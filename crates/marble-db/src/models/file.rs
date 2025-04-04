//! File model representing stored files
//!
//! This module defines the File struct and related functionality.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Represents a file in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    /// Primary key
    pub id: i32,
    /// Foreign key to the user who owns this file
    pub user_id: i32,
    /// Path relative to the user's root folder
    pub path: String,
    /// Content-addressable hash of file contents
    pub content_hash: String,
    /// MIME type of the file
    pub content_type: String,
    /// Size of the file in bytes
    pub size: i32,
    /// When the file was created
    pub created_at: DateTime<Utc>,
    /// When the file was last updated
    pub updated_at: DateTime<Utc>,
    /// Soft deletion flag
    pub is_deleted: bool,
}

impl File {
    /// Create a new file
    pub fn new(
        user_id: i32, 
        path: String, 
        content_hash: String, 
        content_type: String, 
        size: i32
    ) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Will be assigned by database
            user_id,
            path,
            content_hash,
            content_type,
            size,
            created_at: now,
            updated_at: now,
            is_deleted: false,
        }
    }
    
    /// Get the filename from the path
    pub fn name(&self) -> String {
        Path::new(&self.path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.clone())
    }
    
    /// Get the file extension, if any
    pub fn extension(&self) -> Option<String> {
        Path::new(&self.path)
            .extension()
            .map(|ext| ext.to_string_lossy().to_string())
    }
    
    /// Get the folder path containing this file
    pub fn folder_path(&self) -> String {
        let path = PathBuf::from(&self.path);
        path.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string())
    }
    
    /// Update this file's contents
    pub fn update_content(
        &mut self, 
        content_hash: String, 
        content_type: String, 
        size: i32
    ) {
        self.content_hash = content_hash;
        self.content_type = content_type;
        self.size = size;
        self.updated_at = Utc::now();
    }
    
    /// Mark this file as deleted
    pub fn mark_deleted(&mut self) {
        self.is_deleted = true;
        self.updated_at = Utc::now();
    }
    
    /// Restore this file from deletion
    pub fn restore(&mut self) {
        self.is_deleted = false;
        self.updated_at = Utc::now();
    }
    
    /// Check if this is a markdown file
    pub fn is_markdown(&self) -> bool {
        self.content_type == "text/markdown" || 
        self.extension().map_or(false, |ext| ext == "md" || ext == "markdown")
    }
    
    /// Check if this is a canvas file
    pub fn is_canvas(&self) -> bool {
        self.content_type == "application/obsidian-canvas" ||
        self.extension().map_or(false, |ext| ext == "canvas")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_file() {
        let file = File::new(
            1, 
            "/documents/notes.md".to_string(), 
            "abcdef1234567890".to_string(), 
            "text/markdown".to_string(), 
            1024
        );
        
        assert_eq!(file.id, 0);
        assert_eq!(file.user_id, 1);
        assert_eq!(file.path, "/documents/notes.md");
        assert_eq!(file.content_hash, "abcdef1234567890");
        assert_eq!(file.content_type, "text/markdown");
        assert_eq!(file.size, 1024);
        assert!(!file.is_deleted);
    }

    #[test]
    fn test_file_name() {
        let file = File::new(
            1, 
            "/documents/notes.md".to_string(), 
            "abcdef1234567890".to_string(), 
            "text/markdown".to_string(), 
            1024
        );
        
        assert_eq!(file.name(), "notes.md");
    }

    #[test]
    fn test_file_extension() {
        let file = File::new(
            1, 
            "/documents/notes.md".to_string(), 
            "abcdef1234567890".to_string(), 
            "text/markdown".to_string(), 
            1024
        );
        
        assert_eq!(file.extension(), Some("md".to_string()));
    }

    #[test]
    fn test_folder_path() {
        let file = File::new(
            1, 
            "/documents/notes.md".to_string(), 
            "abcdef1234567890".to_string(), 
            "text/markdown".to_string(), 
            1024
        );
        
        assert_eq!(file.folder_path(), "/documents");
    }

    #[test]
    fn test_update_content() {
        let mut file = File::new(
            1, 
            "/documents/notes.md".to_string(), 
            "abcdef1234567890".to_string(), 
            "text/markdown".to_string(), 
            1024
        );
        
        let created_at = file.created_at;
        
        // Short sleep to ensure updated_at changes
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        file.update_content(
            "updated-hash".to_string(),
            "text/markdown".to_string(),
            2048
        );
        
        assert_eq!(file.content_hash, "updated-hash");
        assert_eq!(file.size, 2048);
        assert!(file.updated_at > created_at);
    }

    #[test]
    fn test_is_markdown() {
        let md_file_by_type = File::new(
            1, 
            "/documents/notes.txt".to_string(), 
            "abcdef1234567890".to_string(), 
            "text/markdown".to_string(), 
            1024
        );
        
        let md_file_by_ext = File::new(
            1, 
            "/documents/notes.md".to_string(), 
            "abcdef1234567890".to_string(), 
            "text/plain".to_string(), 
            1024
        );
        
        let not_md_file = File::new(
            1, 
            "/documents/image.png".to_string(), 
            "abcdef1234567890".to_string(), 
            "image/png".to_string(), 
            1024
        );
        
        assert!(md_file_by_type.is_markdown());
        assert!(md_file_by_ext.is_markdown());
        assert!(!not_md_file.is_markdown());
    }

    #[test]
    fn test_is_canvas() {
        let canvas_file_by_type = File::new(
            1, 
            "/documents/diagram.txt".to_string(), 
            "abcdef1234567890".to_string(), 
            "application/obsidian-canvas".to_string(), 
            1024
        );
        
        let canvas_file_by_ext = File::new(
            1, 
            "/documents/diagram.canvas".to_string(), 
            "abcdef1234567890".to_string(), 
            "text/plain".to_string(), 
            1024
        );
        
        let not_canvas_file = File::new(
            1, 
            "/documents/image.png".to_string(), 
            "abcdef1234567890".to_string(), 
            "image/png".to_string(), 
            1024
        );
        
        assert!(canvas_file_by_type.is_canvas());
        assert!(canvas_file_by_ext.is_canvas());
        assert!(!not_canvas_file.is_canvas());
    }
}
