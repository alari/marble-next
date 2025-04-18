// marble-storage crate
// Provides storage abstraction for Marble using OpenDAL

// Re-export the primary traits and types
pub use api::{MarbleStorage, MarbleStorageRef};
pub use api::tenant::{TenantStorage, TenantStorageRef, FileMetadata};
pub use config::{FileSystemConfig, S3Config, StorageBackend, StorageConfig};
pub use error::{StorageError, StorageResult};
pub use mock::MockTenantStorage;
pub use services::hasher::ContentHasher;

// Public modules
pub mod api;
pub mod config;
pub mod error;
pub mod hash;
pub mod mock;

// Internal modules
mod backends;
mod r#impl;
mod services;
#[cfg(test)]
mod tests;

/// Module version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Module name
pub const NAME: &str = env!("CARGO_PKG_NAME");
