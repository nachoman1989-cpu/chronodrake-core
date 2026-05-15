use std::env;

/// Parsed CLI arguments.
///
/// Provides structured access to the subcommand, positional arguments, and
/// flags, replacing the current pattern of manual `args.get(1)` / `args.get(2)`
/// indexing in `main.rs`.
///
/// # Example
///
/// ```ignore
/// let args = ParsedArgs::from_env();
/// match args.command.as_str() {
///     "scan" => { /* ... */ }
///     "load" => {
///         let project_name = args.positional.first().map(|s| s.as_str()).unwrap_or("");
///         // ...
///     }
///     _ => { /* unknown */ }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ParsedArgs {
    /// The subcommand name (e.g., `"scan"`, `"register"`, `"load"`).
    /// Empty string if no subcommand was provided.
    pub command: String,

    /// Positional arguments after the subcommand.
    /// For `chronodrake load my-project`, this would be `["my-project"]`.
    pub positional: Vec<String>,

    /// Flags prefixed with `--` or `-`.
    /// For `chronodrake scan --verbose`, this would be `["--verbose"]`.
    pub flags: Vec<String>,
}

impl ParsedArgs {
    /// Parses the current process arguments into a structured form.
    ///
    /// The first argument (index 0) is the program path and is skipped.
    /// The second argument (index 1) is the subcommand.
    /// Everything after that is classified as positional or flag.
    pub fn from_env() -> Self {
        let raw: Vec<String> = env::args().collect();
        Self::parse(&raw)
    }

    /// Parses raw string arguments into a structured form.
    ///
    /// Useful for testing — allows passing simulated argument lists.
    ///
    /// # Examples
    ///
    /// ```
    /// let args = ParsedArgs::parse(&["chronodrake".into(), "scan".into()]);
    /// assert_eq!(args.command, "scan");
    ///
    /// let args = ParsedArgs::parse(&["chronodrake".into(), "load".into(), "my-project".into()]);
    /// assert_eq!(args.command, "load");
    /// assert_eq!(args.positional, vec!["my-project"]);
    /// ```
    pub fn parse(raw: &[String]) -> Self {
        if raw.len() < 2 {
            return Self {
                command: String::new(),
                positional: Vec::new(),
                flags: Vec::new(),
            };
        }

        let command = raw[1].clone();
        let mut positional = Vec::new();
        let mut flags = Vec::new();

        for arg in &raw[2..] {
            if arg.starts_with("--") || arg.starts_with('-') {
                flags.push(arg.clone());
            } else {
                positional.push(arg.clone());
            }
        }

        Self {
            command,
            positional,
            flags,
        }
    }

    /// Returns `true` if no subcommand was provided.
    pub fn is_empty(&self) -> bool {
        self.command.is_empty()
    }

    /// Returns the first positional argument, if any.
    pub fn first_arg(&self) -> Option<&str> {
        self.positional.first().map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_args() {
        let args = ParsedArgs::parse(&["chronodrake".into()]);
        assert!(args.is_empty());
        assert_eq!(args.command, "");
        assert!(args.positional.is_empty());
        assert!(args.flags.is_empty());
    }

    #[test]
    fn test_scan_command() {
        let args = ParsedArgs::parse(&["chronodrake".into(), "scan".into()]);
        assert_eq!(args.command, "scan");
        assert!(args.positional.is_empty());
        assert!(args.flags.is_empty());
    }

    #[test]
    fn test_load_with_positional() {
        let args = ParsedArgs::parse(&[
            "chronodrake".into(),
            "load".into(),
            "my-project".into(),
        ]);
        assert_eq!(args.command, "load");
        assert_eq!(args.positional, vec!["my-project"]);
        assert_eq!(args.first_arg(), Some("my-project"));
    }

    #[test]
    fn test_with_flags() {
        let args = ParsedArgs::parse(&[
            "chronodrake".into(),
            "scan".into(),
            "--verbose".into(),
            "--output".into(),
            "report.md".into(),
        ]);
        assert_eq!(args.command, "scan");
        assert_eq!(args.flags, vec!["--verbose", "--output"]);
        assert_eq!(args.positional, vec!["report.md"]);
    }

    #[test]
    fn test_help_flag() {
        let args = ParsedArgs::parse(&["chronodrake".into(), "--help".into()]);
        assert_eq!(args.command, "--help");
        assert!(args.positional.is_empty());
    }

    #[test]
    fn test_from_env_does_not_panic() {
        // Just verify it doesn't crash
        let _args = ParsedArgs::from_env();
    }
}
