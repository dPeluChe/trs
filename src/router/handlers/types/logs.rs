//! Log stream parser data structures for command handlers.

// ============================================================
// Log Stream Parser
// ============================================================

/// Log level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LogLevel {
    /// Debug level.
    Debug,
    /// Info level.
    Info,
    /// Warning level.
    Warning,
    /// Error level.
    Error,
    /// Fatal/Critical level.
    Fatal,
    /// Unknown or unclassified level.
    Unknown,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Unknown
    }
}

/// A single parsed log line.
#[derive(Debug, Clone, Default)]
pub(crate) struct LogEntry {
    /// Original line content.
    pub(crate) line: String,
    /// Detected log level.
    pub(crate) level: LogLevel,
    /// Timestamp (if detected).
    pub(crate) timestamp: Option<String>,
    /// Source/logger name (if detected).
    #[allow(dead_code)]
    pub(crate) source: Option<String>,
    /// Message content (without timestamp/level prefix).
    pub(crate) message: String,
    /// Line number in the input.
    pub(crate) line_number: usize,
}

/// Statistics for repeated lines.
#[derive(Debug, Clone)]
pub(crate) struct RepeatedLine {
    /// The repeated line content.
    pub(crate) line: String,
    /// Number of occurrences.
    pub(crate) count: usize,
    /// First occurrence line number.
    pub(crate) first_line: usize,
    /// Last occurrence line number.
    pub(crate) last_line: usize,
}

/// Maximum number of recent critical (ERROR/FATAL) lines to track.
pub(crate) const MAX_RECENT_CRITICAL: usize = 10;

/// Parsed log output.
#[derive(Debug, Clone, Default)]
pub(crate) struct LogsOutput {
    /// All log entries.
    pub(crate) entries: Vec<LogEntry>,
    /// Total line count.
    pub(crate) total_lines: usize,
    /// Count by level.
    pub(crate) debug_count: usize,
    pub(crate) info_count: usize,
    pub(crate) warning_count: usize,
    pub(crate) error_count: usize,
    pub(crate) fatal_count: usize,
    pub(crate) unknown_count: usize,
    /// Repeated lines (collapsed).
    pub(crate) repeated_lines: Vec<RepeatedLine>,
    /// Most recent critical lines (ERROR and FATAL level entries).
    pub(crate) recent_critical: Vec<LogEntry>,
    /// Whether the output is empty.
    pub(crate) is_empty: bool,
}
