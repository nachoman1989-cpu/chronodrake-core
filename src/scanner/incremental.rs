use anyhow::Result;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// Represents the state of a file for incremental scanning.
#[derive(Debug, Clone)]
pub struct FileState {
    /// Relative path of the file.
    pub path: String,
    /// SHA-256 hash of the file contents.
    pub hash: String,
    /// Last modification timestamp (seconds since epoch).
    pub modified: u64,
    /// File size in bytes.
    pub size: u64,
}

/// Result of comparing current file states against a previous snapshot.
#[derive(Debug, Clone, Default)]
pub struct FileChangeSet {
    /// Files that were added since the last scan.
    pub added: Vec<String>,
    /// Files that were modified since the last scan.
    pub modified: Vec<String>,
    /// Files that were deleted since the last scan.
    pub deleted: Vec<String>,
    /// Files that remain unchanged.
    pub unchanged: Vec<String>,
}

impl FileChangeSet {
    /// Returns true if no changes were detected.
    pub fn is_unchanged(&self) -> bool {
        self.added.is_empty() && self.modified.is_empty() && self.deleted.is_empty()
    }

    /// Total number of changed files.
    pub fn change_count(&self) -> usize {
        self.added.len() + self.modified.len() + self.deleted.len()
    }
}

/// Computes the SHA-256 hash of a file's contents.
pub fn hash_file(path: &Path) -> Result<String> {
    let contents = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// Gets the modification timestamp of a file (seconds since epoch).
pub fn get_modified_time(path: &Path) -> Result<u64> {
    let metadata = std::fs::metadata(path)?;
    let modified = metadata.modified()?;
    let duration = modified
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    Ok(duration.as_secs())
}

/// Scans a directory and computes file states for all files matching the given extensions.
pub fn scan_file_states(root_path: &str, extensions: &[&str]) -> Result<HashMap<String, FileState>> {
    let root = Path::new(root_path);
    let mut states = HashMap::new();

    for entry in walkdir::WalkDir::new(root)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_str().unwrap_or("");
            !matches!(
                name,
                "node_modules" | ".git" | "target" | "dist" | "build" | ".cache" | "__pycache__"
            )
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Check extension filter
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if !extensions.is_empty() && !extensions.contains(&ext) {
                continue;
            }
        } else if !extensions.is_empty() {
            continue;
        }

        let relative = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let hash = hash_file(path)?;
        let modified = get_modified_time(path)?;
        let size = std::fs::metadata(path)?.len();

        states.insert(
            relative.clone(),
            FileState {
                path: relative,
                hash,
                modified,
                size,
            },
        );
    }

    Ok(states)
}

/// Compares current file states against a previous snapshot and returns the change set.
pub fn detect_file_changes(
    current: &HashMap<String, FileState>,
    previous: &HashMap<String, FileState>,
) -> FileChangeSet {
    let mut changes = FileChangeSet::default();

    for (path, state) in current {
        match previous.get(path) {
            None => {
                changes.added.push(path.clone());
            }
            Some(prev) => {
                if state.hash != prev.hash || state.modified != prev.modified {
                    changes.modified.push(path.clone());
                } else {
                    changes.unchanged.push(path.clone());
                }
            }
        }
    }

    for path in previous.keys() {
        if !current.contains_key(path) {
            changes.deleted.push(path.clone());
        }
    }

    changes
}

/// Serializes file states to a JSON string for persistent storage.
pub fn serialize_file_states(states: &HashMap<String, FileState>) -> Result<String> {
    Ok(serde_json::to_string(states)?)
}

/// Deserializes file states from a JSON string.
pub fn deserialize_file_states(json: &str) -> Result<HashMap<String, FileState>> {
    Ok(serde_json::from_str(json)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_file_changes_added() {
        let mut current = HashMap::new();
        current.insert(
            "src/main.rs".to_string(),
            FileState {
                path: "src/main.rs".to_string(),
                hash: "abc".to_string(),
                modified: 1000,
                size: 100,
            },
        );

        let previous = HashMap::new();
        let changes = detect_file_changes(&current, &previous);

        assert_eq!(changes.added.len(), 1);
        assert_eq!(changes.added[0], "src/main.rs");
        assert!(changes.modified.is_empty());
        assert!(changes.deleted.is_empty());
    }

    #[test]
    fn test_detect_file_changes_modified() {
        let mut current = HashMap::new();
        current.insert(
            "src/main.rs".to_string(),
            FileState {
                path: "src/main.rs".to_string(),
                hash: "def".to_string(),
                modified: 1001,
                size: 100,
            },
        );

        let mut previous = HashMap::new();
        previous.insert(
            "src/main.rs".to_string(),
            FileState {
                path: "src/main.rs".to_string(),
                hash: "abc".to_string(),
                modified: 1000,
                size: 100,
            },
        );

        let changes = detect_file_changes(&current, &previous);
        assert_eq!(changes.modified.len(), 1);
        assert!(changes.added.is_empty());
        assert!(changes.deleted.is_empty());
    }

    #[test]
    fn test_detect_file_changes_deleted() {
        let current = HashMap::new();

        let mut previous = HashMap::new();
        previous.insert(
            "src/main.rs".to_string(),
            FileState {
                path: "src/main.rs".to_string(),
                hash: "abc".to_string(),
                modified: 1000,
                size: 100,
            },
        );

        let changes = detect_file_changes(&current, &previous);
        assert_eq!(changes.deleted.len(), 1);
        assert!(changes.added.is_empty());
        assert!(changes.modified.is_empty());
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut states = HashMap::new();
        states.insert(
            "test.rs".to_string(),
            FileState {
                path: "test.rs".to_string(),
                hash: "abc123".to_string(),
                modified: 12345,
                size: 500,
            },
        );

        let json = serialize_file_states(&states).unwrap();
        let deserialized = deserialize_file_states(&json).unwrap();

        assert_eq!(states.len(), deserialized.len());
        let original = states.get("test.rs").unwrap();
        let loaded = deserialized.get("test.rs").unwrap();
        assert_eq!(original.hash, loaded.hash);
        assert_eq!(original.modified, loaded.modified);
    }
}
