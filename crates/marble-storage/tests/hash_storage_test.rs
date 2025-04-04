use marble_storage::{create_storage, StorageConfig};
use tempfile::tempdir;
use tokio::test;

#[test]
async fn test_hash_storage_integration() {
    // Create a temporary directory
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config = StorageConfig::new_fs(temp_dir.path().to_path_buf());
    
    // Create the storage
    let storage = create_storage(config).await.expect("Failed to create storage");
    
    // Get the hash storage
    let hash_op = storage.hash_storage();
    
    // Test content
    let content = b"Integration test content";
    let path = "/.hash/test_integration";
    
    // Write content
    hash_op.write(path, content.to_vec()).await.expect("Failed to write content");
    
    // Read content
    let retrieved = hash_op.read(path).await.expect("Failed to read content");
    
    // Verify
    assert_eq!(retrieved, content);
    
    // Check existence
    let exists = hash_op.is_exist(path).await.expect("Failed to check existence");
    assert!(exists, "Content should exist");
    
    // Delete
    hash_op.delete(path).await.expect("Failed to delete content");
    
    // Verify deletion
    let exists_after = hash_op.is_exist(path).await.expect("Failed to check existence");
    assert!(!exists_after, "Content should not exist after deletion");
}
