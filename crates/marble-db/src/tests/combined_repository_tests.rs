//! Tests for all repositories in combination

use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

use crate::Error;
use crate::repositories::{
    Repository,
    UserRepository, 
    SqlxUserRepository,
    FolderRepository,
    SqlxFolderRepository,
    FileRepository,
    SqlxFileRepository,
};
use crate::models::{User, Folder, File};

async fn create_test_pool() -> Result<sqlx::PgPool, Error> {
    // This should be skipped if no test database is available
    let db_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5433/marble_test".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_url)
        .await
        .map_err(Error::ConnectionFailed)?;
        
    Ok(pool)
}

#[tokio::test]
async fn test_combined_repositories() {
    let pool = match create_test_pool().await {
        Ok(pool) => Arc::new(pool),
        Err(_) => {
            println!("Skipping repository test - no test database available");
            return;
        }
    };
    
    // Generate a unique test user name
    let unique_username = format!("combined_test_user_{}", chrono::Utc::now().timestamp());
    println!("Using unique username: {}", unique_username);
    
    // Create repositories
    let user_repo = SqlxUserRepository::new(Arc::clone(&pool));
    let folder_repo = SqlxFolderRepository::new(Arc::clone(&pool));
    let file_repo = SqlxFileRepository::new(Arc::clone(&pool));
    
    // Create test user
    let user = User::new(unique_username.clone(), "passwordhash".to_string());
    let user = user_repo.create(&user).await.unwrap();
    
    // Verify user creation
    let users = user_repo.list(None, None).await.unwrap();
    let matching_users = users.iter().filter(|u| u.username == unique_username).count();
    println!("Users with our test username: {}", matching_users);
    assert_eq!(matching_users, 1, "Should have exactly one user with our test username");
    
    // Create test folder for the user
    let folder = Folder::new(user.id, "/".to_string(), None);
    let _folder = folder_repo.create(&folder).await.unwrap();
    
    // Verify folder creation
    let folders = folder_repo.list_by_user(user.id, None, false).await.unwrap();
    println!("Number of folders after creating one: {}", folders.len());
    assert_eq!(folders.len(), 1, "Should have exactly one folder");
    
    // Create test file for the user
    let file = File::new(
        user.id,
        "/test.md".to_string(),
        "abc123".to_string(),
        "text/markdown".to_string(),
        100
    );
    let _file = file_repo.create(&file).await.unwrap();
    
    // Verify file creation
    let file_count = file_repo.count_by_user(user.id, false).await.unwrap();
    println!("Number of files after creating one: {}", file_count);
    assert_eq!(file_count, 1, "Should have exactly one file");
    
    // Test user listing again to ensure count consistency
    let users_final = user_repo.list(None, None).await.unwrap();
    let matching_users_final = users_final.iter().filter(|u| u.username == unique_username).count();
    println!("Final number of users with our test username: {}", matching_users_final);
    assert_eq!(matching_users_final, 1, "Should still have exactly one user with our test username");
}
