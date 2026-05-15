use anyhow::Result;

use crate::config::ChronoDrakeConfig;
use crate::utils::Logger;

/// Shared context for all CLI commands.
///
/// Created once at startup, this struct owns the resolved root path and
/// loaded configuration, and ensures the logger is initialized exactly once
/// before any command executes.
///
/// # Example
///
/// ```ignore
/// let ctx = CliContext::new()?;
/// Logger::section(&format!("Running command in {}", ctx.root_path));
/// ```
pub struct CliContext {
    /// Absolute path to the current working directory (the project root).
    pub root_path: String,
    /// Loaded ChronoDrake configuration (from `chronodrake.toml` or defaults).
    pub config: ChronoDrakeConfig,
}

impl CliContext {
    /// Creates a new `CliContext` by detecting the current directory, loading
    /// the configuration, and initializing the global tracing logger.
    ///
    /// # Errors
    ///
    /// Returns an error if the current directory cannot be determined or the
    /// configuration file is malformed.
    pub fn new() -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        let root_path = current_dir.to_string_lossy().to_string();
        let config = ChronoDrakeConfig::load(&root_path)?;
        Logger::init(&config.log_level);
        Ok(Self { root_path, config })
    }
}

impl Default for CliContext {
    fn default() -> Self {
        // This should not be used in production — it exists for convenience
        // in tests and future command implementations that need a fallback.
        let root_path = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());
        let config = ChronoDrakeConfig::default();
        Logger::init(&config.log_level);
        Self { root_path, config }
    }
}
