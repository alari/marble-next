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

## Version and Features

- Version: 0.8.3
- Features:
  - `runtime-tokio`: For async operation with Tokio
  - `tls-rustls`: TLS support using Rustls
  - `postgres`: PostgreSQL driver
  - `json`: JSON type support
  - `time`: Timestamp and date handling
  - `uuid`: UUID type support
  - `migrate`: Database migrations

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

## Related Specifications

- [Database Schema](../domain/database_schema.md)
- [marble_db Crate](../crates/marble_db.md)
