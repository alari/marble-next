use crate::dav_handler::DavResponse;
use crate::error::Error;
use crate::headers::{DESTINATION, OVERWRITE};
use crate::operations::utils::get_parent_path;
use bytes::Bytes;
use http::{HeaderMap, Response, StatusCode};
use marble_storage::api::TenantStorageRef;
use marble_storage::StorageError;
use tracing::debug;
use uuid::Uuid;

/// Extract destination path from headers
pub fn extract_destination(headers: &HeaderMap, normalize_fn: impl Fn(&str) -> String) -> Result<String, Error> {
    // Extract the Destination header
    let destination = headers
        .get(&*DESTINATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| Error::WebDav("Destination header missing".to_string()))?;
        
    // Parse the URI to extract the path
    let destination_uri = destination
        .parse::<http::Uri>()
        .map_err(|e| Error::WebDav(format!("Invalid destination URI: {}", e)))?;
        
    // Get the path component
    let path = destination_uri.path();
    
    // Normalize the path
    Ok(normalize_fn(path))
}

/// Copy a file from source to destination
pub async fn copy_file(
    tenant_storage: &TenantStorageRef,
    tenant_id: Uuid,
    source: &str, 
    destination: &str, 
    overwrite: bool
) -> Result<DavResponse, Error> {
    // Read source content
    let content = tenant_storage.read(&tenant_id, source).await?;
    
    // Get source metadata for content type
    let metadata = tenant_storage.metadata(&tenant_id, source).await?;
    let content_type = Some(metadata.content_type.as_str());
    
    // Check if destination exists and delete if overwrite is true
    let dest_exists = tenant_storage.exists(&tenant_id, destination).await?;
    if dest_exists && overwrite {
        tenant_storage.delete(&tenant_id, destination).await?;
    }
    
    // Create parent directory if needed
    let parent = get_parent_path(destination);
    if !parent.is_empty() && parent != "." {
        let parent_exists = tenant_storage.exists(&tenant_id, &parent).await?;
        if !parent_exists {
            tenant_storage.create_directory(&tenant_id, &parent).await?;
        }
    }
    
    // Write content to destination
    tenant_storage.write(&tenant_id, destination, content, content_type).await?;
    
    // Return appropriate status code
    let status = if dest_exists {
        StatusCode::NO_CONTENT // 204 if destination was overwritten
    } else {
        StatusCode::CREATED // 201 if destination was created
    };
    
    let response = Response::builder()
        .status(status)
        .body(Bytes::new())
        .map_err(|e| Error::Internal(format!("Failed to build response: {}", e)))?;
        
    Ok(response)
}

/// Copy a directory recursively from source to destination
pub async fn copy_directory(
    tenant_storage: &TenantStorageRef,
    tenant_id: Uuid,
    source: &str, 
    destination: &str, 
    overwrite: bool
) -> Result<DavResponse, Error> {
    // Check if destination exists
    let dest_exists = tenant_storage.exists(&tenant_id, destination).await?;
    
    // If destination exists but is not a directory, handle overwrite
    if dest_exists {
        let dest_metadata = tenant_storage.metadata(&tenant_id, destination).await?;
        if !dest_metadata.is_directory {
            if overwrite {
                // Delete the file to replace with directory
                tenant_storage.delete(&tenant_id, destination).await?;
            } else {
                return Err(Error::WebDav("Destination exists but is not a directory".to_string()));
            }
        }
    }
    
    // Create destination directory
    tenant_storage.create_directory(&tenant_id, destination).await?;
    
    // List contents of source directory
    let entries = tenant_storage.list(&tenant_id, source).await?;
    
    // Copy each item in the directory
    for entry in entries {
        let source_path = if source == "." {
            entry.clone()
        } else {
            format!("{}/{}", source, entry)
        };
        
        let dest_path = if destination == "." {
            entry.clone()
        } else {
            format!("{}/{}", destination, entry)
        };
        
        // Get metadata to determine if it's a file or directory
        let entry_metadata = tenant_storage.metadata(&tenant_id, &source_path).await?;
        
        if entry_metadata.is_directory {
            // Recursively copy the directory - use Box::pin to avoid infinite recursion
            Box::pin(copy_directory(tenant_storage, tenant_id, &source_path, &dest_path, overwrite)).await?;
        } else {
            // Copy the file
            copy_file(tenant_storage, tenant_id, &source_path, &dest_path, overwrite).await?;
        }
    }
    
    // Return appropriate status code
    let status = if dest_exists {
        StatusCode::NO_CONTENT // 204 if destination was overwritten
    } else {
        StatusCode::CREATED // 201 if destination was created
    };
    
    let response = Response::builder()
        .status(status)
        .body(Bytes::new())
        .map_err(|e| Error::Internal(format!("Failed to build response: {}", e)))?;
        
    Ok(response)
}

/// Handle COPY method to copy a file or directory
pub async fn handle_copy(
    tenant_storage: &TenantStorageRef,
    tenant_id: Uuid, 
    path: &str, 
    headers: HeaderMap,
    normalize_fn: impl Fn(&str) -> String
) -> Result<DavResponse, Error> {
    debug!("COPY request for path: {} by tenant: {}", path, tenant_id);
    
    // Check if source exists
    let exists = tenant_storage.exists(&tenant_id, path).await?;
    if !exists {
        return Err(Error::Storage(StorageError::NotFound(path.to_string())));
    }
    
    // Extract destination from headers
    let destination = extract_destination(&headers, normalize_fn)?;
    debug!("Copy destination: {}", destination);
    
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
    
    // Check if source is a directory
    let source_metadata = tenant_storage.metadata(&tenant_id, path).await?;
    let is_directory = source_metadata.is_directory;
    
    if is_directory {
        // Handle directory copy
        copy_directory(tenant_storage, tenant_id, path, &destination, overwrite).await
    } else {
        // Handle file copy
        copy_file(tenant_storage, tenant_id, path, &destination, overwrite).await
    }
}
