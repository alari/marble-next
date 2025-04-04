# Marble Database Schema - Current Implementation

**Status: [IMPLEMENTED]**

## Overview

This document describes the current PostgreSQL database schema implementation for Marble. The database serves as the metadata layer for the system, tracking file paths, content hashes, relationships, and processing status.

## Current Implementation

The current implementation includes the core tables that form the foundation of the database. Additional tables for content analysis, versioning, and publishing will be implemented in future iterations.

### Core Tables

#### `users`
Stores user authentication information.

| Column          | Type           | Description                                |
|-----------------|----------------|--------------------------------------------|
| id              | SERIAL         | Primary key                                |
| username        | VARCHAR(255)   | Unique username                            |
| password_hash   | VARCHAR(255)   | Securely stored password hash              |
| created_at      | TIMESTAMPTZ    | When the user was created                  |
| last_login      | TIMESTAMPTZ    | When the user last logged in               |

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login TIMESTAMPTZ
);

CREATE INDEX idx_users_username ON users(username);
```

#### `folders`
Tracks folder structure.

| Column          | Type           | Description                                |
|-----------------|----------------|--------------------------------------------|
| id              | SERIAL         | Primary key                                |
| user_id         | INTEGER        | Foreign key to users                       |
| path            | VARCHAR(1024)  | Folder path                                |
| parent_id       | INTEGER        | Foreign key to parent folder               |
| created_at      | TIMESTAMPTZ    | When the folder was created                |
| updated_at      | TIMESTAMPTZ    | When the folder was last updated           |
| is_deleted      | BOOLEAN        | Tombstone flag                             |

```sql
CREATE TABLE folders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    path VARCHAR(1024) NOT NULL,
    parent_id INTEGER REFERENCES folders(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(user_id, path)
);

CREATE INDEX idx_folders_user_path ON folders(user_id, path);
CREATE INDEX idx_folders_parent ON folders(parent_id);
CREATE INDEX idx_folders_user_deleted ON folders(user_id, is_deleted);
```

#### `files`
Tracks current state of each file.

| Column          | Type           | Description                                |
|-----------------|----------------|--------------------------------------------|
| id              | SERIAL         | Primary key                                |
| user_id         | INTEGER        | Foreign key to users                       |
| path            | VARCHAR(1024)  | File path in the vault                     |
| content_hash    | VARCHAR(64)    | Current content hash (links to S3)         |
| content_type    | VARCHAR(255)   | MIME type or file format                   |
| size            | INTEGER        | File size in bytes                         |
| created_at      | TIMESTAMPTZ    | When the file was first created            |
| updated_at      | TIMESTAMPTZ    | When the file was last updated             |
| is_deleted      | BOOLEAN        | Tombstone flag                             |

```sql
CREATE TABLE files (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    path VARCHAR(1024) NOT NULL,
    content_hash VARCHAR(64) NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    size INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(user_id, path)
);

CREATE INDEX idx_files_user_path ON files(user_id, path);
CREATE INDEX idx_files_content_hash ON files(content_hash);
CREATE INDEX idx_files_user_deleted ON files(user_id, is_deleted);
```

## Models Implementation

The database models have been implemented in Rust as follows:

### User Model

```rust
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}
```

Key methods:
- `new(username: String, password_hash: String) -> Self` - Create a new user
- `record_login(&mut self) -> &Self` - Update last login time

### Folder Model

```rust
pub struct Folder {
    pub id: i32,
    pub user_id: i32,
    pub path: String,
    pub parent_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}
```

Key methods:
- `new(user_id: i32, path: String, parent_id: Option<i32>) -> Self` - Create a new folder
- `folder_name(&self) -> &str` - Get folder name from path
- `parent_path(&self) -> Option<&str>` - Get parent path
- `mark_deleted(&mut self) -> &Self` - Mark folder as deleted
- `restore(&mut self) -> &Self` - Restore deleted folder

### File Model

```rust
pub struct File {
    pub id: i32,
    pub user_id: i32,
    pub path: String,
    pub content_hash: String,
    pub content_type: String,
    pub size: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}
```

Key methods:
- `new(user_id: i32, path: String, content_hash: String, content_type: String, size: i32) -> Self` - Create a new file
- `file_name(&self) -> &str` - Get file name from path
- `folder_path(&self) -> &str` - Get folder path
- `file_extension(&self) -> Option<&str>` - Get file extension
- `is_markdown(&self) -> bool` - Check if file is markdown
- `is_canvas(&self) -> bool` - Check if file is canvas
- `update_content(&mut self, content_hash: String, size: i32) -> &Self` - Update file content

## Repository Implementation

The repositories follow a trait-based design:

### Base Repositories

```rust
/// A trait for repositories that can be created from a database pool
pub trait Repository {
    /// Create a new repository instance
    fn new(pool: Arc<PgPool>) -> Self;
}

