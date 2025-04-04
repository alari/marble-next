use thiserror::Error;

/// Storage-related errors for the marble-storage crate
#[derive(Error, Debug)]
pub enum StorageError {
    /// Errors occurring during database operations
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Errors from OpenDAL operations
    #[error("storage operation error: {0}")]
    Storage(String),

    /// Errors related to OpenDAL
    #[error("opendal error: {0}")]
    OpenDal(#[from] opendal::Error),

    /// Errors from content hashing
    #[error("hashing error: {0}")]
    Hashing(String),

    /// Authorization errors (e.g., attempting to access another user's content)
    #[error("authorization error: {0}")]
    Authorization(String),

    /// Configuration errors
    #[error("configuration error: {0}")]
    Configuration(String),

    /// File not found errors
    #[error("file not found: {0}")]
    NotFound(String),

    /// Validation errors
    #[error("validation error: {0}")]
    Validation(String),
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

impl From<std::io::Error> for StorageError {
    fn from(error: std::io::Error) -> Self {
        if error.kind() == std::io::ErrorKind::NotFound {
            StorageError::NotFound(error.to_string())
        } else {
            StorageError::Storage(error.to_string())
        }
    }
}
