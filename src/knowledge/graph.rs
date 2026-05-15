use anyhow::Result;
use crate::persistence::sqlite;
use std::collections::HashMap;

use crate::intelligence::IntelligenceReport;
use crate::models::ProjectScan;
use crate::persistence::sqlite::Database;

use super::modules::ModuleNode;

/// A node in the knowledge graph.
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub category: String,
    pub description: String,
    pub connections: i32,
    pub weight: f64,
}

/// An edge (relationship) in the knowledge graph.
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub rel_type: String,
    pub weight: f64,
    pub metadata: String,
}

/// Builds the complete knowledge graph from scan + intelligence data.
///
/// Creates:
/// - Knowledge nodes for modules, dependencies, technologies, architecture layers
/// - Module relationships (imports, contains)
/// - Dependency relationships (critical, security, AI, blockchain, async)
pub fn build_knowledge_graph(
    db: &Database,
    scan: &ProjectScan,
    intelligence: &IntelligenceReport,
    modules: &[ModuleNode],
    module_edges: &[(String, String, String)],
) -> Result<()> {
    // ── 1. Create knowledge nodes for modules ────────────────
    for module in modules {
        let node_id = format!("module:{}", module.name);
        let connections = count_module_connections(module, module_edges);
        db.insert_knowledge_node(
            &node_id,
            "module",
            &module.name,
            &module.category,
            &module.description,
            connections,
            connections as f64,
        )?;
    }

    // ── 2. Create knowledge nodes for technologies ───────────
    for tech in &scan.technologies {
        if tech.detected {
            let node_id = format!("tech:{}", tech.name);
            db.insert_knowledge_node(
                &node_id,
                "technology",
                &tech.name,
                &tech.category,
                &format!("Detected {} technology", tech.name),
                0,
                1.0,
            )?;
        }
    }

    // ── 3. Create knowledge nodes for architecture layers ────
    for layer in &intelligence.architecture.layers {
        let node_id = format!("layer:{}", layer);
        db.insert_knowledge_node(
            &node_id,
            "architecture",
            layer,
            "layer",
            &format!("Architecture layer: {}", layer),
            0,
            1.0,
        )?;
    }

    // ── 4. Create knowledge nodes for databases ──────────────
    for db_name in &intelligence.architecture.databases {
        let node_id = format!("database:{}", db_name);
        db.insert_knowledge_node(
            &node_id,
            "database",
            db_name,
            "database",
            &format!("Database technology: {}", db_name),
            0,
            1.0,
        )?;
    }

    // ── 5. Create knowledge nodes for maturity scores ────────
    let score_nodes = vec![
        ("maturity:architecture", "Architecture", intelligence.maturity.architecture as f64 / 100.0),
        ("maturity:security", "Security", intelligence.maturity.security as f64 / 100.0),
        ("maturity:scalability", "Scalability", intelligence.maturity.scalability as f64 / 100.0),
        ("maturity:testing", "Testing", intelligence.maturity.testing as f64 / 100.0),
        ("maturity:documentation", "Documentation", intelligence.maturity.documentation as f64 / 100.0),
        ("maturity:overall", "Overall", intelligence.maturity.overall as f64 / 100.0),
    ];
    for (node_id, name, weight) in score_nodes {
        db.insert_knowledge_node(
            node_id,
            "maturity",
            name,
            "score",
            &format!("Maturity score: {}/100", (weight * 100.0) as u8),
            0,
            weight,
        )?;
    }

    // ── 6. Create module relationships ───────────────────────
    for (src, tgt, rel_type) in module_edges {
        let edge_id = format!("rel:{}->{}", src, tgt);
        db.insert_module_relationship(&edge_id, src, tgt, rel_type, 1.0, "{}")?;
    }

    // ── 7. Connect modules to their technologies ─────────────
    for module in modules {
        for import in &module.imports {
            // Check if this import matches a detected technology
            for tech in &scan.technologies {
                if tech.detected && import.to_lowercase().contains(&tech.name.to_lowercase()) {
                    let edge_id = format!("rel:{}->tech:{}", module.name, tech.name);
                    db.insert_module_relationship(
                        &edge_id,
                        &format!("module:{}", module.name),
                        &format!("tech:{}", tech.name),
                        "uses",
                        1.0,
                        "{}",
                    )?;
                }
            }
        }
    }

    Ok(())
}

/// Counts how many connections a module has in the edge list.
fn count_module_connections(module: &ModuleNode, edges: &[(String, String, String)]) -> i32 {
    let mut count = 0;
    for (src, tgt, _) in edges {
        if src == &module.name || tgt == &module.name {
            count += 1;
        }
    }
    // Add submodule connections
    count += module.submodules.len() as i32;
    count
}

/// Generates a KNOWLEDGE_GRAPH.md from the stored graph data.
pub fn generate_knowledge_graph_md(
    nodes: &[sqlite::KnowledgeNodeRow],
    edges: &[sqlite::ModuleRelationshipRow],
) -> String {
    let mut md = String::new();
    md.push_str("# Knowledge Graph\n\n");
    md.push_str("> Generated by ChronoDrake Core v0.4 — Knowledge Graph Engine\n\n");

    // Group nodes by type
    let mut by_type: HashMap<String, Vec<&sqlite::KnowledgeNodeRow>> = HashMap::new();
    for node in nodes {
        by_type.entry(node.node_type.clone()).or_default().push(node);
    }

    md.push_str("## 🧠 Nodes\n\n");

    let type_order = ["module", "technology", "architecture", "database", "maturity"];
    for t in &type_order {
        if let Some(items) = by_type.get(*t) {
            let label = match *t {
                "module" => "Modules",
                "technology" => "Technologies",
                "architecture" => "Architecture Layers",
                "database" => "Databases",
                "maturity" => "Maturity Scores",
                _ => t,
            };
            md.push_str(&format!("### {} ({})\n\n", label, items.len()));
            md.push_str("| Name | Category | Connections | Weight | Description |\n");
            md.push_str("|------|----------|-------------|--------|-------------|\n");
            for node in items {
                md.push_str(&format!(
                    "| `{}` | {} | {} | {:.2} | {} |\n",
                    node.name, node.category, node.connections, node.weight, node.description
                ));
            }
            md.push_str("\n");
        }
    }

    md.push_str("## 🔗 Relationships\n\n");
    md.push_str(&format!("**Total relationships:** {}\n\n", edges.len()));

    if !edges.is_empty() {
        md.push_str("| Source | Target | Type | Weight |\n");
        md.push_str("|--------|--------|------|--------|\n");
        for edge in edges {
            md.push_str(&format!(
                "| `{}` | `{}` | {} | {:.1} |\n",
                edge.source, edge.target, edge.rel_type, edge.weight
            ));
        }
    }

    md.push_str("\n## 📊 Graph Statistics\n\n");
    md.push_str(&format!("- **Total nodes:** {}\n", nodes.len()));
    md.push_str(&format!("- **Total edges:** {}\n", edges.len()));
    let module_count = by_type.get("module").map(|v| v.len()).unwrap_or(0);
    let tech_count = by_type.get("technology").map(|v| v.len()).unwrap_or(0);
    md.push_str(&format!("- **Module nodes:** {}\n", module_count));
    md.push_str(&format!("- **Technology nodes:** {}\n", tech_count));

    md.push_str("\n---\n");
    md.push_str("*This file was automatically generated by ChronoDrake Core v0.4*\n");

    md
}
