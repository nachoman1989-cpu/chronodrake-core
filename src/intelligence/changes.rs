use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::models::ProjectScan;

/// Report of changes between the current scan and the previous snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeReport {
    /// Whether this is the first scan (no previous snapshot).
    pub is_first_scan: bool,
    /// List of changes detected.
    pub changes: Vec<Change>,
    /// Number of changes detected.
    pub change_count: usize,
    /// Human-readable summary.
    pub summary: String,
}

/// A single detected change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Type of change.
    pub change_type: ChangeType,
    /// Category of the change.
    pub category: String,
    /// Description of what changed.
    pub description: String,
}

/// Types of changes that can be detected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Added => write!(f, "Added"),
            ChangeType::Removed => write!(f, "Removed"),
            ChangeType::Modified => write!(f, "Modified"),
        }
    }
}

/// Snapshot of project state saved to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectSnapshot {
    scanned_at: String,
    technologies: Vec<String>,
    config_files: Vec<String>,
    lock_files: Vec<String>,
    has_git: bool,
    has_cargo: bool,
    has_package_json: bool,
    has_tauri: bool,
}

/// Detects changes by comparing the current scan with a previous snapshot.
///
/// Snapshots are stored as JSON files in `memory/snapshots/`.
pub fn detect_changes(scan: &ProjectScan, root_path: &str) -> Result<Option<ChangeReport>> {
    let snapshots_dir = Path::new(root_path).join("memory").join("snapshots");
    let snapshot_file = snapshots_dir.join("latest.json");

    // Build current snapshot
    let current = ProjectSnapshot {
        scanned_at: scan.scanned_at.clone(),
        technologies: scan.technologies.iter().map(|t| t.name.clone()).collect(),
        config_files: scan.findings.get("config_files").cloned().unwrap_or_default(),
        lock_files: scan.findings.get("lock_files").cloned().unwrap_or_default(),
        has_git: scan.has_git,
        has_cargo: scan.has_cargo,
        has_package_json: scan.has_package_json,
        has_tauri: scan.has_tauri,
    };

    // Ensure snapshots directory exists
    std::fs::create_dir_all(&snapshots_dir)?;

    // Check if previous snapshot exists
    if !snapshot_file.exists() {
        // First scan — save snapshot and return no changes
        let json = serde_json::to_string_pretty(&current)?;
        std::fs::write(&snapshot_file, &json)?;

        // Also save a timestamped copy
        let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let ts_file = snapshots_dir.join(format!("snapshot_{}.json", ts));
        std::fs::write(&ts_file, &json)?;

        return Ok(Some(ChangeReport {
            is_first_scan: true,
            changes: Vec::new(),
            change_count: 0,
            summary: "First scan — no previous snapshot to compare against. Baseline snapshot created.".to_string(),
        }));
    }

    // Load previous snapshot
    let previous_json = std::fs::read_to_string(&snapshot_file)?;
    let previous: ProjectSnapshot = serde_json::from_str(&previous_json)?;

    let mut changes: Vec<Change> = Vec::new();

    // Compare technologies
    for tech in &current.technologies {
        if !previous.technologies.contains(tech) {
            changes.push(Change {
                change_type: ChangeType::Added,
                category: "Technology".to_string(),
                description: format!("New technology detected: {}", tech),
            });
        }
    }
    for tech in &previous.technologies {
        if !current.technologies.contains(tech) {
            changes.push(Change {
                change_type: ChangeType::Removed,
                category: "Technology".to_string(),
                description: format!("Technology no longer detected: {}", tech),
            });
        }
    }

    // Compare config files
    for file in &current.config_files {
        if !previous.config_files.contains(file) {
            changes.push(Change {
                change_type: ChangeType::Added,
                category: "Configuration".to_string(),
                description: format!("New config file: {}", file),
            });
        }
    }
    for file in &previous.config_files {
        if !current.config_files.contains(file) {
            changes.push(Change {
                change_type: ChangeType::Removed,
                category: "Configuration".to_string(),
                description: format!("Config file removed: {}", file),
            });
        }
    }

    // Compare lock files
    for file in &current.lock_files {
        if !previous.lock_files.contains(file) {
            changes.push(Change {
                change_type: ChangeType::Added,
                category: "Dependencies".to_string(),
                description: format!("New lock file: {}", file),
            });
        }
    }
    for file in &previous.lock_files {
        if !current.lock_files.contains(file) {
            changes.push(Change {
                change_type: ChangeType::Removed,
                category: "Dependencies".to_string(),
                description: format!("Lock file removed: {}", file),
            });
        }
    }

    // Compare boolean flags
    if current.has_git != previous.has_git {
        changes.push(Change {
            change_type: if current.has_git { ChangeType::Added } else { ChangeType::Removed },
            category: "Version Control".to_string(),
            description: format!("Git repository {}", if current.has_git { "initialized" } else { "removed" }),
        });
    }
    if current.has_cargo != previous.has_cargo {
        changes.push(Change {
            change_type: if current.has_cargo { ChangeType::Added } else { ChangeType::Removed },
            category: "Language".to_string(),
            description: format!("Rust/Cargo {}", if current.has_cargo { "added" } else { "removed" }),
        });
    }
    if current.has_package_json != previous.has_package_json {
        changes.push(Change {
            change_type: if current.has_package_json { ChangeType::Added } else { ChangeType::Removed },
            category: "Language".to_string(),
            description: format!("Node.js/package.json {}", if current.has_package_json { "added" } else { "removed" }),
        });
    }
    if current.has_tauri != previous.has_tauri {
        changes.push(Change {
            change_type: if current.has_tauri { ChangeType::Added } else { ChangeType::Removed },
            category: "Framework".to_string(),
            description: format!("Tauri {}", if current.has_tauri { "added" } else { "removed" }),
        });
    }

    // Save current snapshot as latest
    let json = serde_json::to_string_pretty(&current)?;
    std::fs::write(&snapshot_file, &json)?;

    // Save timestamped copy
    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let ts_file = snapshots_dir.join(format!("snapshot_{}.json", ts));
    std::fs::write(&ts_file, &json)?;

    let change_count = changes.len();
    let summary = if change_count == 0 {
        "No changes detected since last scan.".to_string()
    } else {
        format!(
            "{} change(s) detected since last scan ({} added, {} removed, {} modified).",
            change_count,
            changes.iter().filter(|c| c.change_type == ChangeType::Added).count(),
            changes.iter().filter(|c| c.change_type == ChangeType::Removed).count(),
            changes.iter().filter(|c| c.change_type == ChangeType::Modified).count(),
        )
    };

    Ok(Some(ChangeReport {
        is_first_scan: false,
        changes,
        change_count,
        summary,
    }))
}
