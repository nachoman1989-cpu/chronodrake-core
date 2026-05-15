use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Represents the complete scan result of a project directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectScan {
    /// The root path that was scanned.
    pub root_path: String,
    /// Timestamp of the scan.
    pub scanned_at: String,
    /// Detected technologies found in the project.
    pub technologies: Vec<DetectedTech>,
    /// Whether a Git repository was detected.
    pub has_git: bool,
    /// Whether a Cargo.toml (Rust) was detected.
    pub has_cargo: bool,
    /// Whether a package.json (Node.js) was detected.
    pub has_package_json: bool,
    /// Whether a tauri.conf.json (Tauri) was detected.
    pub has_tauri: bool,
    /// Raw file findings for detailed reporting.
    pub findings: HashMap<String, Vec<String>>,
}

/// Represents a single detected technology or framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTech {
    /// Name of the technology (e.g., "Rust", "Node.js", "Tauri").
    pub name: String,
    /// Category grouping (e.g., "Language", "Framework", "Tool").
    pub category: String,
    /// Whether the detection was successful.
    pub detected: bool,
    /// Optional version or detail string.
    pub detail: Option<String>,
}

impl ProjectScan {
    /// Creates a new empty ProjectScan for the given root path.
    pub fn new(root_path: &str) -> Self {
        Self {
            root_path: root_path.to_string(),
            scanned_at: chrono::Utc::now().to_rfc3339(),
            technologies: Vec::new(),
            has_git: false,
            has_cargo: false,
            has_package_json: false,
            has_tauri: false,
            findings: HashMap::new(),
        }
    }

    /// Adds a detected technology to the scan result.
    pub fn add_tech(&mut self, name: &str, category: &str, detected: bool, detail: Option<String>) {
        self.technologies.push(DetectedTech {
            name: name.to_string(),
            category: category.to_string(),
            detected,
            detail,
        });
    }

    /// Adds a finding entry (file path -> list of matched items).
    pub fn add_finding(&mut self, key: &str, values: Vec<String>) {
        self.findings.insert(key.to_string(), values);
    }
}
