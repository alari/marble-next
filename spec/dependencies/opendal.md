# OpenDAL Specification

**Status:** DRAFT
**Last Updated:** 2025-04-03

## Overview

OpenDAL (Open Data Access Layer) is a unified data access layer for various storage services. It provides a consistent API to interact with multiple storage backends including S3, Azure Blob Storage, Google Cloud Storage, local file systems, and more.

## Usage in Marble

OpenDAL is a critical component in Marble's storage architecture:

- Used to abstract storage operations across both raw and processed content stores
- Provides a unified interface regardless of underlying storage implementation
- Enables easy switching between development (local) and production (S3) environments
- Powers the WebDAV interface through integration with dav-server-opendalfs

## Version and Features

- Current version: 0.43.0
- Required features:
  - `services-s3`: For S3 storage backend
  - `layers-prometheus`: For metrics collection
  - Optional: `runtime-tokio` for async operation support

## Configuration

Basic OpenDAL configuration for S3:

```rust
use opendal::services::S3;
use opendal::Operator;

// Configure S3 backend
let mut builder = S3::default();
builder.bucket("marble-content");
builder.region("us-west-2");
builder.root("/user-content");

// Create the operator
let op = Operator::new(builder)?.finish();
```

For local development:

```rust
use opendal::services::Fs;
use opendal::Operator;

// Configure local filesystem backend
let mut builder = Fs::default();
builder.root("/tmp/marble-data");

// Create the operator
let op = Operator::new(builder)?.finish();
```

## Key APIs and Patterns

### Reading and Writing Content

```rust
// Reading a file
let content = op.read("path/to/file").await?;

// Writing a file
op.write("path/to/file", content).await?;

// Checking if file exists
let exists = op.is_exist("path/to/file").await?;

// Listing directory contents
let entries = op.list("path/to/dir").await?;
```

### Using OpenDAL with WebDAV

Integration with dav-server-opendalfs happens at the server configuration level:

```rust
use dav_server_opendalfs::prelude::*;
use opendal::Operator;

// Create OpenDAL operator
let op = create_opendal_operator()?;

// Create WebDAV handler using OpenDAL
let webdav_handler = DavHandler::builder()
    .backend(OpendalsFs::new(op))
    .locksystem(MemLs::new())
    .build();
```

## Error Handling

OpenDAL provides its own error type that we convert to Marble-specific errors:

```rust
use opendal::Error as OpendalError;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Item not found: {0}")]
    NotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Storage error: {0}")]
    Other(#[from] OpendalError),
}

// Conversion example
fn handle_opendal_error(e: OpendalError) -> StorageError {
    match e {
        e if e.kind() == ErrorKind::NotFound => {
            StorageError::NotFound(e.to_string())
        }
        e if e.kind() == ErrorKind::PermissionDenied => {
            StorageError::PermissionDenied(e.to_string())
        }
        e => StorageError::Other(e),
    }
}
```

## Alternatives Considered

- **rust-s3**: More specific to S3, but lacks the abstraction for other backends
- **object_store**: Similar abstraction but less mature than OpenDAL at the time of selection
- **Custom implementation**: Rejected due to development time and maintenance burden
- **cloud-specific SDKs**: Rejected due to lock-in and inconsistent APIs

## Performance Characteristics

- OpenDAL focuses on minimal overhead for storage operations
- Uses sensible defaults for concurrency and chunking
- Provides opportunity for connection pooling and request reuse

## Security Considerations

- Authentication handled through underlying service SDKs
- Credentials should be provided via environment variables, not hardcoded
- Path sanitization is important for preventing directory traversal attacks

## Related Specifications

- [Storage Architecture](../domain/storage_architecture.md)
- [marble-storage Crate](../crates/marble_storage.md)
- [Marble WebDAV Server](../crates/marble_webdav.md)

## Future Considerations

- OpenDAL is actively developed and may introduce breaking changes
- Consider implementing versioned storage adapters to isolate API changes
- Evaluate specialized features like object locking as they become available
