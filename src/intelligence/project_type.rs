use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::models::ProjectScan;

/// Types of projects ChronoDrake can detect.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectKind {
    DesktopApp,
    CliTool,
    WebApi,
    Frontend,
    Blockchain,
    AiProject,
    SecurityProject,
    Library,
    MobileApp,
    Unknown,
}

impl std::fmt::Display for ProjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectKind::DesktopApp => write!(f, "Desktop Application"),
            ProjectKind::CliTool => write!(f, "CLI Tool"),
            ProjectKind::WebApi => write!(f, "Web API"),
            ProjectKind::Frontend => write!(f, "Frontend Application"),
            ProjectKind::Blockchain => write!(f, "Blockchain / Web3"),
            ProjectKind::AiProject => write!(f, "AI / Machine Learning"),
            ProjectKind::SecurityProject => write!(f, "Security Tool"),
            ProjectKind::Library => write!(f, "Library / Package"),
            ProjectKind::MobileApp => write!(f, "Mobile Application"),
            ProjectKind::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Result of project type detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTypeResult {
    /// The primary detected project type.
    pub primary: ProjectKind,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f64,
    /// All matching types with reasons.
    pub matches: Vec<TypeMatch>,
    /// Human-readable summary.
    pub summary: String,
}

/// A single type match with evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMatch {
    pub kind: ProjectKind,
    pub reason: String,
    pub weight: u32,
}

/// Detects the project type based on files, dependencies, and structure.
pub fn detect_project_type(scan: &ProjectScan, root_path: &str) -> Result<ProjectTypeResult> {
    let mut matches: Vec<TypeMatch> = Vec::new();

    // --- Desktop App ---
    if scan.has_tauri {
        matches.push(TypeMatch {
            kind: ProjectKind::DesktopApp,
            reason: "Tauri configuration detected (tauri.conf.json)".to_string(),
            weight: 90,
        });
    }
    // Check for Electron
    if has_dependency(scan, root_path, "electron")? {
        matches.push(TypeMatch {
            kind: ProjectKind::DesktopApp,
            reason: "Electron dependency detected".to_string(),
            weight: 85,
        });
    }

    // --- CLI Tool ---
    if scan.has_cargo {
        // Check for clap or structopt in Cargo.toml
        if has_cargo_dep(root_path, "clap")? || has_cargo_dep(root_path, "structopt")? {
            matches.push(TypeMatch {
                kind: ProjectKind::CliTool,
                reason: "CLI framework detected (clap/structopt)".to_string(),
                weight: 80,
            });
        }
    }
    if has_dependency(scan, root_path, "commander")?
        || has_dependency(scan, root_path, "yargs")?
    {
        matches.push(TypeMatch {
            kind: ProjectKind::CliTool,
            reason: "Node.js CLI framework detected".to_string(),
            weight: 75,
        });
    }

    // --- Web API ---
    if has_cargo_dep(root_path, "actix-web")?
        || has_cargo_dep(root_path, "axum")?
        || has_cargo_dep(root_path, "rocket")?
        || has_cargo_dep(root_path, "warp")?
    {
        matches.push(TypeMatch {
            kind: ProjectKind::WebApi,
            reason: "Rust web framework detected".to_string(),
            weight: 85,
        });
    }
    if has_dependency(scan, root_path, "express")?
        || has_dependency(scan, root_path, "fastify")?
        || has_dependency(scan, root_path, "nestjs")?
    {
        matches.push(TypeMatch {
            kind: ProjectKind::WebApi,
            reason: "Node.js web framework detected".to_string(),
            weight: 80,
        });
    }

    // --- Frontend ---
    if has_dependency(scan, root_path, "react")?
        || has_dependency(scan, root_path, "vue")?
        || has_dependency(scan, root_path, "svelte")?
        || has_dependency(scan, root_path, "next")?
    {
        matches.push(TypeMatch {
            kind: ProjectKind::Frontend,
            reason: "Frontend framework detected".to_string(),
            weight: 80,
        });
    }
    // Check for vite.config.ts or tsconfig.json as frontend indicators
    if scan.findings.get("config_files").map_or(false, |files| {
        files.iter().any(|f| f.contains("vite.config") || f.contains("tsconfig.json"))
    }) {
        matches.push(TypeMatch {
            kind: ProjectKind::Frontend,
            reason: "Vite/TypeScript frontend config detected".to_string(),
            weight: 60,
        });
    }

    // --- Blockchain ---
    if has_cargo_dep(root_path, "alloy")?
        || has_cargo_dep(root_path, "ethers")?
        || has_cargo_dep(root_path, "solana")?
    {
        matches.push(TypeMatch {
            kind: ProjectKind::Blockchain,
            reason: "Blockchain SDK detected (alloy/ethers/solana)".to_string(),
            weight: 90,
        });
    }
    if has_dependency(scan, root_path, "web3")?
        || has_dependency(scan, root_path, "ethers")?
        || has_dependency(scan, root_path, "hardhat")?
    {
        matches.push(TypeMatch {
            kind: ProjectKind::Blockchain,
            reason: "Web3/Blockchain dependency detected".to_string(),
            weight: 85,
        });
    }

    // --- AI / ML ---
    if has_cargo_dep(root_path, "candle")?
        || has_cargo_dep(root_path, "burn")?
        || has_cargo_dep(root_path, "tch")?
    {
        matches.push(TypeMatch {
            kind: ProjectKind::AiProject,
            reason: "Rust ML framework detected (candle/burn/tch)".to_string(),
            weight: 90,
        });
    }
    if has_dependency(scan, root_path, "tensorflow")?
        || has_dependency(scan, root_path, "torch")?
        || has_dependency(scan, root_path, "transformers")?
        || has_dependency(scan, root_path, "langchain")?
    {
        matches.push(TypeMatch {
            kind: ProjectKind::AiProject,
            reason: "AI/ML dependency detected".to_string(),
            weight: 85,
        });
    }

    // --- Security ---
    if has_cargo_dep(root_path, "aes")?
        || has_cargo_dep(root_path, "sha2")?
        || has_cargo_dep(root_path, "rsa")?
        || has_cargo_dep(root_path, "crypto-vault")?
    {
        matches.push(TypeMatch {
            kind: ProjectKind::SecurityProject,
            reason: "Cryptography/Security crate detected".to_string(),
            weight: 85,
        });
    }

    // --- Library ---
    // If it has Cargo.toml or package.json but no app framework, it's likely a library
    if (scan.has_cargo || scan.has_package_json) && matches.is_empty() {
        matches.push(TypeMatch {
            kind: ProjectKind::Library,
            reason: "Package manifest found without application framework".to_string(),
            weight: 50,
        });
    }

    // Determine primary type (highest weight wins)
    let primary = matches
        .iter()
        .max_by_key(|m| m.weight)
        .map(|m| m.kind.clone())
        .unwrap_or(ProjectKind::Unknown);

    let _total_weight: u32 = matches.iter().map(|m| m.weight).sum();
    let max_weight = matches.iter().map(|m| m.weight).max().unwrap_or(1);
    let confidence = if matches.is_empty() {
        0.0
    } else {
        (max_weight as f64) / 100.0
    };

    let summary = if matches.is_empty() {
        "Unable to determine project type — no recognizable patterns found.".to_string()
    } else {
        let primary_name = &primary.to_string();
        let reasons: Vec<&str> = matches.iter().map(|m| m.reason.as_str()).collect();
        format!(
            "Detected as **{}** (confidence: {:.0}%). Evidence: {}.",
            primary_name,
            confidence * 100.0,
            reasons.join("; ")
        )
    };

    Ok(ProjectTypeResult {
        primary,
        confidence,
        matches,
        summary,
    })
}

