pub mod sqlite;
pub mod snapshots;
pub mod timeline;
pub mod decisions;
pub mod evolution;

use anyhow::Result;
use std::path::Path;

use crate::models::ProjectScan;
use crate::intelligence::IntelligenceReport;

use self::sqlite::Database;
use self::evolution::CurrentScores;

/// Complete result of the persistence pipeline.
pub struct PersistenceResult {
    /// Path to the generated TIMELINE.md (if any events exist).
    pub timeline_path: Option<String>,
    /// Path to the generated RECOMMENDATIONS.md.
    pub recommendations_path: Option<String>,
    /// Number of evolution changes detected.
    pub evolution_count: usize,
    /// Whether this is the first scan in the database.
    pub is_first_scan: bool,
}

/// Runs the full persistence pipeline after a scan + intelligence analysis.
///
/// 1. Opens/creates the SQLite database at `memory/chronodrake.db`
/// 2. Upserts the project record
/// 3. Inserts the scan record with all scores
/// 4. Records snapshot reference
/// 5. Detects and records evolution (score changes)
/// 6. Registers timeline events
/// 7. Generates TIMELINE.md and RECOMMENDATIONS.md
pub fn persist(
    scan: &ProjectScan,
    intelligence: &IntelligenceReport,
    root_path: &str,
) -> Result<PersistenceResult> {
    let db = Database::open(root_path)?;
    let timestamp = &scan.scanned_at;
    let project_name = Path::new(root_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // ── 1. Upsert project ────────────────────────────────────
    let project_type = &intelligence.project_type.primary;
    let project_id = match db.get_project_id(&project_name)? {
        Some(id) => id,
        None => {
            let id = uuid::Uuid::new_v4().to_string();
            db.upsert_project(&id, &project_name, &project_type.to_string(), timestamp)?;
            id
        },
    };

    // ── 2. Check if first scan ────────────────────────────────
    let scan_count = db.get_scan_count(&project_id)?;
    let is_first_scan = scan_count == 0;

    // ── 3. Insert scan record ─────────────────────────────────
    let scan_id = uuid::Uuid::new_v4().to_string();
    let tech_count = scan.technologies.iter().filter(|t| t.detected).count() as i32;
    let dep_count = intelligence.dependencies.total_count as i32;
    let issue_count = intelligence.security.issue_count as i32;

    db.insert_scan(
        &scan_id,
        &project_id,
        timestamp,
        intelligence.maturity.overall as i32,
        intelligence.maturity.security as i32,
        intelligence.maturity.architecture as i32,
        intelligence.maturity.scalability as i32,
        intelligence.maturity.testing as i32,
        intelligence.maturity.documentation as i32,
        tech_count,
        dep_count,
        issue_count,
    )?;

    // ── 4. Record snapshot reference ─────────────────────────
    let snapshot_label = self::snapshots::build_snapshot_label(scan);
    let snapshot_rel_path = format!("memory/snapshots/latest.json");
    self::snapshots::record_snapshot(&db, &scan_id, &snapshot_rel_path)?;

    // ── 5. Detect and record evolution ───────────────────────
    let current_scores = CurrentScores {
        maturity: intelligence.maturity.overall as i32,
        security: intelligence.maturity.security as i32,
        architecture: intelligence.maturity.architecture as i32,
        scalability: intelligence.maturity.scalability as i32,
        testing: intelligence.maturity.testing as i32,
        documentation: intelligence.maturity.documentation as i32,
        tech_count,
        dep_count,
        issue_count,
    };

    let evolution_changes = self::evolution::record_evolution(
        &db, &project_id, &current_scores, timestamp,
    )?;
    let evolution_count = evolution_changes.len();

    // ── 6. Register timeline events ──────────────────────────
    // Always register a scan event
    let scan_event = format!("Scan completed — {} detected", snapshot_label);
    self::timeline::register_event(&db, &scan_event, "Project analysis updated", timestamp)?;

    // Register evolution events
    for (metric, old, new) in &evolution_changes {
        let direction = if new > old { "improved" } else { "degraded" };
        let event = format!("{} {}", metric, direction);
        let impact = format!("{} changed from {:.0} to {:.0}", metric, old, new);
        self::timeline::register_event(&db, &event, &impact, timestamp)?;
    }

    // Register change events from intelligence
    if let Some(ref changes) = intelligence.changes {
        if !changes.is_first_scan && changes.change_count > 0 {
            let event = format!("{} change(s) detected", changes.change_count);
            self::timeline::register_event(&db, &event, &changes.summary, timestamp)?;
        }
    }

    // ── 7. Generate TIMELINE.md ──────────────────────────────
    let timeline_events = self::timeline::get_all_events(&db)?;
    let timeline_path = if !timeline_events.is_empty() {
        let md = self::timeline::generate_timeline_markdown(&timeline_events);
        let path = Path::new(root_path).join("TIMELINE.md");
        std::fs::write(&path, &md)?;
        Some(path.to_string_lossy().to_string())
    } else {
        None
    };

    // ── 8. Generate RECOMMENDATIONS.md ───────────────────────
    let recommendations_path = {
        let md = generate_recommendations(intelligence, is_first_scan);
        let path = Path::new(root_path).join("RECOMMENDATIONS.md");
        std::fs::write(&path, &md)?;
        Some(path.to_string_lossy().to_string())
    };

    Ok(PersistenceResult {
        timeline_path,
        recommendations_path,
        evolution_count,
        is_first_scan,
    })
}

/// Generates a RECOMMENDATIONS.md file based on intelligence analysis.
fn generate_recommendations(intelligence: &IntelligenceReport, is_first_scan: bool) -> String {
    let mut md = String::new();
    md.push_str("# Recommendations\n\n");
    md.push_str("> Generated by ChronoDrake Core v0.3\n\n");

    if is_first_scan {
        md.push_str("_First scan — baseline established. Recommendations will appear after future scans._\n\n");
    }

    // ── Maturity-based recommendations ───────────────────────
    md.push_str("## 📊 Maturity Improvements\n\n");
    let maturity = &intelligence.maturity;
    let mut has_maturity_recs = false;

    if maturity.architecture < 50 {
        md.push_str(&format!("- **Architecture ({}/100):** Consider improving modularity — add more source directories, separate concerns, and introduce service layers.\n", maturity.architecture));
        has_maturity_recs = true;
    }
    if maturity.security < 50 {
        md.push_str(&format!("- **Security ({}/100):** Add .gitignore, .env.example, lockfiles, and security-related dependencies.\n", maturity.security));
        has_maturity_recs = true;
    }
    if maturity.scalability < 50 {
        md.push_str(&format!("- **Scalability ({}/100):** Improve modularity, add async support, and consider service-based architecture.\n", maturity.scalability));
        has_maturity_recs = true;
    }
    if maturity.testing < 50 {
        md.push_str(&format!("- **Testing ({}/100):** Add test directories, testing frameworks (e.g., rstest, jest), and test configuration files.\n", maturity.testing));
        has_maturity_recs = true;
    }
    if maturity.documentation < 50 {
        md.push_str(&format!("- **Documentation ({}/100):** Create README.md, CONTRIBUTING.md, CHANGELOG.md, and add a docs/ directory.\n", maturity.documentation));
        has_maturity_recs = true;
    }

    if !has_maturity_recs {
        md.push_str("_All maturity dimensions are at acceptable levels._\n");
    }

    // ── Security-based recommendations ───────────────────────
    md.push_str("\n## 🔒 Security Gaps\n\n");
    if intelligence.security.issues.is_empty() {
        md.push_str("_No security issues detected._\n");
    } else {
        for issue in &intelligence.security.issues {
            md.push_str(&format!(
                "- **{}** ({}): {} — _Suggestion: {}_\n",
                issue.severity, issue.category, issue.description, issue.suggestion
            ));
        }
    }

    // ── Architecture-based recommendations ───────────────────
    md.push_str("\n## 🏗️ Architecture Improvements\n\n");
    let arch = &intelligence.architecture;
    if !arch.is_modular {
        md.push_str("- **Modularity:** Project is not modular. Consider organizing code into modules (src/, lib/, services/, api/, etc.).\n");
    }
    if arch.layers.len() < 3 {
        md.push_str(&format!("- **Layers:** Only {} layer(s) detected. Adding more layers (models, services, controllers) improves maintainability.\n", arch.layers.len()));
    }
    if !arch.has_frontend_backend_separation {
        md.push_str("- **Frontend/Backend:** No separation detected. For larger projects, consider splitting frontend and backend.\n");
    }
    if !arch.has_services {
        md.push_str("- **Services Pattern:** No services directory found. A services layer helps organize business logic.\n");
    }
    if !arch.has_database {
        md.push_str("- **Database:** No database usage detected. If data persistence is needed, consider adding an ORM (Diesel, SQLx, Prisma).\n");
    }
    if arch.layers.len() >= 3 && arch.is_modular && arch.has_services {
        md.push_str("_Architecture looks well-structured._\n");
    }

    // ── Missing components ───────────────────────────────────
    md.push_str("\n## 🧩 Missing Components\n\n");
    let root = std::path::Path::new(".");
    let mut has_missing = false;

    if !root.join(".gitignore").exists() {
        md.push_str("- **.gitignore:** Missing — add one to prevent committing sensitive files.\n");
        has_missing = true;
    }
    if !root.join("README.md").exists() {
        md.push_str("- **README.md:** Missing — essential for project documentation.\n");
        has_missing = true;
    }
    if !root.join("LICENSE").exists() && !root.join("LICENSE.md").exists() && !root.join("LICENSE.txt").exists() {
        md.push_str("- **LICENSE:** Missing — consider adding an open-source license.\n");
        has_missing = true;
    }
    if !root.join("CHANGELOG.md").exists() {
        md.push_str("- **CHANGELOG.md:** Missing — helps track version history.\n");
        has_missing = true;
    }

    if !has_missing {
        md.push_str("_No critical missing components detected._\n");
    }

    md.push_str("\n---\n");
    md.push_str("*This file was automatically generated by ChronoDrake Core v0.3*\n");

    md
}
