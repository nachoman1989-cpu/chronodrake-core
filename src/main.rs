mod scanner;
mod memory;
mod handoff;
mod models;
mod utils;
mod intelligence;
mod persistence;
mod knowledge;
mod config;
mod errors;
mod cache;
mod migrations;
mod context;
mod doctor;
mod workspace;

use anyhow::Result;
use std::env;

use utils::Logger;
use config::ChronoDrakeConfig;
use cache::CacheManager;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "scan" => cmd_scan().await?,
        "init" => cmd_init().await?,
        "context" => cmd_context().await?,
        "handoff" => cmd_handoff().await?,
        "doctor" => cmd_doctor().await?,
        "register" => cmd_register().await?,
        "projects" => cmd_projects().await?,
        "load" => cmd_load(args.get(2).map(|s| s.as_str()).unwrap_or("")).await?,
        "current" => cmd_current().await?,
        "help" | "--help" | "-h" => print_usage(),
        _ => {
            Logger::failure(&format!("Unknown command: {}", args[1]));
            print_usage();
        }
    }

    Ok(())
}

/// Prints the CLI usage information.
fn print_usage() {
    Logger::section("ChronoDrake Core v0.7");
    Logger::raw("AI Continuity Operating System — Project Registry & Workspace Loader");
    Logger::divider();
    Logger::raw("");
    Logger::raw("  USAGE:");
    Logger::raw("    chronodrake scan        Scan the current directory");
    Logger::raw("    chronodrake init        Generate default chronodrake.toml");
    Logger::raw("    chronodrake context     Generate AI context files from last scan");
    Logger::raw("    chronodrake handoff     Generate AI handoff document");
    Logger::raw("    chronodrake doctor      Run system diagnostics");
    Logger::raw("    chronodrake register    Register current directory as a project");
    Logger::raw("    chronodrake projects    List all registered projects");
    Logger::raw("    chronodrake load <name> Load a project and show its AI context");
    Logger::raw("    chronodrake current     Show the currently active project");
    Logger::raw("    chronodrake help        Show this help message");
    Logger::raw("");
    Logger::divider();
}

/// Initializes a default `chronodrake.toml` configuration file.
async fn cmd_init() -> Result<()> {
    let current_dir = env::current_dir()?;
    let root_path = current_dir.to_string_lossy().to_string();

    Logger::section("ChronoDrake Init");
    Logger::kv("Target", &root_path);
    Logger::divider();

    // Generate default config
    ChronoDrakeConfig::generate_default(&root_path)?;
    Logger::success("chronodrake.toml generated with default settings");

    // Generate default config file
    let config_path = current_dir.join("chronodrake.toml");
    if config_path.exists() {
        Logger::success(&format!("Config file created at {:?}", config_path));
    }

    Logger::divider();
    Logger::section("Init Complete");
    Ok(())
}

/// Executes the `context` command: generates AI context files from the last
/// scan data by re-running the full pipeline and generating the 7 context files.
async fn cmd_context() -> Result<()> {
    let current_dir = env::current_dir()?;
    let root_path = current_dir.to_string_lossy().to_string();

    let config = ChronoDrakeConfig::load(&root_path)?;
    Logger::init(&config.log_level);

    Logger::section("ChronoDrake Context v0.7");
    Logger::kv("Target", &root_path);
    Logger::divider();

    // Run full scan pipeline to get fresh data
    Logger::sub_section("Scanning project");
    let scan = scanner::scan_project(&root_path)?;
    let intelligence = intelligence::analyze(&scan, &root_path)?;
    let persistence_result = persistence::persist(&scan, &intelligence, &root_path)?;

    let db = persistence::sqlite::Database::open(&root_path)?;
    let knowledge_result = knowledge::analyze(&db, &scan, &intelligence, &root_path)?;

    // Generate AI context files
    Logger::sub_section("Generating AI Context files");
    let context_data = context::ContextData {
        scan: &scan,
        intelligence: &intelligence,
        knowledge: &knowledge_result,
        persistence: &persistence_result,
        output_dir: &root_path,
    };

    let generated = context::generate_all(&context_data)?;
    for path in &generated {
        Logger::success(&format!("Generated → {}", path));
    }

    Logger::success(&format!("{} AI context files generated", generated.len()));

    Logger::divider();
    Logger::section("Context Generation Complete");
    Ok(())
}

