use anyhow::Result;
use std::path::Path;

use crate::intelligence::IntelligenceReport;
use crate::knowledge::KnowledgeResult;
use crate::models::ProjectScan;
use crate::persistence::PersistenceResult;

/// Complete context data passed to all generators.
pub struct ContextData<'a> {
    pub scan: &'a ProjectScan,
    pub intelligence: &'a IntelligenceReport,
    pub knowledge: &'a KnowledgeResult,
    pub persistence: &'a PersistenceResult,
    pub output_dir: &'a str,
}

/// Generates all 7 AI context files into the `/ai-context/` directory.
pub fn generate_all(data: &ContextData) -> Result<Vec<String>> {
    let ai_context_dir = Path::new(data.output_dir).join("ai-context");
    std::fs::create_dir_all(&ai_context_dir)?;

    let mut generated = Vec::new();

    let files: Vec<(&str, fn(&ContextData) -> Result<String>)> = vec![
        ("MASTER_CONTEXT.md", generate_master_context),
        ("CURRENT_STATE.md", generate_current_state),
        ("NEXT_STEPS.md", generate_next_steps),
        ("DECISIONS.md", generate_decisions),
        ("ACTIVE_TASK.md", generate_active_task),
        ("PROJECT_IDENTITY.md", generate_project_identity),
        ("AI_BOOT_PROMPT.md", generate_ai_boot_prompt),
    ];

    for (filename, generator) in &files {
        let content = generator(data)?;
        let path = ai_context_dir.join(filename);
        std::fs::write(&path, &content)?;
        generated.push(path.to_string_lossy().to_string());
    }

    Ok(generated)
}

