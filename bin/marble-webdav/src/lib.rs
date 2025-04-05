// Public API
pub mod api;

// Implementation modules
pub mod auth;  // Make public
mod dav_handler;
pub mod error;  // Make public
pub mod lock;  // Make public
mod server;

// Re-export public API
pub use api::*;
pub use error::Error;
pub use server::create_webdav_server;
