//! Folder model representing directory structure
//!
//! This module defines the Folder struct and related functionality.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Represents a folder in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    /// Primary key
    pub id: i32,
    /// Foreign key to the user who owns this folder
    pub user_id: i32,
    /// Path relative to the user's root folder
    pub path: String,
    /// Foreign key to the parent folder, if any
    pub parent_id: Option<i32>,
    /// When the folder was created
    pub created_at: DateTime<Utc>,
    /// When the folder was last updated
    pub updated_at: DateTime<Utc>,
    /// Soft deletion flag
    pub is_deleted: bool,
}

impl Folder {
    /// Create a new folder
    pub fn new(user_id: i32, path: String, parent_id: Option<i32>) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Will be assigned by database
            user_id,
            path,
            parent_id,
            created_at: now,
            updated_at: now,
            is_deleted: false,
        }
    }
    
    /// Check if this is a root folder
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }
    
    /// Get the folder name from the path
    pub fn name(&self) -> String {
        Path::new(&self.path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                if self.path == "/" {
                    "root".to_string()
                } else {
                    self.path.clone()
                }
            })
    }
    
    /// Get the parent path
    pub fn parent_path(&self) -> Option<String> {
        if self.is_root() {
            None
        } else {
            let path = PathBuf::from(&self.path);
            path.parent().map(|p| p.to_string_lossy().to_string())
        }
    }
    
    /// Mark this folder as deleted
    pub fn mark_deleted(&mut self) {
        self.is_deleted = true;
        self.updated_at = Utc::now();
    }
    
    /// Restore this folder from deletion
    pub fn restore(&mut self) {
        self.is_deleted = false;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_folder() {
        let folder = Folder::new(1, "/documents".to_string(), Some(1));
        assert_eq!(folder.id, 0);
        assert_eq!(folder.user_id, 1);
        assert_eq!(folder.path, "/documents");
        assert_eq!(folder.parent_id, Some(1));
        assert!(!folder.is_deleted);
    }

    #[test]
    fn test_root_folder() {
        let folder = Folder::new(1, "/".to_string(), None);
        assert!(folder.is_root());
        assert_eq!(folder.name(), "root");
        assert_eq!(folder.parent_path(), None);
    }

    #[test]
    fn test_folder_name() {
        let folder = Folder::new(1, "/documents/work".to_string(), Some(1));
        assert_eq!(folder.name(), "work");
    }

    #[test]
    fn test_folder_parent_path() {
        let folder = Folder::new(1, "/documents/work".to_string(), Some(1));
        assert_eq!(folder.parent_path(), Some("/documents".to_string()));
    }

    #[test]
    fn test_mark_deleted() {
        let mut folder = Folder::new(1, "/documents".to_string(), Some(1));
        let created_at = folder.created_at;
        
        // Short sleep to ensure updated_at changes
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        folder.mark_deleted();
        assert!(folder.is_deleted);
        assert!(folder.updated_at > created_at);
    }

    #[test]
    fn test_restore() {
        let mut folder = Folder::new(1, "/documents".to_string(), Some(1));
        folder.is_deleted = true;
        let updated_at = folder.updated_at;
        
        // Short sleep to ensure updated_at changes
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        folder.restore();
        assert!(!folder.is_deleted);
        assert!(folder.updated_at > updated_at);
    }
}
