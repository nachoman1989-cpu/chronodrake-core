pub mod filesystem;
pub mod rust;
pub mod node;
pub mod tauri;

use anyhow::Result;
use std::path::Path;

use crate::models::ProjectScan;

/// Orchestrates the full project scan by running all sub-scanners.
///
/// This function walks the given directory, detects technologies,
/// and returns a complete [`ProjectScan`] with all findings.
pub fn scan_project(root_path: &str) -> Result<ProjectScan> {
    let mut scan = ProjectScan::new(root_path);

    // Run filesystem scanner (walks directory, finds key files)
    let findings = filesystem::scan_filesystem(root_path)?;
    scan.findings = findings;

    // Detect Rust / Cargo — search root and common subdirectories
    let cargo_result = detect_any_rust(root_path)?;
    scan.has_cargo = cargo_result;
    scan.add_tech("Rust", "Language", cargo_result, None);

    // Detect Node.js — search root and common subdirectories
    let node_result = detect_any_node(root_path)?;
    scan.has_package_json = node_result;
    scan.add_tech("Node.js", "Runtime", node_result, None);

    // Detect Tauri — search root and src-tauri
    let tauri_result = tauri::detect_tauri(root_path)?;
    scan.has_tauri = tauri_result;
    scan.add_tech("Tauri", "Framework", tauri_result, None);

    // Detect Git
    let git_path = Path::new(root_path).join(".git");
    scan.has_git = git_path.exists();
    scan.add_tech("Git", "VCS", scan.has_git, None);

    Ok(scan)
}

/// Searches for any Cargo.toml in root or common subdirectories.
fn detect_any_rust(root_path: &str) -> Result<bool> {
    // Check root first
    if rust::detect_cargo(root_path)? {
        return Ok(true);
    }

    // Check common subdirectories
    let subdirs = ["src-tauri", "backend", "rust", "server"];
    for sub in &subdirs {
        let sub_path = Path::new(root_path).join(sub);
        if sub_path.exists() && rust::detect_cargo(sub_path.to_str().unwrap_or(""))? {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Searches for any package.json in root or common subdirectories.
fn detect_any_node(root_path: &str) -> Result<bool> {
    // Check root first
    if node::detect_node(root_path)? {
        return Ok(true);
    }

    // Check common subdirectories
    let subdirs = ["frontend", "client", "web", "app", "ui"];
    for sub in &subdirs {
        let sub_path = Path::new(root_path).join(sub);
        if sub_path.exists() && node::detect_node(sub_path.to_str().unwrap_or(""))? {
            return Ok(true);
        }
    }

    Ok(false)
}
