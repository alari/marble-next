use marble_core::error::{MarbleError, DatabaseError};
use marble_storage::StorageError;
use thiserror::Error;

/// Errors that can occur in the WebDAV server
#[derive(Debug, Error)]
pub enum Error {
    /// Authentication errors
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    /// WebDAV protocol errors
    #[error("WebDAV protocol error: {0}")]
    WebDav(String),

    /// Storage errors
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Database errors
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    /// Lock errors
    #[error("Lock error: {0}")]
    Lock(#[from] LockError),
    
    /// Lock operation failed
    #[error("Lock operation failed: {0}")]
    LockFailed(String),
    
    /// Unlock operation failed
    #[error("Unlock operation failed: {0}")]
    UnlockFailed(String),

    /// Internal server errors
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl From<MarbleError> for Error {
    fn from(err: MarbleError) -> Self {
        match err {
            MarbleError::Storage(core_storage_error) => {
                // Manual conversion from core::StorageError to storage::StorageError
                match core_storage_error {
                    marble_core::error::StorageError::NotFound(path) => 
                        Error::Storage(marble_storage::StorageError::NotFound(path)),
                    marble_core::error::StorageError::PermissionDenied(msg) => 
                        Error::Storage(marble_storage::StorageError::Authorization(msg)),
                    marble_core::error::StorageError::InvalidPath(msg) => 
                        Error::Storage(marble_storage::StorageError::Validation(msg)),
                    marble_core::error::StorageError::AlreadyExists(msg) =>
                        Error::Storage(marble_storage::StorageError::Validation(msg)),
                    marble_core::error::StorageError::Io(msg) =>
                        Error::Storage(marble_storage::StorageError::Storage(msg)),
                    marble_core::error::StorageError::Backend(msg) =>
                        Error::Storage(marble_storage::StorageError::Storage(msg)),
                }
            },
            MarbleError::Database(e) => Error::Database(e),
            _ => Error::Internal(format!("Marble error: {}", err)),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Internal(format!("IO error: {}", err))
    }
}

/// Authentication errors
#[derive(Debug, Error)]
pub enum AuthError {
    /// Missing credentials
    #[error("Missing credentials")]
    MissingCredentials,

    /// Invalid credentials
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// User not found
    #[error("User not found")]
    UserNotFound,

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Password verification error
    #[error("Password verification error: {0}")]
    PasswordVerification(String),
}

/// Lock errors
#[derive(Debug, Error)]
pub enum LockError {
    /// Resource is locked by another user
    #[error("Resource is locked by another user")]
    ResourceLocked,

    /// Invalid lock token
    #[error("Invalid lock token")]
    InvalidLockToken,

    /// Lock expired
    #[error("Lock expired")]
    LockExpired,

    /// Internal lock error
    #[error("Internal lock error: {0}")]
    Internal(String),
}
