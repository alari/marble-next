//! Error types for the database crate
//!
//! This module defines error types specific to database operations.

use thiserror::Error;

/// Database-related errors
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to connect to the database
    #[error("Failed to connect to database: {0}")]
    ConnectionFailed(#[source] sqlx::Error),

    /// Failed to run database migrations
    #[error("Failed to run database migrations: {0}")]
    MigrationFailed(#[source] sqlx::migrate::MigrateError),

    /// Failed to execute a database query
    #[error("Failed to execute database query: {0}")]
    QueryFailed(#[source] sqlx::Error),
}
