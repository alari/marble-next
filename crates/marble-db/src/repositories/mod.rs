//! Repository traits and implementations for database operations
//!
//! This module provides repository patterns for accessing and manipulating
//! database entities using the SQLx library.

mod user_repository;
mod folder_repository;
mod file_repository;

pub use user_repository::{UserRepository, SqlxUserRepository};
pub use folder_repository::{FolderRepository, SqlxFolderRepository};
pub use file_repository::{FileRepository, SqlxFileRepository};

use sqlx::postgres::PgPool;
use std::sync::Arc;
use crate::Result;

/// A trait for repositories that can be created from a database pool
pub trait Repository {
    /// Create a new repository instance
    fn new(pool: Arc<PgPool>) -> Self;
}

/// A trait for repositories that need to run transactions
#[async_trait::async_trait]
pub trait TransactionSupport {
    /// Begin a new transaction
    async fn begin_transaction(&self) -> Result<sqlx::Transaction<'static, sqlx::Postgres>>;
    
    /// Commit a transaction
    async fn commit_transaction(
        transaction: sqlx::Transaction<'static, sqlx::Postgres>
    ) -> Result<()>;
    
    /// Rollback a transaction
    async fn rollback_transaction(
        transaction: sqlx::Transaction<'static, sqlx::Postgres>
    ) -> Result<()>;
}

/// Common transaction support implementation that can be used by all repositories
#[async_trait::async_trait]
impl<T: Repository + Send + Sync> TransactionSupport for T 
where 
    Self: BaseRepository,
{
    async fn begin_transaction(&self) -> Result<sqlx::Transaction<'static, sqlx::Postgres>> {
        Ok(self.pool().begin().await.map_err(crate::Error::QueryFailed)?)
    }
    
    async fn commit_transaction(
        transaction: sqlx::Transaction<'static, sqlx::Postgres>
    ) -> Result<()> {
        transaction.commit().await.map_err(crate::Error::QueryFailed)
    }
    
    async fn rollback_transaction(
        transaction: sqlx::Transaction<'static, sqlx::Postgres>
    ) -> Result<()> {
        transaction.rollback().await.map_err(crate::Error::QueryFailed)
    }
}

/// A trait for repositories that have a pool reference
pub trait BaseRepository {
    /// Get a reference to the database pool
    fn pool(&self) -> &PgPool;
}
