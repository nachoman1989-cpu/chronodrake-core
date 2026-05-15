use anyhow::Result;
use std::path::Path;

/// Detects whether the given directory contains a Node.js project.
///
/// Checks for the presence of `package.json` at the root level.
/// Returns `true` if a valid package.json exists.
pub fn detect_node(root_path: &str) -> Result<bool> {
    let pkg_path = Path::new(root_path).join("package.json");

    if !pkg_path.exists() {
        return Ok(false);
    }

    // Validate it's parseable JSON
    let content = std::fs::read_to_string(&pkg_path)?;
    let _parsed: serde_json::Value = serde_json::from_str(&content)?;

    Ok(true)
}

/// Attempts to read the package name from package.json.
pub fn read_package_name(root_path: &str) -> Result<Option<String>> {
    let pkg_path = Path::new(root_path).join("package.json");

    if !pkg_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&pkg_path)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;

    let name = parsed
        .get("name")
        .and_then(|n| n.as_str())
        .map(|s| s.to_string());

    Ok(name)
}
