use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::models::ProjectScan;

/// Result of dependency analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    /// Total number of dependencies found.
    pub total_count: usize,
    /// List of detected dependencies with their details.
    pub dependencies: Vec<DependencyInfo>,
    /// Number of outdated or notable dependencies.
    pub notable_count: usize,
    /// Human-readable summary.
    pub summary: String,
}

/// Information about a single dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    /// Name of the dependency.
    pub name: String,
    /// Version string.
    pub version: String,
    /// Ecosystem (e.g., "cargo", "npm").
    pub ecosystem: String,
    /// Category (e.g., "runtime", "dev", "build").
    pub category: String,
    /// Whether this is a notable dependency.
    pub notable: bool,
    /// Optional description of why it's notable.
    pub note: Option<String>,
}

/// Analyzes the dependencies of a project from Cargo.toml and/or package.json.
pub fn analyze_dependencies(_scan: &ProjectScan, root_path: &str) -> Result<DependencyAnalysis> {
    let mut dependencies: Vec<DependencyInfo> = Vec::new();
    let mut notable_count = 0;

    // Analyze Cargo.toml dependencies
    let cargo_paths = [
        Path::new(root_path).join("Cargo.toml"),
        Path::new(root_path).join("src-tauri").join("Cargo.toml"),
    ];
    for cargo_path in &cargo_paths {
        if cargo_path.exists() {
            if let Ok(content) = std::fs::read_to_string(cargo_path) {
                if let Ok(parsed) = toml::from_str::<toml::Value>(&content) {
                    // Runtime dependencies
                    if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) {
                        for (name, value) in deps {
                            let version = extract_version(value);
                            let notable = is_notable_crate(name);
                            if notable {
                                notable_count += 1;
                            }
                            dependencies.push(DependencyInfo {
                                name: name.clone(),
                                version,
                                ecosystem: "cargo".to_string(),
                                category: "runtime".to_string(),
                                notable,
                                note: if notable { Some(get_crate_note(name)) } else { None },
                            });
                        }
                    }

                    // Dev dependencies
                    if let Some(deps) = parsed.get("dev-dependencies").and_then(|d| d.as_table()) {
                        for (name, value) in deps {
                            dependencies.push(DependencyInfo {
                                name: name.clone(),
                                version: extract_version(value),
                                ecosystem: "cargo".to_string(),
                                category: "dev".to_string(),
                                notable: false,
                                note: None,
                            });
                        }
                    }

                    // Build dependencies
                    if let Some(deps) = parsed.get("build-dependencies").and_then(|d| d.as_table()) {
                        for (name, value) in deps {
                            dependencies.push(DependencyInfo {
                                name: name.clone(),
                                version: extract_version(value),
                                ecosystem: "cargo".to_string(),
                                category: "build".to_string(),
                                notable: false,
                                note: None,
                            });
                        }
                    }
                }
            }
        }
    }

    // Analyze package.json dependencies
    let pkg_path = Path::new(root_path).join("package.json");
    if pkg_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                for (category_key, category_name) in &[("dependencies", "runtime"), ("devDependencies", "dev"), ("peerDependencies", "peer")] {
                    if let Some(deps) = parsed.get(*category_key).and_then(|d| d.as_object()) {
                        for (name, value) in deps {
                            let version = value.as_str().unwrap_or("unknown").to_string();
                            dependencies.push(DependencyInfo {
                                name: name.clone(),
                                version,
                                ecosystem: "npm".to_string(),
                                category: category_name.to_string(),
                                notable: false,
                                note: None,
                            });
                        }
                    }
                }
            }
        }
    }

    let total_count = dependencies.len();
    let summary = format!(
        "{} dependencies found ({} cargo, {} npm). {} notable dependencies detected.",
        total_count,
        dependencies.iter().filter(|d| d.ecosystem == "cargo").count(),
        dependencies.iter().filter(|d| d.ecosystem == "npm").count(),
        notable_count,
    );

    Ok(DependencyAnalysis {
        total_count,
        dependencies,
        notable_count,
        summary,
    })
}

fn extract_version(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        toml::Value::Table(table) => {
            table.get("version").and_then(|v| v.as_str()).unwrap_or("latest").to_string()
        }
        _ => "unknown".to_string(),
    }
}

fn is_notable_crate(name: &str) -> bool {
    matches!(
        name,
        "tokio" | "serde" | "serde_json" | "anyhow" | "thiserror"
            | "tauri" | "tauri-build" | "clap" | "structopt"
            | "diesel" | "sqlx" | "sea-orm" | "rusqlite"
            | "actix-web" | "axum" | "rocket" | "warp"
            | "reqwest" | "hyper" | "tonic"
            | "candle" | "burn" | "tch" | "ort"
            | "alloy" | "ethers" | "solana"
            | "aes" | "sha2" | "rsa" | "bcrypt" | "argon2"
            | "wasm-bindgen" | "wasmer"
            | "egui" | "iced" | "druid"
    )
}

fn get_crate_note(name: &str) -> String {
    match name {
        "tokio" => "Async runtime — core for async Rust applications".to_string(),
        "serde" => "Serialization framework — essential for data handling".to_string(),
        "serde_json" => "JSON serialization".to_string(),
        "anyhow" => "Error handling — flexible error type".to_string(),
        "thiserror" => "Derive macro for custom error types".to_string(),
        "tauri" | "tauri-build" => "Desktop application framework (Tauri)".to_string(),
        "clap" | "structopt" => "CLI argument parsing".to_string(),
        "diesel" | "sqlx" | "sea-orm" => "Database ORM / SQL toolkit".to_string(),
        "actix-web" | "axum" | "rocket" | "warp" => "Web framework".to_string(),
        "reqwest" => "HTTP client".to_string(),
        "candle" | "burn" | "tch" => "Machine learning framework".to_string(),
        "alloy" | "ethers" | "solana" => "Blockchain / Web3 SDK".to_string(),
        "aes" | "sha2" | "rsa" | "bcrypt" | "argon2" => "Cryptography library".to_string(),
        "wasm-bindgen" | "wasmer" => "WebAssembly support".to_string(),
        "egui" | "iced" | "druid" => "GUI framework".to_string(),
        _ => "Notable dependency".to_string(),
    }
}
