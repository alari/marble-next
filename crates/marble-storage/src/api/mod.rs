use std::sync::Arc;

use async_trait::async_trait;
use opendal::Operator;
use uuid::Uuid;

use crate::error::StorageResult;

/// Defines the storage capabilities for Marble.
/// 
/// The storage layer provides access to both raw user content and
/// content-addressable hash-based storage.
#[async_trait]
pub trait MarbleStorage: Send + Sync + 'static {
    /// Get a raw storage operator for a specific user.
    /// 
    /// This operator provides access to the user's files with their original
    /// paths and structure.
    /// 
    /// # Arguments
    /// * `user_id` - The UUID of the user
    /// 
    /// # Returns
    /// * An OpenDAL operator configured for the user's raw storage
    async fn raw_storage(&self, user_id: Uuid) -> StorageResult<Operator>;
    
    /// Get the hash-based storage operator.
    /// 
    /// This operator provides direct access to content by hash.
    /// The hash storage is shared across all users and uses content-based
    /// addressing, which enables deduplication.
    /// 
    /// # Returns
    /// * An OpenDAL operator for hash-based content access
    fn hash_storage(&self) -> Operator;
}

/// Type alias for a boxed MarbleStorage trait object
pub type MarbleStorageRef = Arc<dyn MarbleStorage>;

/// Tenant-isolated storage module
pub mod tenant;
pub use tenant::{TenantStorage, TenantStorageRef, FileMetadata};