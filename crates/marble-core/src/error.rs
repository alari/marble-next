use thiserror::Error;

/// Core error type for the Marble project
#[derive(Debug, Error)]
pub enum MarbleError {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Authorization error
    #[error("Authorization error: {0}")]
    Authorization(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Database error type
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// SQLx error
    #[error("Database error: {0}")]
    Sqlx(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Query error
    #[error("Query error: {0}")]
    Query(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Constraint violation
    #[error("Constraint violation: {0}")]
    Constraint(String),

    /// Not found
    #[error("Record not found")]
    NotFound,
}

/// Storage error type
#[derive(Debug, Error)]
pub enum StorageError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(String),

    /// Not found
    #[error("File not found: {0}")]
    NotFound(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Already exists
    #[error("Already exists: {0}")]
    AlreadyExists(String),

    /// Invalid path
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Backend error
    #[error("Backend error: {0}")]
    Backend(String),
}