/// Executes the `handoff` command: generates the AI_HANDOFF.md document from
/// a fresh scan.
async fn cmd_handoff() -> Result<()> {
    let current_dir = env::current_dir()?;
    let root_path = current_dir.to_string_lossy().to_string();

    let config = ChronoDrakeConfig::load(&root_path)?;
    Logger::init(&config.log_level);

    Logger::section("ChronoDrake Handoff v0.7");
    Logger::kv("Target", &root_path);
    Logger::divider();

    // Run scan to get fresh data
    Logger::sub_section("Scanning project");
    let scan = scanner::scan_project(&root_path)?;

    // Generate handoff document
    Logger::sub_section("Generating AI Handoff");
    let handoff_path = handoff::generator::generate_ai_handoff(&scan, &root_path)?;
    Logger::success(&format!("Generated → {}", handoff_path));

    Logger::divider();
    Logger::section("Handoff Generation Complete");
    Ok(())
}

/// Executes the `doctor` command: runs system diagnostics, detects installed
/// tools, toolchains, and environment, then generates SYSTEM_REPORT.md and
/// DEVELOPMENT_ENVIRONMENT.md.
async fn cmd_doctor() -> Result<()> {
    let current_dir = env::current_dir()?;
    let root_path = current_dir.to_string_lossy().to_string();

    Logger::section("ChronoDrake Doctor v0.7");
    Logger::kv("Target", &root_path);
    Logger::divider();

    Logger::sub_section("Running system diagnostics");
    let report = doctor::run_diagnostics(&root_path)?;

    // Display summary
    Logger::kv("OS", &format!("{} ({})", report.os.name, report.os.arch));
    Logger::kv("Rust", if report.rust_toolchain.installed {
        report.rust_toolchain.rustc_version.as_deref().unwrap_or("installed")
    } else { "❌ Not installed" });
    Logger::kv("Node.js", if report.node_info.installed {
        report.node_info.node_version.as_deref().unwrap_or("installed")
    } else { "❌ Not installed" });
    Logger::kv("Git", if report.git_info.installed {
        report.git_info.version.as_deref().unwrap_or("installed")
    } else { "❌ Not installed" });
    Logger::kv("VSCode", if report.vscode_info.installed { "✅" } else { "❌" });
    Logger::kv("RooCode", if report.roo_code_info.installed { "✅" } else { "❌" });
    Logger::kv("Tauri", if report.tauri_info.installed { "✅" } else { "❌" });
    Logger::kv("SQLite", if report.sqlite_info.installed { "✅" } else { "❌" });

    if !report.missing_deps.is_empty() {
        Logger::warning(&format!("Missing dependencies: {}", report.missing_deps.join(", ")));
    }
    if !report.broken_config.is_empty() {
        Logger::warning(&format!("Broken config: {}", report.broken_config.join(", ")));
    }

    // Generate reports
    Logger::sub_section("Generating reports");
    let sys_path = doctor::generate_system_report(&report, &root_path)?;
    Logger::success(&format!("Generated → {}", sys_path));

    let dev_path = doctor::generate_dev_environment_report(&report, &root_path)?;
    Logger::success(&format!("Generated → {}", dev_path));

    Logger::divider();
    Logger::section("Doctor Diagnostics Complete");
    Ok(())
}

