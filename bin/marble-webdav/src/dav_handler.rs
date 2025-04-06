use crate::api::{AuthServiceRef, LockManagerRef};
use crate::auth::extract_basic_auth;
use crate::error::{AuthError, Error};
use crate::operations;
use bytes::Bytes;
use dav_server::DavMethod;
use http::{HeaderMap, Response, StatusCode};
use marble_storage::api::TenantStorageRef;
use tracing::{info, warn};
use uuid::Uuid;
use std::sync::Arc;

/// Type alias for WebDAV response
pub type DavResponse = Response<Bytes>;

// Tests module
#[cfg(test)]
mod tests {
    // This is a placeholder for the main dav_handler tests
    // All test implementations have been moved to the dedicated tests directory
    // See the tests/ directory for implementation details
}

/// Marble WebDAV handler integrating with TenantStorage
pub struct MarbleDavHandler {
    /// Storage for tenant operations
    tenant_storage: TenantStorageRef,

    /// Authentication service
    auth_service: AuthServiceRef,

    /// Lock manager for WebDAV locks
    lock_manager: LockManagerRef,
}

impl MarbleDavHandler {
    /// Create a new WebDAV handler
    pub fn new(
        tenant_storage: TenantStorageRef,
        auth_service: AuthServiceRef,
        lock_manager: LockManagerRef,
    ) -> Self {
        Self {
            tenant_storage,
            auth_service,
            lock_manager,
        }
    }
    
    // Helper methods for tests
    #[cfg(test)]
    pub(crate) async fn handle_get(&self, tenant_id: Uuid, path: &str) -> Result<DavResponse, Error> {
        operations::handle_get(&self.tenant_storage, tenant_id, path).await
    }
    
    #[cfg(test)]
    pub(crate) async fn handle_put(
        &self,
        tenant_id: Uuid,
        path: &str,
        headers: HeaderMap,
        body: Bytes,
    ) -> Result<DavResponse, Error> {
        operations::handle_put(&self.tenant_storage, tenant_id, path, headers, body).await
    }
    
    #[cfg(test)]
    pub(crate) async fn handle_propfind(
        &self,
        tenant_id: Uuid,
        path: &str,
        body: Bytes,
    ) -> Result<DavResponse, Error> {
        operations::handle_propfind(&self.tenant_storage, tenant_id, path, body).await
    }
    
    #[cfg(test)]
    pub(crate) async fn handle_mkcol(&self, tenant_id: Uuid, path: &str) -> Result<DavResponse, Error> {
        operations::handle_mkcol(&self.tenant_storage, tenant_id, path).await
    }
    
    #[cfg(test)]
    pub(crate) async fn handle_delete(&self, tenant_id: Uuid, path: &str) -> Result<DavResponse, Error> {
        operations::handle_delete(&self.tenant_storage, &self.lock_manager, tenant_id, path).await
    }
    
    #[cfg(test)]
    pub(crate) async fn handle_copy(&self, tenant_id: Uuid, path: &str, headers: HeaderMap) -> Result<DavResponse, Error> {
        operations::handle_copy(
            &self.tenant_storage, 
            tenant_id, 
            path, 
            headers,
            |p| self.normalize_path(p)
        ).await
    }
    
    #[cfg(test)]
    pub(crate) async fn handle_move(&self, tenant_id: Uuid, path: &str, headers: HeaderMap) -> Result<DavResponse, Error> {
        operations::handle_move(
            &self.tenant_storage,
            &self.lock_manager,
            tenant_id,
            path,
            headers,
            |p| self.normalize_path(p)
        ).await
    }
    
    #[cfg(test)]
    pub(crate) async fn handle_lock(&self, tenant_id: Uuid, path: &str, headers: HeaderMap, body: Bytes) -> Result<DavResponse, Error> {
        operations::handle_lock(
            &self.lock_manager,
            tenant_id,
            path,
            headers,
            body
        ).await
    }
    
