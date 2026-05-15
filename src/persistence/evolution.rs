use anyhow::Result;
use crate::persistence::sqlite::{Database, EvolutionRow};

/// Records evolution entries by comparing current scores against previous scan.
/// Returns a list of (metric, old_value, new_value) tuples for any changes.
pub fn record_evolution(
    db: &Database,
    project_id: &str,
    current: &CurrentScores,
    timestamp: &str,
) -> Result<Vec<(String, f64, f64)>> {
    let mut changes: Vec<(String, f64, f64)> = Vec::new();

    if let Some(prev) = db.get_previous_scan_scores(project_id)? {
        // Compare each metric
        let comparisons: Vec<(&str, f64, f64)> = vec![
            ("maturity", prev.maturity_score as f64, current.maturity as f64),
            ("security", prev.security_score as f64, current.security as f64),
            ("architecture", prev.architecture_score as f64, current.architecture as f64),
            ("scalability", prev.scalability_score as f64, current.scalability as f64),
            ("testing", prev.testing_score as f64, current.testing as f64),
            ("documentation", prev.documentation_score as f64, current.documentation as f64),
            ("tech_count", prev.tech_count as f64, current.tech_count as f64),
            ("dep_count", prev.dep_count as f64, current.dep_count as f64),
            ("issue_count", prev.issue_count as f64, current.issue_count as f64),
        ];

        for (metric, old, new) in comparisons {
            if (old - new).abs() > 0.01 {
                let id = uuid::Uuid::new_v4().to_string();
                db.insert_evolution(&id, metric, old, new, timestamp)?;
                changes.push((metric.to_string(), old, new));
            }
        }
    }

    Ok(changes)
}

/// Current scan scores used for evolution comparison.
pub struct CurrentScores {
    pub maturity: i32,
    pub security: i32,
    pub architecture: i32,
    pub scalability: i32,
    pub testing: i32,
    pub documentation: i32,
    pub tech_count: i32,
    pub dep_count: i32,
    pub issue_count: i32,
}

/// Retrieves all evolution records from the database.
pub fn get_all_evolution(db: &Database) -> Result<Vec<EvolutionRow>> {
    db.get_all_evolution()
}

/// Retrieves evolution records for a specific metric.
pub fn get_evolution_by_metric(db: &Database, metric: &str) -> Result<Vec<EvolutionRow>> {
    db.get_evolution_by_metric(metric)
}