/// MASTER_CONTEXT.md — Full project summary, technologies, architecture,
/// security posture, important modules, timeline summary, current maturity,
/// current objectives, pending risks.
fn generate_master_context(data: &ContextData) -> Result<String> {
    let mut md = String::new();
    md.push_str("# MASTER_CONTEXT.md\n\n");
    md.push_str("> **ChronoDrake AI Context — Master Document**\n");
    md.push_str("> This file provides a complete, self-contained summary of the project.\n");
    md.push_str("> Any AI model reading this file can reconstruct full project context.\n\n");
    md.push_str("---\n\n");

    // Project Summary
    md.push_str("## 1. Project Summary\n\n");
    md.push_str(&format!("- **Root Path:** `{}`\n", data.scan.root_path));
    md.push_str(&format!("- **Last Scan:** `{}`\n", data.scan.scanned_at));
    md.push_str(&format!("- **Project Type:** {} (confidence: {:.0}%)\n",
        data.intelligence.project_type.primary,
        data.intelligence.project_type.confidence * 100.0));
    md.push_str(&format!("- **Summary:** {}\n", data.intelligence.project_type.summary));
    md.push_str("\n");

    // Technologies
    md.push_str("## 2. Technologies\n\n");
    md.push_str("| Technology | Category | Status |\n");
    md.push_str("|------------|----------|--------|\n");
    for tech in &data.scan.technologies {
        let status = if tech.detected { "✅ Present" } else { "❌ Absent" };
        let detail = tech.detail.as_deref().unwrap_or("");
        md.push_str(&format!("| {} {} | {} | {} |\n", tech.name, detail, tech.category, status));
    }
    md.push_str("\n");

    // Architecture
    md.push_str("## 3. Architecture\n\n");
    md.push_str(&format!("**{}**\n\n", data.intelligence.architecture.summary));
    if !data.intelligence.architecture.layers.is_empty() {
        md.push_str("### Layers\n\n");
        for layer in &data.intelligence.architecture.layers {
            md.push_str(&format!("- `{}`\n", layer));
        }
        md.push_str("\n");
    }
    if !data.intelligence.architecture.entrypoints.is_empty() {
        md.push_str("### Entry Points\n\n");
        for ep in &data.intelligence.architecture.entrypoints {
            md.push_str(&format!("- `{}`\n", ep));
        }
        md.push_str("\n");
    }
    if !data.intelligence.architecture.databases.is_empty() {
        md.push_str("### Databases\n\n");
        for db in &data.intelligence.architecture.databases {
            md.push_str(&format!("- `{}`\n", db));
        }
        md.push_str("\n");
    }

    // Security Posture
    md.push_str("## 4. Security Posture\n\n");
    md.push_str(&format!("- **Rating:** {}\n", data.intelligence.security.rating));
    md.push_str(&format!("- **Issues Found:** {}\n", data.intelligence.security.issue_count));
    if !data.intelligence.security.issues.is_empty() {
        md.push_str("\n### Issues\n\n");
        for issue in &data.intelligence.security.issues {
            md.push_str(&format!("- [{}] {} — {}\n", issue.severity, issue.description, issue.suggestion));
        }
    }
    md.push_str("\n");

    // Important Modules
    md.push_str("## 5. Important Modules\n\n");
    md.push_str(&format!("- **Modules Detected:** {}\n", data.knowledge.module_count));
    md.push_str(&format!("- **Relationships:** {}\n", data.knowledge.relationship_count));
    md.push_str(&format!("- **Dependencies Classified:** {}\n", data.knowledge.dependency_count));
    md.push_str("\n");

    // Timeline Summary
    md.push_str("## 6. Timeline Summary\n\n");
    md.push_str(&format!("- **Evolution Changes Tracked:** {}\n", data.persistence.evolution_count));
    if data.persistence.is_first_scan {
        md.push_str("- **Status:** First scan — baseline established.\n");
    } else {
        md.push_str("- **Status:** Follow-up scan — evolution data available.\n");
    }
    md.push_str("\n");

    // Current Maturity
    md.push_str("## 7. Current Maturity\n\n");
    md.push_str("| Dimension | Score |\n");
    md.push_str("|-----------|-------|\n");
    md.push_str(&format!("| 🏗️ Architecture | {}/100 |\n", data.intelligence.maturity.architecture));
    md.push_str(&format!("| 🔒 Security | {}/100 |\n", data.intelligence.maturity.security));
    md.push_str(&format!("| 📈 Scalability | {}/100 |\n", data.intelligence.maturity.scalability));
    md.push_str(&format!("| 🧪 Testing | {}/100 |\n", data.intelligence.maturity.testing));
    md.push_str(&format!("| 📝 Documentation | {}/100 |\n", data.intelligence.maturity.documentation));
    md.push_str(&format!("| **Overall** | **{}/100** |\n", data.intelligence.maturity.overall));
    md.push_str("\n");

    // Current Objectives
    md.push_str("## 8. Current Objectives\n\n");
    md.push_str("- Maintain and improve maturity scores across all dimensions\n");
    md.push_str("- Address security issues and improve security rating\n");
    md.push_str("- Expand test coverage and documentation\n");
    md.push_str("- Track evolution and regressions over time\n");
    md.push_str("\n");

    // Pending Risks
    md.push_str("## 9. Pending Risks\n\n");
    if data.intelligence.security.issue_count > 0 {
        md.push_str(&format!("- **{} security issue(s)** require attention\n", data.intelligence.security.issue_count));
    }
    if data.intelligence.maturity.overall < 50 {
        md.push_str("- **Low overall maturity** — structural improvements recommended\n");
    }
    if data.intelligence.maturity.testing < 30 {
        md.push_str("- **Insufficient test coverage** — risk of regressions\n");
    }
    if data.intelligence.maturity.documentation < 30 {
        md.push_str("- **Poor documentation** — knowledge silos risk\n");
    }
    if data.intelligence.maturity.architecture < 30 {
        md.push_str("- **Weak architecture** — scalability and maintainability concerns\n");
    }
    md.push_str("\n---\n");
    md.push_str("*This file was automatically generated by ChronoDrake Core v0.6*\n");

    Ok(md)
}

