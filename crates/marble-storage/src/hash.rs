use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use blake2b_simd::Params;

use crate::error::{StorageError, StorageResult};

/// Length of the hash bytes (32 bytes = 256 bits)
const HASH_BYTES_LENGTH: usize = 32;

/// Generate a content hash using blake2b and base64url encoding
///
/// Uses the following strategy:
/// 1. Hash the content using blake2b with 256 bits output
/// 2. Encode the hash using base64url without padding
///
/// This provides a URL-safe, fixed-length identifier for content
pub fn hash_content(content: &[u8]) -> StorageResult<String> {
    let hash = Params::new()
        .hash_length(HASH_BYTES_LENGTH)
        .hash(content)
        .as_bytes()
        .to_vec();

    let encoded = URL_SAFE_NO_PAD.encode(hash);
    Ok(encoded)
}

/// Converts a content hash to a storage path
///
/// Format: /.hash/{hash}
pub fn hash_to_path(hash: &str) -> String {
    format!("/.hash/{}", hash)
}

/// Extract hash from a storage path
///
/// Extracts hash from path format: /.hash/{hash}
pub fn path_to_hash(path: &str) -> StorageResult<String> {
    // Path should be in the format /.hash/{hash}
    if !path.starts_with("/.hash/") {
        return Err(StorageError::Validation(format!(
            "Invalid hash path format: {}",
            path
        )));
    }

    let hash = path.strip_prefix("/.hash/").unwrap().to_string();
    if hash.is_empty() {
        return Err(StorageError::Validation("Empty hash in path".to_string()));
    }

    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_content() {
        let content = b"Hello, world!";
        let hash = hash_content(content).unwrap();
        
        // Hash should be non-empty
        assert!(!hash.is_empty());
        
        // Same content should produce same hash
        let hash2 = hash_content(content).unwrap();
        assert_eq!(hash, hash2);
        
        // Different content should produce different hash
        let hash3 = hash_content(b"Different content").unwrap();
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_hash_to_path() {
        let hash = "abcdef123456";
        let path = hash_to_path(hash);
        assert_eq!(path, "/.hash/abcdef123456");
    }

    #[test]
    fn test_path_to_hash() {
        let path = "/.hash/abcdef123456";
        let hash = path_to_hash(path).unwrap();
        assert_eq!(hash, "abcdef123456");
        
        // Invalid path
        let result = path_to_hash("/not/a/hash/path");
        assert!(result.is_err());
        
        // Empty hash
        let result = path_to_hash("/.hash/");
        assert!(result.is_err());
    }
}
