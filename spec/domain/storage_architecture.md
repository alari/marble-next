# Storage Architecture Specification

## Overview

Marble uses a hybrid storage architecture combining object storage (S3) for content and a relational database (PostgreSQL) for metadata. This approach enables efficient content storage while providing rich querying capabilities for the metadata.

## Components

### Object Storage (S3)

- **Purpose**: Store actual file content in a hash-based system
- **Implementation**: OpenDAL with S3 backend
- **Organization**:
  - Content is stored using content-addressable approach (hash-based)
  - Content hashing uses blake2b_simd with base64 URL_SAFE_NO_PAD encoding
  - Files with identical content share the same storage object
  - Immutable storage - modifications create new objects with new hashes
  - Orphaned content tracked with tombstone records in database

### Metadata Database (PostgreSQL)

- **Purpose**: Store metadata, relationships, and structural information
- **Implementation**: SQLx for Rust database access
- **Schema** (conceptual):
  - `files`: Tracks file path, current hash, metadata
  - `file_versions`: Historical versions of files (enables future "time machine" functionality)
  - `folder_structure`: Hierarchy information
  - `references`: Links between files (Obsidian references)
  - `embeds`: Embeds between files (Obsidian embeds)
  - `frontmatter`: Extracted frontmatter data
  - `users`: User authentication information
  - `tombstones`: Tracks deleted content for garbage collection

## Data Flow

1. **Write Operations**:
   - Content is hashed and stored in S3
   - Metadata (path, hash, timestamp) is recorded in PostgreSQL
   - Previous versions are tracked in the database

2. **Read Operations**:
   - Path lookup occurs in PostgreSQL to find content hash
   - Content is retrieved from S3 using the hash

3. **Processing Pipeline**:
   - Triggered by write operations
   - Uses database to identify affected files
   - Efficiently processes only changed content and its dependencies
   - Updates processed metadata in the database

## Incremental Processing

The system implements incremental processing to handle large vaults efficiently:

1. **Change Detection**:
   - Each write/update operation is tracked in the database
   - System identifies what specifically changed

2. **Dependency Tracking**:
   - Database records which files reference or embed which other files
   - When a file changes, the system can identify all affected files

3. **Selective Processing**:
   - Only process changed files and their dependents
   - For large vaults (1000+ notes), this approach significantly reduces processing time

4. **Processing Flow**:
   - File is written through WebDAV
   - Storage layer stores content and updates metadata
   - Processor is notified of changes
   - Processor analyzes only affected files
   - Processed output is updated

## Read Model Generation

The read model (processed content) is derived from the write model:

1. Database queries identify published content
2. Content and its dependencies are retrieved
3. Content is transformed according to publishing rules
4. Transformed content is made available through processed API

## Optimization Techniques

### Caching Strategy

- **Read Model Caching**:
  - Processed content cached in S3
  - Cache serves as the read model
  - Public images can be served directly via HTTP from S3
  - Cache invalidation on content changes

- **Processing Queue**:
  - Changes buffer in a queue for 5 seconds after last modification
  - Prevents excessive processing during bulk updates (e.g., during sync)
  - Batch processing of related changes

- **Other Optimizations**:
  - Partial Updates: Only changed portions of the vault are reprocessed
  - Lazy Processing: Some processing deferred until content is requested
  - Parallel Processing: Independent files processed concurrently
  - Streaming for large media objects

### Concurrency Management

- WebDAV lock system (from dav-server-opendalfs) handles concurrent access
- Single-tenant design limits concurrency issues (one user per vault)

## Integration Points

- **WebDAV Server**: Interfaces with storage layer for read/write operations
- **Processor**: Uses database queries to identify changed files
- **Read Side**: Queries database for published content structure
