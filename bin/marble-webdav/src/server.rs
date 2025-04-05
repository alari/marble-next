use axum::{
    Router,
    extract::{Path, State},
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
use marble_storage::api::TenantStorageRef;

// WebDAV server state
pub struct WebDavState {
    dav_handler: Arc<MarbleDavHandler>,
}

// Convert HTTP method to WebDAV method
fn convert_method(method: Method) -> DavMethod {
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
    let dav_method = convert_method(method);
    
    // Extract path from URI
    let path = uri.path();
    
    // Call the WebDAV handler
    match state.dav_handler.handle(dav_method, path, headers, body).await {
        Ok(response) => {
            // This will need to be fleshed out to properly convert DavResponse to axum Response
            // For now, we return a success placeholder
            debug!("Successfully handled WebDAV request");
            (StatusCode::OK, "WebDAV request handled (placeholder)").into_response()
        }
        Err(error) => {
            error!("Error handling WebDAV request: {:?}", error);
            
            // Map error to appropriate status code
            // This is a simplified version and will need to be expanded
            let status_code = match error {
                crate::error::Error::Auth(_) => StatusCode::UNAUTHORIZED,
                crate::error::Error::Storage(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            (status_code, format!("Error: {}", error)).into_response()
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