/// Executes the `scan` command: loads config, initializes tracing, scans the
/// current directory, runs the full intelligence pipeline, persists data to
/// SQLite with schema migrations, builds the knowledge graph, and generates
/// markdown reports.
async fn cmd_scan() -> Result<()> {
    let current_dir = env::current_dir()?;
    let root_path = current_dir.to_string_lossy().to_string();

    // --- Step 0: Load configuration ---
    let config = ChronoDrakeConfig::load(&root_path)?;

    // Initialize tracing with configured log level
    Logger::init(&config.log_level);

    Logger::section("ChronoDrake Scan v0.7");
    Logger::kv("Target", &root_path);
    Logger::kv("Log Level", &config.log_level);
    Logger::kv("Incremental Scan", &config.incremental_scan.to_string());
    Logger::kv("Cache Enabled", &config.enable_cache.to_string());
    Logger::divider();

    // --- Initialize Cache Manager ---
    let cache = CacheManager::new(&root_path, config.enable_cache)?;
    if config.enable_cache {
        let cache_stats = cache.stats()?;
        Logger::kv("Cache Entries", &cache_stats.entry_count.to_string());
        Logger::kv("Cache Size", &format!("{} bytes", cache_stats.total_size_bytes));
    }

    // --- Run schema migrations on existing database ---
    let db_path = current_dir.join("memory").join("chronodrake.db");
    if db_path.exists() {
        Logger::sub_section("Running Schema Migrations");
        let conn = rusqlite::Connection::open(&db_path)?;
        let current_version = migrations::MigrationEngine::get_current_version(&conn)?;
        Logger::kv("Schema Version", &current_version.to_string());

        if current_version < migrations::CURRENT_SCHEMA_VERSION {
            let applied = migrations::MigrationEngine::run_pending(&conn)?;
            for migration in &applied {
                Logger::success(&format!("Migration applied: {}", migration));
            }
            Logger::success(&format!(
                "Schema upgraded from v{} to v{}",
                current_version,
                migrations::CURRENT_SCHEMA_VERSION
            ));
        } else {
            Logger::success("Schema is up to date");
        }
    }

    Logger::divider();

    // --- Step 1: Run scanner (detect stack) ---
    Logger::sub_section("1/7 Scanning project stack");
    let scan = Logger::timed("filesystem scan", || {
        scanner::scan_project(&root_path)
    })?;

    // --- Display detection results ---
    Logger::sub_section("Detection Results");
    for tech in &scan.technologies {
        if tech.detected {
            Logger::success(&format!("{} detected", tech.name));
        } else {
            Logger::failure(&format!("{} not found", tech.name));
        }
    }

    // --- Display findings ---
    if !scan.findings.is_empty() {
        Logger::sub_section("Files Discovered");
        for (category, files) in &scan.findings {
            Logger::info(&format!("{}:", category));
            for file in files {
                Logger::raw(&format!("    └─ {}", file));
            }
        }
    }

    Logger::divider();

    // --- Step 2: Run intelligence pipeline ---
    Logger::sub_section("2/7 Running intelligence pipeline");
    let intelligence = Logger::timed("intelligence analysis", || {
        intelligence::analyze(&scan, &root_path)
    })?;

    // --- Display project type ---
    Logger::sub_section("2/7 Project Type");
    Logger::success(&format!(
        "Primary: {} (confidence: {:.0}%)",
        intelligence.project_type.primary,
        intelligence.project_type.confidence * 100.0
    ));
    Logger::raw(&format!("Summary: {}", intelligence.project_type.summary));

    // --- Display architecture ---
    Logger::sub_section("3/7 Architecture Analysis");
    Logger::success(&format!(
        "Layers: {}",
        intelligence.architecture.layers.len()
    ));
    for layer in &intelligence.architecture.layers {
        Logger::raw(&format!("    └─ {}", layer));
    }
    if !intelligence.architecture.entrypoints.is_empty() {
        Logger::info("Entry points:");
        for ep in &intelligence.architecture.entrypoints {
            Logger::raw(&format!("    └─ {}", ep));
        }
    }
    if !intelligence.architecture.databases.is_empty() {
        Logger::info("Databases:");
        for db in &intelligence.architecture.databases {
            Logger::raw(&format!("    └─ {}", db));
        }
    }

    // --- Display maturity scores ---
    Logger::sub_section("4/7 Maturity Scores");
    Logger::kv("Architecture", &format!("{}/100", intelligence.maturity.architecture));
    Logger::kv("Security", &format!("{}/100", intelligence.maturity.security));
    Logger::kv("Scalability", &format!("{}/100", intelligence.maturity.scalability));
    Logger::kv("Testing", &format!("{}/100", intelligence.maturity.testing));
    Logger::kv("Documentation", &format!("{}/100", intelligence.maturity.documentation));
    Logger::success(&format!("Overall: {}/100", intelligence.maturity.overall));

    // --- Display security findings ---
    Logger::sub_section("5/7 Security Scan");
    Logger::kv("Issues found", &intelligence.security.issue_count.to_string());
    Logger::kv("Rating", &intelligence.security.rating.to_string());
    if !intelligence.security.issues.is_empty() {
        for issue in &intelligence.security.issues {
            let icon = match issue.severity.to_string().as_str() {
                "Critical" => "🔴",
                "Warning" => "🟡",
                _ => "🔵",
            };
            Logger::raw(&format!(
                "    {} [{}] {} — {}",
                icon, issue.severity, issue.description, issue.suggestion
            ));
        }
    }

    // --- Display dependency analysis ---
    Logger::sub_section("6/7 Dependency Analysis");
    Logger::kv("Total dependencies", &intelligence.dependencies.total_count.to_string());
    Logger::kv("Notable", &intelligence.dependencies.notable_count.to_string());
    for dep in &intelligence.dependencies.dependencies {
        if dep.notable {
            Logger::success(&format!(
                "    {} v{} ({}) — {}",
                dep.name, dep.version, dep.ecosystem, dep.note.as_deref().unwrap_or("")
            ));
        }
    }

    // --- Display changes ---
    Logger::sub_section("7/7 Change Detection");
    if let Some(ref changes) = intelligence.changes {
        if changes.is_first_scan {
            Logger::info("First scan — baseline snapshot created.");
        } else if changes.change_count > 0 {
            Logger::warning(&format!("{} changes detected", changes.change_count));
            for change in &changes.changes {
                let icon = match change.change_type.to_string().as_str() {
                    "Added" => "🟢",
                    "Removed" => "🔴",
                    _ => "🟡",
                };
                Logger::raw(&format!(
                    "    {} [{}] {} — {}",
                    icon, change.change_type, change.category, change.description
                ));
            }
        } else {
            Logger::success("No changes detected since last scan.");
        }
    }

    Logger::divider();

    // --- Step 3: Persist to SQLite (history, evolution, timeline) ---
    Logger::sub_section("3/7 Persisting to SQLite");
    let persistence_result = Logger::timed("SQLite persistence", || {
        persistence::persist(&scan, &intelligence, &root_path)
    })?;

    if persistence_result.is_first_scan {
        Logger::info("First scan — database initialized.");
    }
    Logger::success(&format!(
        "Evolution changes tracked: {}",
        persistence_result.evolution_count
    ));

    // --- Step 4: Knowledge Graph Engine ---
    Logger::sub_section("4/7 Building Knowledge Graph");
    let db = persistence::sqlite::Database::open(&root_path)?;
    let knowledge_result = Logger::timed("knowledge graph", || {
        knowledge::analyze(&db, &scan, &intelligence, &root_path)
    })?;

    Logger::success(&format!(
        "Modules detected: {}",
        knowledge_result.module_count
    ));
    Logger::success(&format!(
        "Relationships: {}",
        knowledge_result.relationship_count
    ));
    Logger::success(&format!(
        "Impact events: {}",
        knowledge_result.impact_count
    ));
    Logger::success(&format!(
        "Dependencies classified: {}",
        knowledge_result.dependency_count
    ));

    // --- Generate markdown reports ---
    Logger::sub_section("Generating Reports");
    let status_path = memory::generator::generate_project_status(&scan, &intelligence, &root_path)?;
    Logger::success(&format!("PROJECT_STATUS.md generated → {}", status_path));

    let handoff_path = handoff::generator::generate_ai_handoff(&scan, &root_path)?;
    Logger::success(&format!("AI_HANDOFF.md generated → {}", handoff_path));

    if let Some(changes_path) = memory::generator::generate_changes_report(&intelligence, &root_path)? {
        Logger::success(&format!("CHANGES.md generated → {}", changes_path));
    }

    if let Some(ref timeline_path) = persistence_result.timeline_path {
        Logger::success(&format!("TIMELINE.md generated → {}", timeline_path));
    }

    if let Some(ref recs_path) = persistence_result.recommendations_path {
        Logger::success(&format!("RECOMMENDATIONS.md generated → {}", recs_path));
    }

    if let Some(ref kg_path) = knowledge_result.knowledge_graph_path {
        Logger::success(&format!("KNOWLEDGE_GRAPH.md generated → {}", kg_path));
    }

    if let Some(ref ia_path) = knowledge_result.impact_analysis_path {
        Logger::success(&format!("IMPACT_ANALYSIS.md generated → {}", ia_path));
    }

    if let Some(ref mm_path) = knowledge_result.module_map_path {
        Logger::success(&format!("MODULE_MAP.md generated → {}", mm_path));
    }

    // --- Step 5: Generate AI Context files ---
    Logger::sub_section("5/7 Generating AI Context");
    let context_data = context::ContextData {
        scan: &scan,
        intelligence: &intelligence,
        knowledge: &knowledge_result,
        persistence: &persistence_result,
        output_dir: &root_path,
    };
    let context_files = context::generate_all(&context_data)?;
    for path in &context_files {
        Logger::success(&format!("AI Context → {}", path));
    }

    // --- Cache final stats ---
    if config.enable_cache {
        let cache_stats = cache.stats()?;
        Logger::metric("Cache Entries", &cache_stats.entry_count.to_string());
        Logger::metric("Cache Size (bytes)", &cache_stats.total_size_bytes.to_string());
    }

    Logger::divider();
    Logger::section("Scan Complete");
    Logger::success("ChronoDrake Core v0.7 finished successfully");

    Ok(())
}

