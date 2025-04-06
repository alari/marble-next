#[cfg(test)]
mod lock_tests {
    use crate::operations::{handle_lock, handle_unlock};
    use crate::api::{AuthServiceRef, LockManagerRef};
    use crate::lock::InMemoryLockManager;
    use crate::tests::common::MockTenantStorage;
    use marble_storage::api::TenantStorageRef;
    use marble_core::models::user::UserId;
    use http::{HeaderMap, StatusCode};
    use bytes::Bytes;
    use std::sync::Arc;
    use std::str::FromStr;
    use uuid::Uuid;
    
    // Mock auth service for testing
    struct MockAuthService;
    
    #[async_trait::async_trait]
    impl crate::api::AuthService for MockAuthService {
        async fn authenticate(&self, _username: &str, _password: &str) -> Result<Uuid, crate::error::AuthError> {
            Ok(Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap())
        }
    }
    
    // Setup helper
    fn setup() -> (TenantStorageRef, AuthServiceRef, LockManagerRef, Uuid) {
        let storage = Arc::new(MockTenantStorage::new());
        let auth_service: AuthServiceRef = Arc::new(MockAuthService);
        let lock_manager: LockManagerRef = Arc::new(InMemoryLockManager::new());
        let tenant_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        
        (storage, auth_service, lock_manager, tenant_id)
    }
    
    #[tokio::test]
    async fn test_lock_and_unlock() {
        let (_storage, _auth_service, lock_manager, tenant_id) = setup();
        
        // Create a simple lock XML body
        let lock_body = r#"<?xml version="1.0" encoding="utf-8" ?>
            <D:lockinfo xmlns:D="DAV:">
                <D:lockscope><D:exclusive/></D:lockscope>
                <D:locktype><D:write/></D:locktype>
                <D:owner>Test User</D:owner>
            </D:lockinfo>"#;
        
        // Create headers for lock request
        let mut lock_headers = HeaderMap::new();
        lock_headers.insert("Timeout", "Second-3600".parse().unwrap());
        
        // Test LOCK operation
        let lock_response = handle_lock(
            &lock_manager,
            tenant_id,
            "test/path.md",
            lock_headers,
            Bytes::from(lock_body)
        ).await.unwrap();
        
        // Check response status
        assert_eq!(lock_response.status(), StatusCode::OK);
        
        // Extract lock token from response
        let lock_token = lock_response.headers()
            .get("Lock-Token")
            .and_then(|v| v.to_str().ok())
            .unwrap();
        
        // Create headers for unlock request
        let mut unlock_headers = HeaderMap::new();
        unlock_headers.insert("Lock-Token", lock_token.parse().unwrap());
        
        // Test UNLOCK operation
        let unlock_response = handle_unlock(
            &lock_manager,
            tenant_id,
            "test/path.md",
            unlock_headers
        ).await.unwrap();
        
        // Check response status
        assert_eq!(unlock_response.status(), StatusCode::NO_CONTENT);
    }
    
    #[tokio::test]
    async fn test_lock_conflict() {
        let (_storage, _auth_service, lock_manager, tenant_id) = setup();
        let other_tenant_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();
        
        // Create simple lock XML body
        let lock_body = r#"<?xml version="1.0" encoding="utf-8" ?>
            <D:lockinfo xmlns:D="DAV:">
                <D:lockscope><D:exclusive/></D:lockscope>
                <D:locktype><D:write/></D:locktype>
                <D:owner>Test User</D:owner>
            </D:lockinfo>"#;
        
        // Create headers for lock request
        let mut lock_headers = HeaderMap::new();
        lock_headers.insert("Timeout", "Second-3600".parse().unwrap());
        
        // First user locks the resource
        let lock_response = handle_lock(
            &lock_manager,
            tenant_id,
            "test/path.md",
            lock_headers.clone(),
            Bytes::from(lock_body)
        ).await.unwrap();
        
        // Check response status
        assert_eq!(lock_response.status(), StatusCode::OK);
        
        // Second user tries to lock the same resource
        let lock_result = handle_lock(
            &lock_manager,
            other_tenant_id,
            "test/path.md",
            lock_headers,
            Bytes::from(lock_body)
        ).await;
        
        // Lock should fail
        assert!(lock_result.is_err());
    }
}
