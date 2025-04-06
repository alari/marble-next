use crate::error::Error;
use crate::dav_handler::DavResponse;
use crate::operations::utils::get_parent_path;
use bytes::Bytes;
use http::{HeaderMap, Response, StatusCode};
use marble_storage::api::TenantStorageRef;
use tracing::debug;
use uuid::Uuid;

/// Handle PUT method to create or update a file
pub async fn handle_put(
    tenant_storage: &TenantStorageRef,
    tenant_id: Uuid, 
    path: &str, 
    headers: HeaderMap, 
    body: Bytes
) -> Result<DavResponse, Error> {
    debug!("PUT request for path: {} by tenant: {}", path, tenant_id);
    
    // Check if the path exists and is a directory
    let exists = tenant_storage.exists(&tenant_id, path).await?;
    if exists {
        let metadata = tenant_storage.metadata(&tenant_id, path).await?;
        if metadata.is_directory {
            return Err(Error::WebDav("Cannot PUT to a directory".to_string()));
        }
    }
    
    // Check if the parent directory exists
    let parent_path = get_parent_path(path);
    if !parent_path.is_empty() && parent_path != "." {
        let parent_exists = tenant_storage.exists(&tenant_id, &parent_path).await?;
        if !parent_exists {
            // Parent directory doesn't exist, try to create it
            tenant_storage.create_directory(&tenant_id, &parent_path).await?;
        }
    }
    
    // Get content type from headers or guess from path
    let content_type = headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            // Use mime_guess to determine content type from path
            mime_guess::from_path(path)
                .first_raw()
                .map(|s| s.to_string())
        });
    
    // Write the file
    tenant_storage.write(
        &tenant_id, 
        path, 
        body.to_vec(), 
        content_type.as_deref()
    ).await?;
    
    // Build response
    let status = if exists { 
        StatusCode::NO_CONTENT  // 204 No Content for updates
    } else { 
        StatusCode::CREATED     // 201 Created for new files
    };
    
    let response = Response::builder()
        .status(status)
        .body(Bytes::new())
        .map_err(|e| Error::Internal(format!("Failed to build response: {}", e)))?;
    
    Ok(response)
}
