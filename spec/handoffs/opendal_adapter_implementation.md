# OpenDAL Adapter Implementation Handoff

**Last Updated: 2025-04-04**

## Current Status

We've completed Phase 1 of the OpenDAL adapter implementation. A RawStorageFacade has been created to bridge between OpenDAL's API and our RawStorageBackend, and tests have been implemented to verify the facade's functionality. However, the adapter doesn't yet create a fully functional OpenDAL Operator - this will be completed in the next phase.

## Implementation Approach

After researching OpenDAL's adapter patterns, we've chosen a facade-based approach for implementation:

1. **RawStorageFacade**: A class that adapts between our RawStorageBackend and OpenDAL's expectations, mapping operations like read, write, list, and delete to the corresponding backend methods.

2. **Error Mapping**: A system to convert between our StorageError and OpenDAL Error types.

3. **Path Normalization**: Helper methods to ensure consistent path formats between systems.

4. **Content Type Handling**: Automatic content type detection based on file extensions.

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
- 🔄 Full OpenDAL Operator creation from our facade

### Planned
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

## Next Step: Creating OpenDAL Operator

The most challenging part remaining is creating an actual OpenDAL Operator from our facade. The options are:

1. **Memory-backed Service with Layer**: Use OpenDAL's Memory service as a foundation and add a custom layer that intercepts all operations to use our facade.

2. **Custom Service Implementation**: Implement OpenDAL's `Accessor` trait and related traits for our facade. This is more complex but allows for deeper integration.

We recommend starting with option 1 for a quicker implementation and potentially moving to option 2 if performance or functionality requirements demand it.

## References
- [Updated OpenDAL Documentation](../dependencies/opendal.md)
- [Marble Storage Specification](../crates/marble_storage.md)
- [Storage Architecture](../domain/storage_architecture.md)
