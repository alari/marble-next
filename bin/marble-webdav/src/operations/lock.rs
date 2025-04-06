use crate::api::LockManagerRef;
use crate::error::Error;
use crate::dav_handler::DavResponse;
use crate::operations::utils::{parse_depth, Depth};

use bytes::Bytes;
use http::{HeaderMap, Response, StatusCode};
use tracing::{debug, warn};
use uuid::Uuid;
use std::time::Duration;
use http::header;

/// Handle LOCK WebDAV method
pub async fn handle_lock(
    lock_manager: &LockManagerRef,
    tenant_id: Uuid,
    path: &str,
    headers: HeaderMap,
    body: Bytes,
) -> Result<DavResponse, Error> {
    debug!("LOCK request for: {}", path);
    
    // Parse timeout header if present
    let timeout = parse_timeout_header(&headers)
        .unwrap_or_else(|| Duration::from_secs(3600)); // Default to 1 hour
    
    // Parse depth header
    let depth = parse_depth(&headers).unwrap_or(Depth::Zero);
    
    // Parse XML body to extract lock information
    let (lock_scope, lock_type, owner) = parse_lock_body(&body)?;
    
    // Generate a unique lock token
    let token = format!("urn:uuid:{}", Uuid::new_v4());
    
    // Acquire the lock
    lock_manager.lock(
        &tenant_id,
        path,
        timeout,
        &token
    ).await.map_err(|e| Error::LockFailed(e.to_string()))?;
    
    // Recursive locking not supported yet
    if depth == Depth::Infinity {
        warn!("Recursive locking (Depth: infinity) requested but not fully implemented");
        // In a complete implementation, we would lock all descendants here
    }
    
    // Generate the lock token response header
    let lock_token_header = format!("<{}>", token);
    
    // Create XML response for lockdiscovery
    let lock_discovery = generate_lock_discovery_xml(
        &token,
        &lock_scope,
        &lock_type,
        owner.as_deref(),
        timeout,
        path,
    );
    
    // Build response with proper headers - Response builder approach
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml")
        .header("Lock-Token", lock_token_header)
        .body(Bytes::from(lock_discovery.as_bytes().to_vec()))
        .map_err(|e| Error::Internal(format!("Failed to build response: {}", e)))?;
    
    Ok(response)
}

/// Parse timeout header value into a Duration
/// Format: "Second-xxx" or "Infinite"
fn parse_timeout_header(headers: &HeaderMap) -> Option<Duration> {
    headers.get("Timeout")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            let parts: Vec<&str> = s.split(',').map(|s| s.trim()).collect();
            for part in parts {
                if part.eq_ignore_ascii_case("infinite") {
                    // For infinite, use a very long timeout (1 week)
                    return Some(Duration::from_secs(7 * 24 * 60 * 60));
                } else if part.to_lowercase().starts_with("second-") {
                    if let Ok(secs) = part[7..].parse::<u64>() {
                        return Some(Duration::from_secs(secs));
                    }
                }
            }
            None
        })
}

/// Parse LOCK request XML body to extract lock scope, type, and owner information
fn parse_lock_body(body: &Bytes) -> Result<(String, String, Option<String>), Error> {
    if body.is_empty() {
        // If body is empty, use default values
        return Ok(("exclusive".to_string(), "write".to_string(), None));
    }
    
    // Parse XML with quick-xml
    let xml_str = std::str::from_utf8(body)
        .map_err(|_| Error::WebDav("Invalid XML encoding".to_string()))?;
    
    // This is a simplified parsing approach; in a real implementation you'd use
    // a proper XML parser to extract these values from the lockinfo XML
    
    // Extract lock scope (exclusive or shared)
    let lock_scope = if xml_str.contains("<exclusive") {
        "exclusive".to_string()
    } else if xml_str.contains("<shared") {
        "shared".to_string()
    } else {
        "exclusive".to_string() // Default to exclusive
    };
    
    // Extract lock type (write)
    let lock_type = if xml_str.contains("<write") {
        "write".to_string()
    } else {
        "write".to_string() // Default to write
    };
    
    // Extract owner information (simplified approach)
    let owner = if xml_str.contains("<owner>") {
        // This is a placeholder for owner extraction
        // In a real implementation, you'd properly parse the XML
        Some("unknown".to_string())
    } else {
        None
    };
    
    Ok((lock_scope, lock_type, owner))
}

/// Generate lock discovery XML
fn generate_lock_discovery_xml(
    token: &str,
    lock_scope: &str,
    lock_type: &str,
    owner: Option<&str>,
    timeout: Duration,
    path: &str,
) -> String {
    // Calculate timeout string
    let timeout_str = format!("Second-{}", timeout.as_secs());
    
    // Build XML
    let mut xml = format!(
        r#"<?xml version="1.0" encoding="utf-8" ?>
<D:prop xmlns:D="DAV:">
    <D:lockdiscovery>
        <D:activelock>
            <D:lockscope><D:{}/></D:lockscope>
            <D:locktype><D:{}/></D:locktype>
            <D:depth>0</D:depth>
            <D:timeout>{}</D:timeout>
            <D:locktoken>
                <D:href>{}</D:href>
            </D:locktoken>
            <D:lockroot>
                <D:href>{}</D:href>
            </D:lockroot>"#,
        lock_scope, lock_type, timeout_str, token, path
    );
    
    // Add owner if present
    if let Some(owner_str) = owner {
        xml.push_str(&format!(
            r#"
            <D:owner>
                <D:href>{}</D:href>
            </D:owner>"#,
            owner_str
        ));
    }
    
    // Close tags
    xml.push_str(r#"
        </D:activelock>
    </D:lockdiscovery>
</D:prop>"#);
    
    xml
}