use thiserror::Error;

/// Error type for database operations
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to connect to the database
    #[error("Failed to connect to database: {0}")]
    ConnectionFailed(#[source] sqlx::Error),

    /// Failed to execute a query
    #[error("Query execution failed: {0}")]
    QueryFailed(#[source] sqlx::Error),

    /// Failed to run database migrations
    #[error("Database migration failed: {0}")]
    MigrationFailed(#[source] sqlx::Error),

    /// Failed to create transaction
    #[error("Failed to create transaction: {0}")]
    TransactionFailed(#[source] sqlx::Error),

    /// Failed to commit transaction
    #[error("Failed to commit transaction: {0}")]
    CommitFailed(#[source] sqlx::Error),

    /// Failed to rollback transaction
    #[error("Failed to rollback transaction: {0}")]
    RollbackFailed(#[source] sqlx::Error),

    /// Entity not found
    #[error("Entity not found: {0}")]
    NotFound(String),

    /// Duplicate entity
    #[error("Duplicate entity: {0}")]
    Duplicate(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Export DatabaseError for use in other crates
pub type DatabaseError = Error;
