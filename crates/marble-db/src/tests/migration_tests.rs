//! Tests for database migrations

use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

#[tokio::test]
async fn test_run_migrations() {
    // Skip this test if no test database is available
    let db_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5433/marble_test".to_string());

    // Create a connection pool
    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Could not connect to test database: {}", e);
            eprintln!("Skipping migration test. Run scripts/test_migrations.sh to set up the test database.");
            return;
        }
    };

    // Reset the database
    let result = sqlx::query("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")
        .execute(&pool)
        .await;
    
    if let Err(e) = result {
        eprintln!("Could not reset database: {}", e);
        return;
    }

    // Run migrations
    match crate::MIGRATOR.run(&pool).await {
        Ok(_) => {
            println!("Migrations ran successfully");
        }
        Err(e) => {
            panic!("Failed to run migrations: {}", e);
        }
    }

    // Verify users table exists
    let result = sqlx::query("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await;
    
    assert!(result.is_ok(), "Users table should exist");

    // Verify folders table exists
    let result = sqlx::query("SELECT COUNT(*) FROM folders")
        .fetch_one(&pool)
        .await;
    
    assert!(result.is_ok(), "Folders table should exist");

    // Verify files table exists
    let result = sqlx::query("SELECT COUNT(*) FROM files")
        .fetch_one(&pool)
        .await;
    
    assert!(result.is_ok(), "Files table should exist");
}
