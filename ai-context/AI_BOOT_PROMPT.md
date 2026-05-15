# AI_BOOT_PROMPT.md

> **ChronoDrake AI Context — AI Boot Prompt**
> This file is optimized for ANY AI model to reconstruct full project context.
> Read this file first to understand the project completely.

---

## SYSTEM INSTRUCTION

You are an AI assistant continuing work on this project. Read this entire file to reconstruct full context. Then read the other files in the `/ai-context/` directory for detailed information.

---

## 1. PROJECT OVERVIEW

- **Project:** chronodrake-core
- **Type:** Security Tool (confidence: 85%)
- **Summary:** Detected as **Security Tool** (confidence: 85%). Evidence: Cryptography/Security crate detected.
- **Last Scan:** 2026-05-15T00:44:35.087092200+00:00
- **Root Path:** `E:\PROYECTOS IA\CLAUDE-espacio-de-trabajo\ChronoDrake\chronodrake-core`

## 2. PROJECT GOALS

This project is scanned and tracked by **ChronoDrake**, an AI Continuity Operating System.

### ChronoDrake Goals

- Provide universal AI development continuity
- Enable seamless handoff between any AI models
- Track project evolution, maturity, and technical debt over time
- Generate deterministic, offline-first context for AI assistants
- Maintain local-first, no-cloud, no-telemetry architecture

## 3. CURRENT PROGRESS

- **Overall Maturity:** 30/100
- **Security Rating:** Safe
- **Security Issues:** 1
- **Dependencies Tracked:** 18
- **Modules Detected:** 45
- **Relationships Mapped:** 60
- **Evolution Changes:** 0

## 4. CODING STANDARDS

- **Language:** Rust (stable)
- **Error Handling:** `thiserror` for library errors, `anyhow` for application-level
- **Serialization:** `serde` with `Serialize`/`Deserialize` derives
- **Concurrency:** `tokio` async runtime
- **Database:** `rusqlite` with schema versioning and migrations
- **Logging:** `tracing` + `tracing-subscriber` with env-filter
- **Testing:** Unit tests in-module, integration tests in `/tests/`
- **Determinism:** All outputs must be reproducible — no randomness, no external APIs
- **Offline-first:** No cloud, no telemetry, no internet required

## 5. ARCHITECTURE

### Module Structure

```
src/
├── main.rs              # CLI entry point
├── config/              # Configuration system (chronodrake.toml)
├── errors/              # Typed error system (thiserror)
├── models/              # Data models (ProjectScan, DetectedTech)
├── scanner/             # Technology detection scanners
│   ├── filesystem.rs    # Filesystem scanning
│   ├── incremental.rs   # Incremental scan engine (SHA-256)
│   ├── rust.rs          # Rust/Cargo detection
│   ├── node.rs          # Node.js detection
│   └── tauri.rs         # Tauri detection
├── intelligence/        # Analysis pipeline
│   ├── project_type.rs  # Project type detection
│   ├── architecture.rs  # Architecture analysis
│   ├── maturity.rs      # Maturity scoring
│   ├── security.rs      # Security scanning
│   ├── dependencies.rs  # Dependency analysis
│   └── changes.rs       # Change detection
├── persistence/         # SQLite persistence layer
│   ├── sqlite.rs        # Database operations
│   ├── snapshots.rs     # Snapshot management
│   ├── timeline.rs      # Timeline events
│   ├── decisions.rs     # Decision tracking
│   └── evolution.rs     # Score evolution
├── knowledge/           # Knowledge graph engine
│   ├── modules.rs       # Module detection
│   ├── graph.rs         # Graph building
│   ├── relationships.rs # Relationship detection
│   ├── dependencies.rs  # Dependency classification
│   ├── impacts.rs       # Impact analysis
│   └── queries.rs       # Graph queries
├── context/             # AI Context Engine (v0.6)
│   └── mod.rs           # 7 context file generators
├── doctor/              # Doctor engine (v0.6)
│   └── mod.rs           # System detection
├── cache/               # Cache layer
│   └── mod.rs           # TTL cache with SHA-256 integrity
├── migrations/          # Schema migrations
│   └── mod.rs           # Migration engine
├── memory/              # Report generators
│   ├── generator.rs     # PROJECT_STATUS.md, CHANGES.md
│   └── mod.rs
├── handoff/             # AI handoff generator
│   ├── generator.rs     # AI_HANDOFF.md
│   └── mod.rs
├── utils/               # Utilities
│   └── logger.rs        # Tracing-based logging
└── models/              # Data models
    └── project.rs       # ProjectScan, DetectedTech
```

## 6. NEXT OBJECTIVE

The current objective is **ChronoDrake v0.6 — AI Context Engine**.

### What to do

1. Read all files in `/ai-context/` to understand full project context
2. Check `CURRENT_STATE.md` for the exact implementation status
3. Follow `NEXT_STEPS.md` for the recommended roadmap
4. Review `DECISIONS.md` for architectural decisions and tradeoffs
5. Use `PROJECT_STATUS.md` for detailed scan results
6. Use `AI_HANDOFF.md` for AI-specific handoff instructions

### Constraints

- All outputs must be deterministic and reproducible
- No AI APIs, no embeddings, no cloud, no telemetry
- Must work offline with zero external dependencies
- Must preserve backward compatibility with v0.5

---
*This file was automatically generated by ChronoDrake Core v0.6*
*ChronoDrake — AI Continuity Operating System*
