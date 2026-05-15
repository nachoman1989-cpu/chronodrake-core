use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// ChronoDrake configuration loaded from `chronodrake.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronoDrakeConfig {
    /// Directories to exclude from scanning.
    #[serde(default = "default_excluded_dirs")]
    pub excluded_dirs: Vec<String>,
    /// Which reports to generate.
    #[serde(default)]
    pub reports: ReportConfig,
    /// Snapshot retention policy.
    #[serde(default)]
    pub snapshots: SnapshotConfig,
    /// Maximum scan depth for filesystem traversal.
    #[serde(default = "default_scan_depth")]
    pub scan_depth: usize,
    /// Logging level (trace, debug, info, warn, error).
    #[serde(default = "default_log_level")]
    pub log_level: String,
    /// Whether to enable incremental scanning.
    #[serde(default = "default_true")]
    pub incremental_scan: bool,
    /// Whether to enable the cache layer.
    #[serde(default = "default_true")]
    pub enable_cache: bool,
}

/// Controls which reports are generated after a scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Generate PROJECT_STATUS.md.
    #[serde(default = "default_true")]
    pub project_status: bool,
    /// Generate AI_HANDOFF.md.
    #[serde(default = "default_true")]
    pub ai_handoff: bool,
    /// Generate CHANGES.md.
    #[serde(default = "default_true")]
    pub changes: bool,
    /// Generate TIMELINE.md.
    #[serde(default = "default_true")]
    pub timeline: bool,
    /// Generate RECOMMENDATIONS.md.
    #[serde(default = "default_true")]
    pub recommendations: bool,
    /// Generate KNOWLEDGE_GRAPH.md.
    #[serde(default = "default_true")]
    pub knowledge_graph: bool,
    /// Generate IMPACT_ANALYSIS.md.
    #[serde(default = "default_true")]
    pub impact_analysis: bool,
    /// Generate MODULE_MAP.md.
    #[serde(default = "default_true")]
    pub module_map: bool,
}

/// Snapshot retention configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// Maximum number of snapshots to retain (0 = unlimited).
    #[serde(default = "default_snapshot_retention")]
    pub max_snapshots: usize,
    /// Automatically prune old snapshots.
    #[serde(default = "default_true")]
    pub auto_prune: bool,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            project_status: true,
            ai_handoff: true,
            changes: true,
            timeline: true,
            recommendations: true,
            knowledge_graph: true,
            impact_analysis: true,
            module_map: true,
        }
    }
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            max_snapshots: 10,
            auto_prune: true,
        }
    }
}

impl Default for ChronoDrakeConfig {
    fn default() -> Self {
        Self {
            excluded_dirs: default_excluded_dirs(),
            reports: ReportConfig::default(),
            snapshots: SnapshotConfig::default(),
            scan_depth: default_scan_depth(),
            log_level: default_log_level(),
            incremental_scan: true,
            enable_cache: true,
        }
    }
}

fn default_excluded_dirs() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        ".git".to_string(),
        "target".to_string(),
        "dist".to_string(),
        "build".to_string(),
        ".next".to_string(),
        ".cache".to_string(),
        "__pycache__".to_string(),
        ".venv".to_string(),
        "venv".to_string(),
        ".tox".to_string(),
    ]
}

fn default_scan_depth() -> usize {
    5
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_true() -> bool {
    true
}

fn default_snapshot_retention() -> usize {
    10
}

impl ChronoDrakeConfig {
    /// Loads configuration from `chronodrake.toml` in the given root path.
    /// If the file does not exist, returns the default configuration.
    pub fn load(root_path: &str) -> Result<Self> {
        let config_path = Path::new(root_path).join("chronodrake.toml");
        if !config_path.exists() {
            return Ok(ChronoDrakeConfig::default());
        }
        let content = std::fs::read_to_string(&config_path)?;
        let config: ChronoDrakeConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Generates a default `chronodrake.toml` file at the given path.
    pub fn generate_default(root_path: &str) -> Result<()> {
        let config_path = Path::new(root_path).join("chronodrake.toml");
        if config_path.exists() {
            return Ok(());
        }
        let config = ChronoDrakeConfig::default();
        let toml_string = toml::to_string_pretty(&config)?;
        std::fs::write(&config_path, toml_string)?;
        Ok(())
    }
}