/// CURRENT_STATE.md — Current implementation status, last successful scan,
/// existing modules, existing reports, existing database schema,
/// installed technologies, active dependencies, current blockers.
fn generate_current_state(data: &ContextData) -> Result<String> {
    let mut md = String::new();
    md.push_str("# CURRENT_STATE.md\n\n");
    md.push_str("> **ChronoDrake AI Context — Current State**\n");
    md.push_str("> This file captures the exact state of the project at the last scan.\n\n");
    md.push_str("---\n\n");

    // Implementation Status
    md.push_str("## 1. Implementation Status\n\n");
    md.push_str(&format!("- **Scanner Version:** ChronoDrake Core v0.6\n"));
    md.push_str(&format!("- **Last Scan:** `{}`\n", data.scan.scanned_at));
    md.push_str(&format!("- **Root Path:** `{}`\n", data.scan.root_path));
    md.push_str("\n");

    // Last Successful Scan
    md.push_str("## 2. Last Successful Scan\n\n");
    let detected_count = data.scan.technologies.iter().filter(|t| t.detected).count();
    md.push_str(&format!("- **Technologies Checked:** {}\n", data.scan.technologies.len()));
    md.push_str(&format!("- **Technologies Detected:** {}\n", detected_count));
    md.push_str(&format!("- **Config Files Found:** {}\n", data.scan.findings.get("config_files").map(|v| v.len()).unwrap_or(0)));
    md.push_str(&format!("- **Lock Files Found:** {}\n", data.scan.findings.get("lock_files").map(|v| v.len()).unwrap_or(0)));
    md.push_str(&format!("- **Total Dependencies:** {}\n", data.intelligence.dependencies.total_count));
    md.push_str(&format!("- **Security Issues:** {}\n", data.intelligence.security.issue_count));
    md.push_str("\n");

    // Existing Modules
    md.push_str("## 3. Existing Modules\n\n");
    md.push_str(&format!("- **Modules Detected:** {}\n", data.knowledge.module_count));
    md.push_str(&format!("- **Module Relationships:** {}\n", data.knowledge.relationship_count));
    md.push_str(&format!("- **Impact Events:** {}\n", data.knowledge.impact_count));
    md.push_str("\n");

    // Existing Reports
    md.push_str("## 4. Existing Reports\n\n");
    let reports = vec![
        ("PROJECT_STATUS.md", "Project status and maturity"),
        ("AI_HANDOFF.md", "AI handoff document"),
        ("KNOWLEDGE_GRAPH.md", "Knowledge graph visualization"),
        ("IMPACT_ANALYSIS.md", "Impact analysis"),
        ("MODULE_MAP.md", "Module map"),
        ("TIMELINE.md", "Project timeline"),
        ("RECOMMENDATIONS.md", "Recommendations"),
        ("CHANGES.md", "Change detection"),
    ];
    for (name, desc) in &reports {
        md.push_str(&format!("- **{}** — {}\n", name, desc));
    }
    md.push_str("\n");

    // Database Schema
    md.push_str("## 5. Existing Database Schema\n\n");
    md.push_str("The SQLite database (`memory/chronodrake.db`) contains:\n\n");
    md.push_str("- `projects` — Project records\n");
    md.push_str("- `scans` — Scan history with maturity scores\n");
    md.push_str("- `snapshots` — Snapshot references\n");
    md.push_str("- `decisions` — Architectural decisions\n");
    md.push_str("- `timeline` — Timeline events\n");
    md.push_str("- `evolution` — Score evolution tracking\n");
    md.push_str("- `module_relationships` — Knowledge graph edges\n");
    md.push_str("- `dependency_relationships` — Dependency classifications\n");
    md.push_str("- `impact_events` — Impact analysis events\n");
    md.push_str("- `knowledge_nodes` — Knowledge graph nodes\n");
    md.push_str("- `schema_version` — Schema migration tracking\n");
    md.push_str("\n");

    // Installed Technologies
    md.push_str("## 6. Installed Technologies\n\n");
    md.push_str("| Technology | Category | Status | Detail |\n");
    md.push_str("|------------|----------|--------|--------|\n");
    for tech in &data.scan.technologies {
        let status = if tech.detected { "✅" } else { "❌" };
        let detail = tech.detail.as_deref().unwrap_or("-");
        md.push_str(&format!("| {} | {} | {} | {} |\n", tech.name, tech.category, status, detail));
    }
    md.push_str("\n");

    // Active Dependencies
    md.push_str("## 7. Active Dependencies\n\n");
    if !data.intelligence.dependencies.dependencies.is_empty() {
        md.push_str("| Name | Version | Ecosystem | Category | Notable |\n");
        md.push_str("|------|---------|-----------|----------|---------|\n");
        for dep in data.intelligence.dependencies.dependencies.iter().take(30) {
            let notable = if dep.notable { "⭐" } else { "" };
            md.push_str(&format!("| {} | {} | {} | {} | {} |\n",
                dep.name, dep.version, dep.ecosystem, dep.category, notable));
        }
        if data.intelligence.dependencies.dependencies.len() > 30 {
            md.push_str(&format!("| ... and {} more | | | | |\n",
                data.intelligence.dependencies.dependencies.len() - 30));
        }
    } else {
        md.push_str("_No dependencies detected._\n");
    }
    md.push_str("\n");

    // Current Blockers
    md.push_str("## 8. Current Blockers\n\n");
    let mut has_blockers = false;
    if data.intelligence.security.issue_count > 0 {
        md.push_str(&format!("- 🔴 **{} security issue(s)** — see security scan for details\n", data.intelligence.security.issue_count));
        has_blockers = true;
    }
    if data.intelligence.maturity.overall < 40 {
        md.push_str(&format!("- 🟡 **Low overall maturity ({}/100)** — structural improvements needed\n", data.intelligence.maturity.overall));
        has_blockers = true;
    }
    if !has_blockers {
        md.push_str("_No critical blockers detected._\n");
    }
    md.push_str("\n---\n");
    md.push_str("*This file was automatically generated by ChronoDrake Core v0.6*\n");

    Ok(md)
}

