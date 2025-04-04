//! Raw storage backend implementation that integrates with the database
//!
//! This module provides a raw storage backend that uses the database to map
//! file paths to content hashes, enforcing tenant isolation.

use std::sync::Arc;

use marble_db::models::File;
use marble_db::repositories::{FileRepository, SqlxFileRepository, Repository};

use sqlx::postgres::PgPool;

use crate::error::{StorageError, StorageResult};
use crate::hash::hash_content;
use crate::services::hasher::ContentHasher;

/// Raw storage backend that integrates with the database
pub struct RawStorageBackend {
    /// User ID for tenant isolation
    user_id: i32,
    
    /// Database pool for accessing file metadata
    db_pool: Arc<PgPool>,
    
    /// File repository for database operations
    file_repo: Arc<SqlxFileRepository>,
    
    /// Content hasher for hash computation and storage
    content_hasher: ContentHasher,
}

impl RawStorageBackend {
    /// Create a new raw storage backend for a specific user
    pub fn new(
        user_id: i32,
        db_pool: Arc<PgPool>,
        content_hasher: ContentHasher,
    ) -> Self {
        let file_repo = Arc::new(SqlxFileRepository::new(db_pool.clone()));
        
        Self {
            user_id,
            db_pool,
            file_repo,
            content_hasher,
        }
    }
    
    /// Get a file by path from the database
    async fn get_file_by_path(&self, path: &str) -> StorageResult<Option<File>> {
        match self.file_repo.find_by_path(self.user_id, path).await {
            Ok(file) => Ok(file),
            Err(e) => Err(StorageError::Storage(format!("Database error: {}", e))),
        }
    }
    
    /// Create a new file in the database
    async fn create_file(
        &self,
        path: &str,
        content_hash: &str,
        content_type: &str,
        size: i32,
    ) -> StorageResult<File> {
        let file = File::new(
            self.user_id,
            path.to_string(),
            content_hash.to_string(),
            content_type.to_string(),
            size,
        );
        
        match self.file_repo.create(&file).await {
            Ok(file) => Ok(file),
            Err(e) => Err(StorageError::Storage(format!("Database error: {}", e))),
        }
    }
    
    /// Update an existing file in the database
    async fn update_file(
        &self,
        file: &mut File,
        content_hash: &str,
        content_type: &str,
        size: i32,
    ) -> StorageResult<File> {
        file.update_content(
            content_hash.to_string(),
            content_type.to_string(),
            size,
        );
        
        match self.file_repo.update(file).await {
            Ok(file) => Ok(file),
            Err(e) => Err(StorageError::Storage(format!("Database error: {}", e))),
        }
    }
    
    /// Read a file from raw storage
    pub async fn read_file(&self, path: &str) -> StorageResult<Vec<u8>> {
        // First, lookup the file in the database to get the content hash
        let file = self.get_file_by_path(path).await?
            .ok_or_else(|| StorageError::NotFound(format!("File not found: {}", path)))?;
        
        // Check if the file is marked as deleted
        if file.is_deleted {
            return Err(StorageError::NotFound(format!("File is deleted: {}", path)));
        }
            
        // Now get the content using the hash
        self.content_hasher.get_content(&file.content_hash).await
    }
    
    /// Write a file to raw storage
    pub async fn write_file(
        &self,
        path: &str,
        content: Vec<u8>,
        content_type: &str,
    ) -> StorageResult<()> {
        // Hash the content
        let content_hash = hash_content(&content)?;
        let size = content.len() as i32;
        
        // Store the content using the content hasher (which ensures deduplication)
        self.content_hasher.store_content(&content).await?;
        
        // Check if the file already exists in the database
        let existing_file = self.get_file_by_path(path).await?;
        
        // Update or create the file metadata in the database
        if let Some(mut file) = existing_file {
            self.update_file(&mut file, &content_hash, content_type, size)
                .await?;
        } else {
            self.create_file(path, &content_hash, content_type, size)
                .await?;
        }
        
        Ok(())
    }
    
    /// Check if a file exists
    pub async fn file_exists(&self, path: &str) -> StorageResult<bool> {
        let file = self.get_file_by_path(path).await?;
        
        // The file exists if it's in the database and not marked as deleted
        Ok(file.map(|f| !f.is_deleted).unwrap_or(false))
    }
    
    /// Delete a file
    pub async fn delete_file(&self, path: &str) -> StorageResult<()> {
        // First, lookup the file in the database
        let file = self.get_file_by_path(path).await?
            .ok_or_else(|| StorageError::NotFound(format!("File not found: {}", path)))?;
        
        // Mark the file as deleted in the database
        match self.file_repo.mark_deleted(file.id).await {
            Ok(_) => {},
            Err(e) => return Err(StorageError::Storage(format!("Database error: {}", e))),
        }
        
        // Note: We don't delete the actual content from hash storage since other files
        // might reference the same content. Content garbage collection would be a separate process.
        
        Ok(())
    }
    
