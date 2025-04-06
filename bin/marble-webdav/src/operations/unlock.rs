use crate::api::LockManagerRef;
use crate::error::Error;
use crate::dav_handler::DavResponse;
use crate::operations::utils::create_response;

use bytes::Bytes;
use http::{HeaderMap, StatusCode};
use tracing::debug;
use uuid::Uuid;

/// Handle UNLOCK WebDAV method
pub async fn handle_unlock(
    lock_manager: &LockManagerRef,
    tenant_id: Uuid,
    path: &str,
    headers: HeaderMap,
) -> Result<DavResponse, Error> {
    debug!("UNLOCK request for: {}", path);
    
    // Extract lock token from header
    let lock_token = headers
        .get("Lock-Token")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            // Lock token format is typically "<urn:uuid:...>"
            let s = s.trim();
            if s.starts_with('<') && s.ends_with('>') {
                Some(s[1..s.len()-1].to_string())
            } else {
                Some(s.to_string())
            }
        })
        .ok_or_else(|| Error::WebDav("Missing or invalid Lock-Token header".to_string()))?;
    
    // Release the lock
    lock_manager.unlock(
        &tenant_id,
        path,
        &lock_token
    ).await.map_err(|e| Error::UnlockFailed(e.to_string()))?;
    
    // Return success response
    Ok(create_response(StatusCode::NO_CONTENT, Bytes::new()))
}