/// NEXT_STEPS.md — Immediate next actions, recommended roadmap,
/// suggested implementation order, technical debt.
fn generate_next_steps(data: &ContextData) -> Result<String> {
    let mut md = String::new();
    md.push_str("# NEXT_STEPS.md\n\n");
    md.push_str("> **ChronoDrake AI Context — Next Steps**\n");
    md.push_str("> This file outlines the recommended roadmap and immediate actions.\n\n");
    md.push_str("---\n\n");

    // Immediate Next Actions
    md.push_str("## 1. Immediate Next Actions\n\n");
    let mut actions: Vec<String> = Vec::new();

    if data.intelligence.security.issue_count > 0 {
        actions.push(format!("Address {} security issue(s) — see security scan for details", data.intelligence.security.issue_count));
    }
    if data.intelligence.maturity.testing < 50 {
        actions.push("Improve test coverage — add tests and testing infrastructure".to_string());
    }
    if data.intelligence.maturity.documentation < 50 {
        actions.push("Improve documentation — add README, CONTRIBUTING, CHANGELOG".to_string());
    }
    if data.intelligence.maturity.architecture < 50 {
        actions.push("Improve architecture — add modular structure and service layers".to_string());
    }
    if data.intelligence.maturity.security < 50 {
        actions.push("Improve security posture — add .gitignore, .env.example, lockfiles".to_string());
    }
    if actions.is_empty() {
        actions.push("All maturity dimensions are at acceptable levels — focus on feature development".to_string());
    }

    for (i, action) in actions.iter().enumerate() {
        md.push_str(&format!("{}. {}\n", i + 1, action));
    }
    md.push_str("\n");

    // Recommended Roadmap
    md.push_str("## 2. Recommended Roadmap\n\n");
    md.push_str("### Short-term (next scan cycle)\n\n");
    md.push_str("- Address security issues and low-hanging maturity improvements\n");
    md.push_str("- Run another scan to track evolution\n");
    md.push_str("- Review recommendations from RECOMMENDATIONS.md\n\n");
    md.push_str("### Medium-term\n\n");
    md.push_str("- Improve test coverage and documentation\n");
    md.push_str("- Refactor architecture for better modularity\n");
    md.push_str("- Add CI/CD pipeline and automated scanning\n\n");
    md.push_str("### Long-term\n\n");
    md.push_str("- Achieve maturity scores > 80 across all dimensions\n");
    md.push_str("- Implement full AI context engine integration\n");
    md.push_str("- Scale to multi-project monitoring\n\n");

    // Suggested Implementation Order
    md.push_str("## 3. Suggested Implementation Order\n\n");
    md.push_str("1. **Security fixes** — highest priority, address all security issues\n");
    md.push_str("2. **Testing infrastructure** — prevent regressions during refactoring\n");
    md.push_str("3. **Documentation** — improve knowledge sharing and onboarding\n");
    md.push_str("4. **Architecture improvements** — modularize and decouple\n");
    md.push_str("5. **Feature development** — build on solid foundation\n\n");

    // Technical Debt
    md.push_str("## 4. Technical Debt\n\n");
    let mut debt_items: Vec<String> = Vec::new();
    if data.intelligence.maturity.architecture < 60 {
        debt_items.push("Architecture lacks modularity — consider refactoring into clear layers".to_string());
    }
    if data.intelligence.maturity.testing < 40 {
        debt_items.push("No or minimal test coverage — high regression risk".to_string());
    }
    if data.intelligence.maturity.documentation < 40 {
        debt_items.push("Missing or sparse documentation — knowledge silos".to_string());
    }
    if data.intelligence.maturity.security < 40 {
        debt_items.push("Security gaps — potential vulnerabilities".to_string());
    }
    if debt_items.is_empty() {
        debt_items.push("No significant technical debt detected.".to_string());
    }
    for item in &debt_items {
        md.push_str(&format!("- {}\n", item));
    }
    md.push_str("\n---\n");
    md.push_str("*This file was automatically generated by ChronoDrake Core v0.6*\n");

    Ok(md)
}

