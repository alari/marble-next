# Marble Storage Specification

## Overview

The `marble-storage` crate provides storage abstraction for the Marble platform, implementing tenant-isolated OpenDAL backends for both raw and processed data.

## Responsibilities

- Provide OpenDAL-compatible backends for raw and processed data
- Ensure tenant isolation at the storage level
- Handle paths and organization of stored content
- Support efficient retrieval and modification operations
- Integrate with the WebDAV interface

## Architecture

The `marble-storage` crate implements a hybrid storage architecture combining object storage (S3) for content and a relational database (PostgreSQL) for metadata. It provides two primary storage backends through OpenDAL:

1. **Raw Storage Backend**: *(Initial Implementation Focus)*
   - Read-write access to original user content
   - Content storage:
     - File content stored in S3 with content-addressable (hash-based) approach (`/.hash/{hash}`)
     - Identical content shares the same storage object (deduplication)
   - Metadata storage:
     - File paths, hashes, and relationships stored in PostgreSQL
     - Tracks file versions, folder structure, and dependencies
     - User_id stored with metadata for tenant isolation
   - Preserves original file structure from user's vault
   - Direct mapping to WebDAV paths
   - Contains unmodified content as uploaded by the user

2. **Processed Storage Backend**: *(Future Implementation)*
   - Read-only access to transformed content
   - Dynamically generated from metadata database:
     - Queries identify published content (with `publish: true` in frontmatter) and its dependencies
     - Content retrieved from S3 based on content hashes
     - Transformed according to processing rules
   - Content is restructured based on permalink values:
     - Published markdown files become index files in permalink-named directories
     - Embedded content is included within appropriate directories
   - Obsidian links are converted to standard markdown links
   - Prefixes all paths with username for tenant isolation
   - References to embedded content use fragment anchors

## API Design

### Current Implementation Focus

```rust
// Current implementation focuses on the write side
#[async_trait]
pub trait MarbleStorage: Send + Sync + 'static {
    // Create a raw storage backend for a specific tenant
    async fn raw_storage(&self, user_id: uuid::Uuid) -> StorageResult<Operator>;
    
    // Get the hash backend for direct hash-based access
    fn hash_storage(&self) -> Operator;
}

// Single implementation that supports both filesystem and S3
pub struct MarbleStorageImpl {
    config: StorageConfig,
    hash_operator: Operator,
    content_hasher: ContentHasher,
}

// Configuration supports both backends
pub enum StorageBackend {
    S3(S3Config),
    FileSystem(FileSystemConfig),
}

// Content hashing service
pub struct ContentHasher {
    operator: Operator,
}

impl ContentHasher {
    // Store content and return its hash
    async fn store_content(&self, content: &[u8]) -> StorageResult<String>;
    
    // Retrieve content by its hash
    async fn get_content(&self, hash: &str) -> StorageResult<Vec<u8>>;
    
    // Check if content with the given hash exists
    async fn content_exists(&self, hash: &str) -> StorageResult<bool>;
}

// Error handling
#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("storage operation error: {0}")]
    Storage(String),
    
    #[error("opendal error: {0}")]
    OpenDal(#[from] opendal::Error),
    
    #[error("authorization error: {0}")]
    Authorization(String),
    
    #[error("configuration error: {0}")]
    Configuration(String),
    
    #[error("file not found: {0}")]
    NotFound(String),
    
    #[error("validation error: {0}")]
    Validation(String),
}
```

### OpenDAL Integration Notes

```rust
// Creating an OpenDAL operator requires a two-step process
let operator_builder = Operator::new(builder)?;
let operator = operator_builder.finish();

// Writing content in async functions requires owned data
async fn write_content(op: &Operator, path: &str, content: Vec<u8>) -> Result<()> {
    op.write(path, content).await?;
    Ok(())
}

// Reading content returns a Vec<u8>
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

### Future API Additions (Read Side)

```rust
// To be implemented in future phases
#[async_trait]
pub trait MarbleStorage {
    // Current write-side methods...
    
    // Future read-side methods:
    async fn processed_storage(&self) -> StorageResult<Operator>;
    async fn raw_to_processed_path(&self, user_id: Uuid, path: &str) -> StorageResult<String>;
}
```

## Path Handling

### Raw Storage Paths
- Raw paths match the original file structure: `/path/to/file.md`
- Each tenant has their own isolated raw storage space

### Processed Storage Paths
- All processed paths are prefixed with username: `/{username}/...`
- Published content is organized by permalink:
  - For a file with `permalink: "my-page"`:
    - Main content: `/{username}/my-page/index.md`
    - Embedded content: `/{username}/my-page/embedded-1.md`, etc.
  - For a file without permalink, a default permalink is generated
- References to embedded content within a page use fragment anchors:
  - `/{username}/containing-page#embedded-content-id`

## Configuration Options

- Base paths for file system storage
- S3 bucket configuration (if applicable)
- Database connection strings (if applicable)
- Caching policies
- Performance tuning options

## Integration Points

- Interfaces with `marble-webdav` for WebDAV operations
- Provides backends for content processing pipeline
- Supports user authentication verification

## Future Work

- Define exact OpenDAL operator implementations
- Establish caching strategies
- Determine persistence guarantees
- Define error handling patterns
