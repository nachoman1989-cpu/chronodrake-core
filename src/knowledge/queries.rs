use anyhow::Result;

use crate::persistence::sqlite::Database;

/// A query result entry.
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub category: String,
    pub content: String,
    pub details: String,
}

/// Local query system for the knowledge graph.
///
/// Supports queries like:
/// - What changed?
/// - What impacts security?
/// - Which modules are most connected?
/// - Which dependencies are critical?
pub struct KnowledgeQueries<'a> {
    db: &'a Database,
}

impl<'a> KnowledgeQueries<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Returns all changes (from impact events).
    pub fn what_changed(&self) -> Result<Vec<QueryResult>> {
        let impacts = self.db.get_all_impact_events()?;
        let mut results: Vec<QueryResult> = Vec::new();

        for impact in &impacts {
            results.push(QueryResult {
                category: "change".to_string(),
                content: impact.source.clone(),
                details: format!("{} → {} ({})", impact.description, impact.target, impact.direction),
            });
        }

        Ok(results)
    }

    /// Returns everything that impacts security.
    pub fn what_impacts_security(&self) -> Result<Vec<QueryResult>> {
        let impacts = self.db.get_impacts_by_target("maturity:security")?;
        let mut results: Vec<QueryResult> = Vec::new();

        for impact in &impacts {
            results.push(QueryResult {
                category: "security_impact".to_string(),
                content: impact.source.clone(),
                details: format!("{} ({})", impact.description, impact.direction),
            });
        }

        // Also check dependency relationships
        let deps = self.db.get_all_dependency_relationships()?;
        for dep in &deps {
            if dep.is_security {
                results.push(QueryResult {
                    category: "security_dependency".to_string(),
                    content: dep.dep_name.clone(),
                    details: format!(
                        "Security dependency v{} ({}) — {}",
                        dep.version, dep.ecosystem, dep.category
                    ),
                });
            }
        }

        Ok(results)
    }

    /// Returns modules sorted by connection count (most connected first).
    pub fn most_connected_modules(&self) -> Result<Vec<QueryResult>> {
        let nodes = self.db.get_nodes_by_type("module")?;
        let mut results: Vec<QueryResult> = Vec::new();

        for node in nodes {
            results.push(QueryResult {
                category: "module".to_string(),
                content: node.name.clone(),
                details: format!(
                    "{} connections | weight: {:.2} | {}",
                    node.connections, node.weight, node.description
                ),
            });
        }

        // Sort by connections descending
        results.sort_by(|a, b| {
            let a_conns = a.details.split_whitespace().next().unwrap_or("0").to_string();
            let b_conns = b.details.split_whitespace().next().unwrap_or("0").to_string();
            b_conns.cmp(&a_conns)
        });

        Ok(results)
    }

    /// Returns all critical dependencies.
    pub fn critical_dependencies(&self) -> Result<Vec<QueryResult>> {
        let deps = self.db.get_critical_dependencies()?;
        let mut results: Vec<QueryResult> = Vec::new();

        for dep in &deps {
            let tags = {
                let mut t = Vec::new();
                if dep.is_critical { t.push("critical"); }
                if dep.is_security { t.push("security"); }
                if dep.is_ai { t.push("ai"); }
                if dep.is_blockchain { t.push("blockchain"); }
                if dep.is_async { t.push("async"); }
                t.join(", ")
            };
            results.push(QueryResult {
                category: "critical_dependency".to_string(),
                content: dep.dep_name.clone(),
                details: format!("v{} ({}) — tags: {}", dep.version, dep.ecosystem, tags),
            });
        }

        Ok(results)
    }

    /// Returns all knowledge nodes of a given type.
    pub fn nodes_by_type(&self, node_type: &str) -> Result<Vec<QueryResult>> {
        let nodes = self.db.get_nodes_by_type(node_type)?;
        let mut results: Vec<QueryResult> = Vec::new();

        for node in nodes {
            results.push(QueryResult {
                category: node_type.to_string(),
                content: node.name.clone(),
                details: format!(
                    "{} | {} connections | weight: {:.2}",
                    node.description, node.connections, node.weight
                ),
            });
        }

        Ok(results)
    }

    /// Returns all relationships for a given source.
    pub fn relationships_of(&self, source: &str) -> Result<Vec<QueryResult>> {
        let rels = self.db.get_relationships_by_source(source)?;
        let mut results: Vec<QueryResult> = Vec::new();

        for rel in &rels {
            results.push(QueryResult {
                category: "relationship".to_string(),
                content: format!("{} → {}", rel.source, rel.target),
                details: format!("{} (weight: {:.1}) — {}", rel.rel_type, rel.weight, rel.metadata),
            });
        }

        Ok(results)
    }

    /// Runs all queries and returns a comprehensive report.
    pub fn run_all_queries(&self) -> Result<String> {
        let mut output = String::new();
        output.push_str("# Knowledge Queries\n\n");
        output.push_str("> Generated by ChronoDrake Core v0.4 — Knowledge Graph Engine\n\n");

        // 1. What changed?
        output.push_str("## 🔄 What Changed?\n\n");
        let changes = self.what_changed()?;
        if changes.is_empty() {
            output.push_str("_No changes detected._\n\n");
        } else {
            for c in &changes {
                output.push_str(&format!("- **{}** — {}\n", c.content, c.details));
            }
        }

        // 2. What impacts security?
        output.push_str("\n## 🔒 What Impacts Security?\n\n");
        let security = self.what_impacts_security()?;
        if security.is_empty() {
            output.push_str("_No security impacts detected._\n\n");
        } else {
            for s in &security {
                output.push_str(&format!("- [{}] **{}** — {}\n", s.category, s.content, s.details));
            }
        }

        // 3. Most connected modules
        output.push_str("\n## 🔗 Most Connected Modules\n\n");
        let modules = self.most_connected_modules()?;
        if modules.is_empty() {
            output.push_str("_No modules detected._\n\n");
        } else {
            for (i, m) in modules.iter().enumerate() {
                output.push_str(&format!("{}. **{}** — {}\n", i + 1, m.content, m.details));
            }
        }

        // 4. Critical dependencies
        output.push_str("\n## ⭐ Critical Dependencies\n\n");
        let critical = self.critical_dependencies()?;
        if critical.is_empty() {
            output.push_str("_No critical dependencies detected._\n\n");
        } else {
            for c in &critical {
                output.push_str(&format!("- **{}** — {}\n", c.content, c.details));
            }
        }

        output.push_str("\n---\n");
        output.push_str("*This file was automatically generated by ChronoDrake Core v0.4*\n");

        Ok(output)
    }
}