/// Executes the `register` command: auto-detects project name and stack from
/// the current directory and saves it to the global project registry.
async fn cmd_register() -> Result<()> {
    let current_dir = env::current_dir()?;
    let root_path = current_dir.to_string_lossy().to_string();

    let config = ChronoDrakeConfig::load(&root_path)?;
    Logger::init(&config.log_level);

    Logger::section("ChronoDrake Register v0.7");
    Logger::kv("Target", &root_path);
    Logger::divider();

    Logger::sub_section("Detecting project identity");
    let project = workspace::registry::register_project(&root_path)?;

    Logger::success(&format!("Project registered: {}", project.name));
    Logger::kv("Name", &project.name);
    Logger::kv("Path", &project.path);
    Logger::kv("Stack", &project.stack);

    Logger::divider();
    Logger::section("Registration Complete");
    Ok(())
}

/// Executes the `projects` command: lists all registered projects, marking the
/// active one with `* ACTIVE`.
async fn cmd_projects() -> Result<()> {
    let current_dir = env::current_dir()?;
    let root_path = current_dir.to_string_lossy().to_string();

    let config = ChronoDrakeConfig::load(&root_path)?;
    Logger::init(&config.log_level);

    Logger::section("ChronoDrake Projects v0.7");
    Logger::divider();

    let (projects, active) = workspace::registry::list_projects()?;

    if projects.is_empty() {
        Logger::info("No projects registered yet.");
        Logger::raw("  Use 'chronodrake register' to add your first project.");
        Logger::divider();
        return Ok(());
    }

    Logger::kv("Total", &projects.len().to_string());
    Logger::divider();

    for (i, project) in projects.iter().enumerate() {
        let is_active = active.as_deref() == Some(&project.name);
        let active_marker = if is_active { " * ACTIVE" } else { "" };
        let last = project.last_scan.as_deref().unwrap_or("never");

        Logger::raw(&format!(
            "  {}. {} [{}{}]",
            i + 1,
            project.name,
            project.stack,
            active_marker,
        ));
        Logger::raw(&format!("     Path: {}", project.path));
        Logger::raw(&format!("     Last scan: {}", last));
    }

    Logger::divider();
    Logger::section("Projects List Complete");
    Ok(())
}

