use crate::error::Error;
use crate::dav_handler::DavResponse;
use bytes::Bytes;
use http::{Response, StatusCode};
use marble_storage::api::TenantStorageRef;
use marble_storage::StorageError;
use tracing::debug;
use uuid::Uuid;

/// Handle GET method to retrieve a file
pub async fn handle_get(
    tenant_storage: &TenantStorageRef,
    tenant_id: Uuid, 
    path: &str
) -> Result<DavResponse, Error> {
    debug!("GET request for path: {} by tenant: {}", path, tenant_id);
    
    // First, check if the file exists
    if !tenant_storage.exists(&tenant_id, path).await? {
        return Err(Error::Storage(StorageError::NotFound(path.to_string())));
    }
    
    // Retrieve file metadata to get content type and size
    let metadata = tenant_storage.metadata(&tenant_id, path).await?;
    
    // If it's a directory, return a 405 Method Not Allowed
    if metadata.is_directory {
        return Err(Error::WebDav("Cannot GET a directory".to_string()));
    }
    
    // Read the file content
    let content = tenant_storage.read(&tenant_id, path).await?;
    
    // Build the response with appropriate headers
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(http::header::CONTENT_TYPE, metadata.content_type)
        .header(http::header::CONTENT_LENGTH, content.len().to_string())
        .body(Bytes::from(content))
        .map_err(|e| Error::Internal(format!("Failed to build response: {}", e)))?;
    
    Ok(response)
}
