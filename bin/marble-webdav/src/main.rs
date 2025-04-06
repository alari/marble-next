use std::sync::Arc;
use marble_db::auth::DatabaseAuthService as DbAuthService;
use marble_webdav::auth::WebDavAuthService;
use marble_webdav::lock::InMemoryLockManager;
use marble_webdav::create_webdav_server;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use std::net::SocketAddr;
use dotenv::dotenv;
use marble_storage::api::TenantStorageRef;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();
    
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Starting Marble WebDAV Server");
    
    // Configure the server
    let server_addr = std::env::var("WEBDAV_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:4000".to_string());
    
    // Parse the address
    let addr: SocketAddr = server_addr.parse()?;
    
    // Connect to database
    let db_config = marble_db::config::DatabaseConfig::from_env();
    let db_pool = Arc::new(marble_db::create_pool(db_config).await?);
    
    // Initialize auth service
    let db_auth_service = Arc::new(DbAuthService::from_pool(db_pool.clone()));
    let auth_service = Arc::new(WebDavAuthService::new(db_auth_service));
    
    // Initialize lock manager
    let lock_manager = Arc::new(InMemoryLockManager::new());
    
    // Initialize tenant storage with a simple mock implementation
    info!("Initializing mock tenant storage");
    let tenant_storage: TenantStorageRef = Arc::new(marble_storage::MockTenantStorage::new());
    
    // Create WebDAV server
    let app = create_webdav_server(
        tenant_storage,
        auth_service,
        lock_manager
    );
    
    // Start the server
    info!("WebDAV server listening on {}", addr);
    
    // Build the router with the WebDAV app
    let router = app.into_make_service();
    
    // Start the server (using TcpListener directly since axum 0.8.3 doesn't have Server::bind)
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    
    info!("Marble WebDAV Server - Shutting down");
    Ok(())
}
