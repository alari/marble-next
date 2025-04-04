use std::path::PathBuf;

use crate::error::{StorageError, StorageResult};

/// Configuration for S3 storage backend
#[derive(Clone, Debug)]
pub struct S3Config {
    /// AWS region for S3
    pub region: String,
    
    /// S3 bucket name
    pub bucket: String,
    
    /// Endpoint for S3-compatible storage (optional, for services like MinIO)
    pub endpoint: Option<String>,
    
    /// Path prefix for storage within the bucket
    pub prefix: Option<String>,
    
    /// Access key (if not using instance role/environment credentials)
    pub access_key: Option<String>,
    
    /// Secret key (if not using instance role/environment credentials)
    pub secret_key: Option<String>,
}

/// Configuration for local filesystem storage backend (used for development/testing)
#[derive(Clone, Debug)]
pub struct FileSystemConfig {
    /// Base directory for hash-based storage
    pub hash_base_path: PathBuf,
}

/// Storage backend type
#[derive(Clone, Debug)]
pub enum StorageBackend {
    /// S3 storage backend
    S3(S3Config),
    
    /// Local filesystem storage backend (development/testing)
    FileSystem(FileSystemConfig),
}

/// Configuration for all storage aspects
#[derive(Clone, Debug)]
pub struct StorageConfig {
    /// Storage backend configuration
    pub backend: StorageBackend,
}

impl StorageConfig {
    /// Create a new configuration for S3 storage
    pub fn new_s3(
        region: String,
        bucket: String,
        endpoint: Option<String>,
        prefix: Option<String>,
        access_key: Option<String>,
        secret_key: Option<String>,
    ) -> Self {
        Self {
            backend: StorageBackend::S3(S3Config {
                region,
                bucket,
                endpoint,
                prefix,
                access_key,
                secret_key,
            }),
        }
    }

    /// Create a new configuration for filesystem storage (development/testing)
    pub fn new_fs(hash_base_path: PathBuf) -> Self {
        Self {
            backend: StorageBackend::FileSystem(FileSystemConfig { hash_base_path }),
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> StorageResult<()> {
        match &self.backend {
            StorageBackend::S3(config) => {
                if config.bucket.is_empty() {
                    return Err(StorageError::Configuration(
                        "S3 bucket name cannot be empty".to_string(),
                    ));
                }
                if config.region.is_empty() {
                    return Err(StorageError::Configuration(
                        "S3 region cannot be empty".to_string(),
                    ));
                }
                Ok(())
            }
            StorageBackend::FileSystem(config) => {
                // Check if base path exists and is a directory
                if !config.hash_base_path.exists() {
                    return Err(StorageError::Configuration(format!(
                        "Hash base path does not exist: {}",
                        config.hash_base_path.display()
                    )));
                }
                if !config.hash_base_path.is_dir() {
                    return Err(StorageError::Configuration(format!(
                        "Hash base path is not a directory: {}",
                        config.hash_base_path.display()
                    )));
                }
                Ok(())
            }
        }
    }
}
