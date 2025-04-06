use axum::{
    Router,
    extract::State,
    http::{HeaderMap, Method, StatusCode, Uri},
    response::IntoResponse,
    routing::any,
};
use bytes::Bytes;
use dav_server::DavMethod;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};

use crate::api::{AuthServiceRef, LockManagerRef};
use crate::dav_handler::MarbleDavHandler;
use crate::headers::DAV;
use marble_storage::api::TenantStorageRef;

// WebDAV server state
pub struct WebDavState {
    dav_handler: Arc<MarbleDavHandler>,
}

// Convert HTTP method to WebDAV method
fn convert_method(method: &Method) -> DavMethod {
    match method.as_str() {
        "GET" => DavMethod::Get,
        "PUT" => DavMethod::Put,
        "PROPFIND" => DavMethod::PropFind,
        "PROPPATCH" => DavMethod::PropPatch,
        "MKCOL" => DavMethod::MkCol,
        "COPY" => DavMethod::Copy,
        "MOVE" => DavMethod::Move,
        "DELETE" => DavMethod::Delete,
        "LOCK" => DavMethod::Lock,
        "UNLOCK" => DavMethod::Unlock,
        "HEAD" => DavMethod::Head,
        "OPTIONS" => DavMethod::Options,
        // Handle any other method as a fallback
        _ => DavMethod::Options, // Fallback to OPTIONS as a safe default
    }
}

// Handle WebDAV requests
async fn handle_webdav(
    State(state): State<Arc<WebDavState>>,
    headers: HeaderMap,
    method: Method,
    uri: Uri,
    body: Bytes,
) -> impl IntoResponse {
    info!("Received {} request for {}", method, uri.path());
    
    // Convert HTTP method to WebDAV method
    let dav_method = convert_method(&method);
    
    // Extract path from URI
    let path = uri.path();
    
    // Call the WebDAV handler
    match state.dav_handler.handle(dav_method, path, headers.clone(), body).await {
        Ok(dav_response) => {
            debug!("Successfully handled WebDAV request");
            
            // Convert DavResponse to axum Response
            let mut axum_response = axum::response::Response::builder()
                .status(dav_response.status());
            
            // Copy headers
            for (name, value) in dav_response.headers() {
                axum_response = axum_response.header(name, value);
            }
            
            // Add standard WebDAV headers if not present
            if !dav_response.headers().contains_key(http::header::SERVER) {
                axum_response = axum_response.header(http::header::SERVER, "Marble WebDAV Server");
            }
            
            if method == Method::OPTIONS && !dav_response.headers().contains_key(&*DAV) {
                axum_response = axum_response.header(&*DAV, "1, 2");
                axum_response = axum_response.header("MS-Author-Via", "DAV");
            }
            
            // Set Allow header for OPTIONS requests if not set
            if method == Method::OPTIONS && !dav_response.headers().contains_key(http::header::ALLOW) {
                axum_response = axum_response.header(
                    http::header::ALLOW, 
                    "OPTIONS, GET, HEAD, PUT, PROPFIND, PROPPATCH, MKCOL, DELETE, COPY, MOVE, LOCK, UNLOCK"
                );
            }
            
            // Build final response with body
            axum_response
                .body(axum::body::Body::from(dav_response.into_body()))
                .unwrap_or_else(|_| {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response").into_response()
                })
        }
        Err(error) => {
            error!("Error handling WebDAV request: {:?}", error);
            
            // Map error to appropriate status code and response
            let (status_code, message) = match &error {
                crate::error::Error::Auth(auth_error) => match auth_error {
                    crate::error::AuthError::MissingCredentials => {
                        let mut response = (StatusCode::UNAUTHORIZED, "Missing credentials").into_response();
                        response.headers_mut().insert(
                            http::header::WWW_AUTHENTICATE,
                            http::HeaderValue::from_static("Basic realm=\"Marble WebDAV\"")
                        );
                        return response;
                    },
                    crate::error::AuthError::InvalidCredentials => {
                        let mut response = (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response();
                        response.headers_mut().insert(
                            http::header::WWW_AUTHENTICATE,
                            http::HeaderValue::from_static("Basic realm=\"Marble WebDAV\"")
                        );
                        return response;
                    },
                    _ => (StatusCode::UNAUTHORIZED, format!("Authentication error: {}", auth_error)),
                },
                crate::error::Error::Storage(storage_error) => match storage_error {
                    marble_storage::StorageError::NotFound(_) => {
                        (StatusCode::NOT_FOUND, format!("Resource not found: {}", storage_error))
                    },
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Storage error: {}", storage_error)),
                },
                crate::error::Error::Lock(lock_error) => match lock_error {
                    crate::error::LockError::ResourceLocked => {
                        (StatusCode::LOCKED, "Resource is locked".to_string())
                    },
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Lock error: {}", lock_error)),
                },
                crate::error::Error::WebDav(msg) => {
                    if msg.contains("already exists") {
                        (StatusCode::METHOD_NOT_ALLOWED, msg.clone())
                    } else if msg.contains("Parent directory does not exist") {
                        (StatusCode::CONFLICT, msg.clone())
                    } else if msg.contains("Cannot PUT to a directory") || msg.contains("Cannot GET a directory") {
                        (StatusCode::METHOD_NOT_ALLOWED, msg.clone())
                    } else {
                        (StatusCode::BAD_REQUEST, msg.clone())
                    }
                },
                _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", error)),
            };
            
            (status_code, message).into_response()
        }
    }
}

// Create a WebDAV server with Axum
pub fn create_webdav_server(
    tenant_storage: TenantStorageRef,
    auth_service: AuthServiceRef,
    lock_manager: LockManagerRef,
) -> Router {
    // Create the WebDAV handler
    let dav_handler = Arc::new(MarbleDavHandler::new(
        tenant_storage,
        auth_service,
        lock_manager,
    ));
    
    // Create WebDAV state
    let state = Arc::new(WebDavState {
        dav_handler,
    });
    
    // Create Axum router with Axum 0.8.x syntax
    Router::new()
        .route("/*path", any(handle_webdav))
        .route("/", any(handle_webdav))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
