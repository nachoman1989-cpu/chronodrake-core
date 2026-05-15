use anyhow::Result;

use super::args::ParsedArgs;
use super::commands::CliCommand;
use super::context::CliContext;

/// Registers and dispatches CLI commands.
///
/// The dispatcher maintains a list of registered commands and routes incoming
/// arguments to the correct handler. It also provides built-in `help` and
/// unknown-command handling.
///
/// # Example
///
/// ```ignore
/// let dispatcher = Dispatcher::new();
/// dispatcher.register(Box::new(ScanCommand));
/// dispatcher.register(Box::new(InitCommand));
///
/// let args = ParsedArgs::from_env();
/// let ctx = CliContext::new()?;
/// dispatcher.dispatch(&ctx, &args).await?;
/// ```
pub struct Dispatcher {
    commands: Vec<Box<dyn CliCommand>>,
}

impl Dispatcher {
    /// Creates a new empty dispatcher.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Registers a command with the dispatcher.
    ///
    /// Commands are matched by name (and aliases) during dispatch.
    /// If a command with the same name is already registered, it is replaced.
    pub fn register(&mut self, cmd: Box<dyn CliCommand>) {
        // Remove any existing command with the same name
        self.commands.retain(|c| c.name() != cmd.name());
        self.commands.push(cmd);
    }

    /// Finds a registered command by name or alias.
    pub fn find_command(&self, name: &str) -> Option<&Box<dyn CliCommand>> {
        self.commands.iter().find(|cmd| {
            cmd.name() == name || cmd.aliases().contains(&name)
        })
    }

    /// Dispatches the parsed arguments to the appropriate command handler.
    ///
    /// Returns an error if the command is unknown or if execution fails.
    ///
    /// # Built-in behavior
    ///
    /// - If no command is provided, prints usage and returns `Ok`.
    /// - If the command is `"help"`, `"--help"`, or `"-h"`, prints usage.
    /// - If the command is unknown, prints an error and usage.
    pub async fn dispatch(&self, ctx: &CliContext, args: &ParsedArgs) -> Result<()> {
        // Handle help / no-command cases
        if args.is_empty()
            || args.command == "help"
            || args.command == "--help"
            || args.command == "-h"
        {
            self.print_usage();
            return Ok(());
        }

        // Find the command
        match self.find_command(&args.command) {
            Some(cmd) => {
                cmd.execute(ctx, &args.positional).await?;
                Ok(())
            }
            None => {
                // TODO: Use proper error reporting
                // For now, match the existing main.rs behavior
                crate::utils::Logger::failure(&format!("Unknown command: {}", args.command));
                self.print_usage();
                Ok(())
            }
        }
    }

    /// Prints usage information for all registered commands.
    pub fn print_usage(&self) {
        crate::utils::Logger::section("ChronoDrake Core v0.7");
        crate::utils::Logger::raw("AI Continuity Operating System — Project Registry & Workspace Loader");
        crate::utils::Logger::divider();
        crate::utils::Logger::raw("");
        crate::utils::Logger::raw("  USAGE:");
        crate::utils::Logger::raw("    chronodrake <command> [args]");
        crate::utils::Logger::raw("");

        // Find the longest command name for alignment
        let max_len = self
            .commands
            .iter()
            .map(|c| c.name().len())
            .max()
            .unwrap_or(10);

        for cmd in &self.commands {
            let padding = " ".repeat(max_len - cmd.name().len() + 2);
            crate::utils::Logger::raw(&format!(
                "    chronodrake {}{}{}",
                cmd.name(),
                padding,
                cmd.description()
            ));
        }

        crate::utils::Logger::raw(&format!(
            "    chronodrake {}  {}",
            "help".to_string() + &" ".repeat(max_len - 4 + 2),
            "Show this help message"
        ));
        crate::utils::Logger::raw("");
        crate::utils::Logger::divider();
    }

    /// Returns the number of registered commands.
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    /// Returns an iterator over all registered command names.
    pub fn command_names(&self) -> Vec<&str> {
        self.commands.iter().map(|c| c.name()).collect()
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct TestCommand;

    #[async_trait]
    impl CliCommand for TestCommand {
        fn name(&self) -> &'static str {
            "test"
        }
        fn description(&self) -> &'static str {
            "A test command"
        }
        async fn execute(&self, _ctx: &CliContext, _args: &[String]) -> Result<()> {
            Ok(())
        }
    }

    struct HelpAliasCommand;

    #[async_trait]
    impl CliCommand for HelpAliasCommand {
        fn name(&self) -> &'static str {
            "help"
        }
        fn aliases(&self) -> &[&'static str] {
            &["--help", "-h"]
        }
        fn description(&self) -> &'static str {
            "Show help"
        }
        async fn execute(&self, _ctx: &CliContext, _args: &[String]) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_dispatcher_new_is_empty() {
        let d = Dispatcher::new();
        assert_eq!(d.command_count(), 0);
    }

    #[test]
    fn test_register_command() {
        let mut d = Dispatcher::new();
        d.register(Box::new(TestCommand));
        assert_eq!(d.command_count(), 1);
        assert_eq!(d.command_names(), vec!["test"]);
    }

    #[test]
    fn test_find_command_by_name() {
        let mut d = Dispatcher::new();
        d.register(Box::new(TestCommand));
        assert!(d.find_command("test").is_some());
        assert!(d.find_command("unknown").is_none());
    }

    #[test]
    fn test_find_command_by_alias() {
        let mut d = Dispatcher::new();
        d.register(Box::new(HelpAliasCommand));
        assert!(d.find_command("help").is_some());
        assert!(d.find_command("--help").is_some());
        assert!(d.find_command("-h").is_some());
    }

    #[test]
    fn test_register_replaces_existing() {
        let mut d = Dispatcher::new();
        d.register(Box::new(TestCommand));
        d.register(Box::new(TestCommand)); // same name, should replace
        assert_eq!(d.command_count(), 1);
    }
}