    #[cfg(test)]
    pub(crate) async fn handle_unlock(&self, tenant_id: Uuid, path: &str, headers: HeaderMap) -> Result<DavResponse, Error> {
        operations::handle_unlock(
            &self.lock_manager,
            tenant_id,
            path,
            headers
        ).await
    }

    /// Authenticate a request and return the tenant ID
    async fn authenticate(&self, headers: &HeaderMap) -> Result<Uuid, Error> {
        // Extract Authorization header
        let auth_header = headers
            .get(http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok());

        // If missing, return error
        let auth_header = auth_header.ok_or(Error::Auth(AuthError::MissingCredentials))?;

        // Extract credentials
        let (username, password) = extract_basic_auth(Some(auth_header))
            .ok_or(Error::Auth(AuthError::MissingCredentials))?;

        // Authenticate with auth service
        let tenant_id = self
            .auth_service
            .authenticate(&username, &password)
            .await
            .map_err(Error::Auth)?;

        Ok(tenant_id)
    }

    /// Normalize a WebDAV path to a storage path
    fn normalize_path(&self, path: &str) -> String {
        // Remove leading slash if present
        let path = path.trim_start_matches('/');
        
        // Handle empty path as root
        if path.is_empty() {
            return ".".to_string();
        }
        
        // Replace percent-encoded characters
        // Note: This is a simplification, a real implementation would use proper URL decoding
        let path = path.replace("%20", " ");
        
        path
    }
    
    /// Helper to create a basic response
    fn create_response(&self, status: StatusCode, body: impl Into<Bytes>) -> DavResponse {
        Response::builder()
            .status(status)
            .body(body.into())
            .unwrap()
    }
    
    /// Dispatch WebDAV method to appropriate handler
    pub async fn handle(
        &self,
        method: DavMethod,
        path: &str,
        headers: HeaderMap,
        body: Bytes,
    ) -> Result<DavResponse, Error> {
        info!("Handling {:?} request for path: {}", method, path);
        
        // Extract credentials and get tenant ID
        let tenant_id = self.authenticate(&headers).await?;
        
        // Normalize path
        let normalized_path = self.normalize_path(path);
        
        // Handle method based on tenant ID and normalized path
        match method {
            // Basic file operations
            DavMethod::Get => operations::handle_get(&self.tenant_storage, tenant_id, &normalized_path).await,
            
            DavMethod::Put => operations::handle_put(
                &self.tenant_storage, 
                tenant_id, 
                &normalized_path, 
                headers, 
                body
            ).await,
            
            DavMethod::PropFind => operations::handle_propfind(
                &self.tenant_storage, 
                tenant_id, 
                &normalized_path, 
                body
            ).await,
            
            DavMethod::MkCol => operations::handle_mkcol(
                &self.tenant_storage, 
                tenant_id, 
                &normalized_path
            ).await,
            
            DavMethod::Delete => operations::handle_delete(
                &self.tenant_storage,
                &self.lock_manager,
                tenant_id, 
                &normalized_path
            ).await,
            
            // Advanced operations (implemented)
            DavMethod::Copy => operations::handle_copy(
                &self.tenant_storage,
                tenant_id,
                &normalized_path,
                headers,
                |p| self.normalize_path(p)
            ).await,
            
            DavMethod::Move => operations::handle_move(
                &self.tenant_storage,
                &self.lock_manager,
                tenant_id,
                &normalized_path,
                headers,
                |p| self.normalize_path(p)
            ).await,
            
            // Lock operations
            DavMethod::Lock => operations::handle_lock(
                &self.lock_manager,
                tenant_id,
                &normalized_path,
                headers,
                body
            ).await,
            
            DavMethod::Unlock => operations::handle_unlock(
                &self.lock_manager,
                tenant_id,
                &normalized_path,
                headers
            ).await,
            
            // Other methods will be implemented later
            _ => {
                warn!("Unimplemented method: {:?}", method);
                Err(Error::WebDav(format!("Method not implemented: {:?}", method)))
            }
        }
    }
}
