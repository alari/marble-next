use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::api::{LockInfo, LockManager};
use crate::error::LockError;

/// In-memory lock manager implementation
pub struct InMemoryLockManager {
    locks: Arc<RwLock<HashMap<(Uuid, String), LockInfo>>>,
}

impl InMemoryLockManager {
    /// Create a new in-memory lock manager
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Clean expired locks
    async fn clean_expired_locks(&self) {
        let mut locks = self.locks.write().await;
        let now = Utc::now();
        
        locks.retain(|_, lock_info| lock_info.expires_at > now);
    }
}

#[async_trait]
impl LockManager for InMemoryLockManager {
    async fn lock(
        &self,
        tenant_id: &Uuid,
        path: &str,
        timeout: Duration,
        token: &str,
    ) -> Result<(), LockError> {
        // Clean expired locks first
        self.clean_expired_locks().await;
        
        let mut locks = self.locks.write().await;
        let key = (*tenant_id, path.to_string());
        
        // Check if already locked by someone else
        if let Some(existing_lock) = locks.get(&key) {
            if existing_lock.token != token && existing_lock.expires_at > Utc::now() {
                return Err(LockError::ResourceLocked);
            }
        }
        
        // Calculate expiration time
        let expires_at = Utc::now() + ChronoDuration::from_std(timeout)
            .map_err(|e| LockError::Internal(format!("Invalid duration: {}", e)))?;
        
        // Create or update lock
        let lock_info = LockInfo {
            token: token.to_string(),
            tenant_id: *tenant_id,
            path: path.to_string(),
            expires_at,
        };
        
        locks.insert(key, lock_info);
        
        Ok(())
    }

    async fn unlock(
        &self,
        tenant_id: &Uuid,
        path: &str,
        token: &str,
    ) -> Result<(), LockError> {
        let mut locks = self.locks.write().await;
        let key = (*tenant_id, path.to_string());
        
        // Check if locked and verify token
        if let Some(lock_info) = locks.get(&key) {
            if lock_info.token != token {
                return Err(LockError::InvalidLockToken);
            }
            
            // Remove lock
            locks.remove(&key);
            return Ok(());
        }
        
        // Not locked (which is fine for unlock)
        Ok(())
    }

    async fn is_locked(
        &self,
        tenant_id: &Uuid,
        path: &str,
    ) -> Result<Option<LockInfo>, LockError> {
        // Clean expired locks first
        self.clean_expired_locks().await;
        
        let locks = self.locks.read().await;
        let key = (*tenant_id, path.to_string());
        
        // Check if locked
        if let Some(lock_info) = locks.get(&key) {
            // Clone lock info to return
            return Ok(Some(lock_info.clone()));
        }
        
        // Not locked
        Ok(None)
    }
}
