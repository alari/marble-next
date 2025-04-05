use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use sqlx::types::chrono::Utc;
use uuid::Uuid;
use tempfile::tempdir;

use crate::api::tenant::TenantStorage;
use crate::config::StorageConfig;
use crate::backends::hash::create_hash_storage;
use crate::services::hasher::ContentHasher;
use crate::error::StorageResult;
use crate::create_tenant_storage;

async fn setup_test_db() -> Result<Arc<sqlx::PgPool>, crate::error::StorageError> {
    // This should be skipped if no test database is available
    let db_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5433/marble_test".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .map_err(|e| crate::error::StorageError::Database(e))?;
        
    Ok(Arc::new(pool))
}

async fn setup_test_user(pool: &sqlx::PgPool, username: &str) -> Result<(i32, Uuid), crate::error::StorageError> {
    let test_uuid = Uuid::new_v4();
    
    // Create a test user
    let user_id: i32 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, created_at, uuid) 
         VALUES ($1, $2, $3, $4) 
         RETURNING id"
    )
    .bind(username)
    .bind("test_password_hash")
    .bind(Utc::now())
    .bind(test_uuid)
    .fetch_one(pool)
    .await
    .map_err(|e| crate::error::StorageError::Database(e))?;
    
    Ok((user_id, test_uuid))
}

