use anyhow::{Context, Result};
use std::path::Path;

use crate::utils::Logger;

use super::models::ProjectSummary;
use super::registry;

/// Loads a project by name: finds it in the registry, marks it as active,
/// reads its ai-context/ files, and returns a summary.
pub fn load_project(name: &str) -> Result<ProjectSummary> {
    let project = registry::find_project(name)?
        .with_context(|| format!("Project '{}' not found in registry. Use 'chronodrake register' first.", name))?;

    // Mark as active
    registry::set_current_project(&project.name)?;

    let project_path = &project.path;
    let ai_context_dir = Path::new(project_path).join("ai-context");

    // Read ai-context files
    let has_master = read_context_file(&ai_context_dir, "MASTER_CONTEXT.md").is_ok();
    let has_state = read_context_file(&ai_context_dir, "CURRENT_STATE.md").is_ok();
    let has_steps = read_context_file(&ai_context_dir, "NEXT_STEPS.md").is_ok();
    let has_decisions = read_context_file(&ai_context_dir, "DECISIONS.md").is_ok();
    let has_task = read_context_file(&ai_context_dir, "ACTIVE_TASK.md").is_ok();
    let has_identity = read_context_file(&ai_context_dir, "PROJECT_IDENTITY.md").is_ok();
    let has_boot = read_context_file(&ai_context_dir, "AI_BOOT_PROMPT.md").is_ok();

    // Read AI boot prompt content if available
    let boot_prompt_content = if has_boot {
        read_context_file(&ai_context_dir, "AI_BOOT_PROMPT.md").ok()
    } else {
        None
    };

    // Try to parse scores from MASTER_CONTEXT.md
    let master_content = read_context_file(&ai_context_dir, "MASTER_CONTEXT.md").unwrap_or_default();
    let architecture_score = parse_score(&master_content, "Architecture");
    let security_score = parse_score(&master_content, "Security");
    let maturity_overall = parse_score(&master_content, "Overall Maturity");

    // Try to parse module count from MASTER_CONTEXT.md
    let module_count = parse_module_count(&master_content);

    Ok(ProjectSummary {
        name: project.name.clone(),
        path: project.path.clone(),
        stack: project.stack.clone(),
        last_scan: project.last_scan.clone(),
        architecture_score,
        security_score,
        maturity_overall,
        module_count,
        has_master_context: has_master,
        has_current_state: has_state,
        has_next_steps: has_steps,
        has_decisions,
        has_active_task: has_task,
        has_project_identity: has_identity,
        has_ai_boot_prompt: has_boot,
        boot_prompt_content,
    })
}

/// Reads a single context file from the ai-context directory.
fn read_context_file(dir: &Path, filename: &str) -> Result<String> {
    let path = dir.join(filename);
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {:?}", path))?;
    Ok(content)
}

/// Parses a numeric score from markdown content by looking for a pattern like
/// `**Category:** 85/100` or `Category: 85/100`.
fn parse_score(content: &str, category: &str) -> Option<u8> {
    // Look for patterns like "Architecture: 85/100", "Architecture : 85 / 100",
    // "**Architecture:** 85/100", etc.
    let variants = [
        format!("{}:", category),           // "Architecture:"
        format!("{} :", category),          // "Architecture :"
        format!("**{}:**", category),       // "**Architecture:**"
        format!("**{}:**", category),       // "**Architecture:**"
    ];

    for variant in &variants {
        if let Some(pos) = content.find(variant.as_str()) {
            let after = &content[pos + variant.len()..];
            // Skip any non-digit characters (spaces, etc.)
            let start = after.find(|c: char| c.is_ascii_digit());
            if let Some(digit_start) = start {
                let from_digit = &after[digit_start..];
                let digits: String = from_digit.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(n) = digits.parse::<u8>() {
                    if n <= 100 {
                        return Some(n);
                    }
                }
            }
        }
    }
    None
}

