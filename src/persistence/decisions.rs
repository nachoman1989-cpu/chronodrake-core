use anyhow::Result;
use crate::persistence::sqlite::{Database, DecisionRow};

/// Registers a decision in the database.
pub fn register_decision(
    db: &Database,
    title: &str,
    reason: &str,
    timestamp: &str,
) -> Result<()> {
    let id = uuid::Uuid::new_v4().to_string();
    db.insert_decision(&id, title, reason, timestamp)?;
    Ok(())
}

/// Retrieves all decisions from the database.
pub fn get_all_decisions(db: &Database) -> Result<Vec<DecisionRow>> {
    db.get_all_decisions()
}
