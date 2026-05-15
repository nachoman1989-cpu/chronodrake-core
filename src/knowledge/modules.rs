use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

use crate::models::ProjectScan;

/// A detected source module in the project.
#[derive(Debug, Clone)]
pub struct ModuleNode {
    /// Module name (e.g., "scanner", "persistence", "intelligence").
    pub name: String,
    /// File path relative to project root.
    pub path: String,
    /// Category (e.g., "source", "config", "test").
    pub category: String,
    /// Whether this module has sub-modules.
    pub has_submodules: bool,
    /// Sub-modules if any.
    pub submodules: Vec<String>,
    /// Dependencies this module imports (crate names).
    pub imports: Vec<String>,
    /// Description of what this module does.
    pub description: String,
}

/// Detects and maps all source modules in the project.
///
/// Walks the `src/` directory and identifies Rust modules
/// based on `mod.rs` files and directory structure.
pub fn detect_modules(_scan: &ProjectScan, root_path: &str) -> Result<Vec<ModuleNode>> {
    let src_dir = Path::new(root_path).join("src");
    let mut modules: Vec<ModuleNode> = Vec::new();

    if !src_dir.exists() {
        return Ok(modules);
    }

    // Walk src/ directory recursively to find all modules
    walk_directory(&src_dir, &src_dir, &mut modules)?;

    // Sort: main.rs first, then mod.rs, then alphabetical
    modules.sort_by(|a, b| {
        let a_priority = module_priority(&a.name);
        let b_priority = module_priority(&b.name);
        a_priority.cmp(&b_priority).then(a.name.cmp(&b.name))
    });

    Ok(modules)
}

/// Recursively walks a directory to find Rust modules.
/// Detects both standalone .rs files and directory-based modules (dir/mod.rs).
fn walk_directory(dir: &Path, src_dir: &Path, modules: &mut Vec<ModuleNode>) -> Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files and non-Rust files
        if file_name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            // Check if this directory has a mod.rs (Rust module directory)
            let mod_rs = path.join("mod.rs");
            if mod_rs.exists() {
                let module_name = file_name.to_string();
                let rel_path = format!("src/{}", file_name);

                let imports = detect_imports(&mod_rs);
                let description = infer_module_purpose(&module_name, &imports);
                let submodules = list_submodules(&path);
                let has_submodules = !submodules.is_empty();
                let category = categorize_module(&module_name);

                modules.push(ModuleNode {
                    name: module_name,
                    path: rel_path,
                    category,
                    has_submodules,
                    submodules,
                    imports,
                    description,
                });

                // Recurse into subdirectory for nested modules
                walk_directory(&path, src_dir, modules)?;
            }
        } else if file_name.ends_with(".rs") && file_name != "mod.rs" {
            // Standalone .rs file (e.g., main.rs, lib.rs)
            let module_name = file_name.trim_end_matches(".rs").to_string();
            let rel_path = format!("src/{}", file_name);

            let imports = detect_imports(&path);
            let description = infer_module_purpose(&module_name, &imports);

            // Check for sub-modules (directory with same name + mod.rs)
            let sub_dir = src_dir.join(&module_name);
            let has_submodules = sub_dir.is_dir() && sub_dir.join("mod.rs").exists();
            let submodules = if has_submodules {
                list_submodules(&sub_dir)
            } else {
                Vec::new()
            };

            let category = categorize_module(&module_name);

            modules.push(ModuleNode {
                name: module_name,
                path: rel_path,
                category,
                has_submodules,
                submodules,
                imports,
                description,
            });
        }
    }

    Ok(())
}

/// Builds a structural graph of modules as adjacency pairs.
/// Returns Vec of (source, target, relationship_type).
pub fn build_module_graph(modules: &[ModuleNode]) -> Vec<(String, String, String)> {
    let mut edges: Vec<(String, String, String)> = Vec::new();

    // Map module names for quick lookup
    let module_names: Vec<&str> = modules.iter().map(|m| m.name.as_str()).collect();

    for module in modules {
        // Connect imports to known modules
        for import in &module.imports {
            if module_names.contains(&import.as_str()) {
                edges.push((
                    module.name.clone(),
                    import.clone(),
                    "imports".to_string(),
                ));
            }
        }

        // Connect submodules to parent
        for sub in &module.submodules {
            edges.push((
                module.name.clone(),
                sub.clone(),
                "contains".to_string(),
            ));
        }
    }

    edges
}

