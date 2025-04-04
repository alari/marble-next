//! Repository for file operations
//!
//! This module provides the FileRepository trait and its SQLx implementation.

use sqlx::postgres::{PgPool, PgRow};
use sqlx::{FromRow, Row};
use std::sync::Arc;
use async_trait::async_trait;

use crate::models::File;
use crate::Result;
use crate::Error;
use super::{Repository, BaseRepository};

/// Repository trait for file operations
#[async_trait]
pub trait FileRepository: Repository + BaseRepository + Send + Sync {
    /// Find a file by ID
    async fn find_by_id(&self, id: i32) -> Result<Option<File>>;
    
    /// Find a file by user ID and path
    async fn find_by_path(&self, user_id: i32, path: &str) -> Result<Option<File>>;
    
    /// Find files by content hash
    async fn find_by_content_hash(&self, content_hash: &str) -> Result<Vec<File>>;
    
    /// List files in a folder path for a user
    async fn list_by_folder_path(
        &self, 
        user_id: i32, 
        folder_path: &str, 
        include_deleted: bool
    ) -> Result<Vec<File>>;
    
    /// Create a new file
    async fn create(&self, file: &File) -> Result<File>;
    
    /// Update an existing file
    async fn update(&self, file: &File) -> Result<File>;
    
    /// Mark a file as deleted
    async fn mark_deleted(&self, id: i32) -> Result<bool>;
    
    /// Restore a deleted file
    async fn restore(&self, id: i32) -> Result<bool>;
    
    /// Delete a file permanently (use with caution)
    async fn delete_permanently(&self, id: i32) -> Result<bool>;
    
    /// Count files by user ID
    async fn count_by_user(&self, user_id: i32, include_deleted: bool) -> Result<i64>;
    
    /// Find all markdown files for a user
    async fn find_markdown_files(&self, user_id: i32, include_deleted: bool) -> Result<Vec<File>>;
    
    /// Find all canvas files for a user
    async fn find_canvas_files(&self, user_id: i32, include_deleted: bool) -> Result<Vec<File>>;
}

/// SQLx implementation of the FileRepository
pub struct SqlxFileRepository {
    pool: Arc<PgPool>,
}

