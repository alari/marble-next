use std::collections::HashMap;
use async_trait::async_trait;
use crate::api::AuthService;
use crate::error::AuthError;
use uuid::Uuid;

/// Mock AuthService for testing
pub struct MockAuthService {
    // Map of username -> (password, tenant_id)
    users: HashMap<String, (String, Uuid)>,
}

impl MockAuthService {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        // Add a test user
        users.insert(
            "testuser".to_string(),
            ("password123".to_string(), Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap())
        );
        
        Self { users }
    }
}

#[async_trait]
impl AuthService for MockAuthService {
    async fn authenticate(&self, username: &str, password: &str) -> Result<Uuid, AuthError> {
        if let Some((stored_password, tenant_id)) = self.users.get(username) {
            if stored_password == password {
                return Ok(*tenant_id);
            }
        }
        
        Err(AuthError::InvalidCredentials)
    }
}
