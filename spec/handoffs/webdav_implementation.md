# WebDAV Integration with TenantStorage Handoff

**Last Updated: 2025-04-06** (WebDAV Implementation with LOCK functionality)

## Current Status

We've made significant progress on the WebDAV server implementation:

1. ✅ Completed the `TenantStorage` API implementation with proper tenant isolation
2. ✅ Implemented the WebDAV handler with authentication through a dedicated service
3. ✅ Implemented all core WebDAV methods (GET, PUT, PROPFIND, MKCOL, DELETE)
4. ✅ Implemented advanced WebDAV operations (COPY, MOVE)
5. ✅ Restructured the code for better maintainability with focused modules
6. ✅ Added comprehensive test coverage for all operations

The WebDAV implementation now supports:

- Browsing directories and reading files
- Creating and uploading files
- Creating directories
- Deleting files and directories
- Copying files and directories with recursive support
- Moving/renaming files and directories

We have implemented the LOCK and UNLOCK operations for proper WebDAV locking support, which provides better compatibility with clients that rely on locking for concurrency control. The implementation uses an in-memory lock manager that handles lock token generation, lock expiration, and validation.

## Implementation Strategy

We'll use a layered approach that focuses on incremental delivery of WebDAV functionality.

### Core Architecture

```
Axum HTTP Server → dav-server WebDAV Handler → TenantStorage API → Database & Content Storage
```

This architecture:
- Maintains tenant isolation by authenticating users and passing tenant IDs to storage operations
- Avoids unnecessary abstraction layers between WebDAV and storage
- Leverages existing libraries for WebDAV protocol implementation
- Aligns with our pivot away from OpenDAL adapters

## Implementation Plan

### Phase 1: Core WebDAV Infrastructure

1. **Setup WebDAV Server Framework**
   - Add `dav-server` (not OpenDAL variant) to dependencies
   - Create WebDAV handler structure with TenantStorage integration
   - Implement authentication to extract tenant IDs from requests
   - Create Axum integration for HTTP serving

2. **Implement Basic Authentication**
   - Create `AuthService` trait for user authentication
   - Implement database-backed authentication service
   - Extract credentials from WebDAV requests
   - Map usernames to tenant UUIDs for TenantStorage operations

3. **Basic Path Handling**
   - Implement path normalization between WebDAV and TenantStorage
   - Handle root directory special cases
   - Ensure proper URL encoding/decoding

### Phase 2: Core WebDAV Methods

1. **READ Operations (GET & PROPFIND)**
   - Implement GET method for file retrieval
   - Implement PROPFIND for directory listing
   - Map WebDAV properties to TenantStorage metadata
   - Set proper response headers for content type, length, etc.

2. **WRITE Operations (PUT & MKCOL)**
   - Implement PUT method for file creation/update
   - Implement MKCOL for directory creation
   - Handle parent directory creation as needed
   - Set proper response status codes

3. **DELETE Operation**
   - Implement DELETE method for file/directory removal
   - Ensure proper tenant isolation during deletion
   - Add appropriate error handling

### Phase 3: Advanced WebDAV Functionality

1. **File Movement Operations**
   - Implement COPY for file duplication
   - Implement MOVE for file/directory renaming and movement
   - Ensure database metadata is properly updated
   - Handle destination overwrite scenarios

2. **Lock Management**
   - Create `LockManager` interface for WebDAV locks
   - Implement in-memory or Redis-based lock storage
   - Add LOCK and UNLOCK method handlers
   - Integrate lock checking in PUT, DELETE, and MOVE operations

3. **Advanced WebDAV Properties**
   - Implement remaining WebDAV properties
   - Add support for custom properties if needed
   - Ensure proper XML formatting in responses

### Phase 4: Obsidian-Specific Optimizations

1. **Performance Optimizations**
   - Add caching for frequently accessed metadata
   - Optimize PROPFIND for directory listings (common in Obsidian)
   - Consider batch operations for multiple file updates

2. **Obsidian Compatibility Testing**
   - Test with Obsidian using WebDAV as a remote vault
   - Verify all core operations (read, write, rename, delete)
   - Test with varying file sizes and content types
   - Verify proper handling of Obsidian-specific files (.obsidian folder)

3. **Edge Case Handling**
   - Implement conditional requests (If-Match, If-None-Match)
   - Handle concurrent edit scenarios
   - Add proper error responses for Obsidian compatibility

