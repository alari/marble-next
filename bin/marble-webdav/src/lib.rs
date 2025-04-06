// Public API
pub mod api;

// Implementation modules
pub mod auth;
mod dav_handler;
pub mod error;
pub mod headers;
pub mod lock;
mod operations;
mod server;

// Test modules (only compiled in test mode)
#[cfg(test)]
pub mod tests;

// Re-export public API
pub use api::*;
pub use error::Error;
pub use server::create_webdav_server;

// Type re-export
pub use dav_handler::DavResponse;
