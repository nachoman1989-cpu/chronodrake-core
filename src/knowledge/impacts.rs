use anyhow::Result;

use crate::intelligence::IntelligenceReport;
use crate::persistence::sqlite::Database;

use super::dependencies::DependencyClass;

/// An impact event linking a source change to a target effect.
#[derive(Debug, Clone)]
pub struct ImpactEvent {
    pub source: String,
    pub target: String,
    pub impact_type: String,
    pub direction: String,
    pub description: String,
}

/// Analyzes the impact of changes and dependencies on project scores and quality.
///
/// Relates:
/// - Added dependencies → score changes
/// - Removed dependencies → score changes
/// - New modules → architecture improvements
/// - Security issues → security score degradation
pub fn analyze_impacts(
    db: &Database,
    intelligence: &IntelligenceReport,
    classified: &[DependencyClass],
    timestamp: &str,
) -> Result<Vec<ImpactEvent>> {
    let mut events: Vec<ImpactEvent> = Vec::new();

    // ── 1. Impact from changes (if not first scan) ──────────
    if let Some(ref changes) = intelligence.changes {
        if !changes.is_first_scan {
            for change in &changes.changes {
                let change_lower = change.description.to_lowercase();
                let category_lower = change.category.to_lowercase();

                // Detect security-related changes
                if category_lower.contains("security")
                    || change_lower.contains("crypto")
                    || change_lower.contains("aes")
                    || change_lower.contains("encrypt")
                {
                    events.push(ImpactEvent {
                        source: format!("change:{}", change.description),
                        target: "maturity:security".to_string(),
                        impact_type: "score_impact".to_string(),
                        direction: "increased".to_string(),
                        description: format!(
                            "{} {} → Security score increased",
                            change.change_type, change.description
                        ),
                    });
                }

                // Detect testing-related changes
                if category_lower.contains("test")
                    || change_lower.contains("test")
                    || change_lower.contains("rstest")
                {
                    events.push(ImpactEvent {
                        source: format!("change:{}", change.description),
                        target: "maturity:testing".to_string(),
                        impact_type: "score_impact".to_string(),
                        direction: "increased".to_string(),
                        description: format!(
                            "{} {} → Testing maturity improved",
                            change.change_type, change.description
                        ),
                    });
                }

                // Detect documentation changes
                if category_lower.contains("doc")
                    || change_lower.contains("readme")
                    || change_lower.contains("license")
                {
                    events.push(ImpactEvent {
                        source: format!("change:{}", change.description),
                        target: "maturity:documentation".to_string(),
                        impact_type: "score_impact".to_string(),
                        direction: "increased".to_string(),
                        description: format!(
                            "{} {} → Documentation improved",
                            change.change_type, change.description
                        ),
                    });
                }

                // Detect architecture changes (new modules, services)
                if change_lower.contains("modul")
                    || change_lower.contains("service")
                    || change_lower.contains("layer")
                {
                    events.push(ImpactEvent {
                        source: format!("change:{}", change.description),
                        target: "maturity:architecture".to_string(),
                        impact_type: "score_impact".to_string(),
                        direction: "increased".to_string(),
                        description: format!(
                            "{} {} → Architecture improved",
                            change.change_type, change.description
                        ),
                    });
                }
            }
        }
    }

    // ── 2. Impact from security dependencies ────────────────
    for dc in classified {
        if dc.is_security {
            events.push(ImpactEvent {
                source: format!("dep:{}", dc.name),
                target: "maturity:security".to_string(),
                impact_type: "dependency_impact".to_string(),
                direction: "positive".to_string(),
                description: format!(
                    "{} is a security dependency → strengthens security posture",
                    dc.name
                ),
            });
        }
        if dc.is_async {
            events.push(ImpactEvent {
                source: format!("dep:{}", dc.name),
                target: "maturity:scalability".to_string(),
                impact_type: "dependency_impact".to_string(),
                direction: "positive".to_string(),
                description: format!(
                    "{} enables async operations → improves scalability",
                    dc.name
                ),
            });
        }
    }

    // ── 3. Impact from security issues ──────────────────────
    for issue in &intelligence.security.issues {
        let direction = match issue.severity.to_string().as_str() {
            "Critical" => "negative",
            "Warning" => "negative",
            _ => "neutral",
        };
        events.push(ImpactEvent {
            source: format!("security:{}", issue.category),
            target: "maturity:security".to_string(),
            impact_type: "security_impact".to_string(),
            direction: direction.to_string(),
            description: format!(
                "[{}] {} — {}",
                issue.severity, issue.description, issue.suggestion
            ),
        });
    }

    // ── 4. Impact from architecture ─────────────────────────
    let arch = &intelligence.architecture;
    if arch.is_modular {
        events.push(ImpactEvent {
            source: "architecture:modularity".to_string(),
            target: "maturity:architecture".to_string(),
            impact_type: "architectural_impact".to_string(),
            direction: "positive".to_string(),
            description: "Modular architecture improves maintainability and scalability".to_string(),
        });
    }
    if arch.has_services {
        events.push(ImpactEvent {
            source: "architecture:services".to_string(),
            target: "maturity:architecture".to_string(),
            impact_type: "architectural_impact".to_string(),
            direction: "positive".to_string(),
            description: "Services pattern improves separation of concerns".to_string(),
        });
    }
    if arch.has_database {
        events.push(ImpactEvent {
            source: "architecture:database".to_string(),
            target: "maturity:scalability".to_string(),
            impact_type: "architectural_impact".to_string(),
            direction: "positive".to_string(),
            description: "Database usage enables data persistence and scalability".to_string(),
        });
    }

    // Persist all impact events to database
    for event in &events {
        let id = format!(
            "impact:{}->{}",
            event.source.replace(':', "_").replace(' ', "_"),
            event.target.replace(':', "_")
        );
        db.insert_impact_event(
            &id,
            &event.source,
            &event.target,
            &event.impact_type,
            &event.direction,
            &event.description,
            timestamp,
        )?;
    }

    Ok(events)
}

