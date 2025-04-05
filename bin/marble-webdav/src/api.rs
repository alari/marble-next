use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::error::{AuthError, LockError};

/// Authentication service trait
#[async_trait]
pub trait AuthService: Send + Sync + 'static {
    /// Authenticate a user and return their tenant ID
    async fn authenticate(&self, username: &str, password: &str) -> Result<Uuid, AuthError>;
}

/// Lock information
#[derive(Debug, Clone)]
pub struct LockInfo {
    /// Lock token
    pub token: String,
    
    /// Tenant ID of the lock owner
    pub tenant_id: Uuid,
    
    /// Path that is locked
    pub path: String,
    
    /// When the lock expires
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Lock manager trait
#[async_trait]
pub trait LockManager: Send + Sync + 'static {
    /// Acquire a lock
    async fn lock(
        &self,
        tenant_id: &Uuid,
        path: &str,
        timeout: Duration,
        token: &str,
    ) -> Result<(), LockError>;

    /// Release a lock
    async fn unlock(
        &self,
        tenant_id: &Uuid,
        path: &str,
        token: &str,
    ) -> Result<(), LockError>;

    /// Check if a resource is locked
    async fn is_locked(
        &self,
        tenant_id: &Uuid,
        path: &str,
    ) -> Result<Option<LockInfo>, LockError>;
}

/// Type alias for a reference-counted auth service
pub type AuthServiceRef = Arc<dyn AuthService>;

/// Type alias for a reference-counted lock manager
pub type LockManagerRef = Arc<dyn LockManager>;
