use opendal::Operator;

use crate::backends::hash::{exists_by_hash, get_content_by_hash, put_content_by_hash};
use crate::error::{StorageError, StorageResult};
use crate::hash::hash_content;

/// Service for handling content hashing and storage
#[derive(Clone)]
pub struct ContentHasher {
    /// The OpenDAL operator for the hash storage
    operator: Operator,
}

impl ContentHasher {
    /// Create a new ContentHasher with the given operator
    pub fn new(operator: Operator) -> Self {
        Self { operator }
    }
    
    /// Store content and return its hash
    ///
    /// If the content already exists (based on its hash), it won't be stored again.
    /// This provides automatic deduplication of content.
    pub async fn store_content(&self, content: &[u8]) -> StorageResult<String> {
        // Generate hash for the content
        let hash = hash_content(content)?;
        
        // Store content in hash-based storage
        put_content_by_hash(&self.operator, &hash, content).await?;
        
        Ok(hash)
    }
    
    /// Retrieve content by its hash
    pub async fn get_content(&self, hash: &str) -> StorageResult<Vec<u8>> {
        get_content_by_hash(&self.operator, hash).await
    }
    
    /// Check if content with the given hash exists
    pub async fn content_exists(&self, hash: &str) -> StorageResult<bool> {
        exists_by_hash(&self.operator, hash).await
    }
    
    /// Get the hash for content without storing it
    ///
    /// This is useful when you want to check if content already exists
    /// without actually storing it.
    pub fn compute_hash(&self, content: &[u8]) -> StorageResult<String> {
        hash_content(content)
    }
    
    /// Store content if its hash matches the expected hash
    ///
    /// This is useful for verifying content integrity during uploads.
    pub async fn store_with_verification(
        &self,
        content: &[u8],
        expected_hash: &str,
    ) -> StorageResult<String> {
        let actual_hash = self.compute_hash(content)?;
        
        if actual_hash != expected_hash {
            return Err(StorageError::Validation(format!(
                "Hash mismatch: expected {}, got {}",
                expected_hash, actual_hash
            )));
        }
        
        // Store the content
        put_content_by_hash(&self.operator, &actual_hash, content).await?;
        
        Ok(actual_hash)
    }
    
    /// Get the underlying storage operator
    pub fn operator(&self) -> &Operator {
        &self.operator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::test;
    use crate::backends::hash::create_hash_storage;
    use crate::config::StorageConfig;

    async fn setup_test_hasher() -> (ContentHasher, tempfile::TempDir) {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config = StorageConfig::new_fs(temp_dir.path().to_path_buf());
        
        // Create the storage
        let storage = create_hash_storage(&config).expect("Failed to create storage");
        let hasher = ContentHasher::new(storage);
        
        (hasher, temp_dir)
    }

    #[test]
    async fn test_store_and_retrieve() {
        let (hasher, _temp_dir) = setup_test_hasher().await;
        
        // Test content
        let content = b"Hello, hasher service!";
        
        // Store the content
        let hash = hasher.store_content(content).await.expect("Failed to store content");
        
        // Retrieve the content
        let retrieved = hasher.get_content(&hash).await.expect("Failed to retrieve content");
        
        // Verify
        assert_eq!(retrieved, content);
    }

    #[test]
    async fn test_compute_hash() {
        let (hasher, _temp_dir) = setup_test_hasher().await;
        
        // Test content
        let content = b"Compute hash only";
        
        // Compute hash without storing
        let hash = hasher.compute_hash(content).expect("Failed to compute hash");
        
        // Content should not exist yet
        let exists = hasher.content_exists(&hash).await.expect("Failed to check existence");
        assert!(!exists, "Content should not exist before storing");
        
        // Store it now
        let stored_hash = hasher.store_content(content).await.expect("Failed to store content");
        
        // Hash should be the same
        assert_eq!(hash, stored_hash, "Computed hash should match stored hash");
        
        // Content should exist now
        let exists_now = hasher.content_exists(&hash).await.expect("Failed to check existence");
        assert!(exists_now, "Content should exist after storing");
    }

    #[test]
    async fn test_store_with_verification() {
        let (hasher, _temp_dir) = setup_test_hasher().await;
        
        // Test content
        let content = b"Content for verification";
        
        // Compute the hash
        let hash = hasher.compute_hash(content).expect("Failed to compute hash");
        
        // Store with correct hash
        let result = hasher.store_with_verification(content, &hash).await;
        assert!(result.is_ok(), "Storing with correct hash should succeed");
        
        // Store with incorrect hash
        let wrong_hash = "wrong_hash_value";
        let result = hasher.store_with_verification(content, wrong_hash).await;
        assert!(result.is_err(), "Storing with incorrect hash should fail");
    }

    #[test]
    async fn test_deduplication() {
        let (hasher, _temp_dir) = setup_test_hasher().await;
        
        // Test content
        let content = b"Duplicate content";
        
        // Store the content twice
        let hash1 = hasher.store_content(content).await.expect("First store failed");
        let hash2 = hasher.store_content(content).await.expect("Second store failed");
        
        // Hashes should be the same
        assert_eq!(hash1, hash2, "Hashes should be the same for identical content");
        
        // Content should be retrievable
        let retrieved = hasher.get_content(&hash1).await.expect("Retrieval failed");
        assert_eq!(retrieved, content);
    }
}