## Detailed Technical Design

### WebDAV Handler

```rust
pub struct MarbleDavHandler {
    /// Storage for tenant operations
    tenant_storage: TenantStorageRef,

    /// Authentication service
    auth_service: Arc<dyn AuthService>,

    /// Lock manager for WebDAV locks
    lock_manager: Arc<dyn LockManager>,
}

impl DavHandler for MarbleDavHandler {
    async fn handle(
        &self,
        method: DavMethod,
        path: &str,
        headers: HeaderMap,
        body: Body,
    ) -> Result<DavResponse, Error> {
        // Extract credentials and get tenant ID
        let tenant_id = self.authenticate(&headers).await?;

        // Normalize path
        let normalized_path = normalize_path(path);

        // Handle method based on tenant ID and normalized path
        match method {
            DavMethod::Get => self.handle_get(tenant_id, &normalized_path).await,
            DavMethod::Put => self.handle_put(tenant_id, &normalized_path, headers, body).await,
            DavMethod::Propfind => self.handle_propfind(tenant_id, &normalized_path, body).await,
            // Other methods...
        }
    }
}
```

### Authentication Service

```rust
#[async_trait]
pub trait AuthService: Send + Sync + 'static {
    /// Authenticate a user and return their tenant ID
    async fn authenticate(&self, username: &str, password: &str) -> Result<Uuid, AuthError>;
}

pub struct DatabaseAuthService {
    db_pool: Arc<PgPool>,
}

impl DatabaseAuthService {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl AuthService for DatabaseAuthService {
    async fn authenticate(&self, username: &str, password: &str) -> Result<Uuid, AuthError> {
        // Query user from database
        let user = sqlx::query_as::<_, User>(
            "SELECT id, uuid, username, password_hash FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&*self.db_pool)
        .await?;

        // Verify user exists
        let user = user.ok_or(AuthError::InvalidCredentials)?;

        // Verify password (using bcrypt or similar)
        if !verify_password(password, &user.password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        Ok(user.uuid)
    }
}
```

### Lock Manager

```rust
#[async_trait]
pub trait LockManager: Send + Sync + 'static {
    /// Acquire a lock
    async fn lock(
        &self,
        tenant_id: &Uuid,
        path: &str,
        timeout: Duration,
        token: &str,
    ) -> Result<(), LockError>;

    /// Release a lock
    async fn unlock(
        &self,
        tenant_id: &Uuid,
        path: &str,
        token: &str,
    ) -> Result<(), LockError>;

    /// Check if a resource is locked
    async fn is_locked(
        &self,
        tenant_id: &Uuid,
        path: &str,
    ) -> Result<Option<LockInfo>, LockError>;
}

pub struct InMemoryLockManager {
    locks: Arc<RwLock<HashMap<(Uuid, String), LockInfo>>>,
}

impl InMemoryLockManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl LockManager for InMemoryLockManager {
    // Implementation details...
}
```

### Axum Integration

```rust
pub async fn create_webdav_server(
    tenant_storage: TenantStorageRef,
    auth_service: Arc<dyn AuthService>,
    lock_manager: Arc<dyn LockManager>,
) -> Router {
    // Create the WebDAV handler
    let dav_handler = MarbleDavHandler::new(
        tenant_storage,
        auth_service,
        lock_manager,
    );

    // Create Axum router
    Router::new()
        .route("/*path", any(handle_webdav))
        .layer(Extension(dav_handler))
}

async fn handle_webdav(
    Extension(handler): Extension<MarbleDavHandler>,
    headers: HeaderMap,
    method: Method,
    uri: Uri,
    body: Bytes,
) -> impl IntoResponse {
    // Convert Axum request to dav-server method
    let dav_method = convert_method(method);
    let path = uri.path();

    // Call the WebDAV handler
    match handler.handle(dav_method, path, headers, body).await {
        Ok(response) => convert_to_axum_response(response),
        Err(error) => handle_webdav_error(error),
    }
}
```

## Testing Strategy

1. **Unit Tests**
   - Test WebDAV handler methods in isolation
   - Mock TenantStorage for predictable responses
   - Verify correct tenant isolation in handler methods
   - Test path normalization and error handling

2. **Integration Tests**
   - Test WebDAV handler with actual TenantStorage implementation
   - Verify proper database interactions
   - Test with simulated WebDAV client requests
   - Verify lock functionality with concurrent operations

