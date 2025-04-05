use std::sync::Arc;
use marble_db::auth::DatabaseAuthService as DbAuthService;
use marble_webdav::auth::WebDavAuthService;
use marble_webdav::lock::InMemoryLockManager;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Starting Marble WebDAV Server");
    
    // Configure the server
    let server_addr = "127.0.0.1:4000"; // Should come from config
    
    // For now, just show that we have a skeleton implementation
    // Actual implementation will connect to the database and storage
    info!("Marble WebDAV Server - Skeleton Implementation");
    info!("This is a placeholder. The server will be implemented in future steps.");
    info!("Server would listen on: {}", server_addr);
    
    // The skeleton implementation is complete, but we're not running the server yet
    // In future steps, we'll:
    // 1. Connect to database using marble_db::create_pool
    // 2. Create a TenantStorage instance
    // 3. Create auth and lock services using the marble-db API:
    //    let db_pool = Arc::new(marble_db::create_pool(config).await?);
    //    let db_auth_service = DbAuthService::from_pool(db_pool.clone());
    //    let db_auth_service_arc = Arc::new(db_auth_service);
    //    let webdav_auth_service = WebDavAuthService::new(db_auth_service_arc);
    // 4. Create and run the WebDAV server
    
    info!("Marble WebDAV Server - Shutting down");
    Ok(())
}