/// A trait for repositories that have a pool reference
pub trait BaseRepository {
    /// Get a reference to the database pool
    fn pool(&self) -> &PgPool;
}

/// A trait for repositories that need to run transactions
#[async_trait::async_trait]
pub trait TransactionSupport {
    async fn begin_transaction(&self) -> Result<sqlx::Transaction<'static, sqlx::Postgres>>;
    async fn commit_transaction(transaction: sqlx::Transaction<'static, sqlx::Postgres>) -> Result<()>;
    async fn rollback_transaction(transaction: sqlx::Transaction<'static, sqlx::Postgres>) -> Result<()>;
}
```

### Specific Repository Traits

#### UserRepository
```rust
#[async_trait::async_trait]
pub trait UserRepository: Repository + BaseRepository + Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<User>>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn create(&self, user: &User) -> Result<User>;
    async fn update(&self, user: &User) -> Result<User>;
    async fn delete(&self, id: i32) -> Result<bool>;
    async fn record_login(&self, id: i32) -> Result<bool>;
    async fn list(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<User>>;
}
```

#### FolderRepository
```rust
#[async_trait::async_trait]
pub trait FolderRepository: Repository + BaseRepository + Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<Folder>>;
    async fn find_by_path(&self, user_id: i32, path: &str) -> Result<Option<Folder>>;
    async fn list_by_user(&self, user_id: i32, parent_id: Option<i32>, include_deleted: bool) -> Result<Vec<Folder>>;
    async fn create(&self, folder: &Folder) -> Result<Folder>;
    async fn update(&self, folder: &Folder) -> Result<Folder>;
    async fn mark_deleted(&self, id: i32) -> Result<bool>;
    async fn restore(&self, id: i32) -> Result<bool>;
    async fn has_children(&self, id: i32, include_deleted: bool) -> Result<bool>;
    async fn get_children(&self, id: i32, include_deleted: bool) -> Result<Vec<Folder>>;
    async fn delete_permanently(&self, id: i32) -> Result<bool>;
}
```

#### FileRepository
```rust
#[async_trait::async_trait]
pub trait FileRepository: Repository + BaseRepository + Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<File>>;
    async fn find_by_path(&self, user_id: i32, path: &str) -> Result<Option<File>>;
    async fn find_by_content_hash(&self, content_hash: &str) -> Result<Vec<File>>;
    async fn list_by_folder_path(&self, user_id: i32, folder_path: &str, include_deleted: bool) -> Result<Vec<File>>;
    async fn create(&self, file: &File) -> Result<File>;
    async fn update(&self, file: &File) -> Result<File>;
    async fn mark_deleted(&self, id: i32) -> Result<bool>;
    async fn restore(&self, id: i32) -> Result<bool>;
    async fn delete_permanently(&self, id: i32) -> Result<bool>;
    async fn count_by_user(&self, user_id: i32, include_deleted: bool) -> Result<i64>;
    async fn find_markdown_files(&self, user_id: i32, include_deleted: bool) -> Result<Vec<File>>;
    async fn find_canvas_files(&self, user_id: i32, include_deleted: bool) -> Result<Vec<File>>;
}
```

## Testing

Testing infrastructure has been established with:
- Docker Compose setup for PostgreSQL 17
- Test database on port 5433
- Migration tests to verify schema
- Repository tests with real database interaction
- Test utilities for database operations

## Planned Future Tables

The following tables are planned for future implementation:

1. **Content Analysis Tables**:
   - `frontmatter`: For extracted metadata
   - `document_links`: For file relationships and navigation

2. **Processing Tables**:
   - `processing_queue`: For tracking background tasks
   - `published_content`: For managing published content
   - `cache_invalidations`: For efficient cache management

3. **Version Control Tables**:
   - `file_versions`: For tracking file history

## Related Specifications

- [Full Database Schema Specification](database_schema.md) - Complete database design
- [Marble Database Crate Specification](../crates/marble_db.md) - API design
- [Storage Architecture](storage_architecture.md) - Overall storage approach

## References

- [SQLx Documentation](https://github.com/launchbadge/sqlx)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/17/index.html)
