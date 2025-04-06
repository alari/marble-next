use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::api::{FileMetadata, TenantStorage};
use crate::StorageError;

/// Mock implementation of TenantStorage for testing
#[derive(Default)]
pub struct MockTenantStorage {
    // Maps (tenant_id, path) -> (content, is_directory)
    files: Arc<RwLock<HashMap<(Uuid, String), (Vec<u8>, bool)>>>,
    // Maps (tenant_id, directory_path) -> [entry_names]
    directory_entries: Arc<RwLock<HashMap<(Uuid, String), Vec<String>>>>,
}

impl MockTenantStorage {
    /// Create a new mock tenant storage
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(HashMap::new())),
            directory_entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a file to the storage (for testing)
    pub fn add_file(&self, tenant_id: &Uuid, path: &str, content: Vec<u8>) {
        let mut files = self.files.write().unwrap();
        files.insert((*tenant_id, path.to_string()), (content, false));
        
        // Add to parent directory entries
        let parent_path = self.get_parent_path(path);
        let file_name = self.get_file_name(path);
        
        let mut directory_entries = self.directory_entries.write().unwrap();
        let entries = directory_entries
            .entry((*tenant_id, parent_path))
            .or_insert_with(Vec::new);
        
        if !entries.contains(&file_name) {
            entries.push(file_name);
        }
    }
    
    /// Add a directory to the storage (for testing)
    pub fn add_directory(&self, tenant_id: &Uuid, path: &str) {
        // Add directory entry
        let mut files = self.files.write().unwrap();
        files.insert((*tenant_id, path.to_string()), (Vec::new(), true));
        
        // Add to parent directory entries
        let parent_path = self.get_parent_path(path);
        let dir_name = self.get_file_name(path);
        
        let mut directory_entries = self.directory_entries.write().unwrap();
        
        // Create empty entries list for this directory
        directory_entries.entry((*tenant_id, path.to_string())).or_insert_with(Vec::new);
        
        // Add to parent directory entries
        let entries = directory_entries
            .entry((*tenant_id, parent_path))
            .or_insert_with(Vec::new);
        
        if !entries.contains(&dir_name) {
            entries.push(dir_name);
        }
    }
    
    /// Get parent path of a file path
    fn get_parent_path(&self, path: &str) -> String {
        let path = path.trim_end_matches('/');
        
        if path.is_empty() || path == "." {
            return ".".to_string();
        }
        
        match path.rfind('/') {
            Some(idx) => {
                let parent = &path[..idx];
                if parent.is_empty() {
                    ".".to_string()
                } else {
                    parent.to_string()
                }
            }
            None => ".".to_string()
        }
    }
    
    /// Get file name from a path
    fn get_file_name(&self, path: &str) -> String {
        let path = path.trim_end_matches('/');
        
        if path.is_empty() || path == "." {
            return ".".to_string();
        }
        
        match path.rfind('/') {
            Some(idx) => path[idx + 1..].to_string(),
            None => path.to_string()
        }
    }
}

#[async_trait]
impl TenantStorage for MockTenantStorage {
    async fn exists(&self, tenant_id: &Uuid, path: &str) -> Result<bool, StorageError> {
        let files = self.files.read().unwrap();
        Ok(files.contains_key(&(*tenant_id, path.to_string())))
    }
    
    async fn read(&self, tenant_id: &Uuid, path: &str) -> Result<Vec<u8>, StorageError> {
        let files = self.files.read().unwrap();
        match files.get(&(*tenant_id, path.to_string())) {
            Some((content, is_directory)) => {
                if *is_directory {
                    return Err(StorageError::Validation("Cannot read a directory".to_string()));
                }
                Ok(content.clone())
            }
            None => Err(StorageError::NotFound(path.to_string())),
        }
    }
    
    async fn write(
        &self,
        tenant_id: &Uuid,
        path: &str,
        content: Vec<u8>,
        _content_type: Option<&str>,
    ) -> Result<(), StorageError> {
        self.add_file(tenant_id, path, content);
        Ok(())
    }
    
    async fn delete(&self, tenant_id: &Uuid, path: &str) -> Result<(), StorageError> {
        let mut files = self.files.write().unwrap();
        if files.remove(&(*tenant_id, path.to_string())).is_none() {
            return Err(StorageError::NotFound(path.to_string()));
        }
        
        // Remove from parent directory entries
        let parent_path = self.get_parent_path(path);
        let file_name = self.get_file_name(path);
        
        let mut directory_entries = self.directory_entries.write().unwrap();
        if let Some(entries) = directory_entries.get_mut(&(*tenant_id, parent_path)) {
            entries.retain(|name| name != &file_name);
        }
        
        // Remove directory entries if it was a directory
        directory_entries.remove(&(*tenant_id, path.to_string()));
        
        Ok(())
    }
    
    async fn list(&self, tenant_id: &Uuid, path: &str) -> Result<Vec<String>, StorageError> {
        
        // Check if path exists and is a directory
        let files = self.files.read().unwrap();
        match files.get(&(*tenant_id, path.to_string())) {
            Some((_, is_directory)) => {
                if !*is_directory && path != "." {
                    return Err(StorageError::Validation("Not a directory".to_string()));
                }
            }
            None => {
                if path != "." {
                    return Err(StorageError::NotFound(path.to_string()));
                }
            }
        }
        
        // Get directory entries
        let directory_entries = self.directory_entries.read().unwrap();
        match directory_entries.get(&(*tenant_id, path.to_string())) {
            Some(entries) => Ok(entries.clone()),
            None => Ok(Vec::new()),
        }
    }
    
    async fn create_directory(&self, tenant_id: &Uuid, path: &str) -> Result<(), StorageError> {
        self.add_directory(tenant_id, path);
        Ok(())
    }
    
    async fn metadata(&self, tenant_id: &Uuid, path: &str) -> Result<FileMetadata, StorageError> {
        let files = self.files.read().unwrap();
        match files.get(&(*tenant_id, path.to_string())) {
            Some((content, is_directory)) => {
                let content_type = if *is_directory {
                    "application/x-directory".to_string()
                } else if path.ends_with(".md") {
                    "text/markdown".to_string()
                } else if path.ends_with(".canvas") {
                    "application/json".to_string()
                } else {
                    "application/octet-stream".to_string()
                };
                
                Ok(FileMetadata {
                    path: path.to_string(),
                    content_type,
                    size: content.len() as u64,
                    is_directory: *is_directory,
                    last_modified: None,
                    content_hash: None,
                })
            }
            None => Err(StorageError::NotFound(path.to_string())),
        }
    }
}