impl Repository for SqlxFileRepository {
    fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

impl BaseRepository for SqlxFileRepository {
    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl FromRow<'_, PgRow> for File {
    fn from_row(row: &PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(File {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            path: row.try_get("path")?,
            content_hash: row.try_get("content_hash")?,
            content_type: row.try_get("content_type")?,
            size: row.try_get("size")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            is_deleted: row.try_get("is_deleted")?,
        })
    }
}

#[async_trait]
impl FileRepository for SqlxFileRepository {
    async fn find_by_id(&self, id: i32) -> Result<Option<File>> {
        let file = sqlx::query_as::<_, File>(
            "SELECT id, user_id, path, content_hash, content_type, size, created_at, updated_at, is_deleted 
             FROM files 
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(file)
    }
    
    async fn find_by_path(&self, user_id: i32, path: &str) -> Result<Option<File>> {
        let file = sqlx::query_as::<_, File>(
            "SELECT id, user_id, path, content_hash, content_type, size, created_at, updated_at, is_deleted 
             FROM files 
             WHERE user_id = $1 AND path = $2"
        )
        .bind(user_id)
        .bind(path)
        .fetch_optional(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(file)
    }
    
    async fn find_by_content_hash(&self, content_hash: &str) -> Result<Vec<File>> {
        let files = sqlx::query_as::<_, File>(
            "SELECT id, user_id, path, content_hash, content_type, size, created_at, updated_at, is_deleted 
             FROM files 
             WHERE content_hash = $1"
        )
        .bind(content_hash)
        .fetch_all(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(files)
    }
    
    async fn list_by_folder_path(
        &self, 
        user_id: i32, 
        folder_path: &str, 
        include_deleted: bool
    ) -> Result<Vec<File>> {
        let path_pattern = if folder_path.ends_with('/') {
            format!("{}%", folder_path)
        } else {
            format!("{}/%", folder_path)
        };
        
        let mut query = String::from(
            "SELECT id, user_id, path, content_hash, content_type, size, created_at, updated_at, is_deleted 
             FROM files 
             WHERE user_id = $1 AND path LIKE $2 "
        );
        
        if !include_deleted {
            query.push_str("AND is_deleted = false ");
        }
        
        query.push_str("ORDER BY path");
        
        let files = sqlx::query_as::<_, File>(&query)
            .bind(user_id)
            .bind(path_pattern)
            .fetch_all(self.pool())
            .await
            .map_err(Error::QueryFailed)?;
        
        Ok(files)
    }
    
    async fn create(&self, file: &File) -> Result<File> {
        let now = chrono::Utc::now();
        let created_file = sqlx::query_as::<_, File>(
            "INSERT INTO files (user_id, path, content_hash, content_type, size, created_at, updated_at, is_deleted) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
             RETURNING id, user_id, path, content_hash, content_type, size, created_at, updated_at, is_deleted"
        )
        .bind(file.user_id)
        .bind(&file.path)
        .bind(&file.content_hash)
        .bind(&file.content_type)
        .bind(file.size)
        .bind(now)
        .bind(now)
        .bind(file.is_deleted)
        .fetch_one(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(created_file)
    }
    
    async fn update(&self, file: &File) -> Result<File> {
        let now = chrono::Utc::now();
        let updated_file = sqlx::query_as::<_, File>(
            "UPDATE files 
             SET path = $1, content_hash = $2, content_type = $3, size = $4, updated_at = $5, is_deleted = $6 
             WHERE id = $7 
             RETURNING id, user_id, path, content_hash, content_type, size, created_at, updated_at, is_deleted"
        )
        .bind(&file.path)
        .bind(&file.content_hash)
        .bind(&file.content_type)
        .bind(file.size)
        .bind(now)
        .bind(file.is_deleted)
        .bind(file.id)
        .fetch_one(self.pool())
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(updated_file)
    }
    
    async fn mark_deleted(&self, id: i32) -> Result<bool> {
        let now = chrono::Utc::now();
        let result = sqlx::query(
            "UPDATE files 
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
            "UPDATE files 
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
    
    async fn delete_permanently(&self, id: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM files WHERE id = $1")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(Error::QueryFailed)?;
            
        Ok(result.rows_affected() > 0)
    }
    
    async fn count_by_user(&self, user_id: i32, include_deleted: bool) -> Result<i64> {
        let query = if include_deleted {
            "SELECT COUNT(*) FROM files WHERE user_id = $1"
        } else {
            "SELECT COUNT(*) FROM files WHERE user_id = $1 AND is_deleted = false"
        };
        
        let count: i64 = sqlx::query_scalar(query)
            .bind(user_id)
            .fetch_one(self.pool())
            .await
            .map_err(Error::QueryFailed)?;
        
        Ok(count)
    }
    
    async fn find_markdown_files(&self, user_id: i32, include_deleted: bool) -> Result<Vec<File>> {
        let mut query = String::from(
            "SELECT id, user_id, path, content_hash, content_type, size, created_at, updated_at, is_deleted 
             FROM files 
             WHERE user_id = $1 
             AND (content_type = 'text/markdown' OR path LIKE '%.md' OR path LIKE '%.markdown') "
        );
        
        if !include_deleted {
            query.push_str("AND is_deleted = false ");
        }
        
        query.push_str("ORDER BY path");
        
        let files = sqlx::query_as::<_, File>(&query)
            .bind(user_id)
            .fetch_all(self.pool())
            .await
            .map_err(Error::QueryFailed)?;
        
        Ok(files)
    }
    
    async fn find_canvas_files(&self, user_id: i32, include_deleted: bool) -> Result<Vec<File>> {
        let mut query = String::from(
            "SELECT id, user_id, path, content_hash, content_type, size, created_at, updated_at, is_deleted 
             FROM files 
             WHERE user_id = $1 
             AND (content_type = 'application/obsidian-canvas' OR path LIKE '%.canvas') "
        );
        
        if !include_deleted {
            query.push_str("AND is_deleted = false ");
        }
        
        query.push_str("ORDER BY path");
        
        let files = sqlx::query_as::<_, File>(&query)
            .bind(user_id)
            .fetch_all(self.pool())
            .await
            .map_err(Error::QueryFailed)?;
        
        Ok(files)
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
        .bind("file_test_user")
        .bind("test_password_hash")
        .bind(chrono::Utc::now())
        .fetch_one(pool)
        .await
        .map_err(Error::QueryFailed)?;
        
        Ok(user_id)
    }
    
    #[tokio::test]
    async fn test_file_repository() {
        let pool = match create_test_pool().await {
            Ok(pool) => Arc::new(pool),
            Err(_) => {
                println!("Skipping repository test - no test database available");
                return;
            }
        };
        
        // Clear the files and users table
        let _ = sqlx::query("DELETE FROM files").execute(&*pool).await;
        let _ = sqlx::query("DELETE FROM users WHERE username = 'file_test_user'").execute(&*pool).await;
        
        // Create a test user
        let user_id = match setup_test_user(&pool).await {
            Ok(id) => id,
            Err(_) => {
                println!("Failed to create test user");
                return;
            }
        };
        
        let repo = SqlxFileRepository::new(pool);
        
        // Test creating a file
        let file = File::new(
            user_id,
            "/notes.md".to_string(),
            "abc123".to_string(),
            "text/markdown".to_string(),
            1024
        );
        
        let created_file = repo.create(&file).await.unwrap();
        
        assert!(created_file.id > 0);
        assert_eq!(created_file.path, "/notes.md");
        assert_eq!(created_file.content_hash, "abc123");
        
        // Test finding file by ID
        let found = repo.find_by_id(created_file.id).await.unwrap().unwrap();
        assert_eq!(found.id, created_file.id);
        assert_eq!(found.path, "/notes.md");
        
        // Test finding file by path
        let found = repo.find_by_path(user_id, "/notes.md").await.unwrap().unwrap();
        assert_eq!(found.id, created_file.id);
        
        // Test finding by content hash
        let files = repo.find_by_content_hash("abc123").await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].id, created_file.id);
        
        // Create a canvas file
        let canvas_file = File::new(
            user_id,
            "/diagram.canvas".to_string(),
            "def456".to_string(),
            "application/obsidian-canvas".to_string(),
            2048
        );
        
        let created_canvas = repo.create(&canvas_file).await.unwrap();
        
        // Test listing by folder
        let files = repo.list_by_folder_path(user_id, "/", false).await.unwrap();
        assert_eq!(files.len(), 2);
        
        // Test counting
        let count = repo.count_by_user(user_id, false).await.unwrap();
        assert_eq!(count, 2);
        
        // Test finding markdown files
        let md_files = repo.find_markdown_files(user_id, false).await.unwrap();
        assert_eq!(md_files.len(), 1);
        assert_eq!(md_files[0].id, created_file.id);
        
        // Test finding canvas files
        let canvas_files = repo.find_canvas_files(user_id, false).await.unwrap();
        assert_eq!(canvas_files.len(), 1);
        assert_eq!(canvas_files[0].id, created_canvas.id);
        
        // Test updating
        let mut file_to_update = found;
        file_to_update.content_hash = "updated-hash".to_string();
        file_to_update.size = 2048;
        
        let updated = repo.update(&file_to_update).await.unwrap();
        
        assert_eq!(updated.content_hash, "updated-hash");
        assert_eq!(updated.size, 2048);
        
        // Test marking as deleted
        let result = repo.mark_deleted(created_file.id).await.unwrap();
        assert!(result);
        
        let found = repo.find_by_id(created_file.id).await.unwrap().unwrap();
        assert!(found.is_deleted);
        
        // Test counting with deleted files
        let count_with_deleted = repo.count_by_user(user_id, true).await.unwrap();
        assert_eq!(count_with_deleted, 2);
        
        let count_without_deleted = repo.count_by_user(user_id, false).await.unwrap();
        assert_eq!(count_without_deleted, 1); // Only the canvas file should be counted
        
        // Test restoring
        let result = repo.restore(created_file.id).await.unwrap();
        assert!(result);
        
        let found = repo.find_by_id(created_file.id).await.unwrap().unwrap();
        assert!(!found.is_deleted);
        
        // Test permanent deletion
        let result = repo.delete_permanently(created_file.id).await.unwrap();
        assert!(result);
        
        let not_found = repo.find_by_id(created_file.id).await.unwrap();
        assert!(not_found.is_none());
        
        // Clean up
        let _ = repo.delete_permanently(created_canvas.id).await;
        let _ = sqlx::query("DELETE FROM users WHERE id = $1").bind(user_id).execute(repo.pool()).await;
    }
}
