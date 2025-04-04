use std::path::PathBuf;

use opendal::services::{Fs, S3};
use opendal::Operator;

use crate::config::{StorageBackend, StorageConfig};
use crate::error::{StorageError, StorageResult};
use crate::hash::hash_to_path;

/// Creates a hash-based storage operator based on the configuration
pub fn create_hash_storage(config: &StorageConfig) -> StorageResult<Operator> {
    match &config.backend {
        StorageBackend::FileSystem(fs_config) => {
            let hash_path = fs_config.hash_base_path.clone();
            create_fs_hash_storage(hash_path)
        }
        StorageBackend::S3(s3_config) => {
            let mut builder = S3::default();
            
            // Set the required options
            builder.bucket(&s3_config.bucket);
            builder.region(&s3_config.region);
            
            // Set the optional configurations
            if let Some(ref endpoint) = s3_config.endpoint {
                builder.endpoint(endpoint);
            }
            
            if let Some(ref prefix) = s3_config.prefix {
                let hash_prefix = format!("{}/hash", prefix);
                builder.root(&hash_prefix);
            } else {
                builder.root("/hash");
            }
            
            if let Some(ref access_key) = s3_config.access_key {
                builder.access_key_id(access_key);
            }
            
            if let Some(ref secret_key) = s3_config.secret_key {
                builder.secret_access_key(secret_key);
            }
            
            // Build the operator
            let operator_builder = Operator::new(builder)?;
            Ok(operator_builder.finish())
        }
    }
}

/// Creates a hash-based storage operator using the local filesystem
fn create_fs_hash_storage(base_path: PathBuf) -> StorageResult<Operator> {
    let hash_path = base_path.join("hash");
    
    // Create the directory if it doesn't exist
    if !hash_path.exists() {
        std::fs::create_dir_all(&hash_path)
            .map_err(|e| StorageError::Configuration(format!(
                "Failed to create hash directory: {} - {}", 
                hash_path.display(), e
            )))?;
    }
    
    let mut builder = Fs::default();
    builder.root(hash_path.to_str().ok_or_else(|| {
        StorageError::Configuration(format!(
            "Invalid path: {}",
            hash_path.display()
        ))
    })?);
    
    let operator_builder = Operator::new(builder)?;
    Ok(operator_builder.finish())
}

// Permission layer implementation removed for simplicity
// We'll add a proper Layer implementation in a future phase if needed

/// Put content into hash storage with a given hash
pub async fn put_content_by_hash(
    op: &Operator,
    hash: &str,
    content: Vec<u8>,
) -> StorageResult<()> {
    let path = hash_to_path(hash);
    
    // Check if content already exists (deduplication)
    if op.is_exist(&path).await? {
        // Content already exists, no need to write it again
        return Ok(());
    }
    
    // Write the content
    op.write(&path, content).await?;
    Ok(())
}

/// Get content from hash storage by hash
pub async fn get_content_by_hash(
    op: &Operator,
    hash: &str,
) -> StorageResult<Vec<u8>> {
    let path = hash_to_path(hash);
    let content = op.read(&path).await?;
    Ok(content)
}

/// Get content from hash storage by path
pub async fn get_content_by_path(
    op: &Operator,
    path: &str,
) -> StorageResult<Vec<u8>> {
    let content = op.read(path).await?;
    Ok(content)
}

/// Check if content exists in hash storage
pub async fn exists_by_hash(
    op: &Operator,
    hash: &str,
) -> StorageResult<bool> {
    let path = hash_to_path(hash);
    let exists = op.is_exist(&path).await?;
    Ok(exists)
}

/// Delete content from hash storage by hash
pub async fn delete_by_hash(
    op: &Operator,
    hash: &str,
) -> StorageResult<()> {
    let path = hash_to_path(hash);
    op.delete(&path).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::test;
    use crate::hash::hash_content;

    async fn setup_test_storage() -> (Operator, tempfile::TempDir) {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config = StorageConfig::new_fs(temp_dir.path().to_path_buf());
        
        // Create the storage
        let storage = create_hash_storage(&config).expect("Failed to create storage");
        
        (storage, temp_dir)
    }

    #[test]
    async fn test_put_and_get_content() {
        let (storage, _temp_dir) = setup_test_storage().await;
        
        // Test content
        let content = b"Test content for hash storage";
        
        // Hash the content
        let hash = hash_content(content).expect("Failed to hash content");
        
        // Store the content
        put_content_by_hash(&storage, &hash, content)
            .await
            .expect("Failed to store content");
        
        // Retrieve the content
        let retrieved = get_content_by_hash(&storage, &hash)
            .await
            .expect("Failed to retrieve content");
        
        // Verify the content
        assert_eq!(retrieved, content);
    }

    #[test]
    async fn test_exists_by_hash() {
        let (storage, _temp_dir) = setup_test_storage().await;
        
        // Test content
        let content = b"Test content for checking existence";
        
        // Hash the content
        let hash = hash_content(content).expect("Failed to hash content");
        
        // Check before storing
        let exists_before = exists_by_hash(&storage, &hash)
            .await
            .expect("Failed to check existence");
        assert!(!exists_before, "Content should not exist before storing");
        
        // Store the content
        put_content_by_hash(&storage, &hash, content)
            .await
            .expect("Failed to store content");
        
        // Check after storing
        let exists_after = exists_by_hash(&storage, &hash)
            .await
            .expect("Failed to check existence");
        assert!(exists_after, "Content should exist after storing");
    }

    #[test]
    async fn test_deduplication() {
        let (storage, _temp_dir) = setup_test_storage().await;
        
        // Test content
        let content = b"Test content for deduplication";
        
        // Hash the content
        let hash = hash_content(content).expect("Failed to hash content");
        
        // Store the content twice
        put_content_by_hash(&storage, &hash, content)
            .await
            .expect("Failed to store content first time");
            
        put_content_by_hash(&storage, &hash, content)
            .await
            .expect("Failed to store content second time");
        
        // Should still exist and be retrievable
        let exists = exists_by_hash(&storage, &hash)
            .await
            .expect("Failed to check existence");
        assert!(exists, "Content should exist after storing twice");
        
        let retrieved = get_content_by_hash(&storage, &hash)
            .await
            .expect("Failed to retrieve content");
        assert_eq!(retrieved, content);
    }

    #[test]
    async fn test_delete_by_hash() {
        let (storage, _temp_dir) = setup_test_storage().await;
        
        // Test content
        let content = b"Test content for deletion";
        
        // Hash the content
        let hash = hash_content(content).expect("Failed to hash content");
        
        // Store the content
        put_content_by_hash(&storage, &hash, content)
            .await
            .expect("Failed to store content");
        
        // Verify it exists
        let exists_before = exists_by_hash(&storage, &hash)
            .await
            .expect("Failed to check existence");
        assert!(exists_before, "Content should exist before deletion");
        
        // Delete the content
        delete_by_hash(&storage, &hash)
            .await
            .expect("Failed to delete content");
        
        // Verify it's gone
        let exists_after = exists_by_hash(&storage, &hash)
            .await
            .expect("Failed to check existence");
        assert!(!exists_after, "Content should not exist after deletion");
    }
}
