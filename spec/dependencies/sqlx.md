# SQLx Specification

**Status:** DRAFT
**Last Updated:** 2025-04-03

## Overview

SQLx is an async, pure Rust SQL toolkit with compile-time checked queries. It's used in Marble for database operations with PostgreSQL.

## Usage in Marble

- Database schema operations
- Type-safe SQL queries
- Migrations for schema evolution
- Transaction management
- Custom PostgreSQL type mapping

## Version and Features

- Version: 0.8.3
- Core Features:
  - `runtime-tokio`: For async operation with Tokio
  - `tls-rustls`: TLS support using Rustls
  - `postgres`: PostgreSQL driver
  - `json`: JSON type support
  - `time`: Timestamp and date handling
  - `uuid`: UUID type support
  - `migrate`: Database migrations
  - `macros`: Compile-time checked queries

- PostgreSQL Type Mapping Features:
  - `chrono`: DateTime and other time types
  - `ipnetwork`: IP network types
  - `bit-vec`: Bit string types

## Basic Usage

```rust
// Query with compile-time checking
let users = sqlx::query_as!(User, "SELECT * FROM users WHERE active = $1", true)
    .fetch_all(&pool)
    .await?;

// Transactions
let mut tx = pool.begin().await?;
sqlx::query!("INSERT INTO files (user_id, path, content_hash) VALUES ($1, $2, $3)",
    user_id, path, content_hash)
    .execute(&mut *tx)
    .await?;
tx.commit().await?;

// Migrations
sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;
```

## Custom Type Mapping

SQLx supports mapping between Rust types and PostgreSQL types:

```rust
use sqlx::postgres::PgHasArrayType;
use sqlx::Type;

// Enum with PostgreSQL type mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "file_status", rename_all = "snake_case")]
pub enum FileStatus {
    Active,
    Deleted,
    Processing,
}

// Struct with PostgreSQL type mapping
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Document {
    pub id: i32,
    pub title: String,
    pub tags: Vec<String>,              // maps to text[]
    pub metadata: serde_json::Value,    // maps to jsonb
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// Implementing array type support for custom types
impl PgHasArrayType for FileStatus {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_file_status")
    }
}
```

## Related Specifications

- [Database Schema](../domain/database_schema.md)
- [marble_db Crate](../crates/marble_db.md)
