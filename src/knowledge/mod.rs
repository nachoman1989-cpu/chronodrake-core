pub mod modules;
pub mod graph;
pub mod relationships;
pub mod dependencies;
pub mod impacts;
pub mod queries;

use anyhow::Result;
use std::path::Path;

use crate::intelligence::IntelligenceReport;
use crate::models::ProjectScan;
use crate::persistence::sqlite::Database;

/// Result of the knowledge graph pipeline.
pub struct KnowledgeResult {
    /// Path to the generated KNOWLEDGE_GRAPH.md.
    pub knowledge_graph_path: Option<String>,
    /// Path to the generated IMPACT_ANALYSIS.md.
    pub impact_analysis_path: Option<String>,
    /// Path to the generated MODULE_MAP.md.
    pub module_map_path: Option<String>,
    /// Number of modules detected.
    pub module_count: usize,
    /// Number of relationships detected.
    pub relationship_count: usize,
    /// Number of impact events detected.
    pub impact_count: usize,
    /// Number of classified dependencies.
    pub dependency_count: usize,
}

/// Runs the full knowledge graph pipeline.
///
/// 1. Detects modules from the filesystem
/// 2. Builds module graph (edges between modules)
/// 3. Classifies dependencies (critical, security, AI, blockchain, async)
/// 4. Detects relationships (modules ↔ deps ↔ features ↔ files ↔ scores ↔ architecture)
/// 5. Builds knowledge graph in SQLite
/// 6. Analyzes impacts (changes → effects)
/// 7. Generates KNOWLEDGE_GRAPH.md, IMPACT_ANALYSIS.md, MODULE_MAP.md
pub fn analyze(
    db: &Database,
    scan: &ProjectScan,
    intelligence: &IntelligenceReport,
    root_path: &str,
) -> Result<KnowledgeResult> {
    let timestamp = &scan.scanned_at;

    // Clear previous knowledge graph data before rebuilding
    db.clear_knowledge_graph()?;

    // ── 1. Detect modules ──────────────────────────────────────
    let detected_modules = self::modules::detect_modules(scan, root_path)?;
    let module_count = detected_modules.len();

    // ── 2. Build module graph ──────────────────────────────────
    let module_edges = self::modules::build_module_graph(&detected_modules);

    // ── 3. Classify dependencies ───────────────────────────────
    let classified_deps = self::dependencies::classify_dependencies(db, intelligence)?;
    let dependency_count = classified_deps.len();

    // ── 4. Detect relationships ────────────────────────────────
    let relationships = self::relationships::detect_relationships(
        db, scan, intelligence, &detected_modules,
    )?;
    let relationship_count = relationships.len();

    // ── 5. Build knowledge graph in SQLite ─────────────────────
    self::graph::build_knowledge_graph(db, scan, intelligence, &detected_modules, &module_edges)?;

    // ── 6. Analyze impacts ─────────────────────────────────────
    let impact_events = self::impacts::analyze_impacts(db, intelligence, &classified_deps, timestamp)?;
    let impact_count = impact_events.len();

    // ── 7. Generate reports ────────────────────────────────────

    // KNOWLEDGE_GRAPH.md
    let knowledge_graph_path = {
        let nodes = db.get_all_knowledge_nodes()?;
        let edges = db.get_all_module_relationships()?;
        let md = self::graph::generate_knowledge_graph_md(&nodes, &edges);
        let path = Path::new(root_path).join("KNOWLEDGE_GRAPH.md");
        std::fs::write(&path, &md)?;
        Some(path.to_string_lossy().to_string())
    };

    // IMPACT_ANALYSIS.md
    let impact_analysis_path = {
        let md = self::impacts::generate_impact_analysis_md(&impact_events);
        let path = Path::new(root_path).join("IMPACT_ANALYSIS.md");
        std::fs::write(&path, &md)?;
        Some(path.to_string_lossy().to_string())
    };

    // MODULE_MAP.md
    let module_map_path = {
        let md = self::modules::generate_module_map_md(&detected_modules, &module_edges);
        let path = Path::new(root_path).join("MODULE_MAP.md");
        std::fs::write(&path, &md)?;
        Some(path.to_string_lossy().to_string())
    };

    Ok(KnowledgeResult {
        knowledge_graph_path,
        impact_analysis_path,
        module_map_path,
        module_count,
        relationship_count,
        impact_count,
        dependency_count,
    })
}