/// DECISIONS.md — Architectural decisions, why they were made, tradeoffs,
/// security decisions, rejected alternatives.
fn generate_decisions(_data: &ContextData) -> Result<String> {
    let mut md = String::new();
    md.push_str("# DECISIONS.md\n\n");
    md.push_str("> **ChronoDrake AI Context — Architectural Decisions**\n");
    md.push_str("> This file documents key decisions made during project development.\n\n");
    md.push_str("---\n\n");

    // Architectural Decisions
    md.push_str("## 1. Architectural Decisions\n\n");

    md.push_str("### Decision: Modular Scanner Architecture\n\n");
    md.push_str("- **Context:** Need to support multiple technology stacks (Rust, Node.js, Tauri)\n");
    md.push_str("- **Decision:** Each technology has its own scanner module (`scanner/rust.rs`, `scanner/node.rs`, `scanner/tauri.rs`)\n");
    md.push_str("- **Tradeoff:** More code upfront, but easier to extend with new technologies\n");
    md.push_str("- **Status:** ✅ Implemented\n\n");

    md.push_str("### Decision: SQLite for Persistence\n\n");
    md.push_str("- **Context:** Need offline-first, deterministic, local storage\n");
    md.push_str("- **Decision:** Use SQLite via `rusqlite` — no server, no cloud, no dependencies\n");
    md.push_str("- **Tradeoff:** Not suitable for distributed/cloud scenarios without additional tooling\n");
    md.push_str("- **Status:** ✅ Implemented\n\n");

    md.push_str("### Decision: Schema Versioning with Migrations\n\n");
    md.push_str("- **Context:** Database schema evolves across versions\n");
    md.push_str("- **Decision:** Implement `schema_version` table + sequential migration engine\n");
    md.push_str("- **Tradeoff:** Adds complexity but ensures backward compatibility\n");
    md.push_str("- **Status:** ✅ Implemented\n\n");

    md.push_str("### Decision: Knowledge Graph in SQLite\n\n");
    md.push_str("- **Context:** Need relationship tracking between modules, dependencies, and features\n");
    md.push_str("- **Decision:** Store graph edges directly in SQLite tables (no separate graph DB)\n");
    md.push_str("- **Tradeoff:** Less performant for complex graph queries, but simpler infrastructure\n");
    md.push_str("- **Status:** ✅ Implemented\n\n");

    md.push_str("### Decision: AI Context Engine (v0.6)\n\n");
    md.push_str("- **Context:** Need to provide full project context to any AI model\n");
    md.push_str("- **Decision:** Generate deterministic markdown files in `/ai-context/` directory\n");
    md.push_str("- **Tradeoff:** Static files vs. dynamic API — no real-time updates, but zero infrastructure\n");
    md.push_str("- **Status:** ✅ Implemented\n\n");

    // Why They Were Made
    md.push_str("## 2. Why These Decisions Were Made\n\n");
    md.push_str("- **Offline-first:** All features must work without internet access\n");
    md.push_str("- **Deterministic:** Same input always produces same output — no AI APIs, no randomness\n");
    md.push_str("- **Local-first:** No cloud, no telemetry, no external dependencies\n");
    md.push_str("- **Extensible:** New scanners, analyzers, and generators can be added without modifying existing code\n");
    md.push_str("- **Typed errors:** Using `thiserror` for all error types — no `anyhow` in library code\n");
    md.push_str("- **Strong typing:** `serde` everywhere for serialization\n\n");

    // Security Decisions
    md.push_str("## 3. Security Decisions\n\n");
    md.push_str("- **No AI APIs:** All analysis is local and deterministic\n");
    md.push_str("- **No telemetry:** No data leaves the machine\n");
    md.push_str("- **No cloud:** No external service dependencies\n");
    md.push_str("- **SHA-256 integrity:** Cache entries are verified with content hashing\n");
    md.push_str("- **SQLite file permissions:** Database respects filesystem permissions\n\n");

    // Rejected Alternatives
    md.push_str("## 4. Rejected Alternatives\n\n");
    md.push_str("| Alternative | Reason for Rejection |\n");
    md.push_str("|-------------|---------------------|\n");
    md.push_str("| PostgreSQL | Requires server, not offline-first |\n");
    md.push_str("| MongoDB | Heavy dependency, not deterministic |\n");
    md.push_str("| Neo4j (graph DB) | Overkill for current scale, adds complexity |\n");
    md.push_str("| Redis cache | External dependency, not local-first |\n");
    md.push_str("| Cloud sync | Violates offline-first principle |\n");
    md.push_str("| AI embeddings | Requires API calls, not deterministic |\n");
    md.push_str("| Vector database | Requires AI pipeline, violates local-first |\n");
    md.push_str("| YAML config | TOML is more readable and standard for Rust |\n");
    md.push_str("| JSON schema | TOML is more human-friendly for config |\n");
    md.push_str("\n---\n");
    md.push_str("*This file was automatically generated by ChronoDrake Core v0.6*\n");

    Ok(md)
}

