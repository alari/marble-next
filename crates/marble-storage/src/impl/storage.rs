use std::sync::Arc;

use async_trait::async_trait;
use opendal::Operator;
use uuid::Uuid;

use crate::api::MarbleStorage;
use crate::backends::hash::create_hash_storage;
use crate::config::StorageConfig;
use crate::error::{StorageError, StorageResult};
use crate::services::hasher::ContentHasher;

/// Implementation of the MarbleStorage trait
pub struct MarbleStorageImpl {
    /// Configuration for the storage
    config: StorageConfig,
    
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
            hash_operator,
            content_hasher,
        })
    }
    
    /// Get the content hasher service
    pub fn content_hasher(&self) -> &ContentHasher {
        &self.content_hasher
    }
}

#[async_trait]
impl MarbleStorage for MarbleStorageImpl {
    /// Get a raw storage operator for a specific user
    async fn raw_storage(&self, _user_id: Uuid) -> StorageResult<Operator> {
        // This will be implemented in Phase 3 when we integrate with the database
        // For now, return an error to indicate it's not implemented
        Err(StorageError::Configuration(
            "Raw storage is not yet implemented".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::test;

    #[test]
    async fn test_create_storage() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config = StorageConfig::new_fs(temp_dir.path().to_path_buf());
        
        // Create the storage
        let storage = create_storage(config).await.expect("Failed to create storage");
        
        // Get the hash storage
        let hash_storage = storage.hash_storage();
        assert!(hash_storage.info().scheme() == "fs");
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

    #[test]
    async fn test_raw_storage_not_implemented() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config = StorageConfig::new_fs(temp_dir.path().to_path_buf());
        
        // Create the storage
        let storage_impl = MarbleStorageImpl::new(config).await.expect("Failed to create storage");
        
        // Try to get raw storage
        let result = storage_impl.raw_storage(Uuid::new_v4()).await;
        assert!(result.is_err(), "Raw storage should not be implemented yet");
    }
}
