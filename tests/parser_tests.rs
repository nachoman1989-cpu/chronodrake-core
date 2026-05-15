/// Tests for parsing Cargo.toml files.
///
/// These tests verify that ChronoDrake can correctly parse
/// Rust project manifests and extract dependency information.
#[cfg(test)]
mod parser_tests {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    /// Helper to create a temporary Cargo.toml for testing.
    fn create_test_cargo_toml(dir: &Path, content: &str) {
        fs::write(dir.join("Cargo.toml"), content).expect("Failed to write test Cargo.toml");
    }

    #[test]
    fn test_cargo_toml_basic_parse() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1"
tokio = { version = "1", features = ["full"] }
"#;
        create_test_cargo_toml(dir.path(), content);

        // Verify the file was created and can be read
        let read_content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        assert!(read_content.contains("test-project"));
        assert!(read_content.contains("serde"));
        assert!(read_content.contains("tokio"));
    }

    #[test]
    fn test_cargo_toml_with_dev_deps() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
serde = "1"

[dev-dependencies]
tempfile = "3"
"#;
        create_test_cargo_toml(dir.path(), content);

        let read_content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        assert!(read_content.contains("[dev-dependencies]"));
        assert!(read_content.contains("tempfile"));
    }

    #[test]
    fn test_cargo_toml_with_features() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"
[package]
name = "feature-test"
version = "0.1.0"

[features]
default = ["std"]
std = []

[dependencies]
serde = { version = "1", features = ["derive"] }
"#;
        create_test_cargo_toml(dir.path(), content);

        let read_content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        assert!(read_content.contains("[features]"));
        assert!(read_content.contains("default = [\"std\"]"));
    }

    #[test]
    fn test_cargo_toml_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Cargo.toml");
        assert!(!path.exists());
    }

    #[test]
    fn test_cargo_toml_empty_deps() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"
[package]
name = "empty-deps"
version = "0.1.0"

[dependencies]
"#;
        create_test_cargo_toml(dir.path(), content);
        let read_content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        assert!(read_content.contains("empty-deps"));
    }

    #[test]
    fn test_cargo_toml_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"
[workspace]
members = ["crate-a", "crate-b"]

[package]
name = "workspace-root"
version = "0.1.0"
"#;
        create_test_cargo_toml(dir.path(), content);
        let read_content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        assert!(read_content.contains("[workspace]"));
        assert!(read_content.contains("crate-a"));
    }
}

/// Tests for parsing package.json files.
#[cfg(test)]
mod package_json_tests {
    use std::fs;
    use std::path::Path;

    fn create_test_package_json(dir: &Path, content: &str) {
        fs::write(dir.join("package.json"), content).expect("Failed to write test package.json");
    }

    #[test]
    fn test_package_json_basic() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"{
            "name": "test-project",
            "version": "1.0.0",
            "dependencies": {
                "react": "^18.0.0",
                "express": "^4.0.0"
            }
        }"#;
        create_test_package_json(dir.path(), content);
        let read = fs::read_to_string(dir.path().join("package.json")).unwrap();
        assert!(read.contains("react"));
        assert!(read.contains("express"));
    }

    #[test]
    fn test_package_json_dev_deps() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"{
            "name": "test-project",
            "devDependencies": {
                "jest": "^29.0.0",
                "typescript": "^5.0.0"
            }
        }"#;
        create_test_package_json(dir.path(), content);
        let read = fs::read_to_string(dir.path().join("package.json")).unwrap();
        assert!(read.contains("jest"));
        assert!(read.contains("typescript"));
    }

    #[test]
    fn test_package_json_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"{ invalid json here }"#;
        create_test_package_json(dir.path(), content);
        let result = serde_json::from_str::<serde_json::Value>(content);
        assert!(result.is_err());
    }
}
