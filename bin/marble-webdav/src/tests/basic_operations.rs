use std::sync::Arc;
use bytes::Bytes;
use http::{HeaderMap, StatusCode};
use crate::dav_handler::MarbleDavHandler;
use super::{MockTenantStorage, MockAuthService, MockLockManager};
use uuid::Uuid;

#[tokio::test]
async fn test_get_file() {
    // Create test dependencies
    let tenant_storage = Arc::new(MockTenantStorage::new());
    let auth_service = Arc::new(MockAuthService::new());
    let lock_manager = Arc::new(MockLockManager);
    
    // Create handler
    let handler = MarbleDavHandler::new(
        tenant_storage.clone(),
        auth_service,
        lock_manager
    );
    
    // Set up test data
    let tenant_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    let test_content = b"Test file content".to_vec();
    tenant_storage.add_file(&tenant_id, "test.txt", test_content.clone());
    
    // Call GET method
    let response = handler.handle_get(tenant_id, "test.txt").await.unwrap();
    
    // Verify response
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(http::header::CONTENT_TYPE).unwrap().to_str().unwrap(),
        "text/plain"
    );
    let body_bytes = response.into_body();
    assert_eq!(body_bytes.to_vec(), test_content);
}

#[tokio::test]
async fn test_get_nonexistent_file() {
    // Create test dependencies
    let tenant_storage = Arc::new(MockTenantStorage::new());
    let auth_service = Arc::new(MockAuthService::new());
    let lock_manager = Arc::new(MockLockManager);
    
    // Create handler
    let handler = MarbleDavHandler::new(
        tenant_storage.clone(),
        auth_service,
        lock_manager
    );
    
    // Set up test data
    let tenant_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    
    // Call GET method for nonexistent file
    let result = handler.handle_get(tenant_id, "nonexistent.txt").await;
    
    // Verify error
    assert!(result.is_err());
    match result.unwrap_err() {
        crate::error::Error::Storage(marble_storage::error::StorageError::NotFound(_)) => (),
        err => panic!("Unexpected error: {:?}", err),
    }
}

#[tokio::test]
async fn test_put_file() {
    // Create test dependencies
    let tenant_storage = Arc::new(MockTenantStorage::new());
    let auth_service = Arc::new(MockAuthService::new());
    let lock_manager = Arc::new(MockLockManager);
    
    // Create handler
    let handler = MarbleDavHandler::new(
        tenant_storage.clone(),
        auth_service,
        lock_manager
    );
    
    // Set up test data
    let tenant_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    let test_content = b"New file content".to_vec();
    
    // Empty headers for test
    let headers = HeaderMap::new();
    
    // Call PUT method
    let response = handler.handle_put(
        tenant_id, 
        "new.txt", 
        headers, 
        Bytes::from(test_content.clone())
    ).await.unwrap();
    
    // Verify response
    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Verify file was created
    let stored_content = tenant_storage.read(&tenant_id, "new.txt").await.unwrap();
    assert_eq!(stored_content, test_content);
}

#[tokio::test]
async fn test_mkcol_directory() {
    // Create test dependencies
    let tenant_storage = Arc::new(MockTenantStorage::new());
    let auth_service = Arc::new(MockAuthService::new());
    let lock_manager = Arc::new(MockLockManager);
    
    // Create handler
    let handler = MarbleDavHandler::new(
        tenant_storage.clone(),
        auth_service,
        lock_manager
    );
    
    // Set up test data
    let tenant_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    
    // Call MKCOL method
    let response = handler.handle_mkcol(tenant_id, "test_dir").await.unwrap();
    
    // Verify response
    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Verify directory was created
    let exists = tenant_storage.exists(&tenant_id, "test_dir").await.unwrap();
    assert!(exists);
    
    // Verify it's a directory
    let metadata = tenant_storage.metadata(&tenant_id, "test_dir").await.unwrap();
    assert!(metadata.is_directory);
}

#[tokio::test]
async fn test_delete_file() {
    // Create test dependencies
    let tenant_storage = Arc::new(MockTenantStorage::new());
    let auth_service = Arc::new(MockAuthService::new());
    let lock_manager = Arc::new(MockLockManager);
    
    // Create handler
    let handler = MarbleDavHandler::new(
        tenant_storage.clone(),
        auth_service,
        lock_manager
    );
    
    // Set up test data
    let tenant_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    tenant_storage.add_file(&tenant_id, "to_delete.txt", b"Delete me".to_vec());
    
    // Verify file exists before deletion
    let exists = tenant_storage.exists(&tenant_id, "to_delete.txt").await.unwrap();
    assert!(exists);
    
    // Call DELETE method
    let response = handler.handle_delete(tenant_id, "to_delete.txt").await.unwrap();
    
    // Verify response
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    
    // Verify file was deleted
    let exists = tenant_storage.exists(&tenant_id, "to_delete.txt").await.unwrap();
    assert!(!exists);
}

#[tokio::test]
async fn test_propfind_directory() {
    // Create test dependencies
    let tenant_storage = Arc::new(MockTenantStorage::new());
    let auth_service = Arc::new(MockAuthService::new());
    let lock_manager = Arc::new(MockLockManager);
    
    // Create handler
    let handler = MarbleDavHandler::new(
        tenant_storage.clone(),
        auth_service,
        lock_manager
    );
    
    // Set up test data
    let tenant_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    tenant_storage.add_directory(&tenant_id, "test_dir");
    tenant_storage.add_file(&tenant_id, "test_dir/file1.txt", b"File 1".to_vec());
    tenant_storage.add_file(&tenant_id, "test_dir/file2.txt", b"File 2".to_vec());
    
    // Call PROPFIND method
    let response = handler.handle_propfind(
        tenant_id, 
        "test_dir", 
        Bytes::new()
    ).await.unwrap();
    
    // Verify response
    assert_eq!(response.status(), StatusCode::MULTI_STATUS);
    assert_eq!(
        response.headers().get(http::header::CONTENT_TYPE).unwrap().to_str().unwrap(),
        "application/xml"
    );
    
    // Convert response body to string
    let body = String::from_utf8(response.into_body().to_vec()).unwrap();
    
    // Check that response contains both files
    assert!(body.contains("file1.txt"));
    assert!(body.contains("file2.txt"));
}
