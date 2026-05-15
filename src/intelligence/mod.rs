pub mod project_type;
pub mod architecture;
pub mod maturity;
pub mod security;
pub mod dependencies;
pub mod changes;

use anyhow::Result;

use crate::models::ProjectScan;

use self::project_type::ProjectTypeResult;
use self::architecture::ArchitectureResult;
use self::maturity::MaturityScores;
use self::security::SecurityFindings;
use self::dependencies::DependencyAnalysis;
use self::changes::ChangeReport;

/// Complete result of the intelligence analysis pipeline.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntelligenceReport {
    pub project_type: ProjectTypeResult,
    pub architecture: ArchitectureResult,
    pub maturity: MaturityScores,
    pub security: SecurityFindings,
    pub dependencies: DependencyAnalysis,
    pub changes: Option<ChangeReport>,
}

/// Runs the full intelligence pipeline on a scanned project.
///
/// This function takes a [`ProjectScan`] (from the scanner module) and runs
/// all intelligence analyzers: project type detection, architecture analysis,
/// maturity scoring, security scanning, dependency analysis, and change detection.
pub fn analyze(scan: &ProjectScan, root_path: &str) -> Result<IntelligenceReport> {
    // 1. Detect project type
    let project_type = project_type::detect_project_type(scan, root_path)?;

    // 2. Analyze architecture
    let architecture = architecture::analyze_architecture(scan, root_path)?;

    // 3. Calculate maturity scores
    let maturity = maturity::calculate_maturity(scan, &architecture, root_path)?;

    // 4. Security scan
    let security = security::scan_security(scan, root_path)?;

    // 5. Dependency analysis
    let dependencies = dependencies::analyze_dependencies(scan, root_path)?;

    // 6. Change detection (compare with previous snapshot)
    let changes = changes::detect_changes(scan, root_path)?;

    Ok(IntelligenceReport {
        project_type,
        architecture,
        maturity,
        security,
        dependencies,
        changes,
    })
}