/// Parses the module count from markdown content.
fn parse_module_count(content: &str) -> Option<usize> {
    let variants = [
        "Modules:",
        "Modules :",
        "modules detected:",
        "Module Count:",
    ];

    for variant in &variants {
        if let Some(pos) = content.find(variant) {
            let after = &content[pos + variant.len()..];
            let start = after.find(|c: char| c.is_ascii_digit());
            if let Some(digit_start) = start {
                let from_digit = &after[digit_start..];
                let digits: String = from_digit.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(n) = digits.parse::<usize>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

/// Displays a formatted project summary to the console.
pub fn display_summary(summary: &ProjectSummary) {
    Logger::section(&format!("Project: {}", summary.name));
    Logger::kv("Path", &summary.path);
    Logger::kv("Stack", &summary.stack);

    if let Some(ref last) = summary.last_scan {
        Logger::kv("Last Scan", last);
    } else {
        Logger::kv("Last Scan", "Never");
    }

    Logger::divider();
    Logger::sub_section("Maturity Scores");
    if let Some(arch) = summary.architecture_score {
        Logger::kv("Architecture", &format!("{}/100", arch));
    }
    if let Some(sec) = summary.security_score {
        Logger::kv("Security", &format!("{}/100", sec));
    }
    if let Some(overall) = summary.maturity_overall {
        Logger::kv("Overall", &format!("{}/100", overall));
    }

    if let Some(count) = summary.module_count {
        Logger::kv("Modules", &count.to_string());
    }

    Logger::divider();
    Logger::sub_section("AI Context Files");
    let files = [
        ("MASTER_CONTEXT.md", summary.has_master_context),
        ("CURRENT_STATE.md", summary.has_current_state),
        ("NEXT_STEPS.md", summary.has_next_steps),
        ("DECISIONS.md", summary.has_decisions),
        ("ACTIVE_TASK.md", summary.has_active_task),
        ("PROJECT_IDENTITY.md", summary.has_project_identity),
        ("AI_BOOT_PROMPT.md", summary.has_ai_boot_prompt),
    ];

    for (name, exists) in &files {
        if *exists {
            Logger::success(&format!("    ✅ {}", name));
        } else {
            Logger::raw(&format!("    ❌ {} (not found)", name));
        }
    }
}

/// Displays the AI boot prompt content to the console.
pub fn display_boot_prompt(content: &str) {
    Logger::section("AI Boot Prompt");
    Logger::divider();
    // Print the content line by line using raw logger
    for line in content.lines() {
        Logger::raw(line);
    }
    Logger::divider();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_score_valid() {
        let content = "## Maturity Scores\nArchitecture: 85/100\nSecurity: 72/100";
        assert_eq!(parse_score(content, "Architecture"), Some(85));
        assert_eq!(parse_score(content, "Security"), Some(72));
    }

    #[test]
    fn test_parse_score_bold() {
        let content = "**Architecture:** 85/100";
        assert_eq!(parse_score(content, "Architecture"), Some(85));
    }

    #[test]
    fn test_parse_score_with_spaces() {
        let content = "Architecture : 85 / 100";
        assert_eq!(parse_score(content, "Architecture"), Some(85));
    }

    #[test]
    fn test_parse_score_missing() {
        let content = "No scores here";
        assert_eq!(parse_score(content, "Architecture"), None);
    }

    #[test]
    fn test_parse_score_out_of_range() {
        // Scores > 100 are filtered out by parse_score
        let content = "Architecture: 200/100";
        assert_eq!(parse_score(content, "Architecture"), None);
    }

    #[test]
    fn test_parse_module_count() {
        let content = "Modules: 42\nRelationships: 10";
        assert_eq!(parse_module_count(content), Some(42));
    }

    #[test]
    fn test_parse_module_count_missing() {
        let content = "No modules here";
        assert_eq!(parse_module_count(content), None);
    }

    #[test]
    fn test_read_context_file_nonexistent() {
        let dir = std::path::Path::new("/nonexistent");
        let result = read_context_file(dir, "test.md");
        assert!(result.is_err());
    }
}
