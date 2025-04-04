//! Repository for folder operations
//!
//! This module provides the FolderRepository trait and its SQLx implementation.

use sqlx::postgres::{PgPool, PgRow};
use sqlx::{FromRow, Row};
use std::sync::Arc;
use async_trait::async_trait;

use crate::models::Folder;
use crate::Result;
use crate::Error;
use super::{Repository, BaseRepository};

/// Repository trait for folder operations
#[async_trait]
pub trait FolderRepository: Repository + BaseRepository + Send + Sync {
    /// Find a folder by ID
    async fn find_by_id(&self, id: i32) -> Result<Option<Folder>>;
    
    /// Find a folder by user ID and path
    async fn find_by_path(&self, user_id: i32, path: &str) -> Result<Option<Folder>>;
    
    /// List folders for a user (optionally with a parent ID)
    async fn list_by_user(
        &self, 
        user_id: i32, 
        parent_id: Option<i32>, 
        include_deleted: bool
    ) -> Result<Vec<Folder>>;
    
    /// Create a new folder
    async fn create(&self, folder: &Folder) -> Result<Folder>;
    
    /// Update an existing folder
    async fn update(&self, folder: &Folder) -> Result<Folder>;
    
    /// Mark a folder as deleted
    async fn mark_deleted(&self, id: i32) -> Result<bool>;
    
    /// Restore a deleted folder
    async fn restore(&self, id: i32) -> Result<bool>;
    
    /// Check if a folder has children
    async fn has_children(&self, id: i32, include_deleted: bool) -> Result<bool>;
    
    /// Get a folder's children
    async fn get_children(&self, id: i32, include_deleted: bool) -> Result<Vec<Folder>>;
    
    /// Delete a folder permanently (use with caution)
    async fn delete_permanently(&self, id: i32) -> Result<bool>;
}

/// SQLx implementation of the FolderRepository
pub struct SqlxFolderRepository {
    pool: Arc<PgPool>,
}

