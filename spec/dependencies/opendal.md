# OpenDAL Specification

**Status:** DRAFT
**Last Updated:** 2025-04-03

## Overview

OpenDAL (Open Data Access Layer) provides a consistent API to interact with multiple storage backends. In Marble, it's used to abstract storage operations for both raw and processed content.

## Usage in Marble

- Abstracts storage operations for raw and processed content
- Enables seamless switching between development (local) and production (S3) environments
- Powers the WebDAV interface through integration with dav-server-opendalfs

## Version and Features

- Current version: 0.52.0
- Required features:
  - `services-s3`: For S3 storage backend
  - `runtime-tokio`: For async operation support

## Basic Configuration

```rust
// S3 configuration
let mut builder = S3::default();
builder.bucket("marble-content");
builder.region("us-west-2");
builder.root("/user-content");
let op = Operator::new(builder)?.finish();

// Local filesystem (development)
let mut builder = Fs::default();
builder.root("/tmp/marble-data");
let op = Operator::new(builder)?.finish();
```

## Error Handling

Convert OpenDAL errors to Marble-specific errors when needed:

```rust
fn handle_opendal_error(e: OpendalError) -> StorageError {
    match e {
        e if e.kind() == ErrorKind::NotFound => StorageError::NotFound(e.to_string()),
        e if e.kind() == ErrorKind::PermissionDenied => StorageError::PermissionDenied(e.to_string()),
        e => StorageError::Other(e),
    }
}
```

## Related Specifications

- [Storage Architecture](../domain/storage_architecture.md)
- [marble_storage Crate](../crates/marble_storage.md)
