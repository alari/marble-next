//! Repository for user operations
//!
//! This module provides the UserRepository trait and its SQLx implementation.

use sqlx::postgres::{PgPool, PgRow};
use sqlx::{FromRow, Row};
use std::sync::Arc;
use async_trait::async_trait;

use crate::models::User;
use crate::Result;
use crate::Error;
use super::{Repository, BaseRepository};

/// Repository trait for user operations
#[async_trait]
pub trait UserRepository: Repository + BaseRepository + Send + Sync {
    /// Find a user by ID
    async fn find_by_id(&self, id: i32) -> Result<Option<User>>;
    
    /// Find a user by username
    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;
    
    /// Create a new user
    async fn create(&self, user: &User) -> Result<User>;
    
    /// Update an existing user
    async fn update(&self, user: &User) -> Result<User>;
    
    /// Delete a user by ID
    async fn delete(&self, id: i32) -> Result<bool>;
    
    /// Record a login for a user
    async fn record_login(&self, id: i32) -> Result<bool>;
    
    /// List all users (with optional pagination)
    async fn list(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<User>>;
}

/// SQLx implementation of the UserRepository
pub struct SqlxUserRepository {
    pool: Arc<PgPool>,
}

impl Repository for SqlxUserRepository {
    fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

impl BaseRepository for SqlxUserRepository {
    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl FromRow<'_, PgRow> for User {
    fn from_row(row: &PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(User {
            id: row.try_get("id")?,
            uuid: row.try_get("uuid")?,
            username: row.try_get("username")?,
            password_hash: row.try_get("password_hash")?,
            created_at: row.try_get("created_at")?,
            last_login: row.try_get("last_login")?,
        })
    }
}

#[async_trait]
impl UserRepository for SqlxUserRepository {
    async fn find_by_id(&self, id: i32) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, uuid, username, password_hash, created_at, last_login 
             FROM users 
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(user)
    }
    
    async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, uuid, username, password_hash, created_at, last_login 
             FROM users 
             WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(user)
    }
    
    async fn create(&self, user: &User) -> Result<User> {
        let created_user = sqlx::query_as::<_, User>(
            "INSERT INTO users (uuid, username, password_hash, created_at, last_login) 
             VALUES ($1, $2, $3, $4, $5) 
             RETURNING id, uuid, username, password_hash, created_at, last_login"
        )
        .bind(user.uuid)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(user.created_at)
        .bind(user.last_login)
        .fetch_one(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(created_user)
    }
    
    async fn update(&self, user: &User) -> Result<User> {
        let updated_user = sqlx::query_as::<_, User>(
            "UPDATE users 
             SET username = $1, password_hash = $2, last_login = $3 
             WHERE id = $4 
             RETURNING id, uuid, username, password_hash, created_at, last_login"
        )
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(user.last_login)
        .bind(user.id)
        .fetch_one(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(updated_user)
    }
    
    async fn delete(&self, id: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(Error::QueryFailed)?;
            
        Ok(result.rows_affected() > 0)
    }
    
    async fn record_login(&self, id: i32) -> Result<bool> {
        let now = chrono::Utc::now();
        let result = sqlx::query(
            "UPDATE users 
             SET last_login = $1 
             WHERE id = $2"
        )
        .bind(now)
        .bind(id)
        .execute(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn list(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<User>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        
        let users = sqlx::query_as::<_, User>(
            "SELECT id, uuid, username, password_hash, created_at, last_login 
             FROM users 
             ORDER BY id 
             LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    
    async fn create_test_pool() -> Result<PgPool> {
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
    async fn test_user_repository() {
        let pool = match create_test_pool().await {
            Ok(pool) => Arc::new(pool),
            Err(_) => {
                println!("Skipping repository test - no test database available");
                return;
            }
        };
        
        // Clear the users table
        let _ = sqlx::query("DELETE FROM users").execute(&*pool).await;
        
        let repo = SqlxUserRepository::new(pool);
        
        // Test creating a user
        let user = User::new("testuser".to_string(), "passwordhash".to_string());
        let created = repo.create(&user).await.unwrap();
        
        assert!(created.id > 0);
        assert_eq!(created.username, "testuser");
        
        // Test finding user by ID
        let found = repo.find_by_id(created.id).await.unwrap().unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.username, "testuser");
        
        // Test finding user by username
        let found = repo.find_by_username("testuser").await.unwrap().unwrap();
        assert_eq!(found.id, created.id);
        
        // Test recording login
        let result = repo.record_login(created.id).await.unwrap();
        assert!(result);
        
        let updated = repo.find_by_id(created.id).await.unwrap().unwrap();
        assert!(updated.last_login.is_some());
        
        // Test listing users
        let users = repo.list(None, None).await.unwrap();
        assert_eq!(users.len(), 1);
        
        // Test deleting user
        let result = repo.delete(created.id).await.unwrap();
        assert!(result);
        
        let not_found = repo.find_by_id(created.id).await.unwrap();
        assert!(not_found.is_none());
    }
}
