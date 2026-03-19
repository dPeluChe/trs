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
// Logs - Exception/Stack Trace Fixtures
// ============================================================

/// Returns logs with Java and Python exceptions/stack traces.
pub fn logs_with_exceptions() -> String {
    load_fixture("logs_with_exceptions.txt")
}

/// Returns logs with multiline error messages and stack traces.
pub fn logs_multiline_error() -> String {
    load_fixture("logs_multiline_error.txt")
}

// ============================================================
// Logs - Mixed Format Fixtures
// ============================================================

/// Returns logs with mixed format styles (timestamps, levels).
pub fn logs_mixed_format() -> String {
    load_fixture("logs_mixed_format.txt")
}

/// Returns logs with colon-level format (LEVEL: message).
pub fn logs_colon_format() -> String {
    load_fixture("logs_colon_format.txt")
}

/// Returns logs with pipe-level format (|LEVEL| message).
pub fn logs_pipe_format() -> String {
    load_fixture("logs_pipe_format.txt")
}

// ============================================================
// Logs - Edge Cases
// ============================================================

/// Returns logs with repeated lines (for testing deduplication).
pub fn logs_repeated_lines() -> String {
    load_fixture("logs_repeated_lines.txt")
}

/// Returns logs with many entries (for testing truncation).
pub fn logs_large() -> String {
    load_fixture("logs_large.txt")
}

/// Returns logs in JSON format.
pub fn logs_json_format() -> String {
    load_fixture("logs_json_format.txt")
}

/// Returns logs with source/logger names.
pub fn logs_with_source() -> String {
    load_fixture("logs_with_source.txt")
}

/// Returns application startup/shutdown logs.
pub fn logs_application() -> String {
    load_fixture("logs_application.txt")
}

/// Returns HTTP access logs (Apache/nginx style).
pub fn logs_access() -> String {
    load_fixture("logs_access.txt")
}
