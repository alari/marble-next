use crate::api::LockManagerRef;
use crate::error::{Error, LockError};
use crate::dav_handler::DavResponse;
use bytes::Bytes;
use http::{Response, StatusCode};
use marble_storage::api::TenantStorageRef;
use marble_storage::StorageError;
use tracing::debug;
use uuid::Uuid;

/// Handle DELETE method to remove a file or directory
pub async fn handle_delete(
    tenant_storage: &TenantStorageRef,
    lock_manager: &LockManagerRef,
    tenant_id: Uuid, 
    path: &str
) -> Result<DavResponse, Error> {
    debug!("DELETE request for path: {} by tenant: {}", path, tenant_id);
    
    // Check if path exists
    let exists = tenant_storage.exists(&tenant_id, path).await?;
    if !exists {
        return Err(Error::Storage(StorageError::NotFound(path.to_string())));
    }
    
    // Check if it's locked
    if let Some(_) = lock_manager.is_locked(&tenant_id, path).await? {
        // In a full implementation, we would check the lock token from If header
        // For simplicity, we're just checking if it's locked at all
        return Err(Error::Lock(LockError::ResourceLocked));
    }
    
    // Delete the resource
    tenant_storage.delete(&tenant_id, path).await?;
    
    // Return 204 No Content on success
    let response = Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Bytes::new())
        .map_err(|e| Error::Internal(format!("Failed to build response: {}", e)))?;
    
    Ok(response)
}
