# OpenDAL

**Version:** 0.45.1

## Overview

OpenDAL (Open Data Access Layer) is a data access layer that provides a unified interface for multiple storage services including S3, Azure Blob Storage, Google Cloud Storage, and local filesystem. It allows for the same code to work across different storage backends.

## Core Features Used

- **Common Interface**: Uniform API for different storage services
- **Filesystem Support**: For local development and testing
- **S3 Support**: For production storage
- **Async Operations**: Using Tokio runtime
- **Content-Addressable Storage**: Supporting our hash-based storage approach
- **Custom Adapters**: For integrating with our database-backed tenant isolation

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

// Listing files
async fn list_files(op: &Operator, path: &str) -> Result<Vec<Entry>> {
    let mut entries = Vec::new();
    let mut lister = op.list(path).await?;
    while let Some(entry) = lister.next().await {
        entries.push(entry?);
    }
    Ok(entries)
}

// Deleting files
async fn delete_file(op: &Operator, path: &str) -> Result<()> {
    op.delete(path).await?;
    Ok(())
}
```

## Custom Adapter Implementation

### Understanding OpenDAL Architecture

OpenDAL uses a layered architecture:

1. **Services** - Concrete implementations for different storage providers (S3, Fs, etc.)
2. **Layers** - Middleware-like components that can be applied to any service
3. **Accessor** - The core interface for storage operations
4. **Operator** - The user-facing API that combines services and layers

### Approaches to Custom Integration

There are three main approaches to integrate OpenDAL with custom storage systems:

#### 1. Operator Facade (Preferred Approach for Marble)

Instead of creating a full custom adapter, implement a facade that:
- Coordinates operations between OpenDAL and our database
- Handles tenant isolation through user_id filtering
- Uses OpenDAL's existing services for actual content storage

```rust
struct RawStorageFacade {
    backend: Arc<RawStorageBackend>,
    // Keep a reference to the underlying storage
    hash_operator: Operator,
}

impl RawStorageFacade {
    // Implement methods that map OpenDAL operations to our backend:
    
    async fn read(&self, path: &str) -> Result<Vec<u8>> {
        self.backend.read_file(path).await
    }
    
    async fn write(&self, path: &str, content: Vec<u8>) -> Result<()> {
        let content_type = guess_content_type(path);
        self.backend.write_file(path, content, &content_type).await
    }
    
    // And so on for other operations...
}

// Then provide a function to create an Operator from this facade
fn create_raw_operator(backend: Arc<RawStorageBackend>) -> OpendalResult<Operator> {
    // Use OpenDAL's dynamic dispatch approach to create an operator
    // from our facade implementation
}
```

#### 2. Full Custom Service (More Complex)

Implement a complete custom OpenDAL service:
- Requires implementing multiple OpenDAL traits
- Must handle all storage operations
- More flexible but more complex to implement and maintain

```rust
struct MarbleRawService {
    backend: Arc<RawStorageBackend>,
}

impl Accessor for MarbleRawService {
    // Implement required accessor methods
}

// Then implement any additional required traits for OpenDAL
```

#### 3. Layer Composition (Alternative Approach)

Instead of a custom service, compose existing services with custom layers:
- Use an existing service (like Fs or Memory)
- Add layers to handle tenant isolation, path mapping, etc.
- May be simpler for some use cases

```rust
fn create_layered_operator(backend: Arc<RawStorageBackend>) -> Operator {
    // Create base operator
    let base_op = create_base_operator();
    
    // Add layers for tenant isolation
    let op = base_op
        .layer(TenantIsolationLayer::new(backend))
        .layer(ContentHashingLayer::new());
    
    op
}
```

## Key Considerations for Custom Adapters

1. **Path Translation**: When implementing a custom adapter, you need to handle path mapping between OpenDAL paths and your storage system.

2. **Error Mapping**: Convert between OpenDAL errors and your application's error system consistently.

3. **Operation Permissions**: Implement proper access controls based on tenant isolation requirements.

4. **Metadata Handling**: OpenDAL operations often include metadata like content-type, last modified, etc.

5. **Async Compatibility**: Ensure your adapter works well with async Rust patterns.

6. **Immutable Content**: For content-addressable storage, content should be treated as immutable.

7. **Performance Considerations**:
   - OpenDAL is designed to be efficient with minimal overhead
   - Consider batch operations where available
   - Use proper buffering for large file handling

8. **Tenant Isolation**: Ensure that your adapter properly enforces tenant boundaries:
   - Filter all database queries by user_id
   - Validate access permissions before operations
   - Use proper error handling for authorization failures

## Integration with Marble

In Marble, OpenDAL is used to:

1. **Content-Addressable Storage**: Store content using hash-based paths (`/.hash/{hash}`)
2. **Backend Abstraction**: Support both local filesystem and S3 with the same code
3. **User Isolation**: Enforce tenant boundaries through database metadata
4. **WebDAV Integration**: Support WebDAV operations for file synchronization

### Marble-Specific Implementation Patterns

For our RawStorageBackend adapter:

1. **Metadata in Database**: Store file metadata (paths, permissions, etc.) in PostgreSQL
2. **Content in Object Storage**: Store actual file content in S3 or filesystem
3. **Path Translation**:
   - Convert WebDAV paths to internal paths
   - Map file paths to content hashes
   - Maintain proper directory structure
4. **Tenant Isolation**:
   - Every operation is scoped to a specific user_id
   - Path collisions between tenants are handled through database filtering
   - Content deduplication happens transparently across tenant boundaries

## Common Pitfalls

- **Inheritance vs. Composition**: OpenDAL favors composition over inheritance; use layers when possible
- **Borrowing Rules**: Async operations require owned data (Vec<u8>) rather than borrowed (&[u8])
- **Path Manipulation**: Be careful with path normalization and special characters
- **Error Handling**: OpenDAL errors should be properly mapped to your application errors
- **Concurrent Access**: Be aware of race conditions in database operations
- **Mixed File Types**: Handle both binary and text files correctly
- **Transactions**: Consider database transactions for operations that modify multiple records

## Testing Custom Adapters

1. **Feature Tests**: Test all OpenDAL operations (read, write, list, delete)
2. **Tenant Isolation**: Verify that users cannot access each other's files
3. **Concurrency**: Test multiple concurrent operations
4. **Error Handling**: Test all error paths
5. **Edge Cases**: Test unusual paths, empty files, very large files
6. **Content Types**: Test various content types and encodings
