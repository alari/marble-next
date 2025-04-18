//! Database schema and operations for Marble
//!
//! This crate provides database functionality for the Marble project,
//! including schema migrations, database connections, and query operations.

use sqlx::postgres::{PgPool, PgPoolOptions};

pub mod error;
pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

// Re-export database error as DatabaseError for compatibility
pub type DatabaseError = error::Error;

// Authentication module
pub mod auth;
pub use auth::{AuthService, DatabaseAuthService, AuthError, AuthResult};

// Make PgPool public so it can be used in other crates

pub mod api;
pub mod config;
pub mod models;
pub mod repositories;

#[cfg(test)]
mod tests;

pub use api::{Database, DatabaseApi};
pub use config::DatabaseConfig;

/// Static migrator for database schema migrations
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

/// Create a new database connection pool
pub async fn create_pool(config: DatabaseConfig) -> Result<PgPool> {
    let (acquire_timeout, idle_timeout, max_lifetime) = config::get_timeouts(&config);

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(acquire_timeout)
        .idle_timeout(idle_timeout)
        .max_lifetime(max_lifetime)
        .connect(&config.url)
        .await
        .map_err(Error::ConnectionFailed)?;

    tracing::info!("Connected to database");
    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    tracing::info!("Running database migrations");
    MIGRATOR
        .run(pool)
        .await
        .map_err(Error::MigrationFailed)?;
    tracing::info!("Database migrations complete");
    Ok(())
}

/// Create and initialize a new Database instance
pub async fn connect(config: DatabaseConfig) -> Result<Database> {
    let pool = create_pool(config).await?;
    let db = Database::new(pool);
    db.initialize().await?;
    Ok(db)
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(
            config.url,
            "postgres://postgres:postgres@localhost:5432/marble"
        );
        assert_eq!(config.max_connections, 5);
        assert_eq!(config.acquire_timeout_seconds, 10);
        assert_eq!(config.idle_timeout_seconds, 300);
        assert_eq!(config.max_lifetime_seconds, 1800);
    }
}
