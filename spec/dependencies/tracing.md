# Tracing Specification

**Status:** DRAFT
**Last Updated:** 2025-04-03

## Overview

Tracing is a framework for instrumenting Rust programs to collect structured, event-based diagnostic information. In Marble, it's used for logging, debugging, and performance monitoring.

## Usage in Marble

- Structured logging of database operations
- Performance monitoring for storage operations
- Error tracking and diagnostics
- Request/response logging for WebDAV server

## Version

- tracing: 0.1.41
- tracing-subscriber: 0.3.19 (with env-filter and json features)

## Basic Setup

```rust
// Initialize the tracing subscriber with env-filter
fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,marble_db=debug"));
    
    fmt::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .init();
}

// In main.rs or lib.rs
fn main() {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize tracing
    init_tracing();
    
    // Log events at various levels
    tracing::info!("Application starting up");
    tracing::debug!("Database connection established");
    
    // ...
}
```

## Instrumenting Database Operations

```rust
use tracing::{debug, error, info, instrument, warn};

#[instrument(skip(pool))]
async fn get_user_by_id(pool: &PgPool, user_id: i32) -> Result<User, Error> {
    debug!(user_id, "Fetching user from database");
    
    let result = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await?;
    
    match result {
        Some(user) => {
            info!(user_id, username = %user.username, "User found");
            Ok(user)
        },
        None => {
            warn!(user_id, "User not found");
            Err(Error::NotFound)
        }
    }
}
```

## Span Hierarchy for Context

```rust
async fn process_file(path: &str, content: &[u8]) -> Result<(), Error> {
    // Create a span for the entire operation
    let process_span = tracing::info_span!("process_file", %path, size_bytes = content.len());
    let _guard = process_span.enter();
    
    // Nested spans for sub-operations
    let hash = {
        let hash_span = tracing::debug_span!("calculate_hash");
        let _guard = hash_span.enter();
        calculate_hash(content).await?
    };
    
    // Another nested operation
    {
        let store_span = tracing::debug_span!("store_content", %hash);
        let _guard = store_span.enter();
        store_content(hash, content).await?
    }
    
    // Update metadata with the parent span still active
    update_metadata(path, hash).await?
    
    Ok(())
}
```

## Related Specifications

- [marble_db Crate](../crates/marble_db.md)
- [marble_storage Crate](../crates/marble_storage.md)
