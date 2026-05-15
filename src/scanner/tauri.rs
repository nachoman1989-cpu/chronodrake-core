use anyhow::Result;
use std::path::Path;

/// Detects whether the given directory contains a Tauri project.
///
/// Checks for the presence of `tauri.conf.json` at the root level
/// or inside a `src-tauri/` subdirectory.
pub fn detect_tauri(root_path: &str) -> Result<bool> {
    // Check root level
    let root_config = Path::new(root_path).join("tauri.conf.json");
    if root_config.exists() {
        return Ok(true);
    }

    // Check src-tauri subdirectory (standard Tauri layout)
    let src_tauri_config = Path::new(root_path).join("src-tauri").join("tauri.conf.json");
    if src_tauri_config.exists() {
        return Ok(true);
    }

    Ok(false)
}

/// Attempts to read the Tauri app name from tauri.conf.json.
pub fn read_app_name(root_path: &str) -> Result<Option<String>> {
    let paths = [
        Path::new(root_path).join("tauri.conf.json"),
        Path::new(root_path).join("src-tauri").join("tauri.conf.json"),
    ];

    for config_path in &paths {
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            let parsed: serde_json::Value = serde_json::from_str(&content)?;

            let name = parsed
                .get("package")
                .or_else(|| parsed.get("tauri"))
                .and_then(|p| p.get("productName").or_else(|| p.get("name")))
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());

            return Ok(name);
        }
    }

    Ok(None)
}