/// Checks if a dependency exists in a Cargo.toml by reading it.
fn has_cargo_dep(root_path: &str, dep_name: &str) -> Result<bool> {
    let cargo_path = Path::new(root_path).join("Cargo.toml");
    if !cargo_path.exists() {
        // Also check src-tauri
        let alt = Path::new(root_path).join("src-tauri").join("Cargo.toml");
        if alt.exists() {
            return has_dep_in_file(&alt, dep_name);
        }
        return Ok(false);
    }
    has_dep_in_file(&cargo_path, dep_name)
}

fn has_dep_in_file(path: &Path, dep_name: &str) -> Result<bool> {
    let content = std::fs::read_to_string(path)?;
    let parsed: toml::Value = toml::from_str(&content)?;

    // Check [dependencies]
    if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) {
        if deps.contains_key(dep_name) {
            return Ok(true);
        }
    }

    // Check [dev-dependencies]
    if let Some(deps) = parsed.get("dev-dependencies").and_then(|d| d.as_table()) {
        if deps.contains_key(dep_name) {
            return Ok(true);
        }
    }

    // Check [build-dependencies]
    if let Some(deps) = parsed.get("build-dependencies").and_then(|d| d.as_table()) {
        if deps.contains_key(dep_name) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Checks if a dependency exists in package.json.
fn has_dependency(scan: &ProjectScan, root_path: &str, dep_name: &str) -> Result<bool> {
    if !scan.has_package_json {
        return Ok(false);
    }

    let pkg_path = Path::new(root_path).join("package.json");
    if !pkg_path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&pkg_path)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;

    // Check dependencies
    if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_object()) {
        if deps.contains_key(dep_name) {
            return Ok(true);
        }
    }

    // Check devDependencies
    if let Some(deps) = parsed.get("devDependencies").and_then(|d| d.as_object()) {
        if deps.contains_key(dep_name) {
            return Ok(true);
        }
    }

    // Check peerDependencies
    if let Some(deps) = parsed.get("peerDependencies").and_then(|d| d.as_object()) {
        if deps.contains_key(dep_name) {
            return Ok(true);
        }
    }

    Ok(false)
}
