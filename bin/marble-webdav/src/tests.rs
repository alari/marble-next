
// Test implementations for COPY and MOVE operations

#[tokio::test]
async fn test_copy_file() {
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
    tenant_storage.add_file(&tenant_id, "source.txt", test_content.clone());
    
    // Create headers with Destination
    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::DESTINATION, 
        "/destination.txt".parse().unwrap()
    );
    
    // Call COPY method
    let response = handler.handle_copy(tenant_id, "source.txt", headers).await.unwrap();
    
    // Verify response
    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Verify source file still exists
    let source_exists = tenant_storage.exists(&tenant_id, "source.txt").await.unwrap();
    assert!(source_exists);
    
    // Verify destination file was created with same content
    let dest_exists = tenant_storage.exists(&tenant_id, "destination.txt").await.unwrap();
    assert!(dest_exists);
    
    let dest_content = tenant_storage.read(&tenant_id, "destination.txt").await.unwrap();
    assert_eq!(dest_content, test_content);
}

#[tokio::test]
async fn test_copy_directory() {
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
    tenant_storage.add_directory(&tenant_id, "source_dir");
    tenant_storage.add_file(&tenant_id, "source_dir/file1.txt", b"File 1".to_vec());
    tenant_storage.add_file(&tenant_id, "source_dir/file2.txt", b"File 2".to_vec());
    
    // Create headers with Destination
    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::DESTINATION, 
        "/dest_dir".parse().unwrap()
    );
    
    // Call COPY method
    let response = handler.handle_copy(tenant_id, "source_dir", headers).await.unwrap();
    
    // Verify response
    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Verify source directory and files still exist
    let source_exists = tenant_storage.exists(&tenant_id, "source_dir").await.unwrap();
    assert!(source_exists);
    let source_file1_exists = tenant_storage.exists(&tenant_id, "source_dir/file1.txt").await.unwrap();
    assert!(source_file1_exists);
    
    // Verify destination directory and files were created
    let dest_exists = tenant_storage.exists(&tenant_id, "dest_dir").await.unwrap();
    assert!(dest_exists);
    let dest_file1_exists = tenant_storage.exists(&tenant_id, "dest_dir/file1.txt").await.unwrap();
    assert!(dest_file1_exists);
    let dest_file2_exists = tenant_storage.exists(&tenant_id, "dest_dir/file2.txt").await.unwrap();
    assert!(dest_file2_exists);
    
    // Verify file contents were copied correctly
    let dest_file1_content = tenant_storage.read(&tenant_id, "dest_dir/file1.txt").await.unwrap();
    assert_eq!(dest_file1_content, b"File 1".to_vec());
}

#[tokio::test]
async fn test_move_file() {
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
    tenant_storage.add_file(&tenant_id, "source.txt", test_content.clone());
    
    // Create headers with Destination
    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::DESTINATION, 
        "/moved.txt".parse().unwrap()
    );
    
    // Call MOVE method
    let response = handler.handle_move(tenant_id, "source.txt", headers).await.unwrap();
    
    // Verify response
    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Verify source file no longer exists
    let source_exists = tenant_storage.exists(&tenant_id, "source.txt").await.unwrap();
    assert!(!source_exists);
    
    // Verify destination file was created with same content
    let dest_exists = tenant_storage.exists(&tenant_id, "moved.txt").await.unwrap();
    assert!(dest_exists);
    
    let dest_content = tenant_storage.read(&tenant_id, "moved.txt").await.unwrap();
    assert_eq!(dest_content, test_content);
}

#[tokio::test]
async fn test_move_directory() {
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
    tenant_storage.add_directory(&tenant_id, "source_dir");
    tenant_storage.add_file(&tenant_id, "source_dir/file1.txt", b"File 1".to_vec());
    tenant_storage.add_file(&tenant_id, "source_dir/file2.txt", b"File 2".to_vec());
    
    // Create headers with Destination
    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::DESTINATION, 
        "/moved_dir".parse().unwrap()
    );
    
    // Call MOVE method
    let response = handler.handle_move(tenant_id, "source_dir", headers).await.unwrap();
    
    // Verify response
    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Verify source directory no longer exists
    let source_exists = tenant_storage.exists(&tenant_id, "source_dir").await.unwrap();
    assert!(!source_exists);
    
    // Verify destination directory and files were created
    let dest_exists = tenant_storage.exists(&tenant_id, "moved_dir").await.unwrap();
    assert!(dest_exists);
    let dest_file1_exists = tenant_storage.exists(&tenant_id, "moved_dir/file1.txt").await.unwrap();
    assert!(dest_file1_exists);
    let dest_file2_exists = tenant_storage.exists(&tenant_id, "moved_dir/file2.txt").await.unwrap();
    assert!(dest_file2_exists);
    
    // Verify file contents were moved correctly
    let dest_file1_content = tenant_storage.read(&tenant_id, "moved_dir/file1.txt").await.unwrap();
    assert_eq!(dest_file1_content, b"File 1".to_vec());
}

#[tokio::test]
async fn test_overwrite_existing_file() {
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
    tenant_storage.add_file(&tenant_id, "source.txt", b"Source content".to_vec());
    tenant_storage.add_file(&tenant_id, "dest.txt", b"Original destination content".to_vec());
    
    // Create headers with Destination and Overwrite: T
    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::DESTINATION, 
        "/dest.txt".parse().unwrap()
    );
    headers.insert("Overwrite", "T".parse().unwrap());
    
    // Call COPY method
    let response = handler.handle_copy(tenant_id, "source.txt", headers).await.unwrap();
    
    // Verify response
    assert_eq!(response.status(), StatusCode::NO_CONTENT); // 204 for overwritten content
    
    // Verify destination file was overwritten with source content
    let dest_content = tenant_storage.read(&tenant_id, "dest.txt").await.unwrap();
    assert_eq!(dest_content, b"Source content".to_vec());
}

#[tokio::test]
async fn test_copy_with_no_overwrite() {
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
    tenant_storage.add_file(&tenant_id, "source.txt", b"Source content".to_vec());
    tenant_storage.add_file(&tenant_id, "dest.txt", b"Original destination content".to_vec());
    
    // Create headers with Destination and Overwrite: F (false)
    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::DESTINATION, 
        "/dest.txt".parse().unwrap()
    );
    headers.insert("Overwrite", "F".parse().unwrap());
    
    // Call COPY method - should fail since overwrite is false
    let result = handler.handle_copy(tenant_id, "source.txt", headers).await;
    
    // Verify error
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::WebDav(msg) if msg.contains("Destination already exists and overwrite is false") => (),
        err => panic!("Unexpected error: {:?}", err),
    }
    
    // Verify destination file was not changed
    let dest_content = tenant_storage.read(&tenant_id, "dest.txt").await.unwrap();
    assert_eq!(dest_content, b"Original destination content".to_vec());
}
