use crate::error::Error;
use crate::dav_handler::DavResponse;
use crate::operations::utils::get_parent_path;
use bytes::Bytes;
use http::{Response, StatusCode};
use marble_storage::api::TenantStorageRef;
use tracing::debug;
use uuid::Uuid;

/// Handle MKCOL method to create a directory
pub async fn handle_mkcol(
    tenant_storage: &TenantStorageRef,
    tenant_id: Uuid, 
    path: &str
) -> Result<DavResponse, Error> {
    debug!("MKCOL request for path: {} by tenant: {}", path, tenant_id);
    
    // Check if path already exists
    let exists = tenant_storage.exists(&tenant_id, path).await?;
    if exists {
        // Cannot create collection at an existing path
        return Err(Error::WebDav("Resource already exists".to_string()));
    }
    
    // Check if parent directory exists
    let parent_path = get_parent_path(path);
    if !parent_path.is_empty() && parent_path != "." {
        let parent_exists = tenant_storage.exists(&tenant_id, &parent_path).await?;
        if !parent_exists {
            return Err(Error::WebDav("Parent directory does not exist".to_string()));
        }
        
        // Verify parent is a directory
        let parent_metadata = tenant_storage.metadata(&tenant_id, &parent_path).await?;
        if !parent_metadata.is_directory {
            return Err(Error::WebDav("Parent is not a directory".to_string()));
        }
    }
    
    // Create the directory
    tenant_storage.create_directory(&tenant_id, path).await?;
    
    // Return 201 Created
    let response = Response::builder()
        .status(StatusCode::CREATED)
        .body(Bytes::new())
        .map_err(|e| Error::Internal(format!("Failed to build response: {}", e)))?;
    
    Ok(response)
}
