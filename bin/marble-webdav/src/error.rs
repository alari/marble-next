use marble_core::error::{MarbleError, DatabaseError, StorageError};
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

    /// Internal server errors
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl From<MarbleError> for Error {
    fn from(err: MarbleError) -> Self {
        match err {
            MarbleError::Storage(e) => Error::Storage(e),
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
