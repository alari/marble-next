//! Public API for the database module
//!
//! This module provides the public interface for database operations.
//! All external crates should interact with marble-db through these traits.

use sqlx::postgres::PgPool;
use std::sync::Arc;

use crate::error::Error;
use crate::Result;

/// Core database operations trait
///
/// This trait defines the interface for interacting with the database.
/// Implementations handle connections, transactions, and migrations.
#[async_trait::async_trait]
pub trait DatabaseApi: Send + Sync + 'static {
    /// Initialize the database, running migrations if needed
    async fn initialize(&self) -> Result<()>;

    /// Get a reference to the database pool
    fn pool(&self) -> &PgPool;

    /// Check if the database is healthy
    async fn health_check(&self) -> Result<()>;
}

/// Database implementation that wraps a connection pool
#[derive(Debug, Clone)]
pub struct Database {
    pool: Arc<PgPool>,
}

impl Database {
    /// Create a new Database instance with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
}

#[async_trait::async_trait]
impl DatabaseApi for Database {
    async fn initialize(&self) -> Result<()> {
        crate::run_migrations(self.pool.as_ref()).await
    }

    fn pool(&self) -> &PgPool {
        self.pool.as_ref()
    }

    async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(self.pool.as_ref())
            .await
            .map(|_| ())
            .map_err(Error::QueryFailed)
    }
}
