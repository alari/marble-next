//! Raw storage backend implementation that integrates with the database
//!
//! This module provides a raw storage backend that uses the database to map
//! file paths to content hashes, enforcing tenant isolation.

use std::sync::Arc;

use marble_db::models::File;
use marble_db::repositories::{FileRepository, SqlxFileRepository, Repository};
use sqlx::postgres::PgPool;

use crate::api::tenant::FileMetadata;

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
    
    /// Get metadata for a file
    pub async fn get_file_metadata(&self, path: &str) -> StorageResult<FileMetadata> {
        use crate::api::tenant::FileMetadata;
        
        // Look up the file in the database
        let file = self.get_file_by_path(path).await?
            .ok_or_else(|| StorageError::NotFound(format!("File not found: {}", path)))?;
            
        // Check if the file is marked as deleted
        if file.is_deleted {
            return Err(StorageError::NotFound(format!("File is deleted: {}", path)));
        }
        
        // Determine if it's a directory based on the content type
        let is_directory = 
            file.content_type == "application/vnd.marble.directory" || 
            path.ends_with('/') || 
            path == "/";
            
        // Get the last modified time from the database
        let last_modified = file.updated_at
            .timestamp_millis()
            .try_into()
            .ok();
            
        // Create the metadata
        let metadata = FileMetadata {
            path: file.path,
            size: file.size as u64,
            content_type: file.content_type,
            is_directory,
            last_modified,
            content_hash: Some(file.content_hash),
        };
        
        Ok(metadata)
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
    
    /// Create a directory
    ///
    /// Creates an empty directory by adding a special placeholder file to the database.
    /// Since we don't actually have physical directories (only files), this is
    /// represented as a metadata-only entry in the database with a special content type.
    pub async fn create_directory(&self, dir_path: &str) -> StorageResult<()> {
        // Normalize the directory path to ensure it ends with a slash
        let normalized_dir = if dir_path.ends_with('/') || dir_path == "" {
            dir_path.to_string()
        } else {
            format!("{}/", dir_path)
        };
        
        // Check if the directory already exists by checking for any files with this prefix
        let files = match self.file_repo.list_by_folder_path(self.user_id, &normalized_dir, false).await {
            Ok(files) => files,
            Err(e) => return Err(StorageError::Storage(format!("Database error: {}", e))),
        };
        
        // If there are already files with this prefix, the directory "exists"
        if !files.is_empty() {
            return Ok(());
        }
        
        // Create the parent directories if needed
        let path_parts: Vec<&str> = normalized_dir
            .trim_matches('/')
            .split('/')
            .collect();
            
        // Create parent directories (if needed)
        if path_parts.len() > 1 {
            let mut parent_path = String::from("/");
            for i in 0..path_parts.len() - 1 {
                parent_path.push_str(path_parts[i]);
                parent_path.push('/');
                
                // Check if this parent directory exists
                let parent_files = match self.file_repo.list_by_folder_path(self.user_id, &parent_path, false).await {
                    Ok(files) => files,
                    Err(e) => return Err(StorageError::Storage(format!("Database error: {}", e))),
                };
                
                // If it doesn't exist, create a placeholder
                if parent_files.is_empty() {
                    let placeholder_path = format!("{}/.dir", parent_path.trim_end_matches('/'));
                    let content_hash = hash_content(&[])?;
                    self.create_file(
                        &placeholder_path,
                        &content_hash,
                        "application/vnd.marble.directory",
                        0,
                    ).await?;
                }
            }
        }
        
        // Create an empty directory placeholder
        let placeholder_path = if normalized_dir == "/" {
            "/.dir".to_string()
        } else {
            format!("{}/.dir", normalized_dir.trim_end_matches('/'))
        };
        
        // Create a zero-length file with a special content type to mark it as a directory
        let content_hash = hash_content(&[])?;
        self.create_file(
            &placeholder_path,
            &content_hash,
            "application/vnd.marble.directory",
            0,
        ).await?;
        
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
    use sqlx::types::chrono::Utc;
    use std::time::Duration;
    use tempfile::tempdir;
    use crate::backends::hash::create_hash_storage;
    use crate::config::StorageConfig;
    use crate::api::tenant::FileMetadata;
    
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
        .bind(Utc::now())
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
    async fn test_directory_operations() {
        // Setup the test environment
        let (backend, user_id, _temp_dir) = match setup_test_backend().await {
            Ok(setup) => setup,
            Err(_) => {
                // Skip the test if setup fails
                return;
            }
        };
        
        // Test creating a directory
        backend.create_directory("/test_dir").await.expect("Failed to create directory");
        
        // Test checking if directory exists
        let exists = backend.file_exists("/test_dir/.dir").await.expect("Failed to check directory existence");
        assert!(exists, "Directory placeholder should exist after creation");
        
        // Test creating nested directories
        backend.create_directory("/parent/child/grandchild").await.expect("Failed to create nested directories");
        
        // Verify parent directories were created
        let parent_exists = backend.file_exists("/parent/.dir").await.expect("Failed to check parent directory");
        let child_exists = backend.file_exists("/parent/child/.dir").await.expect("Failed to check child directory");
        let grandchild_exists = backend.file_exists("/parent/child/grandchild/.dir").await.expect("Failed to check grandchild directory");
        
        assert!(parent_exists, "Parent directory should exist");
        assert!(child_exists, "Child directory should exist");
        assert!(grandchild_exists, "Grandchild directory should exist");
        
        // Test writing a file to a directory
        backend.write_file(
            "/parent/child/file.txt",
            b"Test content in directory".to_vec(),
            "text/plain",
        ).await.expect("Failed to write file in directory");
        
        // Test listing files in a directory
        let files = backend.list_files("/parent/child").await.expect("Failed to list directory");
        assert!(files.contains(&"/parent/child/file.txt".to_string()), "Directory listing should include the file");
        assert!(files.contains(&"/parent/child/grandchild/.dir".to_string()), "Directory listing should include subdirectory placeholder");
        
        // Test getting metadata
        let metadata = backend.get_file_metadata("/parent/child/.dir").await.expect("Failed to get directory metadata");
        assert!(metadata.is_directory, "Should be identified as a directory");
        assert_eq!(metadata.size, 0, "Directory should have zero size");
        assert_eq!(metadata.content_type, "application/vnd.marble.directory", "Should have directory content type");
        
        // Clean up
        let _ = sqlx::query("DELETE FROM files WHERE user_id = $1")
            .bind(user_id)
            .execute(&*backend.db_pool)
            .await;
    }
    
    #[tokio::test]
    async fn test_metadata_retrieval() {
        // Setup the test environment
        let (backend, user_id, _temp_dir) = match setup_test_backend().await {
            Ok(setup) => setup,
            Err(_) => {
                // Skip the test if setup fails
                return;
            }
        };
        
        // Test content with known values
        let content = b"Test content for metadata testing".to_vec();
        let content_size = content.len() as u64;
        let expected_content_hash = hash_content(&content).expect("Failed to hash content");
        
        // Write a file
        backend.write_file(
            "/metadata_test.md",
            content.clone(),
            "text/markdown",
        ).await.expect("Failed to write file");
        
        // Get metadata
        let metadata = backend.get_file_metadata("/metadata_test.md").await.expect("Failed to get metadata");
        
        // Verify metadata fields
        assert_eq!(metadata.path, "/metadata_test.md");
        assert_eq!(metadata.size, content_size);
        assert_eq!(metadata.content_type, "text/markdown");
        assert!(!metadata.is_directory);
        assert!(metadata.last_modified.is_some(), "Last modified time should be set");
        assert_eq!(metadata.content_hash.unwrap(), expected_content_hash, "Content hash should match expected value");
        
        // Clean up
        let _ = sqlx::query("DELETE FROM files WHERE user_id = $1")
            .bind(user_id)
            .execute(&*backend.db_pool)
            .await;
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