3. **End-to-End Tests**
   - Test with actual WebDAV client libraries
   - Verify compatibility with standard WebDAV operations
   - Test authentication and authorization
   - Test performance with large directories and files

4. **Obsidian Compatibility Tests**
   - Manual tests with Obsidian using WebDAV connection
   - Verify vault synchronization works correctly
   - Test common Obsidian operations (edit, create, move)
   - Verify proper handling of Obsidian-specific files

## Performance Considerations

1. **Caching Strategy**
   - Cache directory listings to reduce database queries
   - Use ETag headers for conditional requests
   - Invalidate cache on write operations

2. **Batch Operations**
   - Batch database queries when possible
   - Consider implementing BATCH extension for WebDAV

3. **Monitoring**
   - Add telemetry for WebDAV operations
   - Track operation latency and error rates
   - Monitor lock acquisition patterns

## Next Concrete Steps

1. ✅ Create the WebDAV handler skeleton in `marble-webdav` crate
   - Created the MarbleDavHandler with tenant isolation support
   - Implemented the basic structure for handling WebDAV methods
   - Added AuthService and LockManager interfaces with proper abstractions
   - Created placeholder implementations for key WebDAV methods
   - Set up the Axum server integration
   - Created proper dependency documentation for all HTTP/WebDAV dependencies
   - Updated to the latest dependency versions (axum 0.8.3, dav-server 0.7.0, http 1.3.1)
   - Fixed build issues with proper type definitions

2. ✅ Implement the `AuthService` interface and database implementation
   - Created a proper `AuthService` trait in the marble-db crate
   - Moved all database queries to the marble-db crate
   - Implemented the `DatabaseAuthService` with tenant isolation
   - Created an adapter in WebDAV to use the marble-db auth service
   - Added placeholder for password verification to be implemented later with a proper hashing library
   - Ensured proper separation of concerns between database access and WebDAV functionality

3. ✅ Add basic GET and PROPFIND methods for read operations
   - Implemented GET method for file retrieval
   - Implemented PROPFIND for directory listing with XML formatting
   - Connected methods to TenantStorage API with proper error handling
   - Added path conversion utilities for WebDAV/storage path normalization

4. ✅ Add basic WRITE operations (PUT, MKCOL, DELETE)
   - Implemented PUT method for file creation/update with content type handling
   - Implemented MKCOL for directory creation with proper parent directory checking
   - Implemented DELETE method with lock checking
   - Added proper response handling with appropriate status codes

5. ✅ Update server integration
   - Properly convert WebDAV responses to Axum responses
   - Added proper header handling for WebDAV compatibility
   - Improved error mapping for WebDAV-specific errors
   - Updated main function to initialize services and start the server

6. ✅ Add unit tests for the WebDAV methods
   - Created comprehensive test suite with MockTenantStorage
   - Implemented tests for all core WebDAV methods
   - Added utility functions for test data setup
   - Covered success cases and error handling

7. ✅ Implement advanced WebDAV functionality:
   - ✅ COPY operation for file duplication
   - ✅ MOVE operation for renaming and relocating files
   - ✅ LOCK and UNLOCK methods with proper integration
   - ✅ Code reorganization to improve maintainability

8. Add advanced server features:
   - Conditional requests (If-Match, If-None-Match)
   - Custom WebDAV properties
   - Performance optimizations

9. Test with Obsidian:
   - Manual testing with Obsidian WebDAV remote vault
   - Verify compatibility with Obsidian-specific file patterns
   - Test large vault synchronization performance
=======

## Timeline Estimate

- **Phase 1 (Core Infrastructure)**: ✅ Completed
- **Phase 2 (Basic Methods)**: ✅ Completed
- **Phase 3 (Advanced Methods)**: ✅ Completed
  - COPY and MOVE operations ✅
  - Locking functionality (LOCK and UNLOCK) ✅
- **Phase 4 (Optimizations)**: ⏳ Not Started
=======

## References

- [WebDAV RFC 4918](https://tools.ietf.org/html/rfc4918)
- [Authentication Architecture](../domain/authentication.md)
- [dav-server Crate Documentation](https://docs.rs/dav-server/)
- [Axum Documentation](https://docs.rs/axum/)
- [Obsidian WebDAV Sync Documentation](https://help.obsidian.md/Sync+your+notes/WebDAV)
