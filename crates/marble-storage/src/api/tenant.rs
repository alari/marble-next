use std::sync::Arc;
use uuid::Uuid;
use async_trait::async_trait;

use crate::error::StorageResult;

/// TenantStorage provides tenant-isolated storage operations.
///
/// This trait is designed to provide a clean, focused interface for tenant-isolated
/// file storage operations. It makes tenant isolation explicit in the API through
/// tenant_id parameters, ensuring that all operations are properly scoped to a
/// specific tenant.
#[async_trait]
pub trait TenantStorage: Send + Sync + 'static {
    /// Read a file by path for a specific tenant
    ///
    /// # Arguments
    /// * `tenant_id` - The UUID of the tenant
    /// * `path` - The path to the file, relative to the tenant's root
    ///
    /// # Returns
    /// * The file contents as a byte vector
    async fn read(&self, tenant_id: &Uuid, path: &str) -> StorageResult<Vec<u8>>;
    
    /// Create a directory for a specific tenant
    ///
    /// # Arguments
    /// * `tenant_id` - The UUID of the tenant
    /// * `path` - The path to the directory, relative to the tenant's root
    ///
    /// # Returns
    /// * Ok(()) if the directory was created successfully or already exists
    async fn create_directory(&self, tenant_id: &Uuid, path: &str) -> StorageResult<()>;
    
    /// Write a file at path for a specific tenant
    ///
    /// # Arguments
    /// * `tenant_id` - The UUID of the tenant
    /// * `path` - The path to the file, relative to the tenant's root
    /// * `content` - The file contents as a byte vector
    /// * `content_type` - Optional MIME type of the content
    ///
    /// # Returns
    /// * Ok(()) if the write was successful
    async fn write(&self, tenant_id: &Uuid, path: &str, content: Vec<u8>, content_type: Option<&str>) -> StorageResult<()>;
    
    /// Check if a file exists for a tenant
    ///
    /// # Arguments
    /// * `tenant_id` - The UUID of the tenant
    /// * `path` - The path to the file, relative to the tenant's root
    ///
    /// # Returns
    /// * true if the file exists, false otherwise
    async fn exists(&self, tenant_id: &Uuid, path: &str) -> StorageResult<bool>;
    
    /// Delete a file for a tenant
    ///
    /// # Arguments
    /// * `tenant_id` - The UUID of the tenant
    /// * `path` - The path to the file, relative to the tenant's root
    ///
    /// # Returns
    /// * Ok(()) if the delete was successful
    async fn delete(&self, tenant_id: &Uuid, path: &str) -> StorageResult<()>;
    
    /// List files for a tenant in a directory
    ///
    /// # Arguments
    /// * `tenant_id` - The UUID of the tenant
    /// * `dir_path` - The directory path, relative to the tenant's root
    ///
    /// # Returns
    /// * A list of file paths in the directory
    async fn list(&self, tenant_id: &Uuid, dir_path: &str) -> StorageResult<Vec<String>>;
    
    /// Get metadata for a file for a tenant
    ///
    /// # Arguments
    /// * `tenant_id` - The UUID of the tenant
    /// * `path` - The path to the file, relative to the tenant's root
    ///
    /// # Returns
    /// * File metadata including size, content type, etc.
    async fn metadata(&self, tenant_id: &Uuid, path: &str) -> StorageResult<FileMetadata>;
}

/// Metadata for a file
pub struct FileMetadata {
    /// Path to the file
    pub path: String,
    
    /// Size of the file in bytes
    pub size: u64,
    
    /// Content type (MIME type) of the file
    pub content_type: String,
    
    /// Whether the file is a directory
    pub is_directory: bool,
    
    /// Last modified time in milliseconds since epoch
    pub last_modified: Option<u64>,
    
    /// Content hash for verification
    pub content_hash: Option<String>,
}

/// Type alias for a boxed TenantStorage trait object
pub type TenantStorageRef = Arc<dyn TenantStorage>;