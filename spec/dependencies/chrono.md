# Chrono Specification

**Status:** DRAFT
**Last Updated:** 2025-04-03

## Overview

Chrono is a comprehensive date and time library for Rust. In Marble, it's used for handling dates and times in database models, file metadata, and user activity tracking.

## Usage in Marble

- Timestamps for database records
- File creation and modification times
- User activity tracking
- Frontmatter date handling

## Version

- Version: 0.4.40 (with serde feature)

## Basic Usage

```rust
use chrono::{DateTime, Utc, TimeZone, NaiveDate};

// Current time in UTC
let now: DateTime<Utc> = Utc::now();

// Create from timestamp
let timestamp = 1712175000;
let date_time = Utc.timestamp_opt(timestamp, 0).unwrap();

// Format dates
let formatted = now.format("%Y-%m-%d %H:%M:%S").to_string();

// Parse date string
let date_str = "2025-04-01";
let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();

// Date math
let tomorrow = now + chrono::Duration::days(1);
let one_week_ago = now - chrono::Duration::weeks(1);

// PostgreSQL integration with SQLx
async fn get_recently_modified_files(pool: &PgPool, since: DateTime<Utc>) -> Result<Vec<File>, Error> {
    sqlx::query_as!(
        File,
        "SELECT * FROM files WHERE updated_at > $1 ORDER BY updated_at DESC",
        since
    )
    .fetch_all(pool)
    .await
    .map_err(Error::Database)
}

// Serde integration
#[derive(Serialize, Deserialize)]
struct FileMetadata {
    path: String,
    size: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

## PostgreSQL Type Mapping

Chrono types map directly to PostgreSQL timestamp types:

| Chrono Type | PostgreSQL Type |
|-------------|----------------|
| `DateTime<Utc>` | TIMESTAMP WITH TIME ZONE |
| `NaiveDateTime` | TIMESTAMP |
| `NaiveDate` | DATE |
| `NaiveTime` | TIME |

## Related Specifications

- [Database Schema](../domain/database_schema.md)
- [marble_core Crate](../crates/marble_core.md)
- [marble_db Crate](../crates/marble_db.md)
