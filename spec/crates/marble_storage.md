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

## User Identification

Marble uses a dual approach to user identification:

1. **Internal Database IDs (i32)**:
   - Used as primary keys in the database
   - Used for database relationships and foreign keys
   - Used internally by repositories

2. **UUIDs (Universally Unique Identifiers)**:
   - Used for external-facing user identification
   - Used in the `MarbleStorage` API
   - Provides security by not exposing internal database IDs

The system provides utilities to convert between these two ID types, ensuring proper tenant isolation while maintaining a clean external API.

### Authentication
- The `username` field is used for authentication in both write and read sides
- The WebDAV interface uses username/password authentication
- Passwords are stored as hashes in the `password_hash` field
- Authentication happens before storage operations
- Usernames are also used in path structures for processed content

## API Design

### Current Implementation

```rust
// Main storage interface
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
    db_pool: Option<Arc<PgPool>>,
    hash_operator: Operator,
    content_hasher: ContentHasher,
}

// Storage creation API
pub async fn create_storage(config: StorageConfig) -> StorageResult<Arc<dyn MarbleStorage>>;
pub async fn create_storage_with_db(
    config: StorageConfig,
    db_pool: Arc<PgPool>,
) -> StorageResult<Arc<dyn MarbleStorage>>;

// Raw storage backend
pub struct RawStorageBackend {
    user_id: i32,
    db_pool: Arc<PgPool>,
    file_repo: Arc<SqlxFileRepository>,
    content_hasher: ContentHasher,
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

### Raw Storage Backend Operations

The `RawStorageBackend` provides these core operations:

```rust
impl RawStorageBackend {
    // Read a file from raw storage
    pub async fn read_file(&self, path: &str) -> StorageResult<Vec<u8>>;
    
    // Write a file to raw storage
    pub async fn write_file(
        &self,
        path: &str,
        content: Vec<u8>,
        content_type: &str,
    ) -> StorageResult<()>;
    
    // Check if a file exists
    pub async fn file_exists(&self, path: &str) -> StorageResult<bool>;
    
    // Delete a file
    pub async fn delete_file(&self, path: &str) -> StorageResult<()>;
    
    // List files in a directory
    pub async fn list_files(&self, dir_path: &str) -> StorageResult<Vec<String>>;
}
```

These operations enforce tenant isolation through the user_id field and integrate with the database for metadata storage.

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
- Each tenant has their own isolated raw storage space through database metadata
- Physical content is stored in hash-based storage using content-addressable hash

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
- Integrates with marble-db for metadata storage and tenant isolation

## Current Implementation Status

- Content-addressable hash storage is fully implemented
- `ContentHasher` service for content hashing and storage is complete
- `RawStorageBackend` with database integration is implemented
- User ID conversion between UUID and database ID is implemented
- OpenDAL adapter for the `RawStorageBackend` is in progress

## Future Work

- Complete OpenDAL adapter implementation
- Implement the processed storage backend
- Establish caching strategies
- Implement garbage collection for unreferenced content
- Add comprehensive testing for tenant isolation