/// Detects Rust `use` and `mod` statements in a file.
fn detect_imports(path: &Path) -> Vec<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut imports: Vec<String> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();

        // Detect `use crate::xxx` or `use xxx`
        if trimmed.starts_with("use ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let path = parts[1].trim_end_matches(';');
                // Extract the crate/module name (first segment)
                let first_seg = path.split("::").next().unwrap_or("");
                let name = first_seg
                    .strip_prefix("crate::")
                    .unwrap_or(first_seg)
                    .to_string();
                if !name.is_empty() && name != "crate" && !imports.contains(&name) {
                    imports.push(name);
                }
            }
        }

        // Detect `mod xxx;`
        if trimmed.starts_with("mod ") && trimmed.ends_with(';') {
            let name = trimmed
                .strip_prefix("mod ")
                .unwrap_or("")
                .trim_end_matches(';')
                .trim()
                .to_string();
            if !name.is_empty() && !imports.contains(&name) {
                imports.push(name);
            }
        }
    }

    imports
}

/// Infers module purpose based on name and imports.
fn infer_module_purpose(name: &str, imports: &[String]) -> String {
    match name {
        "main" => "Application entry point".to_string(),
        "lib" => "Library root".to_string(),
        "mod" => "Module root".to_string(),
        "scanner" => "Project scanning and technology detection".to_string(),
        "intelligence" => "Project analysis and intelligence pipeline".to_string(),
        "persistence" => "SQLite persistence and data storage".to_string(),
        "memory" => "Memory and snapshot management".to_string(),
        "handoff" => "AI handoff documentation generation".to_string(),
        "models" => "Data models and type definitions".to_string(),
        "utils" => "Utility functions and helpers".to_string(),
        "knowledge" => "Knowledge graph and relationship engine".to_string(),
        _ => {
            // Infer from imports
            if imports.iter().any(|i| i.contains("serde") || i.contains("json")) {
                format!("{} — data serialization module", name)
            } else if imports.iter().any(|i| i.contains("tokio") || i.contains("async")) {
                format!("{} — async operations module", name)
            } else if imports.iter().any(|i| i.contains("rusqlite") || i.contains("sql")) {
                format!("{} — database operations module", name)
            } else {
                format!("{} — project module", name)
            }
        }
    }
}

/// Categorizes a module by its name.
fn categorize_module(name: &str) -> String {
    match name {
        "main" | "lib" => "entrypoint",
        "mod" => "root",
        "scanner" | "intelligence" | "persistence" | "memory" | "handoff" | "knowledge" => "core",
        "models" | "utils" => "foundation",
        "tests" | "test" => "test",
        _ => "module",
    }
    .to_string()
}

/// Priority for sorting (lower = first).
fn module_priority(name: &str) -> u8 {
    match name {
        "main" => 0,
        "lib" => 1,
        "mod" => 2,
        _ => 10,
    }
}

/// Lists sub-modules in a directory.
fn list_submodules(dir: &Path) -> Vec<String> {
    let mut subs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".rs") && name != "mod.rs" {
                subs.push(name.trim_end_matches(".rs").to_string());
            }
        }
    }
    subs
}

/// Generates a MODULE_MAP.md from detected modules.
pub fn generate_module_map_md(modules: &[ModuleNode], edges: &[(String, String, String)]) -> String {
    let mut md = String::new();
    md.push_str("# Module Map\n\n");
    md.push_str("> Generated by ChronoDrake Core v0.4 — Knowledge Graph Engine\n\n");

    md.push_str("## 📦 Modules\n\n");
    md.push_str("| Module | Path | Category | Sub-modules | Description |\n");
    md.push_str("|--------|------|----------|-------------|-------------|\n");

    for m in modules {
        let subs = if m.submodules.is_empty() {
            "—".to_string()
        } else {
            m.submodules.join(", ")
        };
        md.push_str(&format!(
            "| `{}` | `{}` | {} | {} | {} |\n",
            m.name, m.path, m.category, subs, m.description
        ));
    }

    md.push_str("\n## 🔗 Module Graph\n\n");
    md.push_str("```\n");

    // Build adjacency list
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for (src, tgt, _) in edges {
        adj.entry(src.as_str()).or_default().push(tgt.as_str());
    }

    // Print tree
    for module in modules {
        let deps = adj.get(module.name.as_str());
        if let Some(targets) = deps {
            md.push_str(&format!("{} → {}\n", module.name, targets.join(", ")));
        } else {
            md.push_str(&format!("{}\n", module.name));
        }
    }

    md.push_str("```\n");

    md.push_str("\n## 📊 Statistics\n\n");
    md.push_str(&format!("- **Total modules:** {}\n", modules.len()));
    let core_count = modules.iter().filter(|m| m.category == "core").count();
    let foundation_count = modules.iter().filter(|m| m.category == "foundation").count();
    md.push_str(&format!("- **Core modules:** {}\n", core_count));
    md.push_str(&format!("- **Foundation modules:** {}\n", foundation_count));
    md.push_str(&format!("- **Module relationships:** {}\n", edges.len()));

    md.push_str("\n---\n");
    md.push_str("*This file was automatically generated by ChronoDrake Core v0.4*\n");

    md
}
