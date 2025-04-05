//! User model representing authenticated users
//!
//! This module defines the User struct and related functionality.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

/// Represents a user in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Primary key
    pub id: i32,
    /// Unique UUID for tenant identification
    pub uuid: Uuid,
    /// Unique username for identification and login
    pub username: String,
    /// Securely hashed password
    pub password_hash: String,
    /// When the user was created
    pub created_at: DateTime<Utc>,
    /// Most recent login timestamp, if any
    pub last_login: Option<DateTime<Utc>>,
}

impl User {
    /// Create a new user with default timestamps
    pub fn new(username: String, password_hash: String) -> Self {
        Self {
            id: 0, // Will be assigned by database
            uuid: Uuid::new_v4(),
            username,
            password_hash,
            created_at: Utc::now(),
            last_login: None,
        }
    }

    /// Record a login for this user
    pub fn record_login(&mut self) {
        self.last_login = Some(Utc::now());
    }

    /// Get the time elapsed since the last login, if any
    pub fn time_since_last_login(&self) -> Option<chrono::Duration> {
        self.last_login.map(|last| Utc::now() - last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_user() {
        let user = User::new("testuser".to_string(), "passwordhash".to_string());
        assert_eq!(user.id, 0);
        assert_eq!(user.username, "testuser");
        assert_eq!(user.password_hash, "passwordhash");
        assert!(user.last_login.is_none());
    }

    #[test]
    fn test_record_login() {
        let mut user = User::new("testuser".to_string(), "passwordhash".to_string());
        assert!(user.last_login.is_none());
        
        user.record_login();
        assert!(user.last_login.is_some());
        
        let time_since = user.time_since_last_login().unwrap();
        assert!(time_since.num_seconds() >= 0);
    }
}
