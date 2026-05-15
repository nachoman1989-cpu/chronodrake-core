use crate::errors::CacheError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Default time-to-live for cache entries (1 hour).
const DEFAULT_TTL_SECS: u64 = 3600;

/// A generic cache entry with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T: Serialize> {
    /// The cached data.
    pub data: T,
    /// Timestamp when this entry was created (seconds since epoch).
    pub created_at: u64,
    /// Time-to-live in seconds.
    pub ttl: u64,
    /// SHA-256 hash of the serialized data for integrity verification.
    pub integrity_hash: String,
}

/// Cache for parsed dependency information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyCache {
    /// Map of dependency name to version and ecosystem.
    pub dependencies: HashMap<String, (String, String)>,
    /// When this cache was last updated.
    pub updated_at: u64,
}

/// Cache for module graph information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleGraphCache {
    /// Map of module name to its connections.
    pub modules: HashMap<String, Vec<String>>,
    /// When this cache was last updated.
    pub updated_at: u64,
}

/// Cache for file hashes used in incremental scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHashCache {
    /// Map of file path to its SHA-256 hash.
    pub hashes: HashMap<String, String>,
    /// When this cache was last updated.
    pub updated_at: u64,
}

/// The main cache manager for ChronoDrake.
pub struct CacheManager {
    /// Root path for cache storage.
    cache_dir: PathBuf,
    /// Whether caching is enabled.
    enabled: bool,
}

impl CacheManager {
    /// Creates a new CacheManager.
    ///
    /// The cache directory is created at `{root_path}/.chronodrake/cache/`.
    pub fn new(root_path: &str, enabled: bool) -> Result<Self, CacheError> {
        let cache_dir = Path::new(root_path).join(".chronodrake").join("cache");
        if enabled {
            std::fs::create_dir_all(&cache_dir)
                .map_err(|e| CacheError::DirectoryCreation(format!(
                    "Failed to create cache directory {:?}: {}", cache_dir, e
                )))?;
        }
        Ok(Self { cache_dir, enabled })
    }

    /// Returns the path to a cache file for the given key.
    fn cache_path(&self, key: &str) -> PathBuf {
        let hashed_key = self.hash_key(key);
        self.cache_dir.join(format!("{}.json", hashed_key))
    }

    /// Hashes a cache key to a safe filename.
    fn hash_key(&self, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Stores a value in the cache.
    pub fn set<T: Serialize>(&self, key: &str, value: &T, ttl_secs: Option<u64>) -> Result<(), CacheError> {
        if !self.enabled {
            return Ok(());
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::WriteFailed {
                key: key.to_string(),
                details: format!("Serialization failed: {}", e),
            })?;

        let integrity_hash = {
            let mut hasher = Sha256::new();
            hasher.update(serialized.as_bytes());
            hex::encode(hasher.finalize())
        };

        let entry = CacheEntry {
            data: serialized,
            created_at: now,
            ttl: ttl_secs.unwrap_or(DEFAULT_TTL_SECS),
            integrity_hash,
        };

        let entry_json = serde_json::to_string(&entry)
            .map_err(|e| CacheError::WriteFailed {
                key: key.to_string(),
                details: format!("Entry serialization failed: {}", e),
            })?;

        let path = self.cache_path(key);
        std::fs::write(&path, entry_json)
            .map_err(|e| CacheError::WriteFailed {
                key: key.to_string(),
                details: format!("File write failed: {}", e),
            })?;

        Ok(())
    }

    /// Retrieves a value from the cache.
    ///
    /// Returns `None` if the key is not found, TTL has expired, or integrity check fails.
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError> {
        if !self.enabled {
            return Ok(None);
        }

        let path = self.cache_path(key);
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| CacheError::ReadFailed {
                key: key.to_string(),
                details: format!("File read failed: {}", e),
            })?;

        let entry: CacheEntry<String> = serde_json::from_str(&content)
            .map_err(|e| CacheError::ReadFailed {
                key: key.to_string(),
                details: format!("Deserialization failed: {}", e),
            })?;

        // Check TTL
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now - entry.created_at >= entry.ttl {
            // Entry expired — remove it
            let _ = std::fs::remove_file(&path);
            return Ok(None);
        }

        // Verify integrity
        let actual_hash = {
            let mut hasher = Sha256::new();
            hasher.update(entry.data.as_bytes());
            hex::encode(hasher.finalize())
        };

        if actual_hash != entry.integrity_hash {
            // Integrity mismatch — remove corrupted entry
            let _ = std::fs::remove_file(&path);
            return Err(CacheError::IntegrityMismatch {
                key: key.to_string(),
                expected: entry.integrity_hash,
                actual: actual_hash,
            });
        }

        // Deserialize the actual data
        let data: T = serde_json::from_str(&entry.data)
            .map_err(|e| CacheError::ReadFailed {
                key: key.to_string(),
                details: format!("Data deserialization failed: {}", e),
            })?;

        Ok(Some(data))
    }

