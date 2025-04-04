# Marble Database Overview

**Status: [PARTIALLY IMPLEMENTED]**

## Overview

The `marble-db` crate manages the PostgreSQL database layer for Marble, providing the metadata storage that complements the S3-based content storage. It enables multi-tenant data isolation, efficient querying, and relationship tracking between content.

## Core Responsibilities

- Define and manage the database schema
- Provide typed query interfaces for common operations
- Track file metadata, paths, and relationships
- Support user authentication and tenant isolation
- Enable incremental processing through dependency tracking
- Manage content versioning and publishing status

## Key Concepts

- **Repository Pattern**: Trait-based interfaces for database operations
- **Multi-tenant Isolation**: Data separation using user_id foreign keys
- **Path-based Organization**: File and folder organization through paths
- **Content-addressable Storage**: Hash-based content references
- **Transaction Support**: Atomic operations across multiple tables

## API Structure

The primary API is organized around:

1. **Database Configuration**: Connection parameters and pool settings
2. **Core DatabaseApi Trait**: Connection, initialization, and health checks
3. **Repository Traits**: User, File, and Folder operations
4. **Models**: Strongly-typed data structures representing database entities

### Main Interfaces

```rust
// Primary connection and management
pub trait DatabaseApi: Send + Sync + 'static {
    async fn initialize(&self) -> Result<()>;
    fn pool(&self) -> &PgPool;
    async fn health_check(&self) -> Result<()>;
}

// Core repositories
pub trait UserRepository: Repository + BaseRepository + Send + Sync { ... }
pub trait FolderRepository: Repository + BaseRepository + Send + Sync { ... }
pub trait FileRepository: Repository + BaseRepository + Send + Sync { ... }
```

## Implementation Status

### Implemented
- ✅ Database connection and configuration
- ✅ Core models (User, Folder, File)
- ✅ Base repository interfaces and implementations
- ✅ Transaction support
- ✅ Testing infrastructure

### Pending
- ⏳ Advanced model repositories (frontmatter, document links)
- ⏳ Processing queue and publication tracking
- ⏳ Caching and batch operations

## Related Documentation

- [Database Schema](../domain/database_schema.md) - Full schema specification
- [Current Schema Implementation](../domain/database_schema_current.md) - Implemented tables
- [Schema Implementation Handoff](../handoffs/database_schema_implementation.md)
- [Database API Design](marble_db_api.md) - Detailed API interfaces
