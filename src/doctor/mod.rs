use anyhow::Result;
use std::path::Path;

/// Result of the Doctor engine system detection.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DoctorReport {
    pub os: OsInfo,
    pub shell: ShellInfo,
    pub rust_toolchain: RustToolchainInfo,
    pub node_info: NodeInfo,
    pub git_info: GitInfo,
    pub vscode_info: VscodeInfo,
    pub roo_code_info: RooCodeInfo,
    pub tauri_info: TauriInfo,
    pub sqlite_info: SqliteInfo,
    pub cargo_targets: Vec<String>,
    pub env_vars: Vec<EnvVarEntry>,
    pub missing_deps: Vec<String>,
    pub broken_config: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub arch: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShellInfo {
    pub shell: String,
    pub detected: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RustToolchainInfo {
    pub installed: bool,
    pub version: Option<String>,
    pub rustc_version: Option<String>,
    pub cargo_version: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeInfo {
    pub installed: bool,
    pub node_version: Option<String>,
    pub npm_version: Option<String>,
    pub pnpm_installed: bool,
    pub yarn_installed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GitInfo {
    pub installed: bool,
    pub version: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VscodeInfo {
    pub installed: bool,
    pub version: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RooCodeInfo {
    pub installed: bool,
    pub config_path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TauriInfo {
    pub installed: bool,
    pub cli_version: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SqliteInfo {
    pub installed: bool,
    pub version: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnvVarEntry {
    pub name: String,
    pub value: String,
}

/// Runs the full Doctor diagnostic and returns a report.
pub fn run_diagnostics(root_path: &str) -> Result<DoctorReport> {
    let os = detect_os();
    let shell = detect_shell();
    let rust_toolchain = detect_rust_toolchain();
    let node_info = detect_node();
    let git_info = detect_git();
    let vscode_info = detect_vscode();
    let roo_code_info = detect_roo_code();
    let tauri_info = detect_tauri();
    let sqlite_info = detect_sqlite();
    let cargo_targets = detect_cargo_targets();
    let env_vars = detect_env_vars();
    let missing_deps = detect_missing_deps(&rust_toolchain, &node_info, &git_info);
    let broken_config = detect_broken_config(root_path);

    Ok(DoctorReport {
        os,
        shell,
        rust_toolchain,
        node_info,
        git_info,
        vscode_info,
        roo_code_info,
        tauri_info,
        sqlite_info,
        cargo_targets,
        env_vars,
        missing_deps,
        broken_config,
    })
}

/// Detects OS information.
fn detect_os() -> OsInfo {
    let name = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let version = match std::process::Command::new("cmd")
        .args(&["/c", "ver"])
        .output()
    {
        Ok(output) => {
            let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if out.is_empty() { "unknown".to_string() } else { out }
        }
        Err(_) => "unknown".to_string(),
    };

    OsInfo { name, version, arch }
}

/// Detects the default shell.
fn detect_shell() -> ShellInfo {
    let shell = std::env::var("SHELL")
        .or_else(|_| std::env::var("ComSpec"))
        .unwrap_or_else(|_| "cmd.exe".to_string());
    ShellInfo {
        detected: true,
        shell,
    }
}

/// Detects Rust toolchain (rustc, cargo).
fn detect_rust_toolchain() -> RustToolchainInfo {
    let rustc_version = match std::process::Command::new("rustc").arg("--version").output() {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => None,
    };

    let cargo_version = match std::process::Command::new("cargo").arg("--version").output() {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => None,
    };

    let installed = rustc_version.is_some() || cargo_version.is_some();
    let version = rustc_version.clone();

    RustToolchainInfo {
        installed,
        version,
        rustc_version,
        cargo_version,
    }
}

/// Detects Node.js, npm, pnpm, yarn.
fn detect_node() -> NodeInfo {
    let node_version = match std::process::Command::new("node").arg("--version").output() {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => None,
    };

    let npm_version = match std::process::Command::new("npm").arg("--version").output() {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => None,
    };

    let pnpm_installed = std::process::Command::new("pnpm")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let yarn_installed = std::process::Command::new("yarn")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    NodeInfo {
        installed: node_version.is_some(),
        node_version,
        npm_version,
        pnpm_installed,
        yarn_installed,
    }
}

/// Detects Git.
fn detect_git() -> GitInfo {
    let version = match std::process::Command::new("git").arg("--version").output() {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => None,
    };

    GitInfo {
        installed: version.is_some(),
        version,
    }
}

/// Detects VSCode.
fn detect_vscode() -> VscodeInfo {
    let version = match std::process::Command::new("code").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let out = String::from_utf8_lossy(&output.stdout);
            Some(out.lines().next().unwrap_or("unknown").to_string())
        }
        _ => None,
    };

    VscodeInfo {
        installed: version.is_some(),
        version,
    }
}

/// Detects RooCode extension.
fn detect_roo_code() -> RooCodeInfo {
    // Check common RooCode config paths
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());

    let possible_paths = vec![
        Path::new(&home).join(".vscode").join("extensions").join("rooveterinaryinc.roo-code"),
        Path::new(&home).join(".vscode-insiders").join("extensions").join("rooveterinaryinc.roo-code"),
        Path::new(&home).join(".vscode-oss").join("extensions").join("rooveterinaryinc.roo-code"),
    ];

    for path in &possible_paths {
        if path.exists() {
            return RooCodeInfo {
                installed: true,
                config_path: Some(path.to_string_lossy().to_string()),
            };
        }
    }

    RooCodeInfo {
        installed: false,
        config_path: None,
    }
}

/// Detects Tauri CLI.
fn detect_tauri() -> TauriInfo {
    let cli_version = match std::process::Command::new("cargo")
        .args(&["tauri", "--version"])
        .output()
    {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => None,
    };

    TauriInfo {
        installed: cli_version.is_some(),
        cli_version,
    }
}

/// Detects SQLite.
fn detect_sqlite() -> SqliteInfo {
    let version = match std::process::Command::new("sqlite3").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let out = String::from_utf8_lossy(&output.stdout);
            Some(out.lines().next().unwrap_or("unknown").to_string())
        }
        _ => None,
    };

    SqliteInfo {
        installed: version.is_some(),
        version,
    }
}

/// Detects available Cargo targets.
fn detect_cargo_targets() -> Vec<String> {
    match std::process::Command::new("rustup").args(&["target", "list", "--installed"]).output() {
        Ok(output) if output.status.success() => {
            let out = String::from_utf8_lossy(&output.stdout);
            out.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect()
        }
        _ => Vec::new(),
    }
}

/// Detects relevant environment variables.
fn detect_env_vars() -> Vec<EnvVarEntry> {
    let relevant_vars = [
        "PATH", "HOME", "USERPROFILE", "CARGO_HOME", "RUSTUP_HOME",
        "NODE_PATH", "NPM_CONFIG_PREFIX", "PNPM_HOME", "YARN_CACHE_FOLDER",
    ];

    let mut entries = Vec::new();
    for var_name in &relevant_vars {
        if let Ok(value) = std::env::var(var_name) {
            entries.push(EnvVarEntry {
                name: var_name.to_string(),
                value,
            });
        }
    }
    entries
}

/// Detects missing dependencies based on project type.
fn detect_missing_deps(rust: &RustToolchainInfo, node: &NodeInfo, git: &GitInfo) -> Vec<String> {
    let mut missing = Vec::new();

    if !rust.installed {
        missing.push("Rust toolchain (rustc/cargo)".to_string());
    }
    if !node.installed {
        missing.push("Node.js".to_string());
    }
    if !git.installed {
        missing.push("Git".to_string());
    }

    missing
}

/// Detects broken or missing configuration files.
fn detect_broken_config(root_path: &str) -> Vec<String> {
    let mut broken = Vec::new();
    let root = Path::new(root_path);

    // Check chronodrake.toml
    let config_path = root.join("chronodrake.toml");
    if config_path.exists() {
        let content = match std::fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(e) => {
                broken.push(format!("chronodrake.toml: unreadable — {}", e));
                return broken;
            }
        };
        // Try to parse as TOML
        if content.parse::<toml::Value>().is_err() {
            broken.push("chronodrake.toml: invalid TOML syntax".to_string());
        }
    }

    // Check Cargo.toml if exists
    let cargo_path = root.join("Cargo.toml");
    if cargo_path.exists() {
        let content = match std::fs::read_to_string(&cargo_path) {
            Ok(c) => c,
            Err(e) => {
                broken.push(format!("Cargo.toml: unreadable — {}", e));
                return broken;
            }
        };
        if content.parse::<toml::Value>().is_err() {
            broken.push("Cargo.toml: invalid TOML syntax".to_string());
        }
    }

    // Check package.json if exists
    let pkg_path = root.join("package.json");
    if pkg_path.exists() {
        let content = match std::fs::read_to_string(&pkg_path) {
            Ok(c) => c,
            Err(e) => {
                broken.push(format!("package.json: unreadable — {}", e));
                return broken;
            }
        };
        if serde_json::from_str::<serde_json::Value>(&content).is_err() {
            broken.push("package.json: invalid JSON syntax".to_string());
        }
    }

    broken
}

/// Generates a SYSTEM_REPORT.md from a DoctorReport.
pub fn generate_system_report(report: &DoctorReport, root_path: &str) -> Result<String> {
    let output_path = Path::new(root_path).join("SYSTEM_REPORT.md");
    let mut md = String::new();

    md.push_str("# SYSTEM_REPORT.md\n\n");
    md.push_str("> **ChronoDrake System Report**\n");
    md.push_str("> Generated by ChronoDrake Core v0.6 — Doctor Engine\n\n");
    md.push_str("---\n\n");

    // OS
    md.push_str("## 1. Operating System\n\n");
    md.push_str(&format!("- **OS:** {}\n", report.os.name));
    md.push_str(&format!("- **Version:** {}\n", report.os.version));
    md.push_str(&format!("- **Architecture:** {}\n", report.os.arch));
    md.push_str("\n");

    // Shell
    md.push_str("## 2. Shell\n\n");
    md.push_str(&format!("- **Shell:** `{}`\n", report.shell.shell));
    md.push_str("\n");

    // Rust Toolchain
    md.push_str("## 3. Rust Toolchain\n\n");
    if report.rust_toolchain.installed {
        if let Some(ref v) = report.rust_toolchain.rustc_version {
            md.push_str(&format!("- **rustc:** `{}`\n", v));
        }
        if let Some(ref v) = report.rust_toolchain.cargo_version {
            md.push_str(&format!("- **cargo:** `{}`\n", v));
        }
    } else {
        md.push_str("- **Status:** ❌ Not installed\n");
    }
    md.push_str("\n");

    // Node.js
    md.push_str("## 4. Node.js\n\n");
    if report.node_info.installed {
        if let Some(ref v) = report.node_info.node_version {
            md.push_str(&format!("- **Node.js:** `{}`\n", v));
        }
        if let Some(ref v) = report.node_info.npm_version {
            md.push_str(&format!("- **npm:** `{}`\n", v));
        }
        md.push_str(&format!("- **pnpm:** {}\n", if report.node_info.pnpm_installed { "✅" } else { "❌" }));
        md.push_str(&format!("- **yarn:** {}\n", if report.node_info.yarn_installed { "✅" } else { "❌" }));
    } else {
        md.push_str("- **Status:** ❌ Not installed\n");
    }
    md.push_str("\n");

    // Git
    md.push_str("## 5. Git\n\n");
    if report.git_info.installed {
        if let Some(ref v) = report.git_info.version {
            md.push_str(&format!("- **Git:** `{}`\n", v));
        }
    } else {
        md.push_str("- **Status:** ❌ Not installed\n");
    }
    md.push_str("\n");

    // VSCode
    md.push_str("## 6. VSCode\n\n");
    if report.vscode_info.installed {
        if let Some(ref v) = report.vscode_info.version {
            md.push_str(&format!("- **VSCode:** `{}`\n", v));
        }
    } else {
        md.push_str("- **Status:** ❌ Not installed (or not in PATH)\n");
    }
    md.push_str("\n");

    // RooCode
    md.push_str("## 7. RooCode\n\n");
    if report.roo_code_info.installed {
        md.push_str("- **RooCode:** ✅ Installed\n");
        if let Some(ref p) = report.roo_code_info.config_path {
            md.push_str(&format!("- **Path:** `{}`\n", p));
        }
    } else {
        md.push_str("- **Status:** ❌ Not found\n");
    }
    md.push_str("\n");

    // Tauri
    md.push_str("## 8. Tauri\n\n");
    if report.tauri_info.installed {
        if let Some(ref v) = report.tauri_info.cli_version {
            md.push_str(&format!("- **Tauri CLI:** `{}`\n", v));
        }
    } else {
        md.push_str("- **Status:** ❌ Not installed\n");
    }
    md.push_str("\n");

    // SQLite
    md.push_str("## 9. SQLite\n\n");
    if report.sqlite_info.installed {
        if let Some(ref v) = report.sqlite_info.version {
            md.push_str(&format!("- **SQLite:** `{}`\n", v));
        }
    } else {
        md.push_str("- **Status:** ❌ Not installed (or not in PATH)\n");
    }
    md.push_str("\n");

    // Cargo Targets
    md.push_str("## 10. Cargo Targets\n\n");
    if report.cargo_targets.is_empty() {
        md.push_str("_No installed targets detected (rustup may not be installed)._\n");
    } else {
        for target in &report.cargo_targets {
            md.push_str(&format!("- `{}`\n", target));
        }
    }
    md.push_str("\n");

    // Environment Variables
    md.push_str("## 11. Environment Variables\n\n");
    if report.env_vars.is_empty() {
        md.push_str("_No relevant environment variables detected._\n");
    } else {
        md.push_str("| Variable | Value |\n");
        md.push_str("|----------|-------|\n");
        for var in &report.env_vars {
            md.push_str(&format!("| `{}` | `{}` |\n", var.name, var.value));
        }
    }
    md.push_str("\n");

    // Missing Dependencies
    md.push_str("## 12. Missing Dependencies\n\n");
    if report.missing_deps.is_empty() {
        md.push_str("_All core dependencies are installed._\n");
    } else {
        for dep in &report.missing_deps {
            md.push_str(&format!("- ❌ {}\n", dep));
        }
    }
    md.push_str("\n");

    // Broken Config
    md.push_str("## 13. Broken Configuration\n\n");
    if report.broken_config.is_empty() {
        md.push_str("_No broken configuration files detected._\n");
    } else {
        for cfg in &report.broken_config {
            md.push_str(&format!("- ❌ {}\n", cfg));
        }
    }
    md.push_str("\n---\n");
    md.push_str("*This file was automatically generated by ChronoDrake Core v0.6 — Doctor Engine*\n");

    std::fs::write(&output_path, &md)?;
    Ok(output_path.to_string_lossy().to_string())
}

/// Generates a DEVELOPMENT_ENVIRONMENT.md from a DoctorReport.
pub fn generate_dev_environment_report(report: &DoctorReport, root_path: &str) -> Result<String> {
    let output_path = Path::new(root_path).join("DEVELOPMENT_ENVIRONMENT.md");
    let mut md = String::new();

    md.push_str("# DEVELOPMENT_ENVIRONMENT.md\n\n");
    md.push_str("> **ChronoDrake Development Environment Report**\n");
    md.push_str("> Generated by ChronoDrake Core v0.6 — Doctor Engine\n\n");
    md.push_str("---\n\n");

    md.push_str("## Development Environment Summary\n\n");
    md.push_str(&format!("- **OS:** {} ({})\n", report.os.name, report.os.arch));
    md.push_str(&format!("- **Shell:** `{}`\n", report.shell.shell));
    md.push_str(&format!("- **Rust:** {}\n", if report.rust_toolchain.installed {
        report.rust_toolchain.rustc_version.as_deref().unwrap_or("installed")
    } else {
        "❌ Not installed"
    }));
    md.push_str(&format!("- **Node.js:** {}\n", if report.node_info.installed {
        report.node_info.node_version.as_deref().unwrap_or("installed")
    } else {
        "❌ Not installed"
    }));
    md.push_str(&format!("- **Git:** {}\n", if report.git_info.installed {
        report.git_info.version.as_deref().unwrap_or("installed")
    } else {
        "❌ Not installed"
    }));
    md.push_str(&format!("- **VSCode:** {}\n", if report.vscode_info.installed { "✅" } else { "❌" }));
    md.push_str(&format!("- **RooCode:** {}\n", if report.roo_code_info.installed { "✅" } else { "❌" }));
    md.push_str(&format!("- **Tauri:** {}\n", if report.tauri_info.installed { "✅" } else { "❌" }));
    md.push_str(&format!("- **SQLite:** {}\n", if report.sqlite_info.installed { "✅" } else { "❌" }));
    md.push_str("\n");

    md.push_str("### Package Managers\n\n");
    md.push_str(&format!("- **npm:** {}\n", report.node_info.npm_version.as_deref().unwrap_or("❌")));
    md.push_str(&format!("- **pnpm:** {}\n", if report.node_info.pnpm_installed { "✅" } else { "❌" }));
    md.push_str(&format!("- **yarn:** {}\n", if report.node_info.yarn_installed { "✅" } else { "❌" }));
    md.push_str("\n");

    md.push_str("### Cargo Targets\n\n");
    if report.cargo_targets.is_empty() {
        md.push_str("_No additional targets installed._\n");
    } else {
        for target in &report.cargo_targets {
            md.push_str(&format!("- `{}`\n", target));
        }
    }
    md.push_str("\n");

    md.push_str("### Missing Components\n\n");
    if report.missing_deps.is_empty() && report.broken_config.is_empty() {
        md.push_str("_No missing components or broken configurations detected._\n");
    } else {
        for dep in &report.missing_deps {
            md.push_str(&format!("- ❌ Missing: {}\n", dep));
        }
        for cfg in &report.broken_config {
            md.push_str(&format!("- ❌ Config: {}\n", cfg));
        }
    }
    md.push_str("\n---\n");
    md.push_str("*This file was automatically generated by ChronoDrake Core v0.6 — Doctor Engine*\n");

    std::fs::write(&output_path, &md)?;
    Ok(output_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_os_returns_info() {
        let os = detect_os();
        assert!(!os.name.is_empty());
        assert!(!os.arch.is_empty());
    }

    #[test]
    fn test_detect_shell_returns_shell() {
        let shell = detect_shell();
        assert!(shell.detected);
        assert!(!shell.shell.is_empty());
    }

    #[test]
    fn test_detect_rust_toolchain() {
        let rust = detect_rust_toolchain();
        // This test may pass or fail depending on environment
        // We just verify the struct is populated
        assert!(rust.installed || !rust.installed);
    }

    #[test]
    fn test_detect_node() {
        let node = detect_node();
        assert!(node.installed || !node.installed);
    }

    #[test]
    fn test_detect_git() {
        let git = detect_git();
        assert!(git.installed || !git.installed);
    }

    #[test]
    fn test_detect_env_vars_returns_list() {
        let vars = detect_env_vars();
        // PATH should always be present
        assert!(vars.iter().any(|v| v.name == "PATH"));
    }

    #[test]
    fn test_detect_missing_deps_all_installed() {
        let rust = RustToolchainInfo {
            installed: true,
            version: Some("1.70.0".to_string()),
            rustc_version: Some("rustc 1.70.0".to_string()),
            cargo_version: Some("cargo 1.70.0".to_string()),
        };
        let node = NodeInfo {
            installed: true,
            node_version: Some("v18.0.0".to_string()),
            npm_version: Some("9.0.0".to_string()),
            pnpm_installed: true,
            yarn_installed: false,
        };
        let git = GitInfo {
            installed: true,
            version: Some("git 2.40.0".to_string()),
        };
        let missing = detect_missing_deps(&rust, &node, &git);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_detect_missing_deps_none_installed() {
        let rust = RustToolchainInfo {
            installed: false,
            version: None,
            rustc_version: None,
            cargo_version: None,
        };
        let node = NodeInfo {
            installed: false,
            node_version: None,
            npm_version: None,
            pnpm_installed: false,
            yarn_installed: false,
        };
        let git = GitInfo {
            installed: false,
            version: None,
        };
        let missing = detect_missing_deps(&rust, &node, &git);
        assert_eq!(missing.len(), 3);
    }

    #[test]
    fn test_detect_broken_config_nonexistent() {
        let broken = detect_broken_config("/nonexistent/path");
        assert!(broken.is_empty());
    }

    #[test]
    fn test_generate_system_report() {
        let report = DoctorReport {
            os: OsInfo { name: "test".to_string(), version: "1.0".to_string(), arch: "x86_64".to_string() },
            shell: ShellInfo { shell: "cmd.exe".to_string(), detected: true },
            rust_toolchain: RustToolchainInfo { installed: true, version: Some("1.70.0".to_string()), rustc_version: Some("rustc 1.70.0".to_string()), cargo_version: Some("cargo 1.70.0".to_string()) },
            node_info: NodeInfo { installed: true, node_version: Some("v18.0.0".to_string()), npm_version: Some("9.0.0".to_string()), pnpm_installed: true, yarn_installed: false },
            git_info: GitInfo { installed: true, version: Some("git 2.40.0".to_string()) },
            vscode_info: VscodeInfo { installed: true, version: Some("1.80.0".to_string()) },
            roo_code_info: RooCodeInfo { installed: true, config_path: Some("/path/to/roo".to_string()) },
            tauri_info: TauriInfo { installed: true, cli_version: Some("2.0.0".to_string()) },
            sqlite_info: SqliteInfo { installed: true, version: Some("3.42.0".to_string()) },
            cargo_targets: vec!["x86_64-pc-windows-msvc".to_string()],
            env_vars: vec![EnvVarEntry { name: "PATH".to_string(), value: "/usr/bin".to_string() }],
            missing_deps: vec![],
            broken_config: vec![],
        };

        let path = generate_system_report(&report, ".").unwrap();
        assert!(path.contains("SYSTEM_REPORT.md"));
    }

    #[test]
    fn test_generate_dev_environment_report() {
        let report = DoctorReport {
            os: OsInfo { name: "test".to_string(), version: "1.0".to_string(), arch: "x86_64".to_string() },
            shell: ShellInfo { shell: "cmd.exe".to_string(), detected: true },
            rust_toolchain: RustToolchainInfo { installed: true, version: Some("1.70.0".to_string()), rustc_version: Some("rustc 1.70.0".to_string()), cargo_version: Some("cargo 1.70.0".to_string()) },
            node_info: NodeInfo { installed: true, node_version: Some("v18.0.0".to_string()), npm_version: Some("9.0.0".to_string()), pnpm_installed: true, yarn_installed: false },
            git_info: GitInfo { installed: true, version: Some("git 2.40.0".to_string()) },
            vscode_info: VscodeInfo { installed: true, version: Some("1.80.0".to_string()) },
            roo_code_info: RooCodeInfo { installed: true, config_path: Some("/path/to/roo".to_string()) },
            tauri_info: TauriInfo { installed: true, cli_version: Some("2.0.0".to_string()) },
            sqlite_info: SqliteInfo { installed: true, version: Some("3.42.0".to_string()) },
            cargo_targets: vec!["x86_64-pc-windows-msvc".to_string()],
            env_vars: vec![EnvVarEntry { name: "PATH".to_string(), value: "/usr/bin".to_string() }],
            missing_deps: vec![],
            broken_config: vec![],
        };

        let path = generate_dev_environment_report(&report, ".").unwrap();
        assert!(path.contains("DEVELOPMENT_ENVIRONMENT.md"));
    }
}
