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

1. **Raw Storage Backend**:
   - Read-write access to original user content
   - Content storage:
     - File content stored in S3 with content-addressable (hash-based) approach
     - Identical content shares the same storage object
   - Metadata storage:
     - File paths, hashes, and relationships stored in PostgreSQL
     - Tracks file versions, folder structure, and dependencies
   - Preserves original file structure from user's vault
   - Direct mapping to WebDAV paths
   - Contains unmodified content as uploaded by the user

2. **Processed Storage Backend**:
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

```rust
// Example API design (to be refined)
pub trait MarbleStorage {
    // Create a raw storage backend for a specific tenant
    fn raw_storage(&self, tenant: &str) -> Box<dyn OpenDAL>;
    
    // Create a processed storage backend
    fn processed_storage(&self) -> Box<dyn OpenDAL>;
    
    // Convert a raw path to a processed path
    fn raw_to_processed_path(&self, tenant: &str, path: &str) -> String;
}

// Implementation might use configuration to determine underlying storage
pub struct FileSystemStorage {
    raw_base_path: PathBuf,
    processed_base_path: PathBuf,
}

// Other implementations might use S3, databases, etc.
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
