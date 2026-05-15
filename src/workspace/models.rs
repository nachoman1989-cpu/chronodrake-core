use serde::{Deserialize, Serialize};

/// Represents a single registered project in the ChronoDrake workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredProject {
    /// Human-readable project name (auto-detected from Cargo.toml or package.json)
    pub name: String,
    /// Absolute filesystem path to the project root
    pub path: String,
    /// Detected technology stack (e.g., "Rust", "Node", "Tauri", "Rust/Node")
    pub stack: String,
    /// ISO-8601 timestamp of the last scan, if any
    pub last_scan: Option<String>,
}

/// The global project registry stored at `~/.chronodrake/projects.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRegistry {
    /// Name of the last active project (for `current` and `load` commands)
    pub last_project: Option<String>,
    /// All registered projects
    pub projects: Vec<RegisteredProject>,
}

impl ProjectRegistry {
    /// Creates a new, empty registry.
    pub fn new() -> Self {
        ProjectRegistry {
            last_project: None,
            projects: Vec::new(),
        }
    }
}

impl Default for ProjectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary information displayed when loading a project.
#[derive(Debug, Clone)]
pub struct ProjectSummary {
    pub name: String,
    pub path: String,
    pub stack: String,
    pub last_scan: Option<String>,
    pub architecture_score: Option<u8>,
    pub security_score: Option<u8>,
    pub maturity_overall: Option<u8>,
    pub module_count: Option<usize>,
    pub has_master_context: bool,
    pub has_current_state: bool,
    pub has_next_steps: bool,
    pub has_decisions: bool,
    pub has_active_task: bool,
    pub has_project_identity: bool,
    pub has_ai_boot_prompt: bool,
    pub boot_prompt_content: Option<String>,
}
