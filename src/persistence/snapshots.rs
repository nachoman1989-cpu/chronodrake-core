use anyhow::Result;
use crate::persistence::sqlite::Database;
use crate::models::ProjectScan;

/// Saves a snapshot reference into the database.
/// The actual JSON snapshot file is already written by the changes module;
/// this records the path in SQLite for historical queries.
pub fn record_snapshot(
    db: &Database,
    scan_id: &str,
    snapshot_path: &str,
) -> Result<()> {
    let snapshot_id = uuid::Uuid::new_v4().to_string();
    db.insert_snapshot(&snapshot_id, scan_id, snapshot_path)?;
    Ok(())
}

/// Builds a descriptive label for the current snapshot based on scan data.
pub fn build_snapshot_label(scan: &ProjectScan) -> String {
    let techs: Vec<&str> = scan.technologies.iter()
        .filter(|t| t.detected)
        .map(|t| t.name.as_str())
        .collect();
    if techs.is_empty() {
        "empty project".to_string()
    } else {
        techs.join(" + ")
    }
}
