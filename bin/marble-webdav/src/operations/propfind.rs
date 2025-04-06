use crate::error::Error;
use crate::dav_handler::DavResponse;
use bytes::Bytes;
use http::{Response, StatusCode};
use marble_storage::api::TenantStorageRef;
use marble_storage::StorageError;
use tracing::debug;
use uuid::Uuid;

/// Convert a storage path to a WebDAV href
fn path_to_href(path: &str) -> String {
    if path == "." {
        return "/".to_string();
    }
    
    // Ensure the path starts with a slash
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    }
}

/// Handle PROPFIND method to list properties or directory contents
pub async fn handle_propfind(
    tenant_storage: &TenantStorageRef,
    tenant_id: Uuid, 
    path: &str, 
    _body: Bytes
) -> Result<DavResponse, Error> {
    debug!("PROPFIND request for path: {} by tenant: {}", path, tenant_id);
    
    // Check if path exists
    let exists = tenant_storage.exists(&tenant_id, path).await?;
    if !exists {
        return Err(Error::Storage(StorageError::NotFound(path.to_string())));
    }
    
    // Get metadata for the path
    let metadata = tenant_storage.metadata(&tenant_id, path).await?;
    
    // Parse the PROPFIND request to determine depth
    // Assume depth 1 for now (path and immediate children)
    // In a full implementation, we would extract this from headers
    let depth = 1;
    
    // Create XML response for this resource
    let mut xml_content = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <D:multistatus xmlns:D=\"DAV:\">\n\
         <D:response>\n\
         <D:href>{}</D:href>\n\
         <D:propstat>\n\
         <D:prop>\n\
         <D:resourcetype>{}</D:resourcetype>\n\
         <D:getcontentlength>{}</D:getcontentlength>\n\
         <D:getcontenttype>{}</D:getcontenttype>\n\
         <D:getlastmodified>{}</D:getlastmodified>\n\
         </D:prop>\n\
         <D:status>HTTP/1.1 200 OK</D:status>\n\
         </D:propstat>\n\
         </D:response>\n",
        path_to_href(path),
        if metadata.is_directory { "<D:collection/>" } else { "" },
        metadata.size,
        metadata.content_type,
        metadata.last_modified.map_or("".to_string(), |ts| {
            // Convert timestamp to RFC822 format
            // In a real implementation, use a proper date formatting
            format!("{}", ts)
        })
    );
    
    // If it's a directory and depth > 0, add children
    if metadata.is_directory && depth > 0 {
        // List contents of directory
        let entries = tenant_storage.list(&tenant_id, path).await?;
        
        for entry in entries {
            // Get metadata for each child
            let entry_path = if path.ends_with('/') || path == "." {
                if path == "." {
                    entry.clone()
                } else {
                    format!("{}{}", path, entry)
                }
            } else {
                format!("{}/{}", path, entry)
            };
            
            let entry_metadata = match tenant_storage.metadata(&tenant_id, &entry_path).await {
                Ok(m) => m,
                Err(e) => {
                    debug!("Error getting metadata for {}: {}", entry_path, e);
                    continue;
                }
            };
            
            // Add child to XML response
            xml_content.push_str(&format!(
                "<D:response>\n\
                 <D:href>{}</D:href>\n\
                 <D:propstat>\n\
                 <D:prop>\n\
                 <D:resourcetype>{}</D:resourcetype>\n\
                 <D:getcontentlength>{}</D:getcontentlength>\n\
                 <D:getcontenttype>{}</D:getcontenttype>\n\
                 <D:getlastmodified>{}</D:getlastmodified>\n\
                 </D:prop>\n\
                 <D:status>HTTP/1.1 200 OK</D:status>\n\
                 </D:propstat>\n\
                 </D:response>\n",
                path_to_href(&entry_path),
                if entry_metadata.is_directory { "<D:collection/>" } else { "" },
                entry_metadata.size,
                entry_metadata.content_type,
                entry_metadata.last_modified.map_or("".to_string(), |ts| format!("{}", ts))
            ));
        }
    }
    
    // Close the XML document
    xml_content.push_str("</D:multistatus>");
    
    // Build the response
    let response = Response::builder()
        .status(StatusCode::MULTI_STATUS)
        .header(http::header::CONTENT_TYPE, "application/xml")
        .body(Bytes::from(xml_content))
        .map_err(|e| Error::Internal(format!("Failed to build response: {}", e)))?;
    
    Ok(response)
}
