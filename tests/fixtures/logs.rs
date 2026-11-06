#![allow(dead_code)]
use super::load_fixture;

// ============================================================
// Logs - Empty/Clean Fixtures
// ============================================================

/// Returns empty logs output.
pub fn logs_empty() -> String {
    load_fixture("logs_empty.txt")
}

// ============================================================
// Logs - Simple Format Fixtures
// ============================================================

/// Returns simple log output with all levels.
pub fn logs_simple() -> String {
    load_fixture("logs_simple.txt")
}

/// Returns logs with ISO 8601 timestamps.
pub fn logs_iso8601_timestamp() -> String {
    load_fixture("logs_iso8601_timestamp.txt")
}

/// Returns logs in syslog format.
pub fn logs_syslog_format() -> String {
    load_fixture("logs_syslog_format.txt")
}

// ============================================================
// Logs - All Levels Fixtures
// ============================================================

/// Returns logs with all log levels represented.
pub fn logs_all_levels() -> String {
    load_fixture("logs_all_levels.txt")
}

/// Returns logs with only error-level entries.
pub fn logs_errors_only() -> String {
    load_fixture("logs_errors_only.txt")
}

// ============================================================
// Logs - Edge Cases
// ============================================================

/// Returns logs with repeated lines (for testing deduplication).
pub fn logs_repeated_lines() -> String {
    load_fixture("logs_repeated_lines.txt")
}
