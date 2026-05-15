//! CLI command trait and command registry.
//!
//! Each CLI command is implemented as a struct that satisfies the [`CliCommand`]
//! trait. Commands are registered with the [`Dispatcher`](super::dispatcher::Dispatcher)
//! at startup and dispatched by name.
//!
//! # Adding a New Command
//!
//! 1. Create a new file in this module (e.g., `my_command.rs`).
//! 2. Define a struct (e.g., `pub struct MyCommand;`).
//! 3. Implement [`CliCommand`] for it.
//! 4. Register it in the [`Dispatcher`](super::dispatcher::Dispatcher).
//!
//! # Migration Status
//!
//! TODO: Commands are still in `main.rs` as `cmd_*` functions.
//!       They will be migrated here one by one in Phase 2.

use async_trait::async_trait;
use anyhow::Result;

use super::context::CliContext;

/// A single CLI command.
///
/// Each command struct implements this trait to provide its name, description,
/// and execution logic. The [`Dispatcher`](super::dispatcher::Dispatcher) uses
/// these to route user input to the correct handler.
///
/// # Example
///
/// ```ignore
/// pub struct ScanCommand;
///
/// #[async_trait]
/// impl CliCommand for ScanCommand {
///     fn name(&self) -> &'static str { "scan" }
///     fn description(&self) -> &'static str { "Scan the current directory" }
///
///     async fn execute(&self, ctx: &CliContext, args: &[String]) -> Result<()> {
///         // Business logic here (no config/logger init needed)
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait CliCommand: Send + Sync {
    /// The command name used on the CLI (e.g., `"scan"`, `"register"`).
    fn name(&self) -> &'static str;

    /// Optional aliases (e.g., `["--help", "-h"]` for the help command).
    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    /// A short description shown in the help text.
    fn description(&self) -> &'static str;

    /// Execute the command with the given shared context and remaining
    /// positional arguments (everything after the subcommand name).
    async fn execute(&self, ctx: &CliContext, args: &[String]) -> Result<()>;
}

// ---------------------------------------------------------------------------
// TODO: Phase 2 — Migrate commands from main.rs
// ---------------------------------------------------------------------------
//
// Each command will be added as a separate file in this directory:
//
// - scan.rs        → ScanCommand
// - init.rs        → InitCommand
// - context.rs     → ContextCommand
// - handoff.rs     → HandoffCommand
// - doctor.rs      → DoctorCommand
// - register.rs    → RegisterCommand
// - projects.rs    → ProjectsCommand
// - load.rs        → LoadCommand
// - current.rs     → CurrentCommand
//
// After migration, this module will re-export them and provide a list of all
// commands for the dispatcher:
//
// ```rust
// pub use scan::ScanCommand;
// pub use init::InitCommand;
// // ... etc.
//
// pub fn all_commands() -> Vec<Box<dyn CliCommand>> {
//     vec![
//         Box::new(ScanCommand),
//         Box::new(InitCommand),
//         // ...
//     ]
// }
// ```
