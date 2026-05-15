# ChronoDrake v0.7 — CLI Architecture Refactor Plan

> **Status:** Analysis Phase  
> **Target:** ChronoDrake Core v0.7  
> **Goal:** Transition from prototype to professional offline-first AI continuity platform

---

## Table of Contents

1. [Architecture Analysis](#1-architecture-analysis)
2. [Technical Debt Analysis](#2-technical-debt-analysis)
3. [Proposed Folder Structure](#3-proposed-folder-structure)
4. [Refactor Roadmap](#4-refactor-roadmap)
5. [Implementation Priority Order](#5-implementation-priority-order)
6. [Risk Assessment](#6-risk-assessment)

---

## 1. Architecture Analysis

### 1.1 Current Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         main.rs (~639 lines)                        │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    CLI Entry Point                             │   │
│  │  env::args().collect() → match args[1] → cmd_*()              │   │
│  └──────────────────────────────────────────────────────────────┘   │
│         │            │            │           │                     │
│         ▼            ▼            ▼           ▼                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐              │
│  │ cmd_scan │ │cmd_init  │ │cmd_ctx   │ │cmd_hand  │  ... 9 cmds  │
│  │ 277 lines│ │ 22 lines │ │ 40 lines │ │ 24 lines │              │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘              │
│       │            │            │            │                     │
│       ▼            ▼            ▼            ▼                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              Each cmd_* function independently:               │   │
│  │  1. Gets current_dir()                                        │   │
│  │  2. Loads config (ChronoDrakeConfig::load)                    │   │
│  │  3. Initializes Logger (Logger::init)                         │   │
│  │  4. Runs business logic                                       │   │
│  │  5. Prints results via Logger                                 │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    Module Imports (15 mods)                    │   │
│  │  scanner  memory  handoff  models  utils  intelligence        │   │
│  │  persistence  knowledge  config  errors  cache  migrations    │   │
│  │  context  doctor  workspace                                   │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.2 Current Architecture Characteristics

| Aspect | Current State | Assessment |
|--------|--------------|------------|
| **CLI Structure** | Monolithic `match` in `main.rs` | ❌ Tight coupling |
| **Command Dispatch** | Manual string matching | ❌ Not extensible |
| **Argument Parsing** | Manual `args.get(2)` | ❌ Fragile |
| **Config Loading** | Duplicated in each `cmd_*` | ❌ WET (Write Every Time) |
| **Logger Init** | Duplicated in each `cmd_*` | ❌ WET |
| **Error Handling** | `anyhow::Result` throughout | ⚠️ Inconsistent |
| **Error Types** | `thiserror` enums defined but unused | ❌ Dead code |
| **Module Coupling** | Direct function calls across modules | ⚠️ Tight |
| **Testability** | Commands not testable in isolation | ❌ Poor |
| **Extensibility** | Adding a command requires editing main.rs | ❌ Poor |

### 1.3 Command Duplication Pattern

Every `cmd_*` function repeats this exact pattern:

```rust
async fn cmd_xxx() -> Result<()> {
    // 1. Get current directory
    let current_dir = env::current_dir()?;
    let root_path = current_dir.to_string_lossy().to_string();

    // 2. Load config
    let config = ChronoDrakeConfig::load(&root_path)?;

    // 3. Initialize logger
    Logger::init(&config.log_level);

    // 4. Print section header
    Logger::section("ChronoDrake Xxx v0.7");
    Logger::kv("Target", &root_path);
    Logger::divider();

    // 5. Business logic
    // ... (varies per command)

    // 6. Print completion
    Logger::divider();
    Logger::section("Xxx Complete");
    Ok(())
}
```

**Commands affected:** `scan`, `init`, `context`, `handoff`, `doctor`, `register`, `projects`, `load`, `current` — **9 out of 9 commands**.

### 1.4 Module Dependency Graph (Current)

```
main.rs
  ├── scanner ──────────► rust, node, tauri, filesystem, incremental
  ├── intelligence ─────► architecture, security, maturity, dependencies, changes, project_type
  ├── persistence ──────► sqlite, evolution, decisions, snapshots, timeline
  ├── knowledge ────────► modules, graph, dependencies, relationships, impacts, queries
  ├── context ──────────► (generates 7 AI context files)
  ├── memory ───────────► generator
  ├── handoff ──────────► generator
  ├── config
  ├── errors ───────────► (6 error enums, 1 unified enum)
  ├── cache
  ├── migrations
  ├── doctor
  ├── workspace ────────► registry, loader, models
  ├── models ───────────► project
  └── utils ────────────► logger
```

---

## 2. Technical Debt Analysis

### 2.1 Dead Code Inventory

| Category | Location | Impact |
|----------|----------|--------|
| **Unused error types** | `errors/mod.rs` — `ScanError`, `PersistenceError`, `GraphError`, `ParserError`, `ConfigError`, `CacheError` | 6 enum types defined but never directly used (all commands use `anyhow::Result`) |
| **Unused unified error** | `errors/mod.rs` — `ChronoDrakeError` | Wraps all 6 error types but never used as return type |
| **Unused DB methods** | `persistence/sqlite.rs` — `insert_module_relationship`, `get_relationships_by_source`, `insert_dependency_relationship`, `get_critical_dependencies`, `insert_impact_event`, `get_impacts_by_target`, `insert_knowledge_node`, `get_nodes_by_type`, `clear_knowledge_graph`, `insert_decision`, `insert_timeline_event`, `insert_evolution`, `get_evolution_by_metric` | ~13 methods defined but never called |
| **Unused cache types** | `cache/mod.rs` — `DependencyCache`, `ModuleGraphCache`, `FileHashCache` | Structs defined but never instantiated |
| **Unused logger methods** | `utils/logger.rs` — `span()` | Defined but never called |
| **Unused scanner functions** | `scanner/incremental.rs` — `hash_file`, `get_modified_time`, `scan_file_states`, `detect_file_changes` | Module exists but `incremental_scan` config flag is never checked |

**Total estimated dead code:** ~800+ lines across 6+ files.

### 2.2 Architectural Debt Items

| # | Debt Item | Severity | Effort |
|---|-----------|----------|--------|
| D1 | Monolithic `main.rs` with all command logic | 🔴 High | 3 days |
| D2 | Duplicated config/logger init in every command | 🔴 High | 1 day |
| D3 | No command abstraction (trait/struct) | 🔴 High | 2 days |
| D4 | Manual argument parsing (`args.get(2)`) | 🟡 Medium | 1 day |
| D5 | `anyhow::Result` used everywhere (loses type info) | 🟡 Medium | 2 days |
| D6 | Error enums defined but unused | 🟡 Medium | 1 day |
| D7 | No centralized error reporting | 🟡 Medium | 1 day |
| D8 | `cmd_scan` is 277 lines (43% of main.rs) | 🟡 Medium | 2 days |
| D9 | No integration tests for CLI commands | 🟡 Medium | 2 days |
| D10 | `incremental_scan` config flag has no effect | 🔵 Low | 0.5 day |
| D11 | `colored` crate imported but never used directly | 🔵 Low | 0.25 day |
| D12 | No `--version` flag | 🔵 Low | 0.25 day |

### 2.3 Code Smells

1. **Stringly-typed commands**: `args[1].as_str()` match — no type safety for command names
2. **Magic number indexing**: `args.get(2)` for subcommand arguments — breaks if argument order changes
3. **Mixed concerns**: `cmd_scan` handles config loading, logger init, migrations, scanning, intelligence, persistence, knowledge graph, report generation, and AI context generation — all in one function
4. **Hardcoded version strings**: `"v0.7"` appears in 9 places across `main.rs`
5. **No exit codes**: All commands return `Ok(())` even on failure (though `anyhow` propagates errors)

---

## 3. Proposed Folder Structure

### 3.1 Target Architecture

```
src/
├── main.rs                    # ~30 lines: bootstrap only
│
├── cli/                       # NEW: CLI layer
│   ├── mod.rs                 # Re-exports, CLI module root
│   ├── dispatcher.rs          # Command registry + dispatch
│   ├── args.rs                # Argument parsing (subcommand + flags)
│   ├── context.rs             # CliContext: shared config, logger, root_path
│   └── commands/              # One file per command
│       ├── mod.rs             # Re-exports all commands
│       ├── scan.rs            # cmd_scan logic (extracted from main.rs)
│       ├── init.rs            # cmd_init logic
│       ├── context.rs         # cmd_context logic
│       ├── handoff.rs         # cmd_handoff logic
│       ├── doctor.rs          # cmd_doctor logic
│       ├── register.rs        # cmd_register logic
│       ├── projects.rs        # cmd_projects logic
│       ├── load.rs            # cmd_load logic
│       └── current.rs         # cmd_current logic
│
├── scanner/                   # UNCHANGED
├── intelligence/              # UNCHANGED
├── persistence/               # UNCHANGED
├── knowledge/                 # UNCHANGED
├── context/                   # UNCHANGED
├── memory/                    # UNCHANGED
├── handoff/                   # UNCHANGED
├── workspace/                 # UNCHANGED
├── config/                    # UNCHANGED
├── cache/                     # UNCHANGED
├── migrations/                # UNCHANGED
├── doctor/                    # UNCHANGED
├── models/                    # UNCHANGED
├── utils/                     # UNCHANGED
└── errors/                    # REFACTOR: remove unused enums, keep what's needed
```

### 3.2 Key Design: Command Trait

```rust
// src/cli/commands/mod.rs

use async_trait::async_trait;  // or use std::future::Future
use anyhow::Result;
use crate::cli::context::CliContext;

/// A single CLI command.
#[async_trait]
pub trait CliCommand {
    /// The command name (e.g., "scan", "register").
    fn name(&self) -> &'static str;

    /// Optional aliases (e.g., ["--help", "-h"] for help).
    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    /// A short description for the help text.
    fn description(&self) -> &'static str;

    /// Execute the command with the given context and arguments.
    async fn execute(&self, ctx: &CliContext, args: &[String]) -> Result<()>;
}
```

### 3.3 Key Design: CliContext

```rust
// src/cli/context.rs

use anyhow::Result;
use crate::config::ChronoDrakeConfig;
use crate::utils::Logger;

/// Shared context for all CLI commands.
pub struct CliContext {
    /// The absolute path to the current working directory.
    pub root_path: String,
    /// The loaded configuration.
    pub config: ChronoDrakeConfig,
}

impl CliContext {
    /// Creates a new CliContext by detecting the current directory,
    /// loading config, and initializing the logger.
    pub fn new() -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        let root_path = current_dir.to_string_lossy().to_string();
        let config = ChronoDrakeConfig::load(&root_path)?;
        Logger::init(&config.log_level);
        Ok(Self { root_path, config })
    }
}
```

### 3.4 Key Design: Command Dispatcher

```rust
// src/cli/dispatcher.rs

use anyhow::Result;
use super::commands::{CliCommand, ALL_COMMANDS};
use super::context::CliContext;
use super::args::ParsedArgs;

/// Registers and dispatches CLI commands.
pub struct Dispatcher {
    commands: Vec<Box<dyn CliCommand>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    pub fn register(&mut self, cmd: Box<dyn CliCommand>) {
        self.commands.push(cmd);
    }

    pub fn dispatch(&self, ctx: &CliContext, args: &ParsedArgs) -> Result<()> {
        // Find matching command
        // Execute with context and remaining args
    }

    pub fn print_help(&self) {
        // Iterate commands and print usage
    }
}
```

### 3.5 Key Design: Argument Parser

```rust
// src/cli/args.rs

/// Parsed CLI arguments.
pub struct ParsedArgs {
    /// The subcommand name (e.g., "scan", "load").
    pub command: String,
    /// Additional positional arguments (e.g., project name for "load").
    pub positional: Vec<String>,
    /// Named flags (e.g., --verbose, --output).
    pub flags: Vec<String>,
}

impl ParsedArgs {
    pub fn from_env() -> Self {
        let raw: Vec<String> = std::env::args().collect();
        // Parse raw args into structured form
        // ...
    }
}
```

---

## 4. Refactor Roadmap

### Phase 1: Foundation — CLI Module Structure (Days 1-2)

**Goal:** Create the CLI module skeleton without changing any behavior.

| Step | Task | Files | Tests |
|------|------|-------|-------|
| 1.1 | Create `src/cli/mod.rs` | New | — |
| 1.2 | Create `src/cli/context.rs` with `CliContext` | New | Unit test |
| 1.3 | Create `src/cli/args.rs` with `ParsedArgs` | New | Unit test |
| 1.4 | Create `src/cli/dispatcher.rs` with `Dispatcher` | New | Unit test |
| 1.5 | Create `src/cli/commands/mod.rs` with `CliCommand` trait | New | — |
| 1.6 | Create empty command files (one per command) | 9 new files | — |
| 1.7 | Add `mod cli` to `main.rs` | Modified | — |

**Verification:** `cargo build` succeeds. No behavior changes.

### Phase 2: Command Migration — Extract Logic (Days 3-5)

**Goal:** Move each `cmd_*` function from `main.rs` into its own command file.

| Step | Task | Details |
|------|------|---------|
| 2.1 | Migrate `cmd_init` → `cli/commands/init.rs` | Simplest command first |
| 2.2 | Migrate `cmd_handoff` → `cli/commands/handoff.rs` | Small command |
| 2.3 | Migrate `cmd_doctor` → `cli/commands/doctor.rs` | Small command |
| 2.4 | Migrate `cmd_register` → `cli/commands/register.rs` | Small command |
| 2.5 | Migrate `cmd_projects` → `cli/commands/projects.rs` | Small command |
| 2.6 | Migrate `cmd_current` → `cli/commands/current.rs` | Small command |
| 2.7 | Migrate `cmd_context` → `cli/commands/context.rs` | Medium command |
| 2.8 | Migrate `cmd_load` → `cli/commands/load.rs` | Medium command |
| 2.9 | Migrate `cmd_scan` → `cli/commands/scan.rs` | **Largest command last** |

**Pattern for each migration:**

```rust
// Before (in main.rs):
async fn cmd_xxx() -> Result<()> {
    let current_dir = env::current_dir()?;
    let root_path = current_dir.to_string_lossy().to_string();
    let config = ChronoDrakeConfig::load(&root_path)?;
    Logger::init(&config.log_level);
    // ... business logic ...
}

// After (in cli/commands/xxx.rs):
pub struct XxxCommand;

#[async_trait]
impl CliCommand for XxxCommand {
    fn name(&self) -> &'static str { "xxx" }
    fn description(&self) -> &'static str { "Does something" }

    async fn execute(&self, ctx: &CliContext, args: &[String]) -> Result<()> {
        // ... business logic only (no config/logger init) ...
    }
}
```

**Verification:** After each migration, `cargo build` + manual test of the command.

### Phase 3: Dispatcher Integration (Day 6)

**Goal:** Wire the dispatcher into `main.rs` and remove the old match block.

```rust
// main.rs after Phase 3:
mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::args::ParsedArgs::from_env();
    let ctx = cli::context::CliContext::new()?;
    let dispatcher = cli::dispatcher::Dispatcher::new();

    if args.command.is_empty() || args.command == "help" {
        dispatcher.print_help();
        return Ok(());
    }

    dispatcher.dispatch(&ctx, &args).await
}
```

**Verification:** All 9 commands work identically to before. `cargo test` passes.

### Phase 4: Error Handling Overhaul (Days 7-8)

**Goal:** Replace `anyhow::Result` with typed errors where appropriate, remove dead error code.

| Step | Task |
|------|------|
| 4.1 | Audit which `thiserror` enums are actually used |
| 4.2 | Remove unused error variants (ScanError, GraphError, ParserError, ConfigError, CacheError if truly unused) |
| 4.3 | Add `CliError` enum for CLI-specific errors (unknown command, missing argument, etc.) |
| 4.4 | Add `--json` flag support for machine-readable output |
| 4.5 | Implement `Display` for `CliError` with user-friendly messages |
| 4.6 | Add non-zero exit codes on failure |

**Verification:** `cargo build` with zero warnings. Error messages are consistent.

### Phase 5: Cleanup & Testing (Days 9-10)

**Goal:** Remove dead code, add integration tests, final polish.

| Step | Task |
|------|------|
| 5.1 | Remove unused `colored` dependency (if confirmed unused) |
| 5.2 | Remove dead DB methods from `persistence/sqlite.rs` |
| 5.3 | Remove unused cache structs from `cache/mod.rs` |
| 5.4 | Add integration tests for each CLI command |
| 5.5 | Add `--version` flag |
| 5.6 | Centralize version string in a constant |
| 5.7 | Update `ARCHITECTURE.md` to reflect new structure |
| 5.8 | Update `ROADMAP.md` |

**Verification:** `cargo test` passes all tests. `cargo build` has zero warnings.

---

## 5. Implementation Priority Order

### Priority Matrix

| Priority | Item | Effort | Impact | Risk |
|----------|------|--------|--------|------|
| **P0** | CLI module structure (Phase 1) | 2 days | 🔴 High | Low |
| **P0** | Command migration — small commands (Phase 2.1-2.6) | 2 days | 🔴 High | Low |
| **P1** | Command migration — large commands (Phase 2.7-2.9) | 2 days | 🔴 High | Medium |
| **P1** | Dispatcher integration (Phase 3) | 1 day | 🔴 High | Medium |
| **P2** | Error handling overhaul (Phase 4) | 2 days | 🟡 Medium | Medium |
| **P3** | Dead code removal (Phase 5.1-5.3) | 1 day | 🟡 Medium | Low |
| **P3** | Integration tests (Phase 5.4) | 2 days | 🟡 Medium | Low |
| **P4** | Polish (Phase 5.5-5.8) | 1 day | 🔵 Low | Low |

### Recommended Order

```
Week 1:
  Day 1-2:  Phase 1 (CLI module skeleton)
  Day 3-4:  Phase 2.1-2.6 (migrate 6 small commands)
  Day 5:    Phase 2.7-2.9 (migrate 3 large commands)

Week 2:
  Day 6:    Phase 3 (dispatcher integration)
  Day 7-8:  Phase 4 (error handling)
  Day 9:    Phase 5.1-5.4 (cleanup + tests)
  Day 10:   Phase 5.5-5.8 (polish + docs)
```

### Dependency Graph

```
Phase 1 ──► Phase 2 ──► Phase 3 ──► Phase 4 ──► Phase 5
  (foundation)  (migrate)    (wire)      (errors)    (cleanup)
```

Each phase depends on the previous. No parallelization possible within the CLI refactor itself.

---

## 6. Risk Assessment

### Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| **Breaking existing commands** | Medium | 🔴 High | Migrate one command at a time, test each |
| **Async trait complexity** | Low | 🟡 Medium | Use `async-trait` crate or boxed futures |
| **Borrow checker issues** | Medium | 🟡 Medium | `CliContext` owns all data, commands borrow |
| **Dead code removal breaks something** | Low | 🟡 Medium | Remove only after confirming unused via `cargo deadlinks` or manual audit |
| **Scope creep** | High | 🟡 Medium | Strictly follow the plan, defer non-essential changes |

### Guardrails

1. **No behavior changes** — The refactor is structural only. Business logic stays identical.
2. **One command at a time** — Each migration is independently verifiable.
3. **Backward compatibility** — All existing CLI invocations must produce identical output.
4. **Offline-first preserved** — No new dependencies on cloud services, AI APIs, or telemetry.
5. **Deterministic outputs** — All generated files remain reproducible.

---

## Appendix A: Current File Sizes

| File | Lines | % of main.rs |
|------|-------|-------------|
| `src/main.rs` | 639 | 100% |
| `src/main.rs` (cmd_scan only) | 277 | 43% |
| `src/main.rs` (cmd_* functions) | ~560 | 88% |
| `src/main.rs` (match + print_usage) | ~79 | 12% |

## Appendix B: Command Complexity

| Command | Lines | Config Loads | Logger Inits | Dependencies |
|---------|-------|-------------|--------------|--------------|
| `scan` | 277 | Yes | Yes | scanner, intelligence, persistence, knowledge, memory, handoff, context, config, cache, migrations |
| `context` | 40 | Yes | Yes | scanner, intelligence, persistence, knowledge, context, config |
| `handoff` | 24 | Yes | Yes | scanner, handoff, config |
| `doctor` | 46 | Yes | Yes | doctor, config |
| `init` | 22 | No | No | config |
| `register` | 23 | Yes | Yes | workspace::registry, config |
| `projects` | 42 | Yes | Yes | workspace::registry, config |
| `load` | 32 | Yes | Yes | workspace::loader, config |
| `current` | 29 | Yes | Yes | workspace::registry, config |

## Appendix C: Dead Code Detail

### Unused Error Enums (`src/errors/mod.rs`)

```rust
// These enums are defined but NEVER directly constructed in the codebase.
// All functions return anyhow::Result, so these typed errors are dead code.
pub enum ScanError { ... }        // 7 variants, 22 lines
pub enum PersistenceError { ... } // 6 variants, 19 lines
pub enum GraphError { ... }       // 6 variants, 16 lines
pub enum ParserError { ... }      // 6 variants, 17 lines
pub enum ConfigError { ... }      // 5 variants, 16 lines
pub enum CacheError { ... }       // 5 variants, 16 lines
pub enum ChronoDrakeError { ... } // 8 variants, 11 lines
```

**Total: 7 enums, 43 variants, ~117 lines of dead code.**

### Unused DB Methods (`src/persistence/sqlite.rs`)

Methods defined in `impl Database` but never called from anywhere:
- `insert_module_relationship`
- `get_relationships_by_source`
- `insert_dependency_relationship`
- `get_critical_dependencies`
- `insert_impact_event`
- `get_impacts_by_target`
- `insert_knowledge_node`
- `get_nodes_by_type`
- `clear_knowledge_graph`
- `insert_decision`
- `insert_timeline_event`
- `insert_evolution`
- `get_evolution_by_metric`

**Total: ~13 methods, ~200 lines of dead code.**

---

*End of Architecture Refactor Plan*
