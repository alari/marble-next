// marble-storage crate
// Provides storage abstraction for Marble using OpenDAL

// Re-export the primary traits and types
pub use api::{MarbleStorage, MarbleStorageRef};
pub use config::{FileSystemConfig, S3Config, StorageBackend, StorageConfig};
pub use error::{StorageError, StorageResult};

// Public modules
pub mod api;
pub mod config;
pub mod error;
pub mod hash;

// To be implemented in subsequent phases
// mod backends;
// mod impl;

/// Module version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Module name
pub const NAME: &str = env!("CARGO_PKG_NAME");
