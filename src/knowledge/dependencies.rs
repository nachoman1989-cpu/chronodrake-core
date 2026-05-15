use anyhow::Result;

use crate::intelligence::IntelligenceReport;
use crate::persistence::sqlite::Database;

/// Classification of a dependency's role.
#[derive(Debug, Clone)]
pub struct DependencyClass {
    pub name: String,
    pub version: String,
    pub ecosystem: String,
    pub is_critical: bool,
    pub is_security: bool,
    pub is_ai: bool,
    pub is_blockchain: bool,
    pub is_async: bool,
    pub category: String,
}

/// Analyzes dependencies and classifies them into intelligence categories.
///
/// Detects:
/// - Critical crates (core infrastructure)
/// - Security dependencies (encryption, auth, tokens)
/// - AI/ML dependencies
/// - Blockchain dependencies
/// - Async dependencies
pub fn classify_dependencies(
    db: &Database,
    intelligence: &IntelligenceReport,
) -> Result<Vec<DependencyClass>> {
    let mut classified: Vec<DependencyClass> = Vec::new();

    for dep in &intelligence.dependencies.dependencies {
        let name_lower = dep.name.to_lowercase();

        let is_critical = is_critical_crate(&name_lower);
        let is_security = is_security_crate(&name_lower);
        let is_ai = is_ai_crate(&name_lower);
        let is_blockchain = is_blockchain_crate(&name_lower);
        let is_async = is_async_crate(&name_lower);

        let category = determine_category(
            &name_lower, is_critical, is_security, is_ai, is_blockchain, is_async,
        );

        let dc = DependencyClass {
            name: dep.name.clone(),
            version: dep.version.clone(),
            ecosystem: dep.ecosystem.clone(),
            is_critical,
            is_security,
            is_ai,
            is_blockchain,
            is_async,
            category,
        };

        // Persist to database
        let id = format!("dep:{}", dep.name);
        db.insert_dependency_relationship(
            &id,
            &dep.name,
            &dc.category,
            dc.is_critical,
            dc.is_security,
            dc.is_ai,
            dc.is_blockchain,
            dc.is_async,
            &dep.ecosystem,
            &dep.version,
        )?;

        classified.push(dc);
    }

    Ok(classified)
}

/// Determines the primary category of a dependency.
fn determine_category(
    name: &str,
    is_critical: bool,
    is_security: bool,
    is_ai: bool,
    is_blockchain: bool,
    is_async: bool,
) -> String {
    if is_security {
        "security".to_string()
    } else if is_ai {
        "ai".to_string()
    } else if is_blockchain {
        "blockchain".to_string()
    } else if is_async {
        "async".to_string()
    } else if is_critical {
        "critical".to_string()
    } else {
        "standard".to_string()
    }
}

fn is_critical_crate(name: &str) -> bool {
    let critical = [
        "serde", "anyhow", "thiserror", "tokio", "chrono", "serde_json",
        "rusqlite", "uuid", "walkdir", "toml", "colored", "regex",
        "log", "env_logger", "clap", "structopt", "rayon",
    ];
    critical.iter().any(|k| name == *k)
}

fn is_security_crate(name: &str) -> bool {
    let keywords = [
        "aes", "crypto", "sha", "hash", "bcrypt", "argon2", "jwt", "oauth",
        "tls", "ssl", "x509", "pem", "pkcs", "signature", "cipher", "decrypt",
        "encrypt", "password", "auth", "token", "security", "key",
    ];
    keywords.iter().any(|k| name.contains(k))
}

fn is_async_crate(name: &str) -> bool {
    let keywords = ["tokio", "async", "futures", "async-std", "smol"];
    keywords.iter().any(|k| name.contains(k))
}

fn is_ai_crate(name: &str) -> bool {
    let keywords = [
        "tensor", "candle", "burn", "ort", "onnx", "tokenizer", "transformers",
        "llm", "rag", "embedding", "bert", "gpt", "neural", "ml",
    ];
    keywords.iter().any(|k| name.contains(k))
}

fn is_blockchain_crate(name: &str) -> bool {
    let keywords = [
        "ethereum", "web3", "solana", "blockchain", "contract", "wallet",
        "defi", "nft", "evm", "solidity",
    ];
    keywords.iter().any(|k| name.contains(k))
}

/// Generates a dependency intelligence section for reports.
pub fn generate_dependency_intelligence_summary(classified: &[DependencyClass]) -> String {
    let mut md = String::new();

    let critical: Vec<_> = classified.iter().filter(|d| d.is_critical).collect();
    let security: Vec<_> = classified.iter().filter(|d| d.is_security).collect();
    let ai: Vec<_> = classified.iter().filter(|d| d.is_ai).collect();
    let blockchain: Vec<_> = classified.iter().filter(|d| d.is_blockchain).collect();
    let async_deps: Vec<_> = classified.iter().filter(|d| d.is_async).collect();

    md.push_str(&format!("- **Critical dependencies:** {}\n", critical.len()));
    for d in &critical {
        md.push_str(&format!("  - `{}` v{} ({})\n", d.name, d.version, d.ecosystem));
    }

    md.push_str(&format!("\n- **Security dependencies:** {}\n", security.len()));
    for d in &security {
        md.push_str(&format!("  - `{}` v{} ({})\n", d.name, d.version, d.ecosystem));
    }

    if !ai.is_empty() {
        md.push_str(&format!("\n- **AI/ML dependencies:** {}\n", ai.len()));
        for d in &ai {
            md.push_str(&format!("  - `{}` v{} ({})\n", d.name, d.version, d.ecosystem));
        }
    }

    if !blockchain.is_empty() {
        md.push_str(&format!("\n- **Blockchain dependencies:** {}\n", blockchain.len()));
        for d in &blockchain {
            md.push_str(&format!("  - `{}` v{} ({})\n", d.name, d.version, d.ecosystem));
        }
    }

    if !async_deps.is_empty() {
        md.push_str(&format!("\n- **Async dependencies:** {}\n", async_deps.len()));
        for d in &async_deps {
            md.push_str(&format!("  - `{}` v{} ({})\n", d.name, d.version, d.ecosystem));
        }
    }

    md
}
