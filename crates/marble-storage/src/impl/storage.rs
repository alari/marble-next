use std::sync::Arc;

use async_trait::async_trait;
use opendal::{Operator, Scheme};
use sqlx::postgres::PgPool;
use sqlx::types::chrono::Utc;
use uuid::Uuid;

use crate::api::MarbleStorage;
use crate::backends::hash::create_hash_storage;
use crate::backends::raw::RawStorageBackend;
use crate::backends::user::uuid_to_db_id;
use crate::config::StorageConfig;
use crate::error::{StorageError, StorageResult};
use crate::services::hasher::ContentHasher;

/// Implementation of the MarbleStorage trait
pub struct MarbleStorageImpl {
    /// Configuration for the storage
    config: StorageConfig,
    
    /// Database pool for accessing metadata
    db_pool: Option<Arc<PgPool>>,
    
    /// Hash-based storage operator
    hash_operator: Operator,
    
    /// Content hasher service
    content_hasher: ContentHasher,
}

impl MarbleStorageImpl {
    /// Create a new MarbleStorageImpl from the given configuration
    pub async fn new(config: StorageConfig) -> StorageResult<Self> {
        // Validate the configuration
        config.validate()?;
        
        // Create the hash storage operator
        let hash_operator = create_hash_storage(&config)?;
        
        // Create the content hasher
        let content_hasher = ContentHasher::new(hash_operator.clone());
        
        Ok(Self {
            config,
            db_pool: None,
            hash_operator,
            content_hasher,
        })
    }
    
    /// Create a new MarbleStorageImpl with database connection
    pub async fn new_with_db(
        config: StorageConfig,
        db_pool: Arc<PgPool>,
    ) -> StorageResult<Self> {
        // Validate the configuration
        config.validate()?;
        
        // Create the hash storage operator
        let hash_operator = create_hash_storage(&config)?;
        
        // Create the content hasher
        let content_hasher = ContentHasher::new(hash_operator.clone());
        
        Ok(Self {
            config,
            db_pool: Some(db_pool),
            hash_operator,
            content_hasher,
        })
    }
    
    /// Get the content hasher service
    pub fn content_hasher(&self) -> &ContentHasher {
        &self.content_hasher
    }
    
    /// Check if the database connection is available
    fn has_db_connection(&self) -> bool {
        self.db_pool.is_some()
    }
    
    /// Get a reference to the database pool
    fn db_pool(&self) -> StorageResult<&Arc<PgPool>> {
        self.db_pool.as_ref().ok_or_else(|| {
            StorageError::Configuration("Database connection is required but not configured".to_string())
        })
    }
}

#[async_trait]
impl MarbleStorage for MarbleStorageImpl {
    /// Get a raw storage operator for a specific user
    async fn raw_storage(&self, user_id: Uuid) -> StorageResult<Operator> {
        // First, check if we have a database connection
        if !self.has_db_connection() {
            return Err(StorageError::Configuration(
                "Database connection is required for raw storage but not configured".to_string(),
            ));
        }
        
        // Get the database pool
        let db_pool = self.db_pool()?;
        
        // Convert the UUID to a database user ID
        let db_user_id = uuid_to_db_id(db_pool, user_id).await?;
        
        // Create the raw storage backend
        let _backend = Arc::new(RawStorageBackend::new(
            db_user_id,
            db_pool.clone(),
            self.content_hasher.clone(),
        ));
        
        // Create an OpenDAL operator from the backend
        // This is where we would use the OpenDAL adapter, but for now
        // we'll return an error since the adapter is not yet fully implemented
        Err(StorageError::Configuration(
            "OpenDAL adapter for raw storage is not yet fully implemented".to_string(),
        ))
    }
    
    /// Get the hash-based storage operator
    fn hash_storage(&self) -> Operator {
        self.hash_operator.clone()
    }
}

/// Create a new MarbleStorage implementation with the given configuration
pub async fn create_storage(config: StorageConfig) -> StorageResult<Arc<dyn MarbleStorage>> {
    let storage = MarbleStorageImpl::new(config).await?;
    Ok(Arc::new(storage))
}

