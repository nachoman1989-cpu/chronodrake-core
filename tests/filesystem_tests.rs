use std::fs;
use std::path::Path;

/// Integration tests for filesystem scanning operations.
///
/// These tests verify that the filesystem scanner correctly discovers
/// files, respects excluded directories, and handles edge cases.
#[cfg(test)]
mod filesystem_tests {
    use super::*;

    /// Helper to create a test directory structure.
    fn create_test_structure(dir: &Path) {
        // Create source files
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(dir.join("src/lib.rs"), "pub fn hello() {}").unwrap();
        fs::write(dir.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();

        // Create excluded directories
        fs::create_dir_all(dir.join("node_modules")).unwrap();
        fs::write(dir.join("node_modules/package.json"), "{}").unwrap();
        fs::create_dir_all(dir.join("target")).unwrap();
        fs::write(dir.join("target/debug/output"), "binary").unwrap();
        fs::create_dir_all(dir.join(".git")).unwrap();
        fs::write(dir.join(".git/config"), "[core]\n").unwrap();
    }

    #[test]
    fn test_filesystem_scan_basic() {
        let dir = tempfile::tempdir().unwrap();
        create_test_structure(dir.path());

        // Walk the directory and collect files
        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }

        assert!(files.len() >= 4); // src/main.rs, src/lib.rs, Cargo.toml + excluded files
    }

    #[test]
    fn test_filesystem_exclude_dirs() {
        let dir = tempfile::tempdir().unwrap();
        create_test_structure(dir.path());

        let excluded = &["node_modules", "target", ".git"];

        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(dir.path())
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_str().unwrap_or("");
                !excluded.contains(&name)
            })
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }

        // Should only have src/main.rs, src/lib.rs, Cargo.toml
        assert_eq!(files.len(), 3);

        // Verify excluded files are not present
        for file in &files {
            let path_str = file.to_string_lossy();
            assert!(!path_str.contains("node_modules"));
            assert!(!path_str.contains("target"));
            assert!(!path_str.contains(".git"));
        }
    }

    #[test]
    fn test_filesystem_empty_directory() {
        let dir = tempfile::tempdir().unwrap();

        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }

        assert!(files.is_empty());
    }

    #[test]
    fn test_filesystem_extension_filter() {
        let dir = tempfile::tempdir().unwrap();
        create_test_structure(dir.path());

        let extensions = &["rs"];
        let mut rs_files = Vec::new();

        for entry in walkdir::WalkDir::new(dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext) {
                        rs_files.push(path.to_path_buf());
                    }
                }
            }
        }

        assert_eq!(rs_files.len(), 2); // src/main.rs, src/lib.rs
        for file in &rs_files {
            assert_eq!(file.extension().unwrap(), "rs");
        }
    }

    #[test]
    fn test_filesystem_nested_directories() {
        let dir = tempfile::tempdir().unwrap();

        // Create nested structure
        fs::create_dir_all(dir.path().join("src/commands/subcommands")).unwrap();
        fs::write(dir.path().join("src/commands/mod.rs"), "").unwrap();
        fs::write(dir.path().join("src/commands/subcommands/help.rs"), "").unwrap();
        fs::write(dir.path().join("src/commands/subcommands/run.rs"), "").unwrap();

        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }

        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_filesystem_max_depth() {
        let dir = tempfile::tempdir().unwrap();

        // Create deep structure
        let mut current = dir.path().to_path_buf();
        for i in 0..10 {
            current = current.join(format!("level_{}", i));
            fs::create_dir_all(&current).unwrap();
            fs::write(current.join("file.rs"), "").unwrap();
        }

        // Walk with max depth 3
        let mut files_depth_3 = Vec::new();
        for entry in walkdir::WalkDir::new(dir.path())
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                files_depth_3.push(entry.path().to_path_buf());
            }
        }

        // Walk with max depth 10
        let mut files_depth_10 = Vec::new();
        for entry in walkdir::WalkDir::new(dir.path())
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                files_depth_10.push(entry.path().to_path_buf());
            }
        }

        assert!(files_depth_3.len() < files_depth_10.len());
    }

    #[test]
    fn test_filesystem_symlink_handling() {
        let dir = tempfile::tempdir().unwrap();

        // Create a real file and a symlink to it
        fs::write(dir.path().join("real_file.txt"), "content").unwrap();

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(
                dir.path().join("real_file.txt"),
                dir.path().join("link.txt"),
            )
            .unwrap();
        }

        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_file(
                dir.path().join("real_file.txt"),
                dir.path().join("link.txt"),
            )
            .unwrap_or(()); // May fail without admin privileges
        }

        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(dir.path())
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }

        // Should find at least the real file
        assert!(files.len() >= 1);
    }

    #[test]
    fn test_filesystem_hidden_files() {
        let dir = tempfile::tempdir().unwrap();

        fs::write(dir.path().join(".hidden"), "secret").unwrap();
        fs::write(dir.path().join("visible.txt"), "hello").unwrap();

        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }

        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_filesystem_large_directory() {
        let dir = tempfile::tempdir().unwrap();

        // Create 100 files
        for i in 0..100 {
            fs::write(dir.path().join(format!("file_{}.txt", i)), "content").unwrap();
        }

        let count = fs::read_dir(dir.path()).unwrap().count();
        assert_eq!(count, 100);
    }
}
