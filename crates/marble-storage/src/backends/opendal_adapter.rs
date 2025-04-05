//! OpenDAL adapter for the RawStorageBackend
//!
//! This module provides a facade that connects OpenDAL with
//! the RawStorageBackend to enable tenant isolation through
//! database metadata while still using OpenDAL's operator interface.

use std::sync::Arc;
use std::path::PathBuf;

use opendal::{
    ErrorKind,
    Operator,
    Result as OpendalResult,
    Error as OpendalError,
    services::Memory,
    layers::LoggingLayer,
};
use mime_guess::from_path;

use crate::backends::raw::RawStorageBackend;

/// A wrapper for the RawStorageBackend that provides a simplified interface
/// for the OpenDAL adapter.
pub struct RawStorageAdapter {
    /// The underlying storage backend
    backend: Arc<RawStorageBackend>,
    /// The temporary directory for in-memory files (if needed)
    temp_dir: Option<PathBuf>,
}

impl RawStorageAdapter {
    /// Create a new RawStorageAdapter with the given backend
    pub fn new(backend: Arc<RawStorageBackend>) -> Self {
        Self { backend, temp_dir: None }
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
    pub fn normalize_path(path: &str) -> String {
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
}

/// Create an OpenDAL operator from a RawStorageBackend
///
/// This function creates a new OpenDAL operator that integrates with
/// our RawStorageBackend to provide tenant isolation.
///
/// Currently, this is a placeholder implementation that returns a Memory-backed
/// OpenDAL operator. In a real implementation, we would need to:
///
/// 1. Create a custom OpenDAL service or layer that intercepts operations
/// 2. Redirect these operations to our RawStorageBackend
/// 3. Handle metadata appropriately
///
/// However, implementing a full custom OpenDAL adapter requires deep knowledge
/// of the OpenDAL internals and is beyond the scope of this initial implementation.
pub fn create_raw_operator(backend: Arc<RawStorageBackend>) -> OpendalResult<Operator> {
    // Create an adapter wrapping the backend
    let _adapter = RawStorageAdapter::new(backend);
    
    // For now, use a Memory backend since custom adapters are complex
    let memory = Memory::default();
    let op = Operator::new(memory)?.finish();
    
    // Add logging layer for debugging
    #[cfg(debug_assertions)]
    let op = op.layer(LoggingLayer::default());
    
    // This is a placeholder - in a real implementation, we'd create a custom
    // adapter that delegates operations to the RawStorageBackend
    Ok(op)
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
    
    #[test]
    async fn test_path_normalization() {
        assert_eq!(RawStorageAdapter::normalize_path("test.md"), "/test.md");
        assert_eq!(RawStorageAdapter::normalize_path("/test.md"), "/test.md");
        assert_eq!(RawStorageAdapter::normalize_path("/path/"), "/path");
        assert_eq!(RawStorageAdapter::normalize_path("path/"), "/path");
        assert_eq!(RawStorageAdapter::normalize_path("/"), "/");
        assert_eq!(RawStorageAdapter::normalize_path(""), "/");
    }
    
    #[test]
    async fn test_create_raw_operator() {
        // Setup the test environment
        let db_pool = match setup_test_db().await {
            Ok(pool) => pool,
            Err(_) => {
                println!("Skipping test - no test database available");
                return;
            }
        };
        
        // Create a test user
        let user_id = match setup_test_user(&db_pool).await {
            Ok(id) => id,
            Err(_) => {
                println!("Failed to create test user");
                return;
            }
        };
        
        // Create a temp directory for hash storage
        let temp_dir = tempdir().expect("Failed to create temp dir");
        
        // Create the content hasher
        let content_hasher = ContentHasher::new(
            create_hash_storage(&StorageConfig::new_fs(temp_dir.path().to_path_buf())).unwrap()
        );
        
        // Create a raw storage backend
        let backend = Arc::new(RawStorageBackend::new(
            user_id,
            db_pool.clone(),
            content_hasher,
        ));
        
        // Create an operator from the backend
        let operator = create_raw_operator(backend).expect("Failed to create operator");
        
        // Verify the operator was created
        let info = operator.info();
        assert_eq!(info.scheme().to_string(), "memory", "Default placeholder operator should use memory scheme");
        
        // Clean up
        let _ = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&*db_pool)
            .await;
    }
}