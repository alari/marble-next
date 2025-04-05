# Authentication Architecture

**Status:** IMPLEMENTED
**Last Updated:** 2025-04-05

## Overview

The authentication system in Marble provides secure user identification and tenant isolation for the multi-tenant architecture. Authentication is handled by a dedicated `AuthService` in the marble-db crate, separating database concerns from application-specific authentication requirements.

## Architecture

```
┌──────────────────────┐       ┌──────────────────────┐       ┌───────────────────┐
│                      │       │                      │       │                   │
│  WebDAV / Application│       │  Database Layer      │       │  Database         │
│  Layer               │◄──────┤  (marble-db)         │◄──────┤  Storage          │
│                      │       │                      │       │                   │
└──────────────────────┘       └──────────────────────┘       └───────────────────┘
         │                                │                              │
         │  WebDavAuthService             │  DatabaseAuthService         │  User Table
         │  - Adapter for DB AuthService  │  - Core auth implementation  │  - Credentials
         │  - Tenant ID extraction        │  - Password verification     │  - Tenant IDs
         │  - HTTP Basic Auth handling    │  - User lookup               │  - User profiles
         ▼                                ▼                              ▼
```

## Authentication Flow

1. HTTP client submits credentials via Basic Auth header
2. WebDAV handler extracts credentials using `extract_basic_auth` helper
3. `WebDavAuthService` forwards credentials to `DatabaseAuthService` 
4. `DatabaseAuthService` looks up the user and verifies the password
5. Upon successful authentication, the tenant UUID is returned
6. WebDAV handler uses tenant UUID for all subsequent storage operations
7. Tenant isolation is enforced by including tenant ID in all database and storage operations

## Components

### AuthService Interface (marble-db)

The core authentication trait that defines the authentication contract:

```rust
#[async_trait]
pub trait AuthService: Send + Sync + 'static {
    /// Authenticate a user by username and password
    /// Returns the user's UUID if authentication is successful
    async fn authenticate_user(&self, username: &str, password: &str) -> AuthResult<Uuid>;
    
    /// Verify a password against a stored hash
    async fn verify_password(&self, password: &str, password_hash: &str) -> AuthResult<bool>;
}
```

### DatabaseAuthService (marble-db)

An implementation of `AuthService` that uses the database repositories:

```rust
pub struct DatabaseAuthService {
    user_repository: SqlxUserRepository,
}

impl DatabaseAuthService {
    /// Create a new database-backed authentication service
    pub fn new(user_repository: SqlxUserRepository) -> Self { ... }
    
    /// Create a new database-backed authentication service from a pool
    pub fn from_pool(pool: Arc<PgPool>) -> Self { ... }
}
```

Key responsibilities:
- User lookup by username
- Password verification
- Login recording for audit purposes
- Secure tenant ID management

### WebDAV Authentication Service (marble-webdav)

An adapter that connects the WebDAV layer to the database authentication:

```rust
pub struct WebDavAuthService {
    db_auth_service: Arc<dyn DbAuthService>,
}

#[async_trait]
impl AuthService for WebDavAuthService {
    async fn authenticate(&self, username: &str, password: &str) -> Result<Uuid, AuthError> { ... }
}
```

Key responsibilities:
- Adapting between WebDAV and database authentication interfaces
- Error mapping between layers
- Maintaining layer separation

### Basic Auth Helper (marble-webdav)

A utility function that extracts credentials from HTTP Basic Auth headers:

```rust
pub fn extract_basic_auth(auth_header: Option<&str>) -> Option<(String, String)> { ... }
```

## Password Handling

Currently using simple string comparison for development (placeholder):

```rust
async fn verify_password(&self, password: &str, password_hash: &str) -> AuthResult<bool> {
    // TODO: Implement proper password verification with a hashing library
    // In production, this should use a secure password hashing algorithm like bcrypt or Argon2
    Ok(password == password_hash)
}
```

Planned improvements:
- Implement proper password hashing with bcrypt or Argon2
- Add salt management
- Support password upgrades for future algorithm changes

## Error Handling

Authentication errors are categorized in a dedicated error type:

```rust
pub enum AuthError {
    /// Missing credentials
    MissingCredentials,

    /// Invalid credentials
    InvalidCredentials,

    /// User not found
    UserNotFound,

    /// Database error
    Database(#[from] Error),

    /// Password verification error
    PasswordVerification(String),
}
```

This provides:
- Granular error reporting
- Clear distinction between different failure modes
- Proper mapping between layers

## Security Considerations

- Passwords are never logged or exposed in error messages
- Authentication results are clearly separated from authentication failures
- Future improvements will add rate limiting and brute force protection
- All authentication operations are performed securely against the database

## Future Enhancements

1. Add proper password hashing with industry-standard algorithms
2. Implement rate limiting for failed authentication attempts
3. Add support for OAuth or API token authentication for programmatic access
4. Add audit logging for authentication events
5. Implement session management for web interfaces

## Related Specifications

- [WebDAV Implementation](../handoffs/webdav_implementation.md)
- [Database Schema](database_schema.md)
- [Tenant Isolation](storage_architecture.md)
