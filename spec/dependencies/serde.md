# Serde Specification

**Status:** DRAFT
**Last Updated:** 2025-04-03

## Overview

Serde is a framework for serializing and deserializing Rust data structures efficiently and generically. In Marble, it's used for working with structured data formats, especially JSON for database storage and configurations.

## Usage in Marble

- Serializing and deserializing database models
- Handling JSON data in PostgreSQL
- Working with configuration files
- Implementing frontmatter parsing

## Version

- serde: 1.0.219 (with derive feature)
- serde_json: 1.0.140

## Basic Usage

```rust
use serde::{Serialize, Deserialize};

// Simple struct with serde
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub active: bool,
    
    #[serde(default)]
    pub settings: UserSettings,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserSettings {
    #[serde(default)]
    pub theme: String,
    
    #[serde(default)]
    pub notifications_enabled: bool,
}

// Working with JSON
fn process_user_json(json_data: &str) -> Result<User, serde_json::Error> {
    let user: User = serde_json::from_str(json_data)?;
    
    // Process user...
    
    Ok(user)
}

// Serializing to JSON
fn user_to_json(user: &User) -> Result<String, serde_json::Error> {
    let json = serde_json::to_string(user)?;
    Ok(json)
}

// Working with JSON in database
async fn store_user_settings(pool: &PgPool, user_id: i32, settings: &UserSettings) -> Result<(), Error> {
    let settings_json = serde_json::to_value(settings)?;
    
    sqlx::query!(
        "UPDATE users SET settings = $1 WHERE id = $2",
        settings_json,
        user_id
    )
    .execute(pool)
    .await?;
    
    Ok(())
}
```

## Related Specifications

- [Database Schema](../domain/database_schema.md)
- [marble_core Crate](../crates/marble_core.md)
- [marble_db Crate](../crates/marble_db.md)