impl Repository for SqlxFolderRepository {
    fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

impl BaseRepository for SqlxFolderRepository {
    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl FromRow<'_, PgRow> for Folder {
    fn from_row(row: &PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Folder {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            path: row.try_get("path")?,
            parent_id: row.try_get("parent_id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            is_deleted: row.try_get("is_deleted")?,
        })
    }
}

#[async_trait]
impl FolderRepository for SqlxFolderRepository {
    async fn find_by_id(&self, id: i32) -> Result<Option<Folder>> {
        let folder = sqlx::query_as::<_, Folder>(
            "SELECT id, user_id, path, parent_id, created_at, updated_at, is_deleted 
             FROM folders 
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(folder)
    }
    
    async fn find_by_path(&self, user_id: i32, path: &str) -> Result<Option<Folder>> {
        let folder = sqlx::query_as::<_, Folder>(
            "SELECT id, user_id, path, parent_id, created_at, updated_at, is_deleted 
             FROM folders 
             WHERE user_id = $1 AND path = $2"
        )
        .bind(user_id)
        .bind(path)
        .fetch_optional(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(folder)
    }
    
    async fn list_by_user(
        &self, 
        user_id: i32, 
        parent_id: Option<i32>, 
        include_deleted: bool
    ) -> Result<Vec<Folder>> {
        let mut query = String::from(
            "SELECT id, user_id, path, parent_id, created_at, updated_at, is_deleted 
             FROM folders 
             WHERE user_id = $1 "
        );
        
        if let Some(_) = parent_id {
            query.push_str("AND parent_id = $2 ");
        } else {
            query.push_str("AND parent_id IS NULL ");
        }
        
        if !include_deleted {
            query.push_str("AND is_deleted = false ");
        }
        
        query.push_str("ORDER BY path");
        
        let mut q = sqlx::query_as::<_, Folder>(&query)
            .bind(user_id);
        
        if let Some(parent_id) = parent_id {
            q = q.bind(parent_id);
        }
        
        let folders = q.fetch_all(self.pool())
            .await
            .map_err(Error::QueryFailed)?;
        
        Ok(folders)
    }
    
    async fn create(&self, folder: &Folder) -> Result<Folder> {
        let now = chrono::Utc::now();
        let created_folder = sqlx::query_as::<_, Folder>(
            "INSERT INTO folders (user_id, path, parent_id, created_at, updated_at, is_deleted) 
             VALUES ($1, $2, $3, $4, $5, $6) 
             RETURNING id, user_id, path, parent_id, created_at, updated_at, is_deleted"
        )
        .bind(folder.user_id)
        .bind(&folder.path)
        .bind(folder.parent_id)
        .bind(now)
        .bind(now)
        .bind(folder.is_deleted)
        .fetch_one(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(created_folder)
    }
    
    async fn update(&self, folder: &Folder) -> Result<Folder> {
        let now = chrono::Utc::now();
        let updated_folder = sqlx::query_as::<_, Folder>(
            "UPDATE folders 
             SET path = $1, parent_id = $2, updated_at = $3, is_deleted = $4 
             WHERE id = $5 
             RETURNING id, user_id, path, parent_id, created_at, updated_at, is_deleted"
        )
        .bind(&folder.path)
        .bind(folder.parent_id)
        .bind(now)
        .bind(folder.is_deleted)
        .bind(folder.id)
        .fetch_one(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(updated_folder)
    }
    
    async fn mark_deleted(&self, id: i32) -> Result<bool> {
        let now = chrono::Utc::now();
        let result = sqlx::query(
            "UPDATE folders 
             SET is_deleted = true, updated_at = $1 
             WHERE id = $2"
        )
        .bind(now)
        .bind(id)
        .execute(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn restore(&self, id: i32) -> Result<bool> {
        let now = chrono::Utc::now();
        let result = sqlx::query(
            "UPDATE folders 
             SET is_deleted = false, updated_at = $1 
             WHERE id = $2"
        )
        .bind(now)
        .bind(id)
        .execute(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn has_children(&self, id: i32, include_deleted: bool) -> Result<bool> {
        let mut query = String::from(
            "SELECT COUNT(*) > 0 as has_children
             FROM folders 
             WHERE parent_id = $1 "
        );
        
        if !include_deleted {
            query.push_str("AND is_deleted = false");
        }
        
        let has_children: bool = sqlx::query_scalar(&query)
            .bind(id)
            .fetch_one(self.pool())
            .await
            .map_err(Error::QueryFailed)?;
        
        Ok(has_children)
    }
    
    async fn get_children(&self, id: i32, include_deleted: bool) -> Result<Vec<Folder>> {
        let mut query = String::from(
            "SELECT id, user_id, path, parent_id, created_at, updated_at, is_deleted 
             FROM folders 
             WHERE parent_id = $1 "
        );
        
        if !include_deleted {
            query.push_str("AND is_deleted = false ");
        }
        
        query.push_str("ORDER BY path");
        
        let children = sqlx::query_as::<_, Folder>(&query)
            .bind(id)
            .fetch_all(self.pool())
            .await
            .map_err(Error::QueryFailed)?;
        
        Ok(children)
    }
    
    async fn delete_permanently(&self, id: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM folders WHERE id = $1")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(Error::QueryFailed)?;
            
        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    
    async fn create_test_pool() -> Result<PgPool> {
        // This should be skipped if no test database is available
        let db_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5433/marble_test".to_string());
        
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&db_url)
            .await
            .map_err(Error::ConnectionFailed)?;
            
        Ok(pool)
    }
    
    async fn setup_test_user(pool: &PgPool) -> Result<i32> {
        // Create a test user first
        let user_id: i32 = sqlx::query_scalar(
            "INSERT INTO users (username, password_hash, created_at) 
             VALUES ($1, $2, $3) 
             RETURNING id"
        )
        .bind("folder_test_user")
        .bind("test_password_hash")
        .bind(chrono::Utc::now())
        .fetch_one(pool)
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(user_id)
    }
    
    #[tokio::test]
    async fn test_folder_repository() {
        let pool = match create_test_pool().await {
            Ok(pool) => Arc::new(pool),
            Err(_) => {
                println!("Skipping repository test - no test database available");
                return;
            }
        };
        
        // Clear the folders table
        let _ = sqlx::query("DELETE FROM folders").execute(&*pool).await;
        let _ = sqlx::query("DELETE FROM users WHERE username = 'folder_test_user'").execute(&*pool).await;
        
        // Create a test user
        let user_id = match setup_test_user(&pool).await {
            Ok(id) => id,
            Err(_) => {
                println!("Failed to create test user");
                return;
            }
        };
        
        let repo = SqlxFolderRepository::new(pool);
        
        // Test creating a root folder
        let root_folder = Folder::new(user_id, "/".to_string(), None);
        let created_root = repo.create(&root_folder).await.unwrap();
        
        assert!(created_root.id > 0);
        assert_eq!(created_root.path, "/");
        assert!(created_root.parent_id.is_none());
        
        // Test creating a child folder
        let docs_folder = Folder::new(user_id, "/documents".to_string(), Some(created_root.id));
        let created_docs = repo.create(&docs_folder).await.unwrap();
        
        assert!(created_docs.id > 0);
        assert_eq!(created_docs.path, "/documents");
        assert_eq!(created_docs.parent_id, Some(created_root.id));
        
        // Test finding folder by ID
        let found = repo.find_by_id(created_docs.id).await.unwrap().unwrap();
        assert_eq!(found.id, created_docs.id);
        assert_eq!(found.path, "/documents");
        
        // Test finding folder by path
        let found = repo.find_by_path(user_id, "/documents").await.unwrap().unwrap();
        assert_eq!(found.id, created_docs.id);
        
        // Test listing folders by user and parent
        let root_children = repo.list_by_user(user_id, Some(created_root.id), false).await.unwrap();
        assert_eq!(root_children.len(), 1);
        assert_eq!(root_children[0].id, created_docs.id);
        
        // Test has_children
        let has_children = repo.has_children(created_root.id, false).await.unwrap();
        assert!(has_children);
        
        let has_children = repo.has_children(created_docs.id, false).await.unwrap();
        assert!(!has_children);
        
        // Test get_children
        let children = repo.get_children(created_root.id, false).await.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, created_docs.id);
        
        // Test marking as deleted
        let result = repo.mark_deleted(created_docs.id).await.unwrap();
        assert!(result);
        
        let found = repo.find_by_id(created_docs.id).await.unwrap().unwrap();
        assert!(found.is_deleted);
        
        // Test restoring
        let result = repo.restore(created_docs.id).await.unwrap();
        assert!(result);
        
        let found = repo.find_by_id(created_docs.id).await.unwrap().unwrap();
        assert!(!found.is_deleted);
        
        // Test permanent deletion
        let result = repo.delete_permanently(created_docs.id).await.unwrap();
        assert!(result);
        
        let not_found = repo.find_by_id(created_docs.id).await.unwrap();
        assert!(not_found.is_none());
        
        // Clean up
        let _ = repo.delete_permanently(created_root.id).await;
        let _ = sqlx::query("DELETE FROM users WHERE id = $1").bind(user_id).execute(repo.pool()).await;
    }
}