/// ACTIVE_TASK.md — Current active task or objective being worked on.
fn generate_active_task(data: &ContextData) -> Result<String> {
    let mut md = String::new();
    md.push_str("# ACTIVE_TASK.md\n\n");
    md.push_str("> **ChronoDrake AI Context — Active Task**\n");
    md.push_str("> This file tracks the current active objective.\n\n");
    md.push_str("---\n\n");

    md.push_str("## Current Objective\n\n");
    md.push_str("**ChronoDrake v0.6 — AI Context Engine**\n\n");
    md.push_str("### Goal\n\n");
    md.push_str("Transform ChronoDrake from a project scanner into a universal AI development continuity system.\n\n");

    md.push_str("### Key Requirements\n\n");
    md.push_str("- Generate `/ai-context/` directory with 7 context files after every scan\n");
    md.push_str("- Provide deterministic, offline-first AI context for any AI model\n");
    md.push_str("- Add `cargo run -- context`, `cargo run -- handoff`, `cargo run -- doctor` commands\n");
    md.push_str("- Doctor command detects installed tools, toolchains, and environment\n");
    md.push_str("- Generate SYSTEM_REPORT.md and DEVELOPMENT_ENVIRONMENT.md\n");
    md.push_str("- No AI APIs, no embeddings, no cloud, no telemetry\n\n");

    md.push_str("### Status\n\n");
    md.push_str(&format!("- **Last scan:** {}\n", data.scan.scanned_at));
    md.push_str(&format!("- **Project type:** {}\n", data.intelligence.project_type.primary));
    md.push_str(&format!("- **Overall maturity:** {}/100\n", data.intelligence.maturity.overall));
    md.push_str("\n");

    md.push_str("### Next Action\n\n");
    md.push_str("Review the generated AI context files and continue development according to NEXT_STEPS.md\n");
    md.push_str("\n---\n");
    md.push_str("*This file was automatically generated by ChronoDrake Core v0.6*\n");

    Ok(md)
}

/// PROJECT_IDENTITY.md — Project name, version, description, authors, license,
/// repository, language, framework, build system, dependencies.
fn generate_project_identity(data: &ContextData) -> Result<String> {
    let mut md = String::new();
    md.push_str("# PROJECT_IDENTITY.md\n\n");
    md.push_str("> **ChronoDrake AI Context — Project Identity**\n");
    md.push_str("> This file defines the project's identity and key metadata.\n\n");
    md.push_str("---\n\n");

    md.push_str("## Identity\n\n");
    md.push_str(&format!("- **Project Name:** `{}`\n",
        std::path::Path::new(&data.scan.root_path)
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_else(|| std::borrow::Cow::Borrowed("unknown"))));
    md.push_str(&format!("- **Scanner:** ChronoDrake Core v0.6\n"));
    md.push_str(&format!("- **Scan Timestamp:** `{}`\n", data.scan.scanned_at));
    md.push_str(&format!("- **Root Path:** `{}`\n", data.scan.root_path));
    md.push_str("\n");

    md.push_str("## Technology Stack\n\n");
    md.push_str("| Technology | Category | Status |\n");
    md.push_str("|------------|----------|--------|\n");
    for tech in &data.scan.technologies {
        let status = if tech.detected { "✅" } else { "❌" };
        md.push_str(&format!("| {} | {} | {} |\n", tech.name, tech.category, status));
    }
    md.push_str("\n");

    md.push_str("## Project Type\n\n");
    md.push_str(&format!("**{}**\n\n", data.intelligence.project_type.summary));
    if !data.intelligence.project_type.matches.is_empty() {
        md.push_str("| Type | Evidence | Weight |\n");
        md.push_str("|------|----------|--------|\n");
        for m in &data.intelligence.project_type.matches {
            md.push_str(&format!("| {} | {} | {} |\n", m.kind, m.reason, m.weight));
        }
    }
    md.push_str("\n");

    md.push_str("## Build System\n\n");
    if data.scan.has_cargo {
        md.push_str("- **Build System:** Cargo (Rust)\n");
    }
    if data.scan.has_package_json {
        md.push_str("- **Build System:** npm/pnpm/yarn (Node.js)\n");
    }
    if data.scan.has_tauri {
        md.push_str("- **Framework:** Tauri (Desktop Application)\n");
    }
    if data.scan.has_git {
        md.push_str("- **Version Control:** Git\n");
    }
    md.push_str("\n");

    md.push_str("## Maturity Profile\n\n");
    md.push_str(&format!("- **Overall:** {}/100\n", data.intelligence.maturity.overall));
    md.push_str(&format!("- **Architecture:** {}/100\n", data.intelligence.maturity.architecture));
    md.push_str(&format!("- **Security:** {}/100\n", data.intelligence.maturity.security));
    md.push_str(&format!("- **Testing:** {}/100\n", data.intelligence.maturity.testing));
    md.push_str(&format!("- **Documentation:** {}/100\n", data.intelligence.maturity.documentation));
    md.push_str("\n---\n");
    md.push_str("*This file was automatically generated by ChronoDrake Core v0.6*\n");

    Ok(md)
}