    /// List files in a directory
    pub async fn list_files(&self, dir_path: &str) -> StorageResult<Vec<String>> {
        // Normalize the directory path
        let normalized_dir = if !dir_path.ends_with('/') && !dir_path.is_empty() {
            format!("{}/", dir_path)
        } else {
            dir_path.to_string()
        };
        
        // List files from the database
        let files = match self.file_repo.list_by_folder_path(self.user_id, &normalized_dir, false).await {
            Ok(files) => files,
            Err(e) => return Err(StorageError::Storage(format!("Database error: {}", e))),
        };
        
        // Extract just the filenames
        let file_paths = files
            .into_iter()
            .map(|file| file.path)
            .collect();
        
        Ok(file_paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    use tempfile::tempdir;
    use crate::backends::hash::create_hash_storage;
    use crate::config::StorageConfig;
    
    async fn setup_test_db() -> Result<Arc<PgPool>, StorageError> {
        // This should be skipped if no test database is available
        let db_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5433/marble_test".to_string());
        
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&db_url)
            .await
            .map_err(|e| StorageError::Database(e))?;
            
        Ok(Arc::new(pool))
    }
    
    async fn setup_test_user(pool: &PgPool) -> Result<i32, StorageError> {
        // Create a test user first
        let user_id: i32 = sqlx::query_scalar(
            "INSERT INTO users (username, password_hash, created_at) 
             VALUES ($1, $2, $3) 
             RETURNING id"
        )
        .bind("raw_storage_test_user")
        .bind("test_password_hash")
        .bind(chrono::Utc::now())
        .fetch_one(pool)
        .await
        .map_err(|e| StorageError::Database(e))?;
        
        Ok(user_id)
    }
    
    async fn setup_test_backend() -> Result<(RawStorageBackend, i32, tempfile::TempDir), StorageError> {
        // Skip the test if no database is available
        let pool = match setup_test_db().await {
            Ok(pool) => pool,
            Err(e) => {
                println!("Skipping test - no test database available: {}", e);
                return Err(StorageError::Configuration("No test database".to_string()));
            }
        };
        
        // Clear the files and users table
        let _ = sqlx::query("DELETE FROM files").execute(&*pool).await;
        let _ = sqlx::query("DELETE FROM users WHERE username = 'raw_storage_test_user'")
            .execute(&*pool).await;
        
        // Create a test user
        let user_id = match setup_test_user(&pool).await {
            Ok(id) => id,
            Err(e) => {
                println!("Failed to create test user: {}", e);
                return Err(StorageError::Configuration("Failed to create test user".to_string()));
            }
        };
        
        // Create a temp directory for hash storage
        let temp_dir = tempdir().map_err(|e| 
            StorageError::Configuration(format!("Failed to create temp dir: {}", e))
        )?;
        
        let config = StorageConfig::new_fs(temp_dir.path().to_path_buf());
        let hash_operator = create_hash_storage(&config)?;
        let content_hasher = ContentHasher::new(hash_operator.clone());
        
        let backend = RawStorageBackend::new(
            user_id,
            pool,
            content_hasher,
        );
        
        Ok((backend, user_id, temp_dir))
    }
    
    #[tokio::test]
    async fn test_raw_storage_backend() {
        // Setup the test environment
        let (backend, user_id, _temp_dir) = match setup_test_backend().await {
            Ok(setup) => setup,
            Err(_) => {
                // Skip the test if setup fails
                return;
            }
        };
        
        // Test content
        let content = b"Test content for raw storage backend".to_vec();
        
        // Test writing a file
        backend.write_file(
            "/test.md",
            content.clone(),
            "text/markdown",
        ).await.expect("Failed to write file");
        
        // Test checking if a file exists
        let exists = backend.file_exists("/test.md").await.expect("Failed to check existence");
        assert!(exists, "File should exist after writing");
        
        // Test reading a file
        let read_content = backend.read_file("/test.md").await.expect("Failed to read file");
        assert_eq!(read_content, content, "Read content should match written content");
        
        // Test listing files
        let files = backend.list_files("/").await.expect("Failed to list files");
        assert_eq!(files.len(), 1, "Should be one file in the root directory");
        assert_eq!(files[0], "/test.md", "File path should match the written file");
        
        // Test writing to a subdirectory
        backend.write_file(
            "/subdir/nested.md",
            b"Nested content".to_vec(),
            "text/markdown",
        ).await.expect("Failed to write file to subdirectory");
        
        // Test listing files in the subdirectory
        let subdir_files = backend.list_files("/subdir").await.expect("Failed to list subdirectory");
        assert_eq!(subdir_files.len(), 1, "Should be one file in the subdirectory");
        assert_eq!(subdir_files[0], "/subdir/nested.md", "Nested file path should match");
        
        // Test deleting a file
        backend.delete_file("/test.md").await.expect("Failed to delete file");
        
        // Test that the file no longer exists
        let exists_after_delete = backend.file_exists("/test.md").await.expect("Failed to check existence");
        assert!(!exists_after_delete, "File should not exist after deletion");
        
        // Test that reading a deleted file fails
        let read_result = backend.read_file("/test.md").await;
        assert!(read_result.is_err(), "Reading a deleted file should fail");
        
        // Clean up
        let _ = sqlx::query("DELETE FROM files WHERE user_id = $1")
            .bind(user_id)
            .execute(&*backend.db_pool)
            .await;
        let _ = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&*backend.db_pool)
            .await;
    }
}
