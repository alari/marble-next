use crate::api::LockManagerRef;
use crate::dav_handler::DavResponse;
use crate::error::{Error, LockError};
use crate::headers::OVERWRITE;
use crate::operations::copy::{copy_directory, copy_file, extract_destination};
use http::{HeaderMap, Response, StatusCode};
use marble_storage::api::TenantStorageRef;
use marble_storage::StorageError;
use tracing::debug;
use uuid::Uuid;

/// Handle MOVE method to move or rename a file or directory
pub async fn handle_move(
    tenant_storage: &TenantStorageRef,
    lock_manager: &LockManagerRef,
    tenant_id: Uuid, 
    path: &str, 
    headers: HeaderMap,
    normalize_fn: impl Fn(&str) -> String
) -> Result<DavResponse, Error> {
    debug!("MOVE request for path: {} by tenant: {}", path, tenant_id);
    
    // Check if source exists
    let exists = tenant_storage.exists(&tenant_id, path).await?;
    if !exists {
        return Err(Error::Storage(StorageError::NotFound(path.to_string())));
    }
    
    // Check if source is locked
    if let Some(_) = lock_manager.is_locked(&tenant_id, path).await? {
        return Err(Error::Lock(LockError::ResourceLocked));
    }
    
    // Extract destination from headers
    let destination = extract_destination(&headers, normalize_fn)?;
    debug!("Move destination: {}", destination);
    
    // Check if destination already exists
    let dest_exists = tenant_storage.exists(&tenant_id, &destination).await?;
    
    // Get Overwrite header
    let overwrite = headers
        .get(&*OVERWRITE)
        .and_then(|h| h.to_str().ok())
        .map_or(true, |v| v == "T"); // Default to true if not specified
        
    // If destination exists and overwrite is false, return 412 Precondition Failed
    if dest_exists && !overwrite {
        return Err(Error::WebDav("Destination already exists and overwrite is false".to_string()));
    }
    
    // Check if destination is locked
    if let Some(_) = lock_manager.is_locked(&tenant_id, &destination).await? {
        return Err(Error::Lock(LockError::ResourceLocked));
    }
    
    // Get source metadata to determine if it's a file or directory
    let source_metadata = tenant_storage.metadata(&tenant_id, path).await?;
    let is_directory = source_metadata.is_directory;
    
    // Implement move as copy + delete
    let response = if is_directory {
        // Handle directory move
        let copy_result = copy_directory(tenant_storage, tenant_id, path, &destination, overwrite).await?;
        // Delete the source directory after successful copy
        tenant_storage.delete(&tenant_id, path).await?;
        copy_result
    } else {
        // Handle file move
        let copy_result = copy_file(tenant_storage, tenant_id, path, &destination, overwrite).await?;
        // Delete the source file after successful copy
        tenant_storage.delete(&tenant_id, path).await?;
        copy_result
    };
    
    Ok(response)
}
