use std::collections::HashMap;
use anyhow::Result;
use walkdir::WalkDir;

/// Key files to detect during filesystem scanning.
const TARGET_FILES: &[&str] = &[
    "Cargo.toml",
    "package.json",
    "tauri.conf.json",
    ".git",
    "Cargo.lock",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "tsconfig.json",
    "vite.config.ts",
    "webpack.config.js",
    "Dockerfile",
    "docker-compose.yml",
    "Makefile",
    "CMakeLists.txt",
    "build.gradle",
    "pom.xml",
    "go.mod",
    "requirements.txt",
    "pyproject.toml",
    "composer.json",
    "Gemfile",
    ".env.example",
    "README.md",
];

/// Directories to exclude from filesystem scanning.
const EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "dist",
    "build",
    ".next",
    ".cache",
    "__pycache__",
];

/// Scans the filesystem for target files and returns a map of findings.
///
/// The returned HashMap maps category keys (e.g., "config_files", "lock_files")
/// to lists of discovered file paths relative to the root.
/// Automatically excludes `node_modules`, `.git`, `target`, and other common
/// build/dependency directories.
pub fn scan_filesystem(root_path: &str) -> Result<HashMap<String, Vec<String>>> {
    let mut findings: HashMap<String, Vec<String>> = HashMap::new();
    let mut config_files: Vec<String> = Vec::new();
    let mut lock_files: Vec<String> = Vec::new();
    let mut dirs: Vec<String> = Vec::new();

    let root = std::path::Path::new(root_path);

    // Walk the directory tree (max depth 3 to avoid excessive traversal)
    for entry in WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_entry(|e| {
            let name = e
                .file_name()
                .to_str()
                .unwrap_or("");
            // Exclude common dependency/build directories
            !EXCLUDED_DIRS.contains(&name)
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if TARGET_FILES.contains(&file_name) {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            match file_name {
                "Cargo.lock" | "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml" => {
                    lock_files.push(relative);
                }
                ".git" => {
                    dirs.push(relative);
                }
                _ => {
                    config_files.push(relative);
                }
            }
        }
    }

    if !config_files.is_empty() {
        findings.insert("config_files".to_string(), config_files);
    }
    if !lock_files.is_empty() {
        findings.insert("lock_files".to_string(), lock_files);
    }
    if !dirs.is_empty() {
        findings.insert("directories".to_string(), dirs);
    }

    Ok(findings)
}