    /// Removes a specific cache entry.
    pub fn remove(&self, key: &str) -> Result<(), CacheError> {
        if !self.enabled {
            return Ok(());
        }
        let path = self.cache_path(key);
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| CacheError::ReadFailed {
                    key: key.to_string(),
                    details: format!("Failed to remove cache entry: {}", e),
                })?;
        }
        Ok(())
    }

    /// Clears all cache entries.
    pub fn clear(&self) -> Result<(), CacheError> {
        if !self.enabled {
            return Ok(());
        }
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)
                .map_err(|e| CacheError::DirectoryCreation(format!(
                    "Failed to clear cache directory: {}", e
                )))?;
            std::fs::create_dir_all(&self.cache_dir)
                .map_err(|e| CacheError::DirectoryCreation(format!(
                    "Failed to recreate cache directory: {}", e
                )))?;
        }
        Ok(())
    }

    /// Returns the number of cache entries.
    pub fn entry_count(&self) -> Result<usize, CacheError> {
        if !self.enabled || !self.cache_dir.exists() {
            return Ok(0);
        }
        let count = std::fs::read_dir(&self.cache_dir)
            .map_err(|e| CacheError::ReadFailed {
                key: "entry_count".to_string(),
                details: format!("Failed to read cache directory: {}", e),
            })?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
            .count();
        Ok(count)
    }

    /// Returns cache statistics.
    pub fn stats(&self) -> Result<CacheStats, CacheError> {
        let entry_count = self.entry_count()?;
        let size = if self.enabled && self.cache_dir.exists() {
            let total: u64 = std::fs::read_dir(&self.cache_dir)
                .map_err(|e| CacheError::ReadFailed {
                    key: "stats".to_string(),
                    details: format!("Failed to read cache directory: {}", e),
                })?
                .filter_map(|e| e.ok())
                .filter_map(|e| e.metadata().ok())
                .map(|m| m.len())
                .sum();
            total
        } else {
            0
        };

        Ok(CacheStats {
            entry_count,
            total_size_bytes: size,
            cache_dir: self.cache_dir.to_string_lossy().to_string(),
            enabled: self.enabled,
        })
    }
}

/// Statistics about the cache.
#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    /// Number of cache entries.
    pub entry_count: usize,
    /// Total size of cache files in bytes.
    pub total_size_bytes: u64,
    /// Path to the cache directory.
    pub cache_dir: String,
    /// Whether caching is enabled.
    pub enabled: bool,
}

/// Convenience function to create a cache key for dependency analysis.
pub fn dep_cache_key(root_path: &str) -> String {
    format!("deps:{}", root_path)
}

/// Convenience function to create a cache key for module graph.
pub fn module_graph_cache_key(root_path: &str) -> String {
    format!("modules:{}", root_path)
}

/// Convenience function to create a cache key for file hashes.
pub fn file_hash_cache_key(root_path: &str) -> String {
    format!("file_hashes:{}", root_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_cache() -> (tempfile::TempDir, CacheManager) {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheManager::new(dir.path().to_str().unwrap(), true).unwrap();
        (dir, cache)
    }

    #[test]
    fn test_cache_set_get() {
        let (_dir, cache) = setup_test_cache();
        let value = vec!["a", "b", "c"];
        cache.set("test_key", &value, Some(3600)).unwrap();
        let result: Vec<String> = cache.get("test_key").unwrap().unwrap();
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_cache_miss() {
        let (_dir, cache) = setup_test_cache();
        let result: Option<Vec<String>> = cache.get("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_ttl_expiry() {
        let (_dir, cache) = setup_test_cache();
        let value = "test";
        cache.set("ttl_test", &value, Some(0)).unwrap(); // 0 TTL = expired immediately
        let result: Option<String> = cache.get("ttl_test").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_remove() {
        let (_dir, cache) = setup_test_cache();
        cache.set("remove_test", &"value", Some(3600)).unwrap();
        cache.remove("remove_test").unwrap();
        let result: Option<String> = cache.get("remove_test").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_clear() {
        let (_dir, cache) = setup_test_cache();
        cache.set("key1", &"val1", Some(3600)).unwrap();
        cache.set("key2", &"val2", Some(3600)).unwrap();
        assert_eq!(cache.entry_count().unwrap(), 2);
        cache.clear().unwrap();
        assert_eq!(cache.entry_count().unwrap(), 0);
    }

    #[test]
    fn test_cache_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheManager::new(dir.path().to_str().unwrap(), false).unwrap();
        cache.set("key", &"value", Some(3600)).unwrap();
        let result: Option<String> = cache.get("key").unwrap();
        assert!(result.is_none());
        assert_eq!(cache.entry_count().unwrap(), 0);
    }

    #[test]
    fn test_cache_key_generators() {
        let key = dep_cache_key("/test/project");
        assert!(key.starts_with("deps:"));
        let key = module_graph_cache_key("/test/project");
        assert!(key.starts_with("modules:"));
        let key = file_hash_cache_key("/test/project");
        assert!(key.starts_with("file_hashes:"));
    }

    #[test]
    fn test_cache_stats() {
        let (_dir, cache) = setup_test_cache();
        cache.set("stats_test", &"data", Some(3600)).unwrap();
        let stats = cache.stats().unwrap();
        assert!(stats.entry_count >= 1);
        assert!(stats.total_size_bytes > 0);
        assert!(stats.enabled);
    }
}
