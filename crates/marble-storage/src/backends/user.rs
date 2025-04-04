//! User ID conversion and lookup functionality
//!
//! This module provides utilities for working with user IDs, including
//! conversion between UUID and database ID.

use sqlx::postgres::PgPool;
use uuid::Uuid;

use crate::error::{StorageError, StorageResult};

/// Convert a UUID to a database user ID
///
/// Looks up the database ID for a given UUID in the users table.
pub async fn uuid_to_db_id(
    pool: &PgPool,
    uuid: Uuid,
) -> StorageResult<i32> {
    // Query the database for the user ID
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT id FROM users WHERE uuid = $1"
    )
    .bind(uuid)
    .fetch_optional(pool)
    .await;
    
    match result {
        Ok(Some(user_id)) => Ok(user_id),
        Ok(None) => Err(StorageError::Authorization(format!("User with UUID {} not found", uuid))),
        Err(e) => Err(StorageError::Database(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    
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
    
    #[tokio::test]
    async fn test_uuid_to_db_id() {
        // Skip the test if no database is available
        let pool = match setup_test_db().await {
            Ok(pool) => pool,
            Err(_) => {
                println!("Skipping test - no test database available");
                return;
            }
        };
        
        // Generate a test UUID
        let test_uuid = Uuid::new_v4();
        
        // Insert a test user with the UUID
        let inserted_id: i32 = sqlx::query_scalar(
            "INSERT INTO users (username, password_hash, created_at, uuid) 
             VALUES ($1, $2, $3, $4) 
             RETURNING id"
        )
        .bind("uuid_test_user")
        .bind("test_password_hash")
        .bind(chrono::Utc::now())
        .bind(test_uuid)
        .fetch_one(&*pool)
        .await
        .expect("Failed to insert test user");
        
        // Test the conversion
        let db_id = uuid_to_db_id(&pool, test_uuid).await.expect("Failed to convert UUID to DB ID");
        
        // Verify
        assert_eq!(db_id, inserted_id, "Converted DB ID should match inserted ID");
        
        // Test with a non-existent UUID
        let nonexistent_uuid = Uuid::new_v4();
        let result = uuid_to_db_id(&pool, nonexistent_uuid).await;
        assert!(result.is_err(), "Should error for non-existent UUID");
        
        // Clean up
        let _ = sqlx::query("DELETE FROM users WHERE uuid = $1")
            .bind(test_uuid)
            .execute(&*pool)
            .await;
    }
}