/// Create a new MarbleStorage implementation with database connection
pub async fn create_storage_with_db(
    config: StorageConfig,
    db_pool: Arc<PgPool>,
) -> StorageResult<Arc<dyn MarbleStorage>> {
    let storage = MarbleStorageImpl::new_with_db(config, db_pool).await?;
    Ok(Arc::new(storage))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::test;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    #[test]
    async fn test_create_storage() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config = StorageConfig::new_fs(temp_dir.path().to_path_buf());
        
        // Create the storage
        let storage = create_storage(config).await.expect("Failed to create storage");
        
        // Get the hash storage
        let hash_storage = storage.hash_storage();
        assert!(hash_storage.info().scheme() == Scheme::Fs);
    }

    #[test]
    async fn test_hash_storage_operations() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config = StorageConfig::new_fs(temp_dir.path().to_path_buf());
        
        // Create the storage
        let storage_impl = MarbleStorageImpl::new(config).await.expect("Failed to create storage");
        
        // Get the content hasher
        let hasher = storage_impl.content_hasher();
        
        // Test content
        let content = b"Test content for storage operations";
        
        // Store the content
        let hash = hasher.store_content(content).await.expect("Failed to store content");
        
        // Check that it exists
        let exists = hasher.content_exists(&hash).await.expect("Failed to check existence");
        assert!(exists, "Content should exist after storing");
        
        // Retrieve the content
        let retrieved = hasher.get_content(&hash).await.expect("Failed to retrieve content");
        assert_eq!(retrieved, content);
    }

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
    
    async fn setup_test_user(pool: &PgPool) -> Result<(i32, Uuid), StorageError> {
        let test_uuid = Uuid::new_v4();
        
        // Create a test user first
        let user_id: i32 = sqlx::query_scalar(
            "INSERT INTO users (username, password_hash, created_at, uuid) 
             VALUES ($1, $2, $3, $4) 
             RETURNING id"
        )
        .bind("raw_storage_impl_test_user")
        .bind("test_password_hash")
        .bind(Utc::now())
        .bind(test_uuid)
        .fetch_one(pool)
        .await
        .map_err(|e| StorageError::Database(e))?;
        
        Ok((user_id, test_uuid))
    }

    #[test]
    async fn test_raw_storage_with_db() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config = StorageConfig::new_fs(temp_dir.path().to_path_buf());
        
        // Setup the test database
        let db_pool = match setup_test_db().await {
            Ok(pool) => pool,
            Err(_) => {
                println!("Skipping test - no test database available");
                return;
            }
        };
        
        // Create a test user
        let (user_id, user_uuid) = match setup_test_user(&db_pool).await {
            Ok(user) => user,
            Err(_) => {
                println!("Failed to create test user");
                return;
            }
        };
        
        // Create the storage with database connection
        let storage_impl = MarbleStorageImpl::new_with_db(config, db_pool.clone())
            .await
            .expect("Failed to create storage with DB");
        
        // Try to get raw storage
        let result = storage_impl.raw_storage(user_uuid).await;
        assert!(result.is_err(), "Raw storage should not be fully implemented yet");
        assert!(result.unwrap_err().to_string().contains("OpenDAL adapter"), 
                "Error should be about OpenDAL adapter");
        
        // Clean up
        let _ = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&*db_pool)
            .await;
    }
    
    #[test]
    async fn test_raw_storage_without_db() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config = StorageConfig::new_fs(temp_dir.path().to_path_buf());
        
        // Create the storage without database connection
        let storage_impl = MarbleStorageImpl::new(config).await.expect("Failed to create storage");
        
        // Try to get raw storage
        let result = storage_impl.raw_storage(Uuid::new_v4()).await;
        assert!(result.is_err(), "Raw storage should error without DB connection");
        assert!(result.unwrap_err().to_string().contains("Database connection"), 
                "Error should be about missing DB connection");
    }
}
