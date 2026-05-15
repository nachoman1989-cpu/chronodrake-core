use anyhow::Result;
use std::path::Path;

/// Detects whether the given directory contains a Rust project.
///
/// Checks for the presence of `Cargo.toml` at the root level.
/// Returns `true` if a Cargo.toml file exists and can be parsed.
pub fn detect_cargo(root_path: &str) -> Result<bool> {
    let cargo_path = Path::new(root_path).join("Cargo.toml");

    if !cargo_path.exists() {
        return Ok(false);
    }

    // Attempt to parse the Cargo.toml to validate it's a real Rust manifest
    let content = std::fs::read_to_string(&cargo_path)?;
    let _parsed: toml::Value = toml::from_str(&content)?;

    Ok(true)
}

/// Attempts to read the package name from Cargo.toml.
pub fn read_package_name(root_path: &str) -> Result<Option<String>> {
    let cargo_path = Path::new(root_path).join("Cargo.toml");

    if !cargo_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&cargo_path)?;
    let parsed: toml::Value = toml::from_str(&content)?;

    let name = parsed
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string());

    Ok(name)
}
