# Marble Database Specification

## Overview

The `marble-db` crate manages the PostgreSQL database schema and operations for Marble. It provides the metadata storage layer that complements the S3-based content storage, enabling efficient querying, relationship tracking, and incremental processing.

## Responsibilities

- Define and manage the database schema
- Provide typed query interfaces for common operations
- Track file metadata, paths, and relationships
- Support versioning and history tracking
- Facilitate incremental processing through dependency tracking
- Store user authentication data
- Manage garbage collection of orphaned content

## Schema Design

### Core Tables

#### `users`
- Stores user authentication information
- Fields:
  - `id`: Primary key
  - `username`: Unique username
  - `password_hash`: Securely stored password hash
  - `created_at`: Timestamp
  - `last_login`: Timestamp

#### `files`
- Tracks current state of each file
- Fields:
  - `id`: Primary key
  - `user_id`: Foreign key to users
  - `path`: File path in the vault
  - `content_hash`: Current content hash (links to S3)
  - `content_type`: MIME type or file format
  - `size`: File size
  - `created_at`: Timestamp
  - `updated_at`: Timestamp
  - `is_deleted`: Tombstone flag

#### `file_versions`
- Historical record of file changes
- Fields:
  - `id`: Primary key
  - `file_id`: Foreign key to files
  - `content_hash`: Content hash for this version
  - `version_number`: Sequential version number
  - `created_at`: Timestamp when this version was created
  - `metadata`: Additional version metadata

#### `folders`
- Tracks folder structure
- Fields:
  - `id`: Primary key
  - `user_id`: Foreign key to users
  - `path`: Folder path
  - `parent_id`: Foreign key to parent folder
  - `created_at`: Timestamp
  - `updated_at`: Timestamp
  - `is_deleted`: Tombstone flag

### Content Analysis Tables

#### `frontmatter`
- Extracted frontmatter data
- Fields:
  - `id`: Primary key
  - `file_id`: Foreign key to files
  - `publish`: Boolean flag for publishing
  - `permalink`: URL path if specified
  - `tags`: Array of tags
  - `aliases`: Array of alternative names
  - `section`: Section information
  - `description`: Content description
  - `title`: Content title
  - `created_date`: Date from frontmatter
  - `updated_date`: Date from frontmatter
  - `published_date`: Date from frontmatter
  - `layout`: Layout type
  - `other_data`: JSONB for additional fields

#### `document_links`
- Links and embeds between files (combines Obsidian references and embeds)
- Fields:
  - `id`: Primary key
  - `source_file_id`: Foreign key to source file
  - `target_name`: Referenced note title/name
  - `display_text`: Text displayed for the reference
  - `is_embed`: Boolean (true for embeds, false for references)
  - `target_file_id`: Foreign key to target file (NULL if unresolved)
  - `position`: Position in document for ordering
  - `fragment`: Section or block fragment if any

### Processing Tables

#### `processing_queue`
- Tracks files needing processing
- Fields:
  - `id`: Primary key
  - `file_id`: Foreign key to files
  - `operation`: Type of change (create, update, delete)
  - `enqueued_at`: When it was added to queue
  - `priority`: Processing priority
  - `status`: Current status (pending, processing, completed)
  - `last_attempt`: Timestamp of last processing attempt
  - `attempts`: Number of processing attempts

#### `published_content`
- Tracks what content is published
- Fields:
  - `id`: Primary key
  - `file_id`: Foreign key to source file
  - `permalink`: Published path
  - `processed_hash`: Hash of processed content
  - `published_at`: When it was published
  - `invalidated`: Whether it needs reprocessing

#### `cache_invalidations`
- Tracks what needs to be reprocessed
- Fields:
  - `id`: Primary key
  - `path`: Path pattern to invalidate
  - `created_at`: When invalidation was created
  - `processed`: Whether it has been processed

## API Design

The `marble-db` crate provides a trait-based API for database operations. The main interface is the `DatabaseApi` trait:

```rust
/// Core database operations trait
#[async_trait::async_trait]
pub trait DatabaseApi: Send + Sync + 'static {
    /// Initialize the database, running migrations if needed
    async fn initialize(&self) -> Result<()>;

    /// Get a reference to the database pool
    fn pool(&self) -> &PgPool;

    /// Check if the database is healthy
    async fn health_check(&self) -> Result<()>;
}
```

This trait is implemented by the `Database` struct:

```rust
/// Database implementation that wraps a connection pool
#[derive(Debug, Clone)]
pub struct Database {
    pool: Arc<PgPool>,
}

impl Database {
    /// Create a new Database instance with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
}
```

