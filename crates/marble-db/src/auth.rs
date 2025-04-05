//! Authentication services for database users
//!
//! This module provides authentication-related functionality for users
//! in the database, including password verification.

use uuid::Uuid;
use std::sync::Arc;
use async_trait::async_trait;
use sqlx::PgPool;

use crate::error::Error;
use crate::repositories::{SqlxUserRepository, Repository, UserRepository};
use crate::models::User;

/// Error type for authentication operations
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// Missing credentials
    #[error("Missing credentials")]
    MissingCredentials,

    /// Invalid credentials
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// User not found
    #[error("User not found")]
    UserNotFound,

    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] Error),

    /// Password verification error
    #[error("Password verification error: {0}")]
    PasswordVerification(String),
}

/// Result type for authentication operations
pub type AuthResult<T> = std::result::Result<T, AuthError>;

/// Authentication service trait
#[async_trait]
pub trait AuthService: Send + Sync + 'static {
    /// Authenticate a user by username and password
    /// Returns the user's UUID if authentication is successful
    async fn authenticate_user(&self, username: &str, password: &str) -> AuthResult<Uuid>;
    
    /// Verify a password against a stored hash
    async fn verify_password(&self, password: &str, password_hash: &str) -> AuthResult<bool>;
}

/// Database-backed authentication service using SqlxUserRepository
pub struct DatabaseAuthService {
    user_repository: SqlxUserRepository,
}

impl DatabaseAuthService {
    /// Create a new database-backed authentication service
    pub fn new(user_repository: SqlxUserRepository) -> Self {
        Self { user_repository }
    }
    
    /// Create a new database-backed authentication service from a pool
    pub fn from_pool(pool: Arc<PgPool>) -> Self {
        let user_repository = SqlxUserRepository::new(pool);
        Self::new(user_repository)
    }
}

#[async_trait]
impl AuthService for DatabaseAuthService {
    async fn authenticate_user(&self, username: &str, password: &str) -> AuthResult<Uuid> {
        // Find user by username
        let user = self.user_repository
            .find_by_username(username)
            .await?
            .ok_or(AuthError::UserNotFound)?;
        
        // Verify password
        if !self.verify_password(password, &user.password_hash).await? {
            return Err(AuthError::InvalidCredentials);
        }
        
        // Record login (ignoring errors, as authentication still succeeded)
        let _ = self.user_repository.record_login(user.id).await;
        
        Ok(user.uuid)
    }
    
    async fn verify_password(&self, password: &str, password_hash: &str) -> AuthResult<bool> {
        // TODO: Implement proper password verification with a hashing library
        // For now, we just do a simple string comparison as a placeholder
        // In production, this should use a secure password hashing algorithm like bcrypt or Argon2
        Ok(password == password_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    
    async fn create_test_pool() -> crate::Result<PgPool> {
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
    async fn test_auth_service() {
        let pool = match create_test_pool().await {
            Ok(pool) => Arc::new(pool),
            Err(_) => {
                println!("Skipping auth test - no test database available");
                return;
            }
        };
        
        // Clear the users table
        let _ = sqlx::query("DELETE FROM users").execute(&*pool).await;
        
        // Create a user repository
        let user_repository = SqlxUserRepository::new(pool.clone());
        
        // Create a test user
        let user = User::new("testuser".to_string(), "password123".to_string());
        let created = user_repository.create(&user).await.unwrap();
        
        // Create the auth service
        let auth_service = DatabaseAuthService::new(user_repository);
        
        // Test successful authentication
        let uuid = auth_service.authenticate_user("testuser", "password123").await.unwrap();
        assert_eq!(uuid, created.uuid);
        
        // Test failed authentication with wrong password
        let result = auth_service.authenticate_user("testuser", "wrongpassword").await;
        assert!(result.is_err());
        
        // Test failed authentication with wrong username
        let result = auth_service.authenticate_user("nonexistent", "password123").await;
        assert!(result.is_err());
    }
}