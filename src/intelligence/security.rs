use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::models::ProjectScan;

/// Security findings from the project scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFindings {
    /// List of security issues found.
    pub issues: Vec<SecurityIssue>,
    /// Number of issues found.
    pub issue_count: usize,
    /// Overall security rating (safe, warning, critical).
    pub rating: SecurityRating,
    /// Human-readable summary.
    pub summary: String,
}

/// A single security issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    /// Severity level.
    pub severity: SecuritySeverity,
    /// Category of the issue.
    pub category: String,
    /// Description of the issue.
    pub description: String,
    /// Optional file path where the issue was found.
    pub file: Option<String>,
    /// Suggestion for remediation.
    pub suggestion: String,
}

/// Security severity levels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecuritySeverity {
    Critical,
    Warning,
    Info,
}

impl std::fmt::Display for SecuritySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecuritySeverity::Critical => write!(f, "Critical"),
            SecuritySeverity::Warning => write!(f, "Warning"),
            SecuritySeverity::Info => write!(f, "Info"),
        }
    }
}

/// Overall security rating.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityRating {
    Safe,
    Warning,
    Critical,
}

impl std::fmt::Display for SecurityRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityRating::Safe => write!(f, "Safe"),
            SecurityRating::Warning => write!(f, "Warning"),
            SecurityRating::Critical => write!(f, "Critical"),
        }
    }
}

/// Scans a project for common security issues.
///
/// This is a deterministic, filesystem-based security scan.
/// It does NOT use AI or external APIs.
pub fn scan_security(scan: &ProjectScan, root_path: &str) -> Result<SecurityFindings> {
    let mut issues: Vec<SecurityIssue> = Vec::new();
    let root = Path::new(root_path);

    // 1. Check for .env files (potential secret leakage)
    if root.join(".env").exists() {
        issues.push(SecurityIssue {
            severity: SecuritySeverity::Critical,
            category: "Secret Management".to_string(),
            description: ".env file detected in project root. This file may contain secrets and should not be committed to version control.".to_string(),
            file: Some(".env".to_string()),
            suggestion: "Add .env to .gitignore and use .env.example for documentation.".to_string(),
        });
    }

    // 2. Check for .gitignore
    if !root.join(".gitignore").exists() {
        issues.push(SecurityIssue {
            severity: SecuritySeverity::Warning,
            category: "Version Control".to_string(),
            description: "No .gitignore file found. Sensitive files may be accidentally committed.".to_string(),
            file: None,
            suggestion: "Create a .gitignore file appropriate for your technology stack.".to_string(),
        });
    }

    // 3. Check for hardcoded port numbers in config files
    let config_files_to_check = [
        "tauri.conf.json",
        "src-tauri/tauri.conf.json",
        "vite.config.ts",
        "webpack.config.js",
    ];
    for config_file in &config_files_to_check {
        let cfg_path = root.join(config_file);
        if cfg_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&cfg_path) {
                // Check for common default ports
                if content.contains(":3000") || content.contains(":8080") || content.contains(":5432") {
                    issues.push(SecurityIssue {
                        severity: SecuritySeverity::Info,
                        category: "Configuration".to_string(),
                        description: format!("Default port detected in {}. Consider using environment variables for port configuration.", config_file),
                        file: Some(config_file.to_string()),
                        suggestion: "Use environment variables (e.g., PORT=3000) instead of hardcoded ports.".to_string(),
                    });
                }
            }
        }
    }

    // 4. Check for exposed API keys or tokens in config files
    let sensitive_patterns = ["api_key", "apiKey", "API_KEY", "secret", "SECRET", "password", "PASSWORD", "token", "TOKEN"];
    for entry in walkdir::WalkDir::new(root)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy();
                // Only check config/source files
                if matches!(&*ext_str, "rs" | "ts" | "js" | "json" | "toml" | "yaml" | "yml" | "env") {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        let lower = content.to_lowercase();
                        for pattern in &sensitive_patterns {
                            if lower.contains(&pattern.to_lowercase()) {
                                // Check if it has an assignment (potential hardcoded value)
                                if content.contains(&format!("{} =", pattern))
                                    || content.contains(&format!("{}:", pattern))
                                    || content.contains(&format!("\"{}\"", pattern))
                                {
                                    let relative = path.strip_prefix(root)
                                        .unwrap_or(path)
                                        .to_string_lossy()
                                        .to_string();
                                    issues.push(SecurityIssue {
                                        severity: SecuritySeverity::Warning,
                                        category: "Hardcoded Secrets".to_string(),
                                        description: format!("Potential hardcoded secret '{}' found in {}", pattern, relative),
                                        file: Some(relative),
                                        suggestion: "Use environment variables or a secrets manager instead of hardcoding sensitive values.".to_string(),
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 5. Check for outdated dependency files without lockfiles
    if scan.has_cargo && !scan.findings.get("lock_files").map_or(false, |f| f.iter().any(|lf| lf.contains("Cargo.lock"))) {
        issues.push(SecurityIssue {
            severity: SecuritySeverity::Warning,
            category: "Dependency Management".to_string(),
            description: "Cargo.toml found but no Cargo.lock. Lockfile ensures reproducible builds.".to_string(),
            file: None,
            suggestion: "Run 'cargo build' to generate Cargo.lock and commit it to version control.".to_string(),
        });
    }

    if scan.has_package_json && !scan.findings.get("lock_files").map_or(false, |f| f.iter().any(|lf| lf.contains("package-lock") || lf.contains("yarn.lock") || lf.contains("pnpm-lock"))) {
        issues.push(SecurityIssue {
            severity: SecuritySeverity::Warning,
            category: "Dependency Management".to_string(),
            description: "package.json found but no lockfile (package-lock.json, yarn.lock, or pnpm-lock.yaml).".to_string(),
            file: None,
            suggestion: "Run 'npm install' to generate package-lock.json and commit it.".to_string(),
        });
    }

    // Determine rating
    let critical_count = issues.iter().filter(|i| i.severity == SecuritySeverity::Critical).count();
    let warning_count = issues.iter().filter(|i| i.severity == SecuritySeverity::Warning).count();

    let rating = if critical_count > 0 {
        SecurityRating::Critical
    } else if warning_count > 2 {
        SecurityRating::Warning
    } else {
        SecurityRating::Safe
    };

    let summary = format!(
        "Security rating: **{}** — {} issues found ({} critical, {} warnings, {} info)",
        rating,
        issues.len(),
        critical_count,
        warning_count,
        issues.iter().filter(|i| i.severity == SecuritySeverity::Info).count(),
    );

    Ok(SecurityFindings {
        issue_count: issues.len(),
        issues,
        rating,
        summary,
    })
}
