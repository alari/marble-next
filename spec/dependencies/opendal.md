# OpenDAL

**Version:** 0.45.1

## Overview

OpenDAL (Open Data Access Layer) is a data access layer that provides a unified interface for multiple storage services including S3, Azure Blob Storage, Google Cloud Storage, and local filesystem. It allows for the same code to work across different storage backends.

## Core Features Used

- **Common Interface**: Uniform API for different storage services
- **Filesystem Support**: For local development and testing
- **S3 Support**: For production storage
- **Async Operations**: Using Tokio runtime

## Usage Pattern

### Creating Operators

OpenDAL uses a two-step process to create storage operators:

```rust
// Step 1: Create a builder for the service (S3, Fs, etc.)
let mut builder = S3::default();
builder.bucket("bucket-name");
builder.region("us-east-1");
// Configure other settings...

// Step 2: Create an operator using the builder
let operator_builder = Operator::new(builder)?;
let operator = operator_builder.finish();
```

### Content Operations

Operations for reading, writing, and checking existence:

```rust
// Writing content (requires Vec<u8> for async)
async fn write_content(op: &Operator, path: &str, content: Vec<u8>) -> Result<()> {
    op.write(path, content).await?;
    Ok(())
}

// Reading content
async fn read_content(op: &Operator, path: &str) -> Result<Vec<u8>> {
    let content = op.read(path).await?;
    Ok(content)
}

// Checking if content exists
async fn exists(op: &Operator, path: &str) -> Result<bool> {
    let exists = op.is_exist(path).await?;
    Ok(exists)
}
```

## Key Considerations

- **Borrowing Rules**: When working with async functions, content must be owned (Vec<u8>) rather than borrowed (&[u8])
- **Error Handling**: OpenDAL provides its own error type that should be converted in your application's error system
- **Feature Flags**: Required features must be enabled for different storage backends (`services-s3`, `services-fs`, etc.)
- **Layer System**: OpenDAL provides a layer system for adding middleware-like functionality
- **Read vs Write**: Some operations are inherently async (like reading/writing) while others can be synchronous

## Integration with Marble

In Marble, OpenDAL is used to:

1. Provide content-addressable storage using hashes (`/.hash/{hash}`)
2. Abstract away the differences between local filesystem (development) and S3 (production)
3. Support the WebDAV interface for vault synchronization

## Common Pitfalls

- Not using the proper two-step process for creating operators
- Trying to use borrowed data in async write operations
- Missing feature flags for specific storage backends
- Not handling path differences between storage providers correctly
