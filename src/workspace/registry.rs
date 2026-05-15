use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::scanner;
use crate::utils::Logger;

use super::models::{ProjectRegistry, RegisteredProject};

/// Returns the path to the global ChronoDrake directory (`~/.chronodrake/`).
pub fn chronodrake_dir() -> Result<PathBuf> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .context("Could not determine home directory (USERPROFILE or HOME not set)")?;

    let dir = PathBuf::from(home).join(".chronodrake");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create ChronoDrake directory at {:?}", dir))?;
    }
    Ok(dir)
}

/// Returns the full path to the projects registry JSON file.
pub fn registry_path() -> Result<PathBuf> {
    Ok(chronodrake_dir()?.join("projects.json"))
}

/// Loads the project registry from disk.
///
/// If the file does not exist, returns a default (empty) registry.
pub fn load_registry() -> Result<ProjectRegistry> {
    let path = registry_path()?;

    if !path.exists() {
        return Ok(ProjectRegistry::new());
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read registry at {:?}", path))?;

    let registry: ProjectRegistry = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse registry at {:?}", path))?;

    Ok(registry)
}

/// Saves the project registry to disk.
pub fn save_registry(registry: &ProjectRegistry) -> Result<()> {
    let path = registry_path()?;

    let content = serde_json::to_string_pretty(registry)
        .context("Failed to serialize registry")?;

    std::fs::write(&path, &content)
        .with_context(|| format!("Failed to write registry to {:?}", path))?;

    Ok(())
}

/// Registers a new project by auto-detecting its name and stack from the given
/// directory path. Avoids duplicates by name — if a project with the same name
/// already exists, its path and stack are updated instead.
pub fn register_project(path: &str) -> Result<RegisteredProject> {
    let name = detect_project_name(path)?;
    let stack = detect_project_stack(path)?;

    let mut registry = load_registry()?;

    // Check for duplicate by name
    if let Some(existing) = registry.projects.iter().position(|p| p.name == name) {
        let project = &mut registry.projects[existing];
        project.path = path.to_string();
        project.stack = stack.clone();
        let result = project.clone();
        Logger::info(&format!("Project '{}' already registered — updated path and stack", name));
        save_registry(&registry)?;
        return Ok(result);
    }

    let project = RegisteredProject {
        name: name.clone(),
        path: path.to_string(),
        stack,
        last_scan: None,
    };

    registry.projects.push(project.clone());
    save_registry(&registry)?;

    Ok(project)
}

/// Lists all registered projects. Returns the list and the name of the active
/// project (if any).
pub fn list_projects() -> Result<(Vec<RegisteredProject>, Option<String>)> {
    let registry = load_registry()?;
    let active = registry.last_project.clone();
    Ok((registry.projects, active))
}

/// Returns the currently active project (the one that was last loaded).
pub fn get_current_project() -> Result<Option<RegisteredProject>> {
    let registry = load_registry()?;

    match registry.last_project {
        Some(ref name) => {
            let project = registry.projects.iter()
                .find(|p| p.name == *name)
                .cloned();
            Ok(project)
        }
        None => Ok(None),
    }
}

/// Sets the given project name as the active (last loaded) project.
pub fn set_current_project(name: &str) -> Result<()> {
    let mut registry = load_registry()?;
    registry.last_project = Some(name.to_string());
    save_registry(&registry)?;
    Ok(())
}

/// Finds a registered project by name (case-insensitive partial match).
pub fn find_project(name: &str) -> Result<Option<RegisteredProject>> {
    let registry = load_registry()?;
    let lower = name.to_lowercase();

    // Try exact match first
    if let Some(p) = registry.projects.iter().find(|p| p.name.to_lowercase() == lower) {
        return Ok(Some(p.clone()));
    }

    // Try partial match
    if let Some(p) = registry.projects.iter().find(|p| p.name.to_lowercase().contains(&lower)) {
        return Ok(Some(p.clone()));
    }

    Ok(None)
}

/// Updates the `last_scan` timestamp for a project.
pub fn update_last_scan(name: &str, timestamp: &str) -> Result<()> {
    let mut registry = load_registry()?;

    if let Some(project) = registry.projects.iter_mut().find(|p| p.name == name) {
        project.last_scan = Some(timestamp.to_string());
        save_registry(&registry)?;
    }

    Ok(())
}

/// Auto-detects the project name from Cargo.toml, package.json, or falls back
/// to the directory name.
pub fn detect_project_name(root_path: &str) -> Result<String> {
    // Try Cargo.toml first
    if let Some(name) = scanner::rust::read_package_name(root_path)? {
        return Ok(name);
    }

    // Try package.json
    if let Some(name) = scanner::node::read_package_name(root_path)? {
        return Ok(name);
    }

    // Try Tauri app name
    if let Some(name) = scanner::tauri::read_app_name(root_path)? {
        return Ok(name);
    }

    // Fallback: use directory name
    let path = std::path::Path::new(root_path);
    let dir_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed-project".to_string());

    Ok(dir_name)
}

/// Auto-detects the technology stack of a project by checking for Cargo.toml,
/// package.json, and tauri.conf.json.
pub fn detect_project_stack(root_path: &str) -> Result<String> {
    let mut parts: Vec<&str> = Vec::new();

    let has_cargo = scanner::rust::detect_cargo(root_path).unwrap_or(false);
    let has_node = scanner::node::detect_node(root_path).unwrap_or(false);
    let has_tauri = scanner::tauri::detect_tauri(root_path).unwrap_or(false);

    if has_cargo {
        parts.push("Rust");
    }
    if has_node {
        parts.push("Node");
    }
    if has_tauri {
        parts.push("Tauri");
    }

    if parts.is_empty() {
        // Check for other common indicators
        let path = std::path::Path::new(root_path);
        if path.join("Makefile").exists() || path.join("CMakeLists.txt").exists() {
            parts.push("C/C++");
        } else if path.join("go.mod").exists() {
            parts.push("Go");
        } else if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() {
            parts.push("Python");
        } else {
            parts.push("Unknown");
        }
    }

    Ok(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new_is_empty() {
        let registry = ProjectRegistry::new();
        assert!(registry.projects.is_empty());
        assert!(registry.last_project.is_none());
    }

    #[test]
    fn test_registry_default_is_empty() {
        let registry = ProjectRegistry::default();
        assert!(registry.projects.is_empty());
        assert!(registry.last_project.is_none());
    }

    #[test]
    fn test_serialize_deserialize_registry() {
        let mut registry = ProjectRegistry::new();
        registry.projects.push(RegisteredProject {
            name: "test-project".into(),
            path: "/tmp/test".into(),
            stack: "Rust".into(),
            last_scan: None,
        });
        registry.last_project = Some("test-project".into());

        let json = serde_json::to_string_pretty(&registry).unwrap();
        let deserialized: ProjectRegistry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.projects.len(), 1);
        assert_eq!(deserialized.projects[0].name, "test-project");
        assert_eq!(deserialized.last_project.unwrap(), "test-project");
    }

    #[test]
    fn test_detect_project_name_fallback_to_dir() {
        // Use a temp dir to test fallback
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();
        let name = detect_project_name(&path).unwrap();
        // The name should be the temp dir's name
        assert!(!name.is_empty());
        assert_ne!(name, "unnamed-project");
    }
}
