# dotenv Specification

**Status:** DRAFT
**Last Updated:** 2025-04-03

## Overview

dotenv is a library for loading environment variables from a `.env` file at runtime. In Marble, it's used for configuration management, particularly for database connections and service credentials.

## Usage in Marble

- Loading database connection strings
- Setting environment-specific configuration
- Managing S3 credentials for storage
- Configuring server parameters

## Version

- Version: 0.15.0

## Basic Usage

```rust
// Load .env file at the start of the application
dotenv::dotenv().ok();

// Access environment variables
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

// PostgreSQL connection configuration from environment
let pg_config = PgConnectOptions::new()
    .host(&std::env::var("PG_HOST").unwrap_or_else(|_| "localhost".into()))
    .port(std::env::var("PG_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(5432))
    .username(&std::env::var("PG_USER").unwrap_or_else(|_| "postgres".into()))
    .password(&std::env::var("PG_PASS").unwrap_or_default())
    .database(&std::env::var("PG_DATABASE").unwrap_or_else(|_| "marble".into()));
```

## Example .env File

```
# Database configuration
DATABASE_URL=postgres://user:password@localhost:5432/marble
PG_HOST=localhost
PG_PORT=5432
PG_USER=marble_user
PG_PASS=secure_password
PG_DATABASE=marble

# S3 configuration
S3_BUCKET=marble-content
S3_REGION=us-west-2
S3_ACCESS_KEY=your_access_key
S3_SECRET_KEY=your_secret_key

# Server configuration
HOST=0.0.0.0
PORT=8080
LOG_LEVEL=info
```

## Related Specifications

- [Database Schema](../domain/database_schema.md)
- [marble_db Crate](../crates/marble_db.md)
