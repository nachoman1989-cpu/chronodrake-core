use anyhow::Result;

use crate::intelligence::IntelligenceReport;
use crate::models::ProjectScan;
use crate::persistence::sqlite::Database;

use super::modules::ModuleNode;

/// A detected relationship between two entities.
#[derive(Debug, Clone)]
pub struct Relationship {
    pub source: String,
    pub target: String,
    pub rel_type: String,
    pub weight: f64,
    pub metadata: String,
}

/// Detects all relationships between modules, dependencies, features, files,
/// scores, and architecture.
pub fn detect_relationships(
    db: &Database,
    scan: &ProjectScan,
    intelligence: &IntelligenceReport,
    modules: &[ModuleNode],
) -> Result<Vec<Relationship>> {
    let mut relationships: Vec<Relationship> = Vec::new();

    // ── 1. Module ↔ Technology relationships ────────────────
    for module in modules {
        for tech in &scan.technologies {
            if tech.detected {
                // Check if module name or imports relate to this tech
                let module_lower = module.name.to_lowercase();
                let tech_lower = tech.name.to_lowercase();
                if module_lower.contains(&tech_lower) || tech_lower.contains(&module_lower) {
                    relationships.push(Relationship {
                        source: format!("module:{}", module.name),
                        target: format!("tech:{}", tech.name),
                        rel_type: "implements".to_string(),
                        weight: 1.0,
                        metadata: format!("Module {} relates to technology {}", module.name, tech.name),
                    });
                }
            }
        }
    }

    // ── 2. Module ↔ Architecture layer relationships ────────
    for module in modules {
        for layer in &intelligence.architecture.layers {
            let layer_lower = layer.to_lowercase();
            let module_lower = module.name.to_lowercase();
            if module_lower.contains(&layer_lower) || layer_lower.contains(&module_lower) {
                relationships.push(Relationship {
                    source: format!("module:{}", module.name),
                    target: format!("layer:{}", layer),
                    rel_type: "belongs_to".to_string(),
                    weight: 1.0,
                    metadata: format!("Module {} is in layer {}", module.name, layer),
                });
            }
        }
    }

    // ── 3. Dependency ↔ Security relationships ──────────────
    for dep in &intelligence.dependencies.dependencies {
        if dep.notable {
            // Check if dependency relates to security
            let dep_lower = dep.name.to_lowercase();
            let is_security = is_security_crate(&dep_lower);
            let is_async = is_async_crate(&dep_lower);
            let is_ai = is_ai_crate(&dep_lower);
            let is_blockchain = is_blockchain_crate(&dep_lower);

            if is_security {
                relationships.push(Relationship {
                    source: format!("dep:{}", dep.name),
                    target: "maturity:security".to_string(),
                    rel_type: "impacts".to_string(),
                    weight: 1.5,
                    metadata: format!("{} is a security-related dependency", dep.name),
                });
            }
            if is_async {
                relationships.push(Relationship {
                    source: format!("dep:{}", dep.name),
                    target: "maturity:scalability".to_string(),
                    rel_type: "impacts".to_string(),
                    weight: 1.2,
                    metadata: format!("{} enables async operations", dep.name),
                });
            }
            if is_ai {
                relationships.push(Relationship {
                    source: format!("dep:{}", dep.name),
                    target: "tech:AI".to_string(),
                    rel_type: "enables".to_string(),
                    weight: 2.0,
                    metadata: format!("{} is an AI/ML dependency", dep.name),
                });
            }
            if is_blockchain {
                relationships.push(Relationship {
                    source: format!("dep:{}", dep.name),
                    target: "tech:Blockchain".to_string(),
                    rel_type: "enables".to_string(),
                    weight: 2.0,
                    metadata: format!("{} is a blockchain dependency", dep.name),
                });
            }
        }
    }

    // ── 4. File ↔ Module relationships ──────────────────────
    for (category, files) in &scan.findings {
        for file in files {
            // Extract module name from file path
            if let Some(module_name) = extract_module_from_path(file) {
                if modules.iter().any(|m| m.name == module_name) {
                    relationships.push(Relationship {
                        source: format!("file:{}", file),
                        target: format!("module:{}", module_name),
                        rel_type: "belongs_to".to_string(),
                        weight: 1.0,
                        metadata: format!("File {} belongs to module {}", file, module_name),
                    });
                }
            }

            // Connect config files to architecture
            if category == "config_files" {
                relationships.push(Relationship {
                    source: format!("file:{}", file),
                    target: "layer:config".to_string(),
                    rel_type: "configures".to_string(),
                    weight: 1.0,
                    metadata: format!("Config file: {}", file),
                });
            }
        }
    }

    // ── 5. Score ↔ Score relationships (correlations) ───────
    let scores = &intelligence.maturity;
    if scores.testing > 0 && scores.documentation > 0 {
        relationships.push(Relationship {
            source: "maturity:testing".to_string(),
            target: "maturity:documentation".to_string(),
            rel_type: "correlates".to_string(),
            weight: 0.5,
            metadata: "Testing and documentation scores correlate".to_string(),
        });
    }
    if scores.security > 0 && scores.overall > 0 {
        relationships.push(Relationship {
            source: "maturity:security".to_string(),
            target: "maturity:overall".to_string(),
            rel_type: "contributes_to".to_string(),
            weight: 0.8,
            metadata: "Security contributes to overall maturity".to_string(),
        });
    }

    // ── 6. Dependency ↔ Module relationships ────────────────
    for module in modules {
        for import in &module.imports {
            for dep in &intelligence.dependencies.dependencies {
                if import.to_lowercase() == dep.name.to_lowercase() {
                    relationships.push(Relationship {
                        source: format!("module:{}", module.name),
                        target: format!("dep:{}", dep.name),
                        rel_type: "uses".to_string(),
                        weight: 1.0,
                        metadata: format!("Module {} uses dependency {}", module.name, dep.name),
                    });
                }
            }
        }
    }

    // Persist all relationships to the database
    for rel in &relationships {
        let edge_id = format!(
            "rel:{}->{}",
            rel.source.replace(':', "_"),
            rel.target.replace(':', "_")
        );
        db.insert_module_relationship(
            &edge_id,
            &rel.source,
            &rel.target,
            &rel.rel_type,
            rel.weight,
            &rel.metadata,
        )?;
    }

    Ok(relationships)
}

/// Extracts a module name from a file path.
fn extract_module_from_path(path: &str) -> Option<String> {
    // e.g., "src/scanner/mod.rs" -> "scanner"
    // e.g., "src/main.rs" -> "main"
    let path = path.replace('\\', "/");
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() >= 2 && parts[0] == "src" {
        let name = parts[1].trim_end_matches(".rs");
        if name == "mod" && parts.len() >= 3 {
            // src/scanner/mod.rs -> scanner
            Some(parts[1].to_string())
        } else if name != "mod" {
            Some(name.to_string())
        } else {
            None
        }
    } else {
        None
    }
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