/// Executes the `load <project>` command: finds a project by name, marks it as
/// active, reads its ai-context/ files, displays a summary with maturity scores,
/// and shows the AI boot prompt.
async fn cmd_load(project_name: &str) -> Result<()> {
    if project_name.is_empty() {
        Logger::failure("Usage: chronodrake load <project-name>");
        Logger::raw("  Use 'chronodrake projects' to see available projects.");
        return Ok(());
    }

    let current_dir = env::current_dir()?;
    let root_path = current_dir.to_string_lossy().to_string();

    let config = ChronoDrakeConfig::load(&root_path)?;
    Logger::init(&config.log_level);

    Logger::section(&format!("ChronoDrake Load: {}", project_name));
    Logger::divider();

    Logger::sub_section("Loading project context");
    let summary = workspace::loader::load_project(project_name)?;

    // Display summary
    workspace::loader::display_summary(&summary);

    // Display AI boot prompt if available
    if let Some(ref boot) = summary.boot_prompt_content {
        Logger::divider();
        workspace::loader::display_boot_prompt(boot);
    }

    Logger::divider();
    Logger::section(&format!("Project '{}' loaded successfully", summary.name));
    Ok(())
}

/// Executes the `current` command: shows the currently active project.
async fn cmd_current() -> Result<()> {
    let current_dir = env::current_dir()?;
    let root_path = current_dir.to_string_lossy().to_string();

    let config = ChronoDrakeConfig::load(&root_path)?;
    Logger::init(&config.log_level);

    Logger::section("ChronoDrake Current v0.7");
    Logger::divider();

    match workspace::registry::get_current_project()? {
        Some(project) => {
            Logger::success(&format!("Active project: {}", project.name));
            Logger::kv("Name", &project.name);
            Logger::kv("Path", &project.path);
            Logger::kv("Stack", &project.stack);
            Logger::kv("Last Scan", project.last_scan.as_deref().unwrap_or("never"));
        }
        None => {
            Logger::info("No active project.");
            Logger::raw("  Use 'chronodrake load <name>' to load a project.");
            Logger::raw("  Use 'chronodrake register' to register a new project.");
        }
    }

    Logger::divider();
    Logger::section("Current Project Info");
    Ok(())
}
