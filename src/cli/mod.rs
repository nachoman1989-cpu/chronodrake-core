//! CLI layer for ChronoDrake.
//!
//! This module provides a modular command dispatcher architecture that
//! separates CLI concerns (argument parsing, command dispatch, shared context)
//! from business logic.
//!
//! # Structure
//!
//! - [`args`] — Argument parsing (subcommand + flags + positional args)
//! - [`context`] — Shared [`CliContext`] (root path, config, initialized logger)
//! - [`dispatcher`] — Command registry and dispatch
//! - [`commands`] — Individual command implementations (one per file)
//!
//! # Future
//!
//! Once all commands are migrated, `main.rs` will shrink to:
//!
//! ```ignore
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let args = cli::args::ParsedArgs::from_env();
//!     let ctx = cli::context::CliContext::new()?;
//!     let dispatcher = cli::dispatcher::Dispatcher::new();
//!     dispatcher.dispatch(&ctx, &args).await
//! }
//! ```

pub mod args;
pub mod commands;
pub mod context;
pub mod dispatcher;

pub use args::ParsedArgs;
pub use commands::CliCommand;
pub use context::CliContext;
pub use dispatcher::Dispatcher;
