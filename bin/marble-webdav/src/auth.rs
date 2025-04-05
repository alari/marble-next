use base64::Engine;
use std::sync::Arc;
use marble_db::auth::{AuthService as DbAuthService, AuthError as DbAuthError};
use uuid::Uuid;

use crate::api::AuthService;
use crate::error::AuthError;

/// WebDAV authentication service that adapts the marble-db AuthService
pub struct WebDavAuthService {
    db_auth_service: Arc<dyn DbAuthService>,
}

impl WebDavAuthService {
    /// Create a new WebDAV authentication service
    pub fn new(db_auth_service: Arc<dyn DbAuthService>) -> Self {
        Self { db_auth_service }
    }
}

#[async_trait::async_trait]
impl AuthService for WebDavAuthService {
    async fn authenticate(&self, username: &str, password: &str) -> Result<Uuid, AuthError> {
        // Use the database auth service for authentication
        self.db_auth_service
            .authenticate_user(username, password)
            .await
            .map_err(|e| match e {
                DbAuthError::MissingCredentials => AuthError::MissingCredentials,
                DbAuthError::InvalidCredentials => AuthError::InvalidCredentials,
                DbAuthError::UserNotFound => AuthError::UserNotFound,
                DbAuthError::Database(e) => AuthError::Database(format!("Database error: {}", e)),
                DbAuthError::PasswordVerification(e) => AuthError::PasswordVerification(e),
            })
    }
}

/// Helper function to extract Basic Auth credentials from headers
pub fn extract_basic_auth(auth_header: Option<&str>) -> Option<(String, String)> {
    let auth_header = auth_header?;
    
    // Check if it's a Basic auth header
    if !auth_header.starts_with("Basic ") {
        return None;
    }
    
    // Extract the base64 encoded credentials
    let encoded = auth_header.trim_start_matches("Basic ").trim();
    
    // Use the engine API from base64 0.22.1
    let decoded = match base64::engine::general_purpose::STANDARD.decode(encoded) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
        Err(_) => return None,
    };
    
    // Split into username and password
    let parts: Vec<&str> = decoded.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }
    
    Some((parts[0].to_string(), parts[1].to_string()))
}
