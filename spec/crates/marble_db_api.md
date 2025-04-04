# Marble Database API Design

**Status: [PARTIALLY IMPLEMENTED]**

## Repository Pattern Implementation

The `marble-db` crate uses a trait-based repository pattern to provide clean abstractions over database operations. Each domain entity has a corresponding repository trait with implementations backed by SQLx.

## Core Traits

### Repository Creation
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
```

### Transaction Support
```rust
/// A trait for repositories that need to run transactions
#[async_trait::async_trait]
pub trait TransactionSupport {
    async fn begin_transaction(&self) -> Result<sqlx::Transaction<'static, sqlx::Postgres>>;
    async fn commit_transaction(transaction: sqlx::Transaction<'static, sqlx::Postgres>) -> Result<()>;
    async fn rollback_transaction(transaction: sqlx::Transaction<'static, sqlx::Postgres>) -> Result<()>;
}
```

## Implemented Repository Interfaces

### UserRepository
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

### FolderRepository
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

### FileRepository
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

## Database Struct

The main `Database` struct serves as a facade for all repositories:

```rust
/// Database implementation that wraps a connection pool
#[derive(Debug, Clone)]
pub struct Database {
    pool: Arc<PgPool>,
    user_repository: Arc<dyn UserRepository>,
    folder_repository: Arc<dyn FolderRepository>,
    file_repository: Arc<dyn FileRepository>,
}

impl Database {
    /// Create a new Database instance with the given connection pool
    pub fn new(pool: PgPool) -> Self { ... }
    
    /// Get a reference to the user repository
    pub fn users(&self) -> &dyn UserRepository { ... }
    
    /// Get a reference to the folder repository
    pub fn folders(&self) -> &dyn FolderRepository { ... }
    
    /// Get a reference to the file repository
    pub fn files(&self) -> &dyn FileRepository { ... }
}
```

## Configuration

```rust
/// Configuration for a database connection
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

## Future Repositories

The following repository traits are planned for future implementation:

```rust
// Planned additional traits
pub trait FrontmatterRepository: Repository + BaseRepository + Send + Sync { ... }
pub trait DocumentLinkRepository: Repository + BaseRepository + Send + Sync { ... }
pub trait FileVersionRepository: Repository + BaseRepository + Send + Sync { ... }
pub trait ProcessingQueueRepository: Repository + BaseRepository + Send + Sync { ... }
pub trait PublishedContentRepository: Repository + BaseRepository + Send + Sync { ... }
```

## Testing Infrastructure

```rust
// Testing utilities interface
pub mod testing {
    // Create a test database configuration
    pub fn test_config() -> DatabaseConfig { ... }
    
    // Create a test database connection
    pub async fn create_test_db() -> Database { ... }
    
    // Run migrations on a test database
    pub async fn run_test_migrations(pool: &PgPool) -> Result<()> { ... }
}
```
