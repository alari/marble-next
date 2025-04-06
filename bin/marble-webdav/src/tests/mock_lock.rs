use std::time::Duration;
use async_trait::async_trait;
use crate::api::{LockManager, LockInfo};
use crate::error::LockError;
use uuid::Uuid;

/// Mock LockManager for testing
pub struct MockLockManager;

#[async_trait]
impl LockManager for MockLockManager {
    async fn lock(
        &self,
        _tenant_id: &Uuid,
        _path: &str,
        _timeout: Duration,
        _token: &str,
    ) -> Result<(), LockError> {
        Ok(())  // No-op for tests
    }
    
    async fn unlock(
        &self,
        _tenant_id: &Uuid,
        _path: &str,
        _token: &str,
    ) -> Result<(), LockError> {
        Ok(())  // No-op for tests
    }
    
    async fn is_locked(
        &self,
        _tenant_id: &Uuid,
        _path: &str,
    ) -> Result<Option<LockInfo>, LockError> {
        Ok(None)  // Always unlocked in tests
    }
}
