use bytes::Bytes;
use http::{HeaderMap, Response, StatusCode};

/// Depth value for WebDAV operations
#[derive(Debug, PartialEq, Eq)]
pub enum Depth {
    /// Current resource only
    Zero,
    /// Current resource and its children
    One,
    /// Current resource and all descendants
    Infinity,
}

/// Parse Depth header
pub fn parse_depth(headers: &HeaderMap) -> Option<Depth> {
    headers.get("Depth")
        .and_then(|v| v.to_str().ok())
        .map(|s| {
            match s {
                "0" => Depth::Zero,
                "1" => Depth::One,
                "infinity" => Depth::Infinity,
                _ => Depth::Infinity, // Default to infinity per WebDAV spec
            }
        })
}

/// Create a simple response with status code and body
pub fn create_response(status: StatusCode, body: impl Into<Bytes>) -> Response<Bytes> {
    Response::builder()
        .status(status)
        .body(body.into())
        .unwrap()
}

/// Get the parent path of a given path
pub fn get_parent_path(path: &str) -> String {
    let path = path.trim_end_matches('/');
    
    if path.is_empty() || path == "." {
        return String::new();
    }
    
    match path.rfind('/') {
        Some(idx) => {
            let parent = &path[..idx];
            if parent.is_empty() {
                ".".to_string()
            } else {
                parent.to_string()
            }
        }
        None => ".".to_string()
    }
}