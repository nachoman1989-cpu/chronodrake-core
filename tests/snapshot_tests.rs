use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Integration tests for snapshot creation, serialization, and management.
#[cfg(test)]
mod snapshot_tests {
    use super::*;

    /// A simplified snapshot structure for testing.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestSnapshot {
        id: String,
        timestamp: String,
        files: HashMap<String, String>, // path → hash
        metadata: SnapshotMetadata,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct SnapshotMetadata {
        file_count: usize,
        total_size: u64,
        scan_version: String,
    }

    fn create_test_snapshot(id: &str) -> TestSnapshot {
        let mut files = HashMap::new();
        files.insert("src/main.rs".to_string(), "abc123".to_string());
        files.insert("src/lib.rs".to_string(), "def456".to_string());
        files.insert("Cargo.toml".to_string(), "ghi789".to_string());

        TestSnapshot {
            id: id.to_string(),
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            files,
            metadata: SnapshotMetadata {
                file_count: 3,
                total_size: 1024,
                scan_version: "0.5.0".to_string(),
            },
        }
    }

    #[test]
    fn test_snapshot_create() {
        let snapshot = create_test_snapshot("snap-001");
        assert_eq!(snapshot.id, "snap-001");
        assert_eq!(snapshot.metadata.file_count, 3);
        assert_eq!(snapshot.files.len(), 3);
    }

    #[test]
    fn test_snapshot_serialize_deserialize() {
        let snapshot = create_test_snapshot("snap-002");
        let json = serde_json::to_string_pretty(&snapshot).unwrap();
        let deserialized: TestSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, snapshot.id);
        assert_eq!(deserialized.timestamp, snapshot.timestamp);
        assert_eq!(deserialized.files.len(), snapshot.files.len());
        assert_eq!(deserialized.metadata.file_count, snapshot.metadata.file_count);
    }

    #[test]
    fn test_snapshot_write_read_file() {
        let dir = tempfile::tempdir().unwrap();
        let snap_path = dir.path().join("snapshot.json");

        let snapshot = create_test_snapshot("snap-003");
        let json = serde_json::to_string_pretty(&snapshot).unwrap();
        fs::write(&snap_path, &json).unwrap();

        let read_content = fs::read_to_string(&snap_path).unwrap();
        let deserialized: TestSnapshot = serde_json::from_str(&read_content).unwrap();

        assert_eq!(deserialized.id, "snap-003");
        assert!(deserialized.files.contains_key("src/main.rs"));
    }

    #[test]
    fn test_snapshot_file_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let snap_path = dir.path().join("nonexistent.json");
        assert!(!snap_path.exists());
    }

    #[test]
    fn test_snapshot_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let snap_path = dir.path().join("invalid.json");
        fs::write(&snap_path, "{ invalid json }").unwrap();

        let content = fs::read_to_string(&snap_path).unwrap();
        let result: Result<TestSnapshot, _> = serde_json::from_str(&content);
        assert!(result.is_err());
    }

    #[test]
    fn test_snapshot_empty_state() {
        let snapshot = TestSnapshot {
            id: "snap-empty".to_string(),
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            files: HashMap::new(),
            metadata: SnapshotMetadata {
                file_count: 0,
                total_size: 0,
                scan_version: "0.5.0".to_string(),
            },
        };

        assert_eq!(snapshot.files.len(), 0);
        assert_eq!(snapshot.metadata.file_count, 0);

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: TestSnapshot = serde_json::from_str(&json).unwrap();
        assert!(deserialized.files.is_empty());
    }

    #[test]
    fn test_snapshot_multiple_snapshots() {
        let dir = tempfile::tempdir().unwrap();

        let snapshots = vec![
            create_test_snapshot("snap-001"),
            create_test_snapshot("snap-002"),
            create_test_snapshot("snap-003"),
        ];

        for snap in &snapshots {
            let path = dir.path().join(format!("{}.json", snap.id));
            let json = serde_json::to_string_pretty(snap).unwrap();
            fs::write(&path, json).unwrap();
        }

        // Count snapshot files
        let count = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
            .count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_snapshot_large_file_set() {
        let mut files = HashMap::new();
        for i in 0..1000 {
            files.insert(
                format!("src/module_{}/file_{}.rs", i % 10, i),
                format!("hash_{}", i),
            );
        }

        let snapshot = TestSnapshot {
            id: "snap-large".to_string(),
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            files,
            metadata: SnapshotMetadata {
                file_count: 1000,
                total_size: 1_000_000,
                scan_version: "0.5.0".to_string(),
            },
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.len() > 1000);

        let deserialized: TestSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.files.len(), 1000);
    }

    #[test]
    fn test_snapshot_metadata_integrity() {
        let snapshot = create_test_snapshot("snap-meta");
        assert_eq!(snapshot.metadata.file_count, snapshot.files.len() as usize);
    }
}
