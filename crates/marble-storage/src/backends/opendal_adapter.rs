//! OpenDAL adapter for the RawStorageBackend
//!
//! This module provides a facade that connects OpenDAL with
//! the RawStorageBackend to enable tenant isolation through
//! database metadata while still using OpenDAL's operator interface.

use std::sync::Arc;

use async_trait::async_trait;
use opendal::{
    ErrorKind,
    Metadata,
    Operator,
    OperatorInfo,
    Result as OpendalResult,
    Error as OpendalError,
    Entry,
    EntryMode,
    Reader,
    Writer,
    Lister,
};
use mime_guess::from_path;

use crate::backends::raw::RawStorageBackend;

/// A facade that adapts the RawStorageBackend to the OpenDAL interface.
///
/// This adapter implements a simplified approach by:
/// 1. Mapping OpenDAL operations to RawStorageBackend methods
/// 2. Converting error types between systems
/// 3. Handling metadata appropriately
pub struct RawStorageFacade {
    /// The underlying storage backend
    backend: Arc<RawStorageBackend>,
}

impl RawStorageFacade {
    /// Create a new RawStorageFacade with the given backend
    pub fn new(backend: Arc<RawStorageBackend>) -> Self {
        Self { backend }
    }

    /// Helper to convert our storage errors to OpenDAL errors
    fn convert_error(err: crate::error::StorageError) -> OpendalError {
        match err {
            crate::error::StorageError::NotFound(msg) => {
                OpendalError::new(ErrorKind::NotFound, &msg)
            },
            crate::error::StorageError::Authorization(msg) => {
                OpendalError::new(ErrorKind::PermissionDenied, &msg)
            },
            crate::error::StorageError::Validation(msg) => {
                OpendalError::new(ErrorKind::InvalidInput, &msg)
            },
            _ => OpendalError::new(ErrorKind::Unexpected, &format!("{}", err)),
        }
    }

    /// Helper to normalize paths for OpenDAL
    fn normalize_path(path: &str) -> String {
        let path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };

        // Remove trailing slash unless it's the root path
        if path.len() > 1 && path.ends_with('/') {
            path[0..path.len() - 1].to_string()
        } else {
            path
        }
    }
    
    /// Guess the content type based on file extension
    fn guess_content_type(path: &str) -> String {
        match from_path(path).first() {
            Some(mime) => mime.to_string(),
            None => "application/octet-stream".to_string(),
        }
    }
    
    /// Check if path refers to a directory
    fn is_directory(path: &str) -> bool {
        path.ends_with('/') || path.is_empty() || path == "/"
    }

    /// Read a file from storage
    pub async fn read(&self, path: &str) -> OpendalResult<Vec<u8>> {
        let normalized_path = Self::normalize_path(path);
        self.backend.read_file(&normalized_path)
            .await
            .map_err(Self::convert_error)
    }

    /// Write a file to storage
    pub async fn write(&self, path: &str, content: Vec<u8>) -> OpendalResult<()> {
        let normalized_path = Self::normalize_path(path);
        let content_type = Self::guess_content_type(&normalized_path);

        self.backend.write_file(&normalized_path, content, &content_type)
            .await
            .map_err(Self::convert_error)
    }

    /// Check if a file exists
    pub async fn exists(&self, path: &str) -> OpendalResult<bool> {
        let normalized_path = Self::normalize_path(path);
        self.backend.file_exists(&normalized_path)
            .await
            .map_err(Self::convert_error)
    }
    
    /// Delete a file
    pub async fn delete(&self, path: &str) -> OpendalResult<()> {
        let normalized_path = Self::normalize_path(path);
        self.backend.delete_file(&normalized_path)
            .await
            .map_err(Self::convert_error)
    }
    
    /// List files in a directory
    pub async fn list(&self, path: &str) -> OpendalResult<Vec<String>> {
        let normalized_path = Self::normalize_path(path);
        let dir_path = if normalized_path.ends_with('/') {
            normalized_path
        } else {
            format!("{}/", normalized_path)
        };
        
        self.backend.list_files(&dir_path)
            .await
            .map_err(Self::convert_error)
    }
    
    /// Get file metadata
    pub async fn metadata(&self, path: &str) -> OpendalResult<Option<FileInfo>> {
        let normalized_path = Self::normalize_path(path);
        
        // Check if file exists
        let exists = self.backend.file_exists(&normalized_path)
            .await
            .map_err(Self::convert_error)?;
            
        if !exists {
            return Ok(None);
        }
        
        // If it's a directory, return directory info
        if Self::is_directory(&normalized_path) {
            return Ok(Some(FileInfo {
                path: normalized_path,
                is_dir: true,
                size: 0,
                content_type: "application/x-directory".to_string(),
            }));
        }
        
        // For files, we need to read from the database
        // This is a simplified approach - in a real implementation we'd add a method
        // to RawStorageBackend to get file metadata directly without reading content
        let content = self.backend.read_file(&normalized_path)
            .await
            .map_err(Self::convert_error)?;
            
        let content_type = Self::guess_content_type(&normalized_path);
        
        Ok(Some(FileInfo {
            path: normalized_path,
            is_dir: false,
            size: content.len() as u64,
            content_type,
        }))
    }
}

