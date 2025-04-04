//! Database configuration
//!
//! This module provides configuration structures and utilities for database connections.

use std::env;
use std::time::Duration;

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

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://postgres:postgres@localhost:5432/marble".to_string(),
            max_connections: 5,
            acquire_timeout_seconds: 10,
            idle_timeout_seconds: 300,
            max_lifetime_seconds: 1800,
        }
    }
}

impl DatabaseConfig {
    /// Create a new DatabaseConfig from environment variables with default fallbacks
    pub fn from_env() -> Self {
        // Load environment variables from .env file if present
        dotenv::dotenv().ok();

        Self {
            url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/marble".to_string()
            }),
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            acquire_timeout_seconds: env::var("DATABASE_ACQUIRE_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            idle_timeout_seconds: env::var("DATABASE_IDLE_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
            max_lifetime_seconds: env::var("DATABASE_MAX_LIFETIME")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1800),
        }
    }

    /// Create a DatabaseConfig for testing
    pub fn for_test() -> Self {
        Self {
            url: "postgres://postgres:postgres@localhost:5432/marble_test".to_string(),
            max_connections: 2,
            acquire_timeout_seconds: 5,
            idle_timeout_seconds: 60,
            max_lifetime_seconds: 300,
        }
    }
}

/// Create timeouts from config values
pub(crate) fn get_timeouts(config: &DatabaseConfig) -> (Duration, Duration, Duration) {
    (
        Duration::from_secs(config.acquire_timeout_seconds),
        Duration::from_secs(config.idle_timeout_seconds),
        Duration::from_secs(config.max_lifetime_seconds),
    )
}
