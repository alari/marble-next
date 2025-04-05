use std::sync::Arc;

use async_trait::async_trait;
use mime_guess::from_path;
use sqlx::postgres::PgPool;
use uuid::Uuid;

use crate::api::tenant::{FileMetadata, TenantStorage};
use crate::backends::raw::RawStorageBackend;
use crate::backends::user::uuid_to_db_id;
use crate::error::{StorageError, StorageResult};
use crate::services::hasher::ContentHasher;

/// Implementation of the TenantStorage trait
///
/// This implementation uses the existing RawStorageBackend and ContentHasher
/// to provide tenant-isolated storage operations.
pub struct MarbleTenantStorage {
    /// Database pool for metadata operations
    db_pool: Arc<PgPool>,
    
    /// Content hasher for deduplication and storage
    content_hasher: ContentHasher,
}

impl MarbleTenantStorage {
    /// Create a new MarbleTenantStorage
    pub fn new(db_pool: Arc<PgPool>, content_hasher: ContentHasher) -> Self {
        Self {
            db_pool,
            content_hasher,
        }
    }
    
    /// Helper to create a RawStorageBackend for a specific tenant
    async fn get_backend_for_tenant(&self, tenant_id: &Uuid) -> StorageResult<RawStorageBackend> {
        // Convert UUID to database ID
        let db_user_id = uuid_to_db_id(&self.db_pool, *tenant_id).await?;
        
        // Create and return the backend
        Ok(RawStorageBackend::new(
            db_user_id,
            self.db_pool.clone(),
            self.content_hasher.clone(),
        ))
    }
    
    /// Helper to normalize paths
    fn normalize_path(path: &str) -> String {
        let path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };

        // Remove trailing slash unless it's the root path
        if path.len() > 1 && path.ends_with('/') {
            path[0..path.len() - 1].to_string()
        } else {
            path
        }
    }
    
    /// Helper to guess content type from path
    fn guess_content_type(path: &str) -> String {
        match from_path(path).first() {
            Some(mime) => mime.to_string(),
            None => "application/octet-stream".to_string(),
        }
    }
}

#[async_trait]
impl TenantStorage for MarbleTenantStorage {
    async fn read(&self, tenant_id: &Uuid, path: &str) -> StorageResult<Vec<u8>> {
        let backend = self.get_backend_for_tenant(tenant_id).await?;
        let normalized_path = Self::normalize_path(path);
        backend.read_file(&normalized_path).await
    }
    
    async fn write(&self, tenant_id: &Uuid, path: &str, content: Vec<u8>, content_type: Option<&str>) -> StorageResult<()> {
        let backend = self.get_backend_for_tenant(tenant_id).await?;
        let normalized_path = Self::normalize_path(path);
        
        // Use provided content type or guess from path
        let content_type = content_type
            .map(|ct| ct.to_string())
            .unwrap_or_else(|| Self::guess_content_type(&normalized_path));
        
        backend.write_file(&normalized_path, content, &content_type).await
    }
    
    async fn exists(&self, tenant_id: &Uuid, path: &str) -> StorageResult<bool> {
        let backend = self.get_backend_for_tenant(tenant_id).await?;
        let normalized_path = Self::normalize_path(path);
        backend.file_exists(&normalized_path).await
    }
    
    async fn delete(&self, tenant_id: &Uuid, path: &str) -> StorageResult<()> {
        let backend = self.get_backend_for_tenant(tenant_id).await?;
        let normalized_path = Self::normalize_path(path);
        backend.delete_file(&normalized_path).await
    }
    
    async fn list(&self, tenant_id: &Uuid, dir_path: &str) -> StorageResult<Vec<String>> {
        let backend = self.get_backend_for_tenant(tenant_id).await?;
        let normalized_path = Self::normalize_path(dir_path);
        
        // Ensure path ends with slash for directory listing
        let dir_path = if normalized_path.ends_with('/') {
            normalized_path
        } else {
            format!("{}/", normalized_path)
        };
        
        backend.list_files(&dir_path).await
    }
    
    async fn create_directory(&self, tenant_id: &Uuid, path: &str) -> StorageResult<()> {
        let backend = self.get_backend_for_tenant(tenant_id).await?;
        let normalized_path = Self::normalize_path(path);
        backend.create_directory(&normalized_path).await
    }
    
    async fn metadata(&self, tenant_id: &Uuid, path: &str) -> StorageResult<FileMetadata> {
        let backend = self.get_backend_for_tenant(tenant_id).await?;
        let normalized_path = Self::normalize_path(path);
        
        // Use the new get_file_metadata method from RawStorageBackend
        backend.get_file_metadata(&normalized_path).await
    }
}

/// Create a new TenantStorage implementation
pub async fn create_tenant_storage(
    db_pool: Arc<PgPool>,
    content_hasher: ContentHasher,
) -> StorageResult<Arc<dyn TenantStorage>> {
    let storage = MarbleTenantStorage::new(db_pool, content_hasher);
    Ok(Arc::new(storage))
}