/// Simple file information structure
pub struct FileInfo {
    /// File path
    pub path: String,
    /// Is this a directory?
    pub is_dir: bool,
    /// File size in bytes
    pub size: u64,
    /// Content type (MIME)
    pub content_type: String,
}

/// Create an OpenDAL operator from a RawStorageBackend
///
/// This function creates a new OpenDAL operator that integrates with
/// our RawStorageBackend to provide tenant isolation.
pub fn create_raw_operator(backend: Arc<RawStorageBackend>) -> OpendalResult<Operator> {
    // Create the facade
    let facade = Arc::new(RawStorageFacade::new(backend));
    
    // We would normally create an Operator from our facade here,
    // but that requires implementing several OpenDAL traits
    // and is beyond the scope of this initial implementation.
    //
    // For now, we'll return an error to indicate this is not yet implemented
    Err(OpendalError::new(ErrorKind::Unsupported, 
        "Creating an OpenDAL Operator from RawStorageFacade is not yet implemented"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    use tempfile::tempdir;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    use sqlx::types::chrono::Utc;
    use crate::config::StorageConfig;
    use crate::backends::hash::create_hash_storage;
    use crate::services::hasher::ContentHasher;
    
    async fn setup_test_db() -> Result<Arc<sqlx::PgPool>, crate::error::StorageError> {
        // This should be skipped if no test database is available
        let db_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5433/marble_test".to_string());
        
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&db_url)
            .await
            .map_err(|e| crate::error::StorageError::Database(e))?;
            
        Ok(Arc::new(pool))
    }
    
    async fn setup_test_user(pool: &sqlx::PgPool) -> Result<i32, crate::error::StorageError> {
        // Create a test user first
        let user_id: i32 = sqlx::query_scalar(
            "INSERT INTO users (username, password_hash, created_at) 
             VALUES ($1, $2, $3) 
             RETURNING id"
        )
        .bind("adapter_test_user")
        .bind("test_password_hash")
        .bind(Utc::now())
        .fetch_one(pool)
        .await
        .map_err(|e| crate::error::StorageError::Database(e))?;
        
        Ok(user_id)
    }
    
    async fn setup_test_facade() -> Option<(RawStorageFacade, i32, Arc<sqlx::PgPool>, tempfile::TempDir)> {
        // Skip the test if no database is available
        let pool = match setup_test_db().await {
            Ok(pool) => pool,
            Err(_) => {
                println!("Skipping test - no test database available");
                return None;
            }
        };
        
        // Clear the files and users table
        let _ = sqlx::query("DELETE FROM files").execute(&*pool).await;
        let _ = sqlx::query("DELETE FROM users WHERE username = 'adapter_test_user'")
            .execute(&*pool).await;
        
        // Create a test user
        let user_id = match setup_test_user(&pool).await {
            Ok(id) => id,
            Err(_) => {
                println!("Failed to create test user");
                return None;
            }
        };
        
        // Create a temp directory for hash storage
        let temp_dir = match tempdir() {
            Ok(dir) => dir,
            Err(_) => {
                println!("Failed to create temp dir");
                return None;
            }
        };
        
        let config = StorageConfig::new_fs(temp_dir.path().to_path_buf());
        let hash_operator = match create_hash_storage(&config) {
            Ok(op) => op,
            Err(_) => {
                println!("Failed to create hash storage");
                return None;
            }
        };
        
        let content_hasher = ContentHasher::new(hash_operator.clone());
        
        let backend = Arc::new(RawStorageBackend::new(
            user_id,
            pool.clone(),
            content_hasher,
        ));
        
        let facade = RawStorageFacade::new(backend);
        
        Some((facade, user_id, pool, temp_dir))
    }
    
    #[test]
    async fn test_facade_write_read() {
        // Setup the test environment
        let (facade, user_id, pool, _temp_dir) = match setup_test_facade().await {
            Some(setup) => setup,
            None => {
                // Skip the test if setup fails
                return;
            }
        };
        
        // Test content
        let content = b"Test content for facade adapter".to_vec();
        
        // Write a file
        facade.write("/test.md", content.clone())
            .await
            .expect("Failed to write file");
        
        // Check if it exists
        let exists = facade.exists("/test.md")
            .await
            .expect("Failed to check existence");
        assert!(exists, "File should exist after writing");
        
        // Read the file
        let read_content = facade.read("/test.md")
            .await
            .expect("Failed to read file");
        assert_eq!(read_content, content, "Read content should match written content");
        
        // Clean up
        let _ = sqlx::query("DELETE FROM files WHERE user_id = $1")
            .bind(user_id)
            .execute(&*pool)
            .await;
        let _ = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&*pool)
            .await;
    }
    
    #[test]
    async fn test_facade_list_delete() {
        // Setup the test environment
        let (facade, user_id, pool, _temp_dir) = match setup_test_facade().await {
            Some(setup) => setup,
            None => {
                // Skip the test if setup fails
                return;
            }
        };
        
        // Write multiple files
        facade.write("/test1.md", b"Test content 1".to_vec())
            .await
            .expect("Failed to write file 1");
            
        facade.write("/test2.md", b"Test content 2".to_vec())
            .await
            .expect("Failed to write file 2");
            
        facade.write("/subdir/nested.md", b"Nested content".to_vec())
            .await
            .expect("Failed to write nested file");
        
        // List files in root directory
        let root_files = facade.list("/")
            .await
            .expect("Failed to list root files");
        assert_eq!(root_files.len(), 3, "Should be 3 files in total");
        assert!(root_files.contains(&"/test1.md".to_string()), "Missing test1.md");
        assert!(root_files.contains(&"/test2.md".to_string()), "Missing test2.md");
        assert!(root_files.contains(&"/subdir/nested.md".to_string()), "Missing nested.md");
        
        // List files in subdirectory
        let subdir_files = facade.list("/subdir")
            .await
            .expect("Failed to list subdirectory");
        assert_eq!(subdir_files.len(), 1, "Should be 1 file in subdirectory");
        assert!(subdir_files.contains(&"/subdir/nested.md".to_string()), "Missing nested.md in subdir");
        
        // Delete a file
        facade.delete("/test1.md")
            .await
            .expect("Failed to delete file");
            
        // Verify it's gone
        let exists = facade.exists("/test1.md")
            .await
            .expect("Failed to check existence");
        assert!(!exists, "File should not exist after deletion");
        
        // List should now have one less file
        let root_files_after_delete = facade.list("/")
            .await
            .expect("Failed to list root files after delete");
        assert_eq!(root_files_after_delete.len(), 2, "Should be 2 files after deletion");
        assert!(!root_files_after_delete.contains(&"/test1.md".to_string()), "test1.md should be gone");
        
        // Clean up
        let _ = sqlx::query("DELETE FROM files WHERE user_id = $1")
            .bind(user_id)
            .execute(&*pool)
            .await;
        let _ = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&*pool)
            .await;
    }
    
    #[test]
    async fn test_path_normalization() {
        assert_eq!(RawStorageFacade::normalize_path("test.md"), "/test.md");
        assert_eq!(RawStorageFacade::normalize_path("/test.md"), "/test.md");
        assert_eq!(RawStorageFacade::normalize_path("/path/"), "/path");
        assert_eq!(RawStorageFacade::normalize_path("path/"), "/path");
        assert_eq!(RawStorageFacade::normalize_path("/"), "/");
        assert_eq!(RawStorageFacade::normalize_path(""), "/");
    }
}
