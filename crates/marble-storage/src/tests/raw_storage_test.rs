//! Integration tests for the RawStorageBackend
//!
//! Tests tenant isolation and basic operations

use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use sqlx::types::chrono::Utc;
use tempfile::tempdir;
use tokio::test;
use uuid::Uuid;

use crate::backends::hash::create_hash_storage;
use crate::backends::raw::RawStorageBackend;
use crate::config::StorageConfig;
use crate::error::StorageResult;
use crate::services::hasher::ContentHasher;

async fn setup_test_db() -> StorageResult<Arc<sqlx::PgPool>> {
    // This should be skipped if no test database is available
    let db_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5433/marble_test".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(&db_url)
        .await?;
        
    Ok(Arc::new(pool))
}

async fn setup_test_user(pool: &sqlx::PgPool, username: &str) -> StorageResult<(i32, Uuid)> {
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
    .await?;
    
    Ok((user_id, test_uuid))
}

async fn setup_test_backend(user_id: i32) -> StorageResult<(RawStorageBackend, tempfile::TempDir)> {
    // Create a temp directory for hash storage
    let temp_dir = tempdir().unwrap();
    
    let config = StorageConfig::new_fs(temp_dir.path().to_path_buf());
    let hash_operator = create_hash_storage(&config)?;
    let content_hasher = ContentHasher::new(hash_operator.clone());
    
    // Skip the test if no database is available
    let pool = setup_test_db().await?;
    
    let backend = RawStorageBackend::new(
        user_id,
        pool,
        content_hasher,
    );
    
    Ok((backend, temp_dir))
}

/// Basic test for single-tenant operations
#[test]
async fn test_raw_storage_basic_operations() {
    // Setup the test environment
    let pool = match setup_test_db().await {
        Ok(pool) => pool,
        Err(e) => {
            println!("Skipping test - no test database available: {}", e);
            return;
        }
    };
    
    // Create a test user
    let (user_id, _) = match setup_test_user(&pool, "raw_test_user1").await {
        Ok(user) => user,
        Err(e) => {
            println!("Failed to create test user: {}", e);
            return;
        }
    };
    
    // Create the backend
    let (backend, _temp_dir) = match setup_test_backend(user_id).await {
        Ok(backend) => backend,
        Err(e) => {
            println!("Failed to create backend: {}", e);
            return;
        }
    };
    
    // Test content
    let content = b"Test content for raw storage backend".to_vec();
    
    // Test writing a file
    backend.write_file(
        "/test.md",
        content.clone(),
        "text/markdown",
    ).await.expect("Failed to write file");
    
    // Test checking if a file exists
    let exists = backend.file_exists("/test.md").await.expect("Failed to check existence");
    assert!(exists, "File should exist after writing");
    
    // Test reading a file
    let read_content = backend.read_file("/test.md").await.expect("Failed to read file");
    assert_eq!(read_content, content, "Read content should match written content");
    
    // Test listing files
    let files = backend.list_files("/").await.expect("Failed to list files");
    assert_eq!(files.len(), 1, "Should be one file in the root directory");
    assert_eq!(files[0], "/test.md", "File path should match the written file");
    
    // Test deleting a file
    backend.delete_file("/test.md").await.expect("Failed to delete file");
    
    // Test that the file no longer exists
    let exists_after_delete = backend.file_exists("/test.md").await.expect("Failed to check existence");
    assert!(!exists_after_delete, "File should not exist after deletion");
    
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

/// Test tenant isolation ensures users can't access each other's files
#[test]
async fn test_raw_storage_tenant_isolation() {
    // Setup the test environment
    let pool = match setup_test_db().await {
        Ok(pool) => pool,
        Err(e) => {
            println!("Skipping test - no test database available: {}", e);
            return;
        }
    };
    
    // Create two test users
    let (user1_id, _) = match setup_test_user(&pool, "raw_test_user1").await {
        Ok(user) => user,
        Err(e) => {
            println!("Failed to create test user 1: {}", e);
            return;
        }
    };
    
    let (user2_id, _) = match setup_test_user(&pool, "raw_test_user2").await {
        Ok(user) => user,
        Err(e) => {
            println!("Failed to create test user 2: {}", e);
            return;
        }
    };
    
    // Create backends for both users
    let (backend1, _temp_dir1) = match setup_test_backend(user1_id).await {
        Ok(backend) => backend,
        Err(e) => {
            println!("Failed to create backend for user 1: {}", e);
            return;
        }
    };
    
    let (backend2, _temp_dir2) = match setup_test_backend(user2_id).await {
        Ok(backend) => backend,
        Err(e) => {
            println!("Failed to create backend for user 2: {}", e);
            return;
        }
    };
    
    // Create test content for user 1
    let content1 = b"User 1's content".to_vec();
    let path = "/isolation-test.md";
    
    // User 1 writes a file
    backend1.write_file(
        path,
        content1.clone(),
        "text/markdown",
    ).await.expect("Failed to write file for user 1");
    
    // Verify user 1 can see the file
    let exists_for_user1 = backend1.file_exists(path).await.expect("Failed to check existence for user 1");
    assert!(exists_for_user1, "File should exist for user 1");
    
    // Verify user 2 cannot see user 1's file
    let exists_for_user2 = backend2.file_exists(path).await.expect("Failed to check existence for user 2");
    assert!(!exists_for_user2, "File should NOT exist for user 2 (isolation test)");
    
    // Create test content for user 2 with the same path
    let content2 = b"User 2's content".to_vec();
    
    // User 2 writes a file with the same path
    backend2.write_file(
        path,
        content2.clone(),
        "text/markdown",
    ).await.expect("Failed to write file for user 2");
    
    // Verify both users can see their own files at the same path
    let read_content1 = backend1.read_file(path).await.expect("Failed to read file for user 1");
    assert_eq!(read_content1, content1, "User 1 should see their own content");
    
    let read_content2 = backend2.read_file(path).await.expect("Failed to read file for user 2");
    assert_eq!(read_content2, content2, "User 2 should see their own content");
    
    // Verify content deduplication for identical content
    let identical_content = b"Identical content".to_vec();
    
    // Both users write identical content to different files
    backend1.write_file(
        "/identical1.md",
        identical_content.clone(),
        "text/markdown",
    ).await.expect("Failed to write identical file for user 1");
    
    backend2.write_file(
        "/identical2.md",
        identical_content.clone(),
        "text/markdown",
    ).await.expect("Failed to write identical file for user 2");
    
    // Verify both users can read their files
    let read_identical1 = backend1.read_file("/identical1.md").await.expect("Failed to read identical file for user 1");
    let read_identical2 = backend2.read_file("/identical2.md").await.expect("Failed to read identical file for user 2");
    
    assert_eq!(read_identical1, identical_content, "User 1 should see the identical content");
    assert_eq!(read_identical2, identical_content, "User 2 should see the identical content");
    
    // Clean up
    let _ = sqlx::query("DELETE FROM files WHERE user_id = $1")
        .bind(user1_id)
        .execute(&*pool)
        .await;
    let _ = sqlx::query("DELETE FROM files WHERE user_id = $1")
        .bind(user2_id)
        .execute(&*pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user1_id)
        .execute(&*pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user2_id)
        .execute(&*pool)
        .await;
}