Additional functionality will be added through specialized traits for different database operations:

```rust
// Planned additional traits (to be implemented)
pub trait UserOperations: DatabaseApi {
    async fn create_user(&self, username: &str, password_hash: &str) -> Result<User>;
    async fn get_user(&self, username: &str) -> Result<User>;
    async fn authenticate_user(&self, username: &str, password_hash: &str) -> Result<User>;
}

pub trait FileOperations: DatabaseApi {
    async fn get_file(&self, user_id: i32, path: &str) -> Result<File>;
    async fn create_file(&self, user_id: i32, path: &str, hash: &str) -> Result<File>;
    async fn update_file(&self, file_id: i32, hash: &str) -> Result<File>;
    async fn mark_deleted(&self, file_id: i32) -> Result<()>;
}

pub trait VersionOperations: DatabaseApi {
    async fn get_file_versions(&self, file_id: i32) -> Result<Vec<FileVersion>>;
    async fn get_file_at_version(&self, file_id: i32, version: i32) -> Result<FileVersion>;
}

pub trait ContentAnalysisOperations: DatabaseApi {
    async fn get_frontmatter(&self, file_id: i32) -> Result<Frontmatter>;
    async fn update_frontmatter(&self, file_id: i32, frontmatter: &Frontmatter) -> Result<()>;
    async fn get_document_links(&self, file_id: i32, include_embeds: bool) -> Result<Vec<DocumentLink>>;
    async fn get_referencing_files(&self, file_id: i32, include_embeds: bool) -> Result<Vec<DocumentLink>>;
    async fn get_next_published_link(&self, source_file_id: i32, current_position: i32) -> Result<Option<DocumentLink>>;
    async fn get_prev_published_link(&self, source_file_id: i32, current_position: i32) -> Result<Option<DocumentLink>>;
    async fn get_recursive_embeds(&self, file_id: i32) -> Result<Vec<DocumentLink>>;
}

pub trait ProcessingOperations: DatabaseApi {
    async fn enqueue_file(&self, file_id: i32, operation: OperationType) -> Result<()>;
    async fn get_processing_batch(&self, max_items: i32) -> Result<Vec<QueueItem>>;
    async fn mark_processed(&self, queue_id: i32) -> Result<()>;
}

pub trait PublicationOperations: DatabaseApi {
    async fn get_published_content(&self, user_id: i32) -> Result<Vec<PublishedContent>>;
    async fn invalidate_cache(&self, path_pattern: &str) -> Result<()>;
}
```

### Database Configuration

Configuration for database connections is handled through the `DatabaseConfig` struct:

```rust
/// Configuration for a database connection
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Acquire timeout in seconds
    pub acquire_timeout_seconds: u64,
    /// Idle timeout in seconds
    pub idle_timeout_seconds: u64,
    /// Maximum lifetime of connections in seconds
    pub max_lifetime_seconds: u64,
}
```

The configuration can be loaded from environment variables using the `from_env()` method, which uses dotenv for configuration management.

## Integration Points

- **Storage Layer**: Uses database to map paths to content hashes
- **Processor**: Queries for changed files and their dependencies
- **WebDAV Server**: Uses database for authentication and path resolution

## Testing Approach

The `marble-db` crate uses a comprehensive testing strategy with a dedicated PostgreSQL 17 database:

### Test Environment

- Docker Compose-based PostgreSQL 17 instance for testing
- Isolated test database on port 5433 (separate from development)
- SQL query logging for debugging and performance analysis
- Automated setup script for test environment initialization

### Testing Utilities

```rust
// Planned testing utilities
mod testing {
    // Create a test database configuration
    pub fn test_config() -> DatabaseConfig { ... }
    
    // Create and initialize a test database connection
    pub async fn create_test_db() -> Database { ... }
    
    // Create standard test data
    pub async fn create_test_user(db: &Database) -> i64 { ... }
    
    // Run a test function with a database connection
    pub fn with_db<F, Fut, R>(test_fn: F) -> R { ... }
}
```

### Test Types

1. **Schema Tests**: Verify database structure matches expectations
2. **Query Tests**: Confirm SQL queries function correctly
3. **API Tests**: Test the public DatabaseApi interface
4. **Integration Tests**: End-to-end tests of database operations
5. **Performance Tests**: Benchmark critical database operations

### Snapshot Testing

Database schema changes will be verified using snapshot testing to ensure schema modifications are intentional and documented.

## Future Considerations

- Optimize indices for common query patterns
- Consider partitioning for multi-tenant scalability
- Implement efficient batch operations for bulk updates
- Design migration strategy for schema evolution
- Benchmark scaling with large dataset volumes
