use std::collections::HashMap;
use std::sync::Mutex;
use async_trait::async_trait;
use marble_storage::api::{TenantStorage, FileMetadata};
use marble_storage::error::StorageResult;
use uuid::Uuid;

/// Mock TenantStorage for testing
#[derive(Default)]
pub struct MockTenantStorage {
    // Simulates a simple file system with tenant_id -> path -> content
    files: Mutex<HashMap<Uuid, HashMap<String, Vec<u8>>>>,
    
    // Simulates directories with tenant_id -> directory path
    directories: Mutex<HashMap<Uuid, Vec<String>>>,
}

impl MockTenantStorage {
    pub fn new() -> Self {
        Self::default()
    }
    
    // Helper to set up test data
    pub fn add_file(&self, tenant_id: &Uuid, path: &str, content: Vec<u8>) {
        let mut files = self.files.lock().unwrap();
        let tenant_files = files.entry(*tenant_id).or_insert_with(HashMap::new);
        tenant_files.insert(path.to_string(), content);
        
        // Ensure parent directories exist
        let parent = if path.contains('/') {
            let parts: Vec<&str> = path.split('/').collect();
            let parent = parts[..parts.len()-1].join("/");
            if parent.is_empty() { "." } else { &parent }
        } else {
            "."
        };
        
        let mut directories = self.directories.lock().unwrap();
        let tenant_dirs = directories.entry(*tenant_id).or_insert_with(Vec::new);
        
        if !tenant_dirs.contains(&parent.to_string()) {
            tenant_dirs.push(parent.to_string());
        }
    }
    
    pub fn add_directory(&self, tenant_id: &Uuid, path: &str) {
        let mut directories = self.directories.lock().unwrap();
        let tenant_dirs = directories.entry(*tenant_id).or_insert_with(Vec::new);
        
        if !tenant_dirs.contains(&path.to_string()) {
            tenant_dirs.push(path.to_string());
        }
    }
}

#[async_trait]
impl TenantStorage for MockTenantStorage {
    async fn read(&self, tenant_id: &Uuid, path: &str) -> StorageResult<Vec<u8>> {
        let files = self.files.lock().unwrap();
        
        if let Some(tenant_files) = files.get(tenant_id) {
            if let Some(content) = tenant_files.get(path) {
                return Ok(content.clone());
            }
        }
        
        Err(marble_storage::error::StorageError::NotFound(path.to_string()))
    }
    
    async fn create_directory(&self, tenant_id: &Uuid, path: &str) -> StorageResult<()> {
        let mut directories = self.directories.lock().unwrap();
        let tenant_dirs = directories.entry(*tenant_id).or_insert_with(Vec::new);
        
        if !tenant_dirs.contains(&path.to_string()) {
            tenant_dirs.push(path.to_string());
        }
        
        Ok(())
    }
    
    async fn write(&self, tenant_id: &Uuid, path: &str, content: Vec<u8>, _content_type: Option<&str>) -> StorageResult<()> {
        // Create parent directories if needed
        if path.contains('/') {
            let parent = path.rsplit_once('/').unwrap().0;
            if !parent.is_empty() {
                self.create_directory(tenant_id, parent).await?;
            }
        }
        
        let mut files = self.files.lock().unwrap();
        let tenant_files = files.entry(*tenant_id).or_insert_with(HashMap::new);
        tenant_files.insert(path.to_string(), content);
        
        Ok(())
    }
    
    async fn exists(&self, tenant_id: &Uuid, path: &str) -> StorageResult<bool> {
        let files = self.files.lock().unwrap();
        let directories = self.directories.lock().unwrap();
        
        // Check files
        if let Some(tenant_files) = files.get(tenant_id) {
            if tenant_files.contains_key(path) {
                return Ok(true);
            }
        }
        
        // Check directories
        if let Some(tenant_dirs) = directories.get(tenant_id) {
            if tenant_dirs.contains(&path.to_string()) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    async fn delete(&self, tenant_id: &Uuid, path: &str) -> StorageResult<()> {
        // Check if it exists first
        if !self.exists(tenant_id, path).await? {
            return Err(marble_storage::error::StorageError::NotFound(path.to_string()));
        }
        
        // Try to remove as a file
        let mut files = self.files.lock().unwrap();
        if let Some(tenant_files) = files.get_mut(tenant_id) {
            if tenant_files.remove(path).is_some() {
                return Ok(());
            }
        }
        
        // Try to remove as a directory
        let mut directories = self.directories.lock().unwrap();
        if let Some(tenant_dirs) = directories.get_mut(tenant_id) {
            if let Some(index) = tenant_dirs.iter().position(|p| p == path) {
                tenant_dirs.remove(index);
                return Ok(());
            }
        }
        
        // This shouldn't happen if exists() returned true
        Err(marble_storage::error::StorageError::NotFound(path.to_string()))
    }
    
    async fn list(&self, tenant_id: &Uuid, dir_path: &str) -> StorageResult<Vec<String>> {
        let files = self.files.lock().unwrap();
        let mut results = Vec::new();
        
        // Check if directory exists
        let directories = self.directories.lock().unwrap();
        if let Some(tenant_dirs) = directories.get(tenant_id) {
            if !tenant_dirs.contains(&dir_path.to_string()) && dir_path != "." {
                return Err(marble_storage::error::StorageError::NotFound(dir_path.to_string()));
            }
        }
        
        // Get files in this directory
        if let Some(tenant_files) = files.get(tenant_id) {
            for path in tenant_files.keys() {
                let parent = if path.contains('/') {
                    path.rsplit_once('/').unwrap().0
                } else {
                    "."
                };
                
                if parent == dir_path {
                    let file_name = if path.contains('/') {
                        path.rsplit_once('/').unwrap().1
                    } else {
                        path
                    };
                    results.push(file_name.to_string());
                }
            }
        }
        
        // Get subdirectories
        if let Some(tenant_dirs) = directories.get(tenant_id) {
            for dir in tenant_dirs {
                if dir == dir_path {
                    continue;  // Skip the current directory
                }
                
                let parent = if dir.contains('/') {
                    dir.rsplit_once('/').unwrap().0
                } else {
                    "."
                };
                
                if parent == dir_path {
                    let dir_name = if dir.contains('/') {
                        dir.rsplit_once('/').unwrap().1
                    } else {
                        dir
                    };
                    if !results.contains(&dir_name.to_string()) {
                        results.push(dir_name.to_string());
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    async fn metadata(&self, tenant_id: &Uuid, path: &str) -> StorageResult<FileMetadata> {
        let files = self.files.lock().unwrap();
        let directories = self.directories.lock().unwrap();
        
        // Check if it's a file
        if let Some(tenant_files) = files.get(tenant_id) {
            if let Some(content) = tenant_files.get(path) {
                return Ok(FileMetadata {
                    path: path.to_string(),
                    size: content.len() as u64,
                    content_type: mime_guess::from_path(path).first_or_octet_stream().to_string(),
                    is_directory: false,
                    last_modified: Some(chrono::Utc::now().timestamp_millis() as u64),
                    content_hash: None,
                });
            }
        }
        
        // Check if it's a directory
        if let Some(tenant_dirs) = directories.get(tenant_id) {
            if tenant_dirs.contains(&path.to_string()) || path == "." {
                return Ok(FileMetadata {
                    path: path.to_string(),
                    size: 0,
                    content_type: "application/x-directory".to_string(),
                    is_directory: true,
                    last_modified: Some(chrono::Utc::now().timestamp_millis() as u64),
                    content_hash: None,
                });
            }
        }
        
        Err(marble_storage::error::StorageError::NotFound(path.to_string()))
    }
}
