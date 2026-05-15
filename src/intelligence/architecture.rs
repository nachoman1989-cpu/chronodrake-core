use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::models::ProjectScan;

/// Result of architecture analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureResult {
    /// Whether the project has a clear frontend/backend separation.
    pub has_frontend_backend_separation: bool,
    /// Detected project layers.
    pub layers: Vec<String>,
    /// Entry points found.
    pub entrypoints: Vec<String>,
    /// Whether services pattern is detected.
    pub has_services: bool,
    /// Whether database usage is detected.
    pub has_database: bool,
    /// Detected database technologies.
    pub databases: Vec<String>,
    /// Whether the project appears modular.
    pub is_modular: bool,
    /// Number of top-level source directories (indicator of modularity).
    pub module_count: usize,
    /// Human-readable summary.
    pub summary: String,
}

/// Analyzes the architecture of a project based on its structure and files.
pub fn analyze_architecture(_scan: &ProjectScan, root_path: &str) -> Result<ArchitectureResult> {
    let root = Path::new(root_path);

    // Detect layers
    let mut layers: Vec<String> = Vec::new();
    let mut entrypoints: Vec<String> = Vec::new();
    let mut databases: Vec<String> = Vec::new();
    let mut has_services = false;
    let mut has_database = false;
    let mut module_count = 0;

    // Check for common source directories (modularity indicators)
    let src_dirs = ["src", "lib", "app", "components", "modules", "services", "api", "routes", "controllers", "models", "views", "pages", "core", "utils", "helpers", "middleware", "hooks", "store", "state", "types", "interfaces", "config", "db", "database", "migrations", "seeds", "tests", "spec", "fixtures", "mocks"];

    for dir_name in &src_dirs {
        let dir_path = root.join(dir_name);
        if dir_path.is_dir() {
            module_count += 1;
            layers.push(dir_name.to_string());

            // Check for services pattern
            if *dir_name == "services" {
                has_services = true;
            }

            // Check for database
            if matches!(*dir_name, "db" | "database" | "migrations") {
                has_database = true;
            }
        }
    }

    // Check for frontend/backend separation
    let has_frontend_dir = root.join("frontend").is_dir()
        || root.join("client").is_dir()
        || root.join("web").is_dir()
        || root.join("ui").is_dir();
    let has_backend_dir = root.join("backend").is_dir()
        || root.join("server").is_dir()
        || root.join("api").is_dir();
    let has_frontend_backend_separation = has_frontend_dir && has_backend_dir;

    // Detect entry points
    let entry_candidates = [
        "main.rs", "main.ts", "index.ts", "index.js", "app.ts", "app.js",
        "server.ts", "server.js", "cli.rs", "cli.ts", "cli.js",
        "lib.rs", "lib.ts", "lib.js", "entry.ts", "entry.js",
        "index.html", "App.tsx", "App.jsx", "App.vue",
    ];
    for candidate in &entry_candidates {
        let candidate_path = root.join(candidate);
        if candidate_path.exists() {
            entrypoints.push(candidate.to_string());
        }
        // Also check in src/
        let src_candidate = root.join("src").join(candidate);
        if src_candidate.exists() {
            entrypoints.push(format!("src/{}", candidate));
        }
    }

    // Detect database usage by scanning for common ORM/db files
    let db_indicators = [
        "prisma", "diesel", "sqlx", "sea-orm", "typeorm", "sequelize",
        "mongoose", "knex", "database.rs", "db.rs", "schema.rs",
        "migrations", "seed.rs", "seeds",
    ];
    for indicator in &db_indicators {
        let ind_path = root.join(indicator);
        if ind_path.exists() {
            databases.push(indicator.to_string());
            has_database = true;
        }
        // Check in src/
        let src_ind = root.join("src").join(indicator);
        if src_ind.exists() {
            databases.push(format!("src/{}", indicator));
            has_database = true;
        }
    }

    // Check for database-related dependencies in Cargo.toml
    let cargo_db_crates = ["diesel", "sqlx", "sea-orm", "rusqlite", "mongodb", "redis"];
    for db_crate in &cargo_db_crates {
        if has_cargo_dep(root_path, db_crate) {
            databases.push(db_crate.to_string());
            has_database = true;
        }
    }

    // Check for database-related dependencies in package.json
    let npm_db_pkgs = ["prisma", "typeorm", "sequelize", "mongoose", "knex", "pg", "mysql2", "redis"];
    for db_pkg in &npm_db_pkgs {
        if has_npm_dep(root_path, db_pkg) {
            databases.push(db_pkg.to_string());
            has_database = true;
        }
    }

    // Modularity score: more source directories = more modular
    let is_modular = module_count >= 3;

    let summary = format!(
        "Architecture: {} layers detected ({}), {} entrypoints, frontend/backend separation: {}, modular: {}, database: {}",
        layers.len(),
        layers.join(", "),
        entrypoints.len(),
        if has_frontend_backend_separation { "yes" } else { "no" },
        if is_modular { "yes" } else { "no" },
        if has_database { format!("yes ({})", databases.join(", ")) } else { "no".to_string() },
    );

    Ok(ArchitectureResult {
        has_frontend_backend_separation,
        layers,
        entrypoints,
        has_services,
        has_database,
        databases,
        is_modular,
        module_count,
        summary,
    })
}

fn has_cargo_dep(root_path: &str, dep_name: &str) -> bool {
    let paths = [
        Path::new(root_path).join("Cargo.toml"),
        Path::new(root_path).join("src-tauri").join("Cargo.toml"),
    ];
    for cargo_path in &paths {
        if cargo_path.exists() {
            if let Ok(content) = std::fs::read_to_string(cargo_path) {
                if let Ok(parsed) = toml::from_str::<toml::Value>(&content) {
                    if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) {
                        if deps.contains_key(dep_name) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

fn has_npm_dep(root_path: &str, dep_name: &str) -> bool {
    let pkg_path = Path::new(root_path).join("package.json");
    if !pkg_path.exists() {
        return false;
    }
    if let Ok(content) = std::fs::read_to_string(&pkg_path) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
            for key in &["dependencies", "devDependencies", "peerDependencies"] {
                if let Some(deps) = parsed.get(*key).and_then(|d| d.as_object()) {
                    if deps.contains_key(dep_name) {
                        return true;
                    }
                }
            }
        }
    }
    false
}
