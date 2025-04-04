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
pub trait MarbleStorage {
    // Create a raw storage backend for a specific tenant
    fn raw_storage(&self, user_id: uuid::Uuid) -> Result<Box<dyn OpenDAL>, StorageError>;
    
    // Get the hash backend for direct hash-based access
    fn hash_storage(&self) -> Box<dyn OpenDAL>;
}

// Initial implementation will use file system for local development
pub struct FileSystemStorage {
    hash_base_path: PathBuf,
    db_pool: sqlx::PgPool,
}

// Production implementation will use S3
pub struct S3Storage {
    bucket_name: String,
    client: opendal::services::S3,
    db_pool: sqlx::PgPool,
}

// Error handling
#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("storage operation error: {0}")]
    Storage(String),
    
    #[error("authorization error: {0}")]
    Authorization(String),
    
    #[error("configuration error: {0}")]
    Configuration(String),
}
```

### Future API Additions (Read Side)

```rust
// To be implemented in future phases
pub trait MarbleStorage {
    // Current write-side methods...
    
    // Future read-side methods:
    fn processed_storage(&self) -> Box<dyn OpenDAL>;
    fn raw_to_processed_path(&self, user_id: uuid::Uuid, path: &str) -> Result<String, StorageError>;
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