/// AI_BOOT_PROMPT.md — Optimized for ANY AI model. Reconstruct full project
/// context, explain project goals, current progress, coding standards,
/// architecture, next objective. Allow seamless continuation in another AI.
fn generate_ai_boot_prompt(data: &ContextData) -> Result<String> {
    let mut md = String::new();
    md.push_str("# AI_BOOT_PROMPT.md\n\n");
    md.push_str("> **ChronoDrake AI Context — AI Boot Prompt**\n");
    md.push_str("> This file is optimized for ANY AI model to reconstruct full project context.\n");
    md.push_str("> Read this file first to understand the project completely.\n\n");
    md.push_str("---\n\n");

    md.push_str("## SYSTEM INSTRUCTION\n\n");
    md.push_str("You are an AI assistant continuing work on this project. ");
    md.push_str("Read this entire file to reconstruct full context. ");
    md.push_str("Then read the other files in the `/ai-context/` directory for detailed information.\n\n");

    md.push_str("---\n\n");

    // Project Overview
    md.push_str("## 1. PROJECT OVERVIEW\n\n");
    md.push_str(&format!("- **Project:** {}\n",
        std::path::Path::new(&data.scan.root_path)
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_else(|| std::borrow::Cow::Borrowed("unknown"))));
    md.push_str(&format!("- **Type:** {} (confidence: {:.0}%)\n",
        data.intelligence.project_type.primary,
        data.intelligence.project_type.confidence * 100.0));
    md.push_str(&format!("- **Summary:** {}\n", data.intelligence.project_type.summary));
    md.push_str(&format!("- **Last Scan:** {}\n", data.scan.scanned_at));
    md.push_str(&format!("- **Root Path:** `{}`\n", data.scan.root_path));
    md.push_str("\n");

    // Goals
    md.push_str("## 2. PROJECT GOALS\n\n");
    md.push_str("This project is scanned and tracked by **ChronoDrake**, an AI Continuity Operating System.\n\n");
    md.push_str("### ChronoDrake Goals\n\n");
    md.push_str("- Provide universal AI development continuity\n");
    md.push_str("- Enable seamless handoff between any AI models\n");
    md.push_str("- Track project evolution, maturity, and technical debt over time\n");
    md.push_str("- Generate deterministic, offline-first context for AI assistants\n");
    md.push_str("- Maintain local-first, no-cloud, no-telemetry architecture\n\n");

    // Current Progress
    md.push_str("## 3. CURRENT PROGRESS\n\n");
    md.push_str(&format!("- **Overall Maturity:** {}/100\n", data.intelligence.maturity.overall));
    md.push_str(&format!("- **Security Rating:** {}\n", data.intelligence.security.rating));
    md.push_str(&format!("- **Security Issues:** {}\n", data.intelligence.security.issue_count));
    md.push_str(&format!("- **Dependencies Tracked:** {}\n", data.intelligence.dependencies.total_count));
    md.push_str(&format!("- **Modules Detected:** {}\n", data.knowledge.module_count));
    md.push_str(&format!("- **Relationships Mapped:** {}\n", data.knowledge.relationship_count));
    md.push_str(&format!("- **Evolution Changes:** {}\n", data.persistence.evolution_count));
    md.push_str("\n");

    // Coding Standards
    md.push_str("## 4. CODING STANDARDS\n\n");
    md.push_str("- **Language:** Rust (stable)\n");
    md.push_str("- **Error Handling:** `thiserror` for library errors, `anyhow` for application-level\n");
    md.push_str("- **Serialization:** `serde` with `Serialize`/`Deserialize` derives\n");
    md.push_str("- **Concurrency:** `tokio` async runtime\n");
    md.push_str("- **Database:** `rusqlite` with schema versioning and migrations\n");
    md.push_str("- **Logging:** `tracing` + `tracing-subscriber` with env-filter\n");
    md.push_str("- **Testing:** Unit tests in-module, integration tests in `/tests/`\n");
    md.push_str("- **Determinism:** All outputs must be reproducible — no randomness, no external APIs\n");
    md.push_str("- **Offline-first:** No cloud, no telemetry, no internet required\n\n");

    // Architecture
    md.push_str("## 5. ARCHITECTURE\n\n");
    md.push_str("### Module Structure\n\n");
    md.push_str("```\n");
    md.push_str("src/\n");
    md.push_str("├── main.rs              # CLI entry point\n");
    md.push_str("├── config/              # Configuration system (chronodrake.toml)\n");
    md.push_str("├── errors/              # Typed error system (thiserror)\n");
    md.push_str("├── models/              # Data models (ProjectScan, DetectedTech)\n");
    md.push_str("├── scanner/             # Technology detection scanners\n");
    md.push_str("│   ├── filesystem.rs    # Filesystem scanning\n");
    md.push_str("│   ├── incremental.rs   # Incremental scan engine (SHA-256)\n");
    md.push_str("│   ├── rust.rs          # Rust/Cargo detection\n");
    md.push_str("│   ├── node.rs          # Node.js detection\n");
    md.push_str("│   └── tauri.rs         # Tauri detection\n");
    md.push_str("├── intelligence/        # Analysis pipeline\n");
    md.push_str("│   ├── project_type.rs  # Project type detection\n");
    md.push_str("│   ├── architecture.rs  # Architecture analysis\n");
    md.push_str("│   ├── maturity.rs      # Maturity scoring\n");
    md.push_str("│   ├── security.rs      # Security scanning\n");
    md.push_str("│   ├── dependencies.rs  # Dependency analysis\n");
    md.push_str("│   └── changes.rs       # Change detection\n");
    md.push_str("├── persistence/         # SQLite persistence layer\n");
    md.push_str("│   ├── sqlite.rs        # Database operations\n");
    md.push_str("│   ├── snapshots.rs     # Snapshot management\n");
    md.push_str("│   ├── timeline.rs      # Timeline events\n");
    md.push_str("│   ├── decisions.rs     # Decision tracking\n");
    md.push_str("│   └── evolution.rs     # Score evolution\n");
    md.push_str("├── knowledge/           # Knowledge graph engine\n");
    md.push_str("│   ├── modules.rs       # Module detection\n");
    md.push_str("│   ├── graph.rs         # Graph building\n");
    md.push_str("│   ├── relationships.rs # Relationship detection\n");
    md.push_str("│   ├── dependencies.rs  # Dependency classification\n");
    md.push_str("│   ├── impacts.rs       # Impact analysis\n");
    md.push_str("│   └── queries.rs       # Graph queries\n");
    md.push_str("├── context/             # AI Context Engine (v0.6)\n");
    md.push_str("│   └── mod.rs           # 7 context file generators\n");
    md.push_str("├── doctor/              # Doctor engine (v0.6)\n");
    md.push_str("│   └── mod.rs           # System detection\n");
    md.push_str("├── cache/               # Cache layer\n");
    md.push_str("│   └── mod.rs           # TTL cache with SHA-256 integrity\n");
    md.push_str("├── migrations/          # Schema migrations\n");
    md.push_str("│   └── mod.rs           # Migration engine\n");
    md.push_str("├── memory/              # Report generators\n");
    md.push_str("│   ├── generator.rs     # PROJECT_STATUS.md, CHANGES.md\n");
    md.push_str("│   └── mod.rs\n");
    md.push_str("├── handoff/             # AI handoff generator\n");
    md.push_str("│   ├── generator.rs     # AI_HANDOFF.md\n");
    md.push_str("│   └── mod.rs\n");
    md.push_str("├── utils/               # Utilities\n");
    md.push_str("│   └── logger.rs        # Tracing-based logging\n");
    md.push_str("└── models/              # Data models\n");
    md.push_str("    └── project.rs       # ProjectScan, DetectedTech\n");
    md.push_str("```\n\n");

    // Next Objective
    md.push_str("## 6. NEXT OBJECTIVE\n\n");
    md.push_str("The current objective is **ChronoDrake v0.6 — AI Context Engine**.\n\n");
    md.push_str("### What to do\n\n");
    md.push_str("1. Read all files in `/ai-context/` to understand full project context\n");
    md.push_str("2. Check `CURRENT_STATE.md` for the exact implementation status\n");
    md.push_str("3. Follow `NEXT_STEPS.md` for the recommended roadmap\n");
    md.push_str("4. Review `DECISIONS.md` for architectural decisions and tradeoffs\n");
    md.push_str("5. Use `PROJECT_STATUS.md` for detailed scan results\n");
    md.push_str("6. Use `AI_HANDOFF.md` for AI-specific handoff instructions\n\n");

    md.push_str("### Constraints\n\n");
    md.push_str("- All outputs must be deterministic and reproducible\n");
    md.push_str("- No AI APIs, no embeddings, no cloud, no telemetry\n");
    md.push_str("- Must work offline with zero external dependencies\n");
    md.push_str("- Must preserve backward compatibility with v0.5\n\n");

    md.push_str("---\n");
    md.push_str("*This file was automatically generated by ChronoDrake Core v0.6*\n");
    md.push_str("*ChronoDrake — AI Continuity Operating System*\n");

    Ok(md)
}