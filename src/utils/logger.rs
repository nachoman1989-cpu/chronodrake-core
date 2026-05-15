use std::time::Instant;
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::fmt::format::FmtSpan;

/// A structured logger built on top of `tracing`.
///
/// Provides convenience methods for common logging patterns used throughout
/// ChronoDrake, while delegating to the `tracing` crate for actual output.
pub struct Logger;

impl Logger {
    /// Initializes the global tracing subscriber with sensible defaults.
    ///
    /// - Logs to stdout with a compact format (level, target, message).
    /// - Supports `RUST_LOG` environment variable for filtering.
    /// - Includes span events (new, close) for performance tracking.
    pub fn init(log_level: &str) {
        let level = log_level.parse::<Level>().unwrap_or(Level::INFO);

        tracing_subscriber::fmt()
            .with_target(true)
            .with_level(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_env_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(level.into())
                    .from_env_lossy(),
            )
            .compact()
            .init();
    }

    /// Prints a section header.
    pub fn section(title: &str) {
        info!("═══════════════════════════════════════════════");
        info!("  {}", title);
        info!("═══════════════════════════════════════════════");
    }

    /// Prints a sub-section header.
    pub fn sub_section(title: &str) {
        info!("─── {} ───", title);
    }

    /// Prints a key-value pair.
    pub fn kv(key: &str, value: &str) {
        info!("  {}: {}", key, value);
    }

    /// Prints a success message (green).
    pub fn success(msg: &str) {
        info!("✓ {}", msg);
    }

    /// Prints a warning message (yellow).
    pub fn warning(msg: &str) {
        warn!("⚠ {}", msg);
    }

    /// Prints a failure/error message (red).
    pub fn failure(msg: &str) {
        error!("✗ {}", msg);
    }

    /// Prints an informational message (blue).
    pub fn info(msg: &str) {
        info!("ℹ {}", msg);
    }

    /// Prints a debug message.
    pub fn debug(msg: &str) {
        debug!("  {}", msg);
    }

    /// Prints a raw message at info level.
    pub fn raw(msg: &str) {
        info!("{}", msg);
    }

    /// Prints a visual divider.
    pub fn divider() {
        info!("───────────────────────────────────────────────");
    }

    /// Times the execution of a closure and logs the duration.
    ///
    /// Returns the result of the closure.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = Logger::timed("scanning files", || {
    ///     scan_filesystem("/path")
    /// });
    /// ```
    pub fn timed<T, F: FnOnce() -> T>(label: &str, f: F) -> T {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        info!("⏱ {} completed in {:?}", label, duration);
        result
    }

    /// Creates a tracing span for performance tracking.
    ///
    /// Use this for tracking execution time of async operations.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let span = Logger::span("database_query");
    /// let _guard = span.enter();
    /// // ... do work ...
    /// drop(_guard); // span closes, duration logged
    /// ```
    pub fn span(name: &str) -> tracing::Span {
        tracing::info_span!("{}", name)
    }

    /// Logs a performance metric.
    pub fn metric(name: &str, value: &str) {
        info!("📊 {} = {}", name, value);
    }
}

/// A simple timer for measuring and logging execution duration.
pub struct ScopedTimer {
    label: String,
    start: Instant,
}

impl ScopedTimer {
    /// Creates a new timer with the given label.
    /// The timer starts immediately.
    pub fn new(label: &str) -> Self {
        let start = Instant::now();
        debug!("▶ {} started", label);
        Self {
            label: label.to_string(),
            start,
        }
    }

    /// Stops the timer and logs the elapsed duration.
    pub fn stop(&self) {
        let duration = self.start.elapsed();
        info!("⏱ {} took {:?}", self.label, duration);
    }

    /// Returns the elapsed duration without logging.
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        debug!("◼ {} finished in {:?}", self.label, duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoped_timer() {
        let timer = ScopedTimer::new("test_operation");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed.as_millis() >= 10);
    }

    #[test]
    fn test_logger_init() {
        // Just verify it doesn't panic
        Logger::init("info");
        Logger::section("Test Section");
        Logger::sub_section("Test Subsection");
        Logger::success("Test success");
        Logger::warning("Test warning");
        Logger::info("Test info");
        Logger::raw("Test raw");
        Logger::divider();
    }
}
