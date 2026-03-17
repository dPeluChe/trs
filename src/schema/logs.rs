//! Log output schema types.

use serde::{Deserialize, Serialize};

use super::SchemaVersion;

// ============================================================
// Logs Output Schema
// ============================================================

/// Schema for log/tail output.
///
/// # Example JSON
///
/// ```json
/// {
///   "schema": { "version": "1.0.0", "type": "logs_output" },
///   "is_empty": false,
///   "entries": [
///     {
///       "line": "2024-01-15 10:30:00 [INFO] Application started",
///       "level": "info",
///       "timestamp": "2024-01-15 10:30:00",
///       "source": null,
///       "message": "Application started",
///       "line_number": 1
///     }
///   ],
///   "counts": {
///     "total_lines": 100,
///     "debug": 10,
///     "info": 50,
///     "warning": 5,
///     "error": 3,
///     "fatal": 0,
///     "unknown": 32
///   },
///   "recent_critical": [],
///   "repeated_lines": []
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogsOutputSchema {
    /// Schema version information.
    pub schema: SchemaVersion,
    /// Whether the output is empty.
    pub is_empty: bool,
    /// All log entries.
    #[serde(default)]
    pub entries: Vec<LogEntry>,
    /// Count summary.
    pub counts: LogCounts,
    /// Most recent critical lines (ERROR and FATAL level entries).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent_critical: Vec<LogEntry>,
    /// Repeated lines (collapsed).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repeated_lines: Vec<RepeatedLine>,
}

impl LogsOutputSchema {
    /// Create a new logs output schema.
    pub fn new() -> Self {
        Self {
            schema: SchemaVersion::new("logs_output"),
            is_empty: true,
            entries: Vec::new(),
            counts: LogCounts::default(),
            recent_critical: Vec::new(),
            repeated_lines: Vec::new(),
        }
    }
}

impl Default for LogsOutputSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Log level classification.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
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
    #[default]
    Unknown,
}

/// A single parsed log line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogEntry {
    /// Original line content.
    pub line: String,
    /// Detected log level.
    pub level: LogLevel,
    /// Timestamp (if detected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Source/logger name (if detected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Message content (without timestamp/level prefix).
    pub message: String,
    /// Line number in the input.
    pub line_number: usize,
}

impl LogEntry {
    /// Create a new log entry.
    pub fn new(line: &str, line_number: usize) -> Self {
        Self {
            line: line.to_string(),
            level: LogLevel::Unknown,
            timestamp: None,
            source: None,
            message: line.to_string(),
            line_number,
        }
    }
}

/// Statistics for repeated lines.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepeatedLine {
    /// The repeated line content.
    pub line: String,
    /// Number of occurrences.
    pub count: usize,
    /// First occurrence line number.
    pub first_line: usize,
    /// Last occurrence line number.
    pub last_line: usize,
}

/// Count summary for log output.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogCounts {
    /// Total line count.
    pub total_lines: usize,
    /// Number of debug level lines.
    #[serde(default)]
    pub debug: usize,
    /// Number of info level lines.
    #[serde(default)]
    pub info: usize,
    /// Number of warning level lines.
    #[serde(default)]
    pub warning: usize,
    /// Number of error level lines.
    #[serde(default)]
    pub error: usize,
    /// Number of fatal level lines.
    #[serde(default)]
    pub fatal: usize,
    /// Number of unknown level lines.
    #[serde(default)]
    pub unknown: usize,
}