/// Create tenant storage with two test users
async fn setup_tenant_storage_test() -> Option<(Arc<dyn TenantStorage>, Uuid, Uuid, Arc<sqlx::PgPool>)> {
    // Create the test database connection
    let db_pool = match setup_test_db().await {
        Ok(pool) => pool,
        Err(_) => {
            println!("Skipping test - no test database available");
            return None;
        }
    };
    
    // Clean up any existing test users
    let _ = sqlx::query("DELETE FROM files WHERE user_id IN (SELECT id FROM users WHERE username IN ('tenant_test_user1', 'tenant_test_user2'))")
        .execute(&*db_pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username IN ('tenant_test_user1', 'tenant_test_user2')")
        .execute(&*db_pool)
        .await;
    
    // Create two test users
    let (_, user1_uuid) = match setup_test_user(&db_pool, "tenant_test_user1").await {
        Ok(user) => user,
        Err(_) => {
            println!("Failed to create test user 1");
            return None;
        }
    };
    
    let (_, user2_uuid) = match setup_test_user(&db_pool, "tenant_test_user2").await {
        Ok(user) => user,
        Err(_) => {
            println!("Failed to create test user 2");
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
    
    // Create a content hasher
    let config = StorageConfig::new_fs(temp_dir.path().to_path_buf());
    let hash_operator = match create_hash_storage(&config) {
        Ok(op) => op,
        Err(_) => {
            println!("Failed to create hash storage");
            return None;
        }
    };
    
    let content_hasher = ContentHasher::new(hash_operator);
    
    // Create the tenant storage
    let tenant_storage = match create_tenant_storage(db_pool.clone(), content_hasher).await {
        Ok(storage) => storage,
        Err(_) => {
            println!("Failed to create tenant storage");
            return None;
        }
    };
    
    Some((tenant_storage, user1_uuid, user2_uuid, db_pool))
}

/// Clean up test data
async fn cleanup_tenant_storage_test(db_pool: &Arc<sqlx::PgPool>) {
    let _ = sqlx::query("DELETE FROM files WHERE user_id IN (SELECT id FROM users WHERE username IN ('tenant_test_user1', 'tenant_test_user2'))")
        .execute(&**db_pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username IN ('tenant_test_user1', 'tenant_test_user2')")
        .execute(&**db_pool)
        .await;
}

/// Test basic operations with the tenant storage
#[tokio::test]
async fn test_tenant_storage_basic_operations() {
    // Setup the test environment
    let (tenant_storage, user1_uuid, _, db_pool) = match setup_tenant_storage_test().await {
        Some(setup) => setup,
        None => {
            // Skip the test if setup fails
            return;
        }
    };
    
    // Test content
    let test_content = b"Test content for tenant storage".to_vec();
    
    // Write a file
    tenant_storage.write(&user1_uuid, "/test.md", test_content.clone(), None)
        .await
        .expect("Failed to write file");
    
    // Check if it exists
    let exists = tenant_storage.exists(&user1_uuid, "/test.md")
        .await
        .expect("Failed to check existence");
    assert!(exists, "File should exist after writing");
    
    // Read the file
    let read_content = tenant_storage.read(&user1_uuid, "/test.md")
        .await
        .expect("Failed to read file");
    assert_eq!(read_content, test_content, "Read content should match written content");
    
    // Get metadata
    let metadata = tenant_storage.metadata(&user1_uuid, "/test.md")
        .await
        .expect("Failed to get metadata");
    assert_eq!(metadata.size, test_content.len() as u64);
    assert_eq!(metadata.content_type, "text/markdown");
    assert!(!metadata.is_directory);
    
    // Clean up
    cleanup_tenant_storage_test(&db_pool).await;
}

/// Test tenant isolation with directories
#[tokio::test]
async fn test_tenant_directory_isolation() {
    // Setup the test environment
    let (tenant_storage, user1_uuid, user2_uuid, db_pool) = match setup_tenant_storage_test().await {
        Some(setup) => setup,
        None => {
            // Skip the test if setup fails
            return;
        }
    };
    
    // Create the same directory path for both tenants
    tenant_storage.create_directory(&user1_uuid, "/shared_dir_name")
        .await
        .expect("Failed to create directory for tenant 1");
    
    tenant_storage.create_directory(&user2_uuid, "/shared_dir_name")
        .await
        .expect("Failed to create directory for tenant 2");
    
    // Add different files in each tenant's directory
    tenant_storage.write(&user1_uuid, "/shared_dir_name/tenant1.txt", b"Tenant 1 file".to_vec(), None)
        .await
        .expect("Failed to write file for tenant 1");
    
    tenant_storage.write(&user2_uuid, "/shared_dir_name/tenant2.txt", b"Tenant 2 file".to_vec(), None)
        .await
        .expect("Failed to write file for tenant 2");
    
    // Verify tenant 1 can only see their own file
    let files1 = tenant_storage.list(&user1_uuid, "/shared_dir_name")
        .await
        .expect("Failed to list directory for tenant 1");
    assert_eq!(files1.len(), 1, "Tenant 1 should see only 1 file");
    assert!(files1.contains(&"/shared_dir_name/tenant1.txt".to_string()), "Tenant 1 should see their own file");
    
    // Verify tenant 2 can only see their own file
    let files2 = tenant_storage.list(&user2_uuid, "/shared_dir_name")
        .await
        .expect("Failed to list directory for tenant 2");
    assert_eq!(files2.len(), 1, "Tenant 2 should see only 1 file");
    assert!(files2.contains(&"/shared_dir_name/tenant2.txt".to_string()), "Tenant 2 should see their own file");
    
    // Clean up
    cleanup_tenant_storage_test(&db_pool).await;
}

/// Test tenant isolation
#[tokio::test]
async fn test_tenant_storage_isolation() {
    // Setup the test environment
    let (tenant_storage, user1_uuid, user2_uuid, db_pool) = match setup_tenant_storage_test().await {
        Some(setup) => setup,
        None => {
            // Skip the test if setup fails
            return;
        }
    };
    
    // Test content
    let test_content1 = b"Test content for tenant 1".to_vec();
    let test_content2 = b"Test content for tenant 2".to_vec();
    
    // Write files for both tenants at the same path
    tenant_storage.write(&user1_uuid, "/isolation_test.md", test_content1.clone(), None)
        .await
        .expect("Failed to write file for tenant 1");
    
    tenant_storage.write(&user2_uuid, "/isolation_test.md", test_content2.clone(), None)
        .await
        .expect("Failed to write file for tenant 2");
    
    // Verify tenant 1 can only see their own content
    let read_content1 = tenant_storage.read(&user1_uuid, "/isolation_test.md")
        .await
        .expect("Failed to read file for tenant 1");
    assert_eq!(read_content1, test_content1, "Tenant 1 should see their own content");
    
    // Verify tenant 2 can only see their own content
    let read_content2 = tenant_storage.read(&user2_uuid, "/isolation_test.md")
        .await
        .expect("Failed to read file for tenant 2");
    assert_eq!(read_content2, test_content2, "Tenant 2 should see their own content");
    
    // Delete tenant 1's file
    tenant_storage.delete(&user1_uuid, "/isolation_test.md")
        .await
        .expect("Failed to delete file for tenant 1");
    
    // Verify it's gone for tenant 1
    let exists1 = tenant_storage.exists(&user1_uuid, "/isolation_test.md")
        .await
        .expect("Failed to check existence for tenant 1");
    assert!(!exists1, "File should not exist for tenant 1 after deletion");
    
    // But still exists for tenant 2
    let exists2 = tenant_storage.exists(&user2_uuid, "/isolation_test.md")
        .await
        .expect("Failed to check existence for tenant 2");
    assert!(exists2, "File should still exist for tenant 2");
    
    // Clean up
    cleanup_tenant_storage_test(&db_pool).await;
}

/// Test directory creation and listing
#[tokio::test]
async fn test_tenant_storage_directory_operations() {
    // Setup the test environment
    let (tenant_storage, user1_uuid, _, db_pool) = match setup_tenant_storage_test().await {
        Some(setup) => setup,
        None => {
            // Skip the test if setup fails
            return;
        }
    };
    
    // Create an empty directory
    tenant_storage.create_directory(&user1_uuid, "/empty_dir")
        .await
        .expect("Failed to create directory");
    
    // Check if directory exists
    let exists = tenant_storage.exists(&user1_uuid, "/empty_dir")
        .await
        .expect("Failed to check directory existence");
    assert!(exists, "Directory should exist after creation");
    
    // Get metadata for the directory
    let metadata = tenant_storage.metadata(&user1_uuid, "/empty_dir")
        .await
        .expect("Failed to get directory metadata");
    assert!(metadata.is_directory, "Should be recognized as a directory");
    assert_eq!(metadata.size, 0, "Directory should have zero size");
    
    // Create nested directories
    tenant_storage.create_directory(&user1_uuid, "/parent/child/grandchild")
        .await
        .expect("Failed to create nested directories");
    
    // Create a file in the nested directory
    tenant_storage.write(&user1_uuid, "/parent/child/grandchild/test.txt", b"Test content".to_vec(), None)
        .await
        .expect("Failed to write file in nested directory");
    
    // List files in the nested directory
    let files = tenant_storage.list(&user1_uuid, "/parent/child/grandchild")
        .await
        .expect("Failed to list nested directory");
    assert_eq!(files.len(), 1, "Should be 1 file in nested directory");
    assert!(files.contains(&"/parent/child/grandchild/test.txt".to_string()), "Missing test.txt in nested directory");
    
    // Clean up
    cleanup_tenant_storage_test(&db_pool).await;
}

/// Test metadata operations
#[tokio::test]
async fn test_tenant_storage_metadata() {
    // Setup the test environment
    let (tenant_storage, user1_uuid, _, db_pool) = match setup_tenant_storage_test().await {
        Some(setup) => setup,
        None => {
            // Skip the test if setup fails
            return;
        }
    };
    
    // Write a file
    let content = b"Test content for metadata testing".to_vec();
    tenant_storage.write(&user1_uuid, "/metadata_test.md", content.clone(), Some("text/markdown"))
        .await
        .expect("Failed to write file");
    
    // Get metadata for the file
    let metadata = tenant_storage.metadata(&user1_uuid, "/metadata_test.md")
        .await
        .expect("Failed to get file metadata");
    
    // Verify metadata
    assert_eq!(metadata.path, "/metadata_test.md");
    assert_eq!(metadata.size, content.len() as u64);
    assert_eq!(metadata.content_type, "text/markdown");
    assert!(!metadata.is_directory);
    assert!(metadata.last_modified.is_some(), "Last modified time should be set");
    assert!(metadata.content_hash.is_some(), "Content hash should be set");
    
    // Clean up
    cleanup_tenant_storage_test(&db_pool).await;
}

/// Test directory listing
#[tokio::test]
async fn test_tenant_storage_list() {
    // Setup the test environment
    let (tenant_storage, user1_uuid, _, db_pool) = match setup_tenant_storage_test().await {
        Some(setup) => setup,
        None => {
            // Skip the test if setup fails
            return;
        }
    };
    
    // Write multiple files
    tenant_storage.write(&user1_uuid, "/list_test1.md", b"Test content 1".to_vec(), None)
        .await
        .expect("Failed to write file 1");
        
    tenant_storage.write(&user1_uuid, "/list_test2.md", b"Test content 2".to_vec(), None)
        .await
        .expect("Failed to write file 2");
        
    tenant_storage.write(&user1_uuid, "/subdir/nested.md", b"Nested content".to_vec(), None)
        .await
        .expect("Failed to write nested file");
    
    // List files in root directory
    let root_files = tenant_storage.list(&user1_uuid, "/")
        .await
        .expect("Failed to list root files");
    
    // Verify we get all the files
    assert!(root_files.contains(&"/list_test1.md".to_string()), "Missing list_test1.md");
    assert!(root_files.contains(&"/list_test2.md".to_string()), "Missing list_test2.md");
    assert!(root_files.contains(&"/subdir/nested.md".to_string()), "Missing nested.md");
    
    // List files in subdirectory
    let subdir_files = tenant_storage.list(&user1_uuid, "/subdir")
        .await
        .expect("Failed to list subdirectory");
    assert_eq!(subdir_files.len(), 1, "Should be 1 file in subdirectory");
    assert!(subdir_files.contains(&"/subdir/nested.md".to_string()), "Missing nested.md in subdir");
    
    // Clean up
    cleanup_tenant_storage_test(&db_pool).await;
}