/// Generates an IMPACT_ANALYSIS.md from impact events.
pub fn generate_impact_analysis_md(events: &[ImpactEvent]) -> String {
    let mut md = String::new();
    md.push_str("# Impact Analysis\n\n");
    md.push_str("> Generated by ChronoDrake Core v0.4 — Knowledge Graph Engine\n\n");

    if events.is_empty() {
        md.push_str("_No impact events detected._\n\n");
        md.push_str("---\n");
        md.push_str("*This file was automatically generated by ChronoDrake Core v0.4*\n");
        return md;
    }

    // Group by impact type
    let mut by_type: std::collections::BTreeMap<String, Vec<&ImpactEvent>> =
        std::collections::BTreeMap::new();
    for event in events {
        by_type
            .entry(event.impact_type.clone())
            .or_default()
            .push(event);
    }

    for (impact_type, type_events) in &by_type {
        let label = match impact_type.as_str() {
            "score_impact" => "📊 Score Impacts",
            "dependency_impact" => "📦 Dependency Impacts",
            "security_impact" => "🔒 Security Impacts",
            "architectural_impact" => "🏗️ Architectural Impacts",
            _ => impact_type,
        };
        md.push_str(&format!("## {}\n\n", label));

        let positive = type_events.iter().filter(|e| e.direction == "positive" || e.direction == "increased").count();
        let negative = type_events.iter().filter(|e| e.direction == "negative" || e.direction == "decreased").count();

        md.push_str(&format!("- **Positive impacts:** {}\n", positive));
        md.push_str(&format!("- **Negative impacts:** {}\n", negative));
        md.push_str("\n| Source | Target | Direction | Description |\n");
        md.push_str("|--------|--------|-----------|-------------|\n");

        for event in type_events {
            let icon = match event.direction.as_str() {
                "positive" | "increased" => "🟢",
                "negative" | "decreased" => "🔴",
                _ => "🟡",
            };
            md.push_str(&format!(
                "| {} `{}` | `{}` | {} {} | {} |\n",
                icon, event.source, event.target, icon, event.direction, event.description
            ));
        }
        md.push_str("\n");
    }

    md.push_str("## 📊 Summary\n\n");
    md.push_str(&format!("- **Total impact events:** {}\n", events.len()));
    let positive_total = events.iter().filter(|e| e.direction == "positive" || e.direction == "increased").count();
    let negative_total = events.iter().filter(|e| e.direction == "negative" || e.direction == "decreased").count();
    md.push_str(&format!("- **Positive:** {}\n", positive_total));
    md.push_str(&format!("- **Negative:** {}\n", negative_total));

    md.push_str("\n---\n");
    md.push_str("*This file was automatically generated by ChronoDrake Core v0.4*\n");

    md
}
