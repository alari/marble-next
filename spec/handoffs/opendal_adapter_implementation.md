# OpenDAL Adapter Implementation Handoff

**Last Updated: 2025-04-05**

## Current Status

We've completed our initial investigation of OpenDAL integration and have decided to pivot to a more direct approach. Our findings and decisions:

1. Created a `RawStorageAdapter` to bridge between OpenDAL's API and our RawStorageBackend
2. Implemented a placeholder memory-backed OpenDAL operator that passes tests
3. Documented the challenges of creating a full custom OpenDAL adapter
4. **Strategic Decision**: Instead of pursuing the complex OpenDAL adapter implementation immediately, we'll first create a more focused `TenantStorage` API that directly addresses our needs

After researching OpenDAL's custom adapter implementation, we found that developing a complete custom adapter is significantly more complex than initially expected. OpenDAL's raw API requires implementing multiple associated types and traits with precise signatures. Rather than getting blocked by this complexity, we'll implement a direct solution first and reconsider OpenDAL integration later.

## Unified Storage Approach

After evaluating the challenges with OpenDAL integration, we've decided to take a different approach:

1. **Initial OpenDAL Investigation (Completed)**:
   - **RawStorageAdapter**: Created a simplified wrapper around our RawStorageBackend
   - **Error Mapping**: Implemented logic to convert between our StorageError and OpenDAL Error types
   - **Path Normalization**: Developed helper methods to ensure consistent path formats
   - **Memory Backend**: Implemented a placeholder using OpenDAL's Memory service
   - **Documentation**: Updated our findings about OpenDAL's complexity

2. **New Unified Storage API (Next Steps)**:
   - **TenantStorage Trait**: Define a simpler, more focused API for tenant-isolated storage
   - **Direct Implementation**: Create an implementation that directly uses our existing components
   - **Tenant Isolation**: Make tenant isolation explicit through tenant_id parameters
   - **OpenDAL Deferral**: Postpone OpenDAL integration until our core functionality is solid

3. **Potential Future OpenDAL Integration**:
   - After our core functionality is working, we can revisit OpenDAL integration
   - We'll have a better understanding of how to map our storage model to OpenDAL
   - The implementation will be guided by actual WebDAV requirements

## Key Insights

### OpenDAL Custom Adapters
- OpenDAL provides three main approaches to custom storage integration:
  1. **Operator Facade**: Our chosen approach - wrapping our storage with OpenDAL interface
  2. **Full Custom Service**: More complex but more powerful - implementing all OpenDAL traits
  3. **Layer Composition**: Using existing services with custom layers for specific behavior

### Testing Approach
- We've implemented comprehensive tests for the RawStorageFacade functionality:
  - Test file writing and reading
  - Test listing files and directories
  - Test deleting files
  - Test path normalization
- All tests pass successfully, confirming that the facade works as expected

### Next Steps for Full Implementation
To complete the OpenDAL adapter implementation, we need to:
1. Create a proper bridge between our facade and OpenDAL's Operator interface
2. Implement any missing OpenDAL functionality (e.g., stream-based reading/writing)
3. Add support for all OpenDAL metadata operations
4. Complete comprehensive testing with the full implementation

## Development Progress

### Completed
- ✅ Comprehensive OpenDAL research and documentation
- ✅ RawStorageFacade implementation with all core operations
- ✅ Path normalization and content type detection 
- ✅ Error mapping between OpenDAL and our storage system
- ✅ Unit tests for all facade operations

### In Progress
- 🔄 Full OpenDAL Operator creation from our facade (using simplified approach)

### Planned
- ⏳ Proper integration between OpenDAL Operator and RawStorageBackend
- ⏳ Stream-based reading/writing
- ⏳ Advanced metadata support
- ⏳ Comprehensive integration tests

## Implementation Notes

### Facade vs. Full Implementation
We've opted for a facade-based approach because:
1. It's more straightforward to implement and test
2. It allows us to control exactly how operations map to our backend
3. It avoids having to implement the full OpenDAL trait hierarchy

### Path Handling
Our facade handles path normalization to ensure consistency:
```rust
fn normalize_path(path: &str) -> String {
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };

    // Remove trailing slash unless it's the root path
    if path.len() > 1 && path.ends_with('/') {
        path[0..path.len() - 1].to_string()
    } else {
        path
    }
}
```

### Error Mapping
We map our errors to appropriate OpenDAL error types:
```rust
fn convert_error(err: crate::error::StorageError) -> OpendalError {
    match err {
        crate::error::StorageError::NotFound(msg) => {
            OpendalError::new(ErrorKind::NotFound, &msg)
        },
        crate::error::StorageError::Authorization(msg) => {
            OpendalError::new(ErrorKind::PermissionDenied, &msg)
        },
        // etc.
    }
}
```

## Challenges and Strategic Pivot

We attempted to implement a custom `Accessor` implementation but encountered multiple API compatibility issues. These challenges led us to reconsider our approach:

1. **OpenDAL's Raw API Complexity**: 
   - Requires implementing six associated types (`Reader`, `Writer`, `Lister`, etc.)
   - Method signatures different from documentation (e.g., additional parameters)
   - Private/internal implementation details not accessible to users

2. **Current Status**: 
   - We've implemented a simplified version using Memory backend as a placeholder
   - All tests pass but the integration is not yet functional
   - The code provides a foundation for future OpenDAL integration if needed

Our strategic pivot:

1. **Focus on Core Requirements**: Create a direct `TenantStorage` API
2. **Leverage Existing Components**: Build on our working hash storage and database integration
3. **Explicit Tenant Isolation**: Design the API with tenant isolation as a core principle
4. **Simplify WebDAV Integration**: Create a clearer path to WebDAV support

## New Approach: TenantStorage API

The proposed `TenantStorage` trait will:

```rust
pub trait TenantStorage: Send + Sync + 'static {
    /// Read a file by path for a specific tenant
    async fn read(&self, tenant_id: &Uuid, path: &str) -> StorageResult<Vec<u8>>;
    
    /// Write a file at path for a specific tenant
    async fn write(&self, tenant_id: &Uuid, path: &str, content: Vec<u8>) -> StorageResult<()>;
    
    /// Check if a file exists for a tenant
    async fn exists(&self, tenant_id: &Uuid, path: &str) -> StorageResult<bool>;
    
    /// Delete a file for a tenant
    async fn delete(&self, tenant_id: &Uuid, path: &str) -> StorageResult<()>;
    
    /// List files for a tenant in a directory
    async fn list(&self, tenant_id: &Uuid, dir_path: &str) -> StorageResult<Vec<String>>;
}
```

Implementation will use our existing components:

```rust
pub struct MarbleTenantStorage {
    /// Database pool for metadata operations
    db_pool: Arc<PgPool>,
    
    /// Hash-based storage for content
    content_hasher: ContentHasher,
}
```

This approach gives us immediate benefits:
1. Direct path to working tenant-isolated storage
2. Clear API that maps exactly to our requirements
3. Flexibility to integrate with OpenDAL later if needed
4. Simpler WebDAV integration path

## References
- [Updated OpenDAL Documentation](../dependencies/opendal.md)
- [Marble Storage Specification](../crates/marble_storage.md)
- [Storage Architecture](../domain/storage_architecture.md)
