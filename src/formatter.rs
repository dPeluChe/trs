//! Formatter system for TARS CLI.
//!
//! This module provides a centralized formatter interface for producing output
//! in different formats (Compact, JSON, CSV, TSV, Agent, Raw).
//!
//! # Architecture
//!
//! The formatter system is built around a trait-based design:
//!
//! - `Formatter` - Core trait for formatting data to string output
//! - `CompactFormatter` - Formats data in a human-readable compact format
//! - `JsonFormatter` - Formats data as JSON
//! - `CsvFormatter` - Formats data as CSV
//! - `TsvFormatter` - Formats data as TSV
//! - `AgentFormatter` - Formats data for AI consumption
//! - `RawFormatter` - Formats data with minimal processing
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::formatter::{Formatter, CompactFormatter};
//! use crate::OutputFormat;
//!
//! let status = GitStatus { /* ... */ };
//! let output = CompactFormatter::format_git_status(&status);
//! println!("{}", output);
//! ```

use crate::OutputFormat;

// ============================================================
// Core Formatter Trait
// ============================================================

/// Core trait for formatting data to string output.
///
/// This trait defines the interface that all formatters must implement.
/// Each formatter handles a specific output format (Compact, JSON, etc.).
pub trait Formatter {
    /// Returns the name of this formatter.
    fn name() -> &'static str;

    /// Returns the output format this formatter handles.
    fn format() -> OutputFormat;
}

// ============================================================
// Compact Formatter
// ============================================================

/// Formatter for compact, human-readable output.
///
/// The compact formatter produces output that is:
/// - Concise and information-dense
/// - Easy for humans to read and scan
/// - Focused on essential information
/// - Suitable as the default output format
///
/// # Example Output
///
/// ```text
/// branch: main
/// status: clean
/// ```
///
/// Or for dirty state:
///
/// ```text
/// branch: feature/new-thing
/// counts: staged=2 unstaged=3 untracked=5 unmerged=0
/// staged (2):
///   M src/main.rs
///   A src/new_file.rs
/// unstaged (3):
///   M src/lib.rs
/// ```
pub struct CompactFormatter;

impl Formatter for CompactFormatter {
    fn name() -> &'static str {
        "compact"
    }

    fn format() -> OutputFormat {
        OutputFormat::Compact
    }
}

impl CompactFormatter {
    /// Format a simple message/status line.
    pub fn format_message(key: &str, value: &str) -> String {
        format!("{}: {}\n", key, value)
    }

    /// Format a count summary line.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CompactFormatter;
    /// let output = CompactFormatter::format_counts("counts", &[("passed", 10), ("failed", 2)]);
    /// assert_eq!(output, "counts: passed=10 failed=2\n");
    /// ```
    pub fn format_counts(label: &str, counts: &[(&str, usize)]) -> String {
        let parts: Vec<String> = counts
            .iter()
            .filter(|(_, c)| *c > 0)
            .map(|(name, count)| format!("{}={}", name, count))
            .collect();
        if parts.is_empty() {
            String::new()
        } else {
            format!("{}: {}\n", label, parts.join(" "))
        }
    }

    /// Format a section header with an optional count.
    pub fn format_section_header(name: &str, count: Option<usize>) -> String {
        match count {
            Some(c) => format!("{} ({}):\n", name, c),
            None => format!("{}:\n", name),
        }
    }

    /// Format an indented list item.
    pub fn format_item(status: &str, path: &str) -> String {
        format!("  {} {}\n", status, path)
    }

    /// Format an indented list item with rename info.
    pub fn format_item_renamed(status: &str, old_path: &str, new_path: &str) -> String {
        format!("  {} {} -> {}\n", status, old_path, new_path)
    }

    /// Format a test result summary.
    pub fn format_test_summary(passed: usize, failed: usize, skipped: usize, duration_ms: u64) -> String {
        let mut parts = Vec::new();
        if passed > 0 {
            parts.push(format!("passed={}", passed));
        }
        if failed > 0 {
            parts.push(format!("failed={}", failed));
        }
        if skipped > 0 {
            parts.push(format!("skipped={}", skipped));
        }

        let mut output = String::new();
        if !parts.is_empty() {
            output.push_str(&format!("tests: {}\n", parts.join(" ")));
        }
        output.push_str(&format!("duration: {}\n", format_duration(duration_ms)));
        output
    }

    /// Format a success/failure indicator.
    pub fn format_status(success: bool) -> &'static str {
        if success {
            "status: passed\n"
        } else {
            "status: failed\n"
        }
    }

    /// Format a list of failing tests.
    pub fn format_failures(failures: &[String]) -> String {
        let mut output = String::new();
        if !failures.is_empty() {
            output.push_str(&format!("failures ({}):\n", failures.len()));
            for failure in failures {
                output.push_str(&format!("  {}\n", failure));
            }
        }
        output
    }

    /// Format log level counts.
    pub fn format_log_levels(error: usize, warn: usize, info: usize, debug: usize) -> String {
        let mut parts = Vec::new();
        if error > 0 {
            parts.push(format!("error={}", error));
        }
        if warn > 0 {
            parts.push(format!("warn={}", warn));
        }
        if info > 0 {
            parts.push(format!("info={}", info));
        }
        if debug > 0 {
            parts.push(format!("debug={}", debug));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("levels: {}\n", parts.join(" "))
        }
    }

    /// Format a grep match line.
    pub fn format_grep_match(file: &str, line: Option<usize>, content: &str) -> String {
        match line {
            Some(l) => format!("{}:{}: {}\n", file, l, content.trim()),
            None => format!("{}: {}\n", file, content.trim()),
        }
    }

    /// Format a grep file header.
    pub fn format_grep_file(file: &str, match_count: usize) -> String {
        format!("{} ({} matches):\n", file, match_count)
    }

    /// Format a diff file entry.
    pub fn format_diff_file(path: &str, change_type: &str, additions: usize, deletions: usize) -> String {
        format!("  {} {} (+{} -{})\n", change_type, path, additions, deletions)
    }

    /// Format a diff summary.
    pub fn format_diff_summary(files_changed: usize, insertions: usize, deletions: usize) -> String {
        format!(
            "diff: {} files changed, {} insertions, {} deletions\n",
            files_changed, insertions, deletions
        )
    }

    /// Format a clean state indicator.
    pub fn format_clean() -> String {
        "status: clean\n".to_string()
    }

    /// Format a dirty state indicator with counts.
    pub fn format_dirty(staged: usize, unstaged: usize, untracked: usize, unmerged: usize) -> String {
        format!(
            "status: dirty (staged={} unstaged={} untracked={} unmerged={})\n",
            staged, unstaged, untracked, unmerged
        )
    }

    /// Format branch info with ahead/behind.
    pub fn format_branch_with_tracking(branch: &str, ahead: usize, behind: usize) -> String {
        let mut tracking = String::new();
        if ahead > 0 {
            tracking.push_str(&format!("ahead {}", ahead));
        }
        if behind > 0 {
            if !tracking.is_empty() {
                tracking.push_str(", ");
            }
            tracking.push_str(&format!("behind {}", behind));
        }
        if tracking.is_empty() {
            format!("branch: {}\n", branch)
        } else {
            format!("branch: {} ({})\n", branch, tracking)
        }
    }

    /// Format an empty result.
    pub fn format_empty() -> String {
        "(empty)\n".to_string()
    }

    /// Format a truncation warning.
    pub fn format_truncated(shown: usize, total: usize) -> String {
        format!("... showing {} of {} total\n", shown, total)
    }
}

// ============================================================
// Helper Functions for Compact Formatting
// ============================================================

/// Format a count with label, only showing if count > 0.
pub fn format_count_if_positive(label: &str, count: usize) -> Option<String> {
    if count > 0 {
        Some(format!("{}={}", label, count))
    } else {
        None
    }
}

/// Format a list of items with a header and count.
pub fn format_list_with_count(label: &str, items: &[String]) -> String {
    let mut output = String::new();
    if !items.is_empty() {
        output.push_str(&format!("{} ({}):\n", label, items.len()));
        for item in items {
            output.push_str(&format!("  {}\n", item));
        }
    }
    output
}

/// Format a key-value pair with optional label.
pub fn format_key_value(key: &str, value: &str, label: Option<&str>) -> String {
    match label {
        Some(l) => format!("{} [{}]: {}\n", key, l, value),
        None => format!("{}: {}\n", key, value),
    }
}

/// Format a simple key-value line.
pub fn format_line(key: &str, value: impl std::fmt::Display) -> String {
    format!("{}: {}\n", key, value)
}

/// Truncate a string to a maximum length with ellipsis.
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Format a duration in human-readable form.
pub fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60000 {
        format!("{:.2}s", ms as f64 / 1000.0)
    } else {
        let mins = ms / 60000;
        let secs = (ms % 60000) / 1000;
        format!("{}m {}s", mins, secs)
    }
}

/// Format a byte count in human-readable form.
pub fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    const GB: usize = 1024 * MB;

    if bytes >= GB {
        format!("{:.2}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

// ============================================================
// JSON Formatter
// ============================================================

/// Formatter for JSON output.
///
/// The JSON formatter produces structured JSON output that:
/// - Is machine-readable
/// - Can be parsed by other tools
/// - Contains all available fields
/// - Uses consistent schemas
pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn name() -> &'static str {
        "json"
    }

    fn format() -> OutputFormat {
        OutputFormat::Json
    }
}

// ============================================================
// CSV Formatter
// ============================================================

/// Formatter for CSV (Comma-Separated Values) output.
///
/// The CSV formatter produces tabular output that:
/// - Has a header row
/// - Uses commas as delimiters
/// - Properly escapes special characters
/// - Is compatible with spreadsheet tools
pub struct CsvFormatter;

impl Formatter for CsvFormatter {
    fn name() -> &'static str {
        "csv"
    }

    fn format() -> OutputFormat {
        OutputFormat::Csv
    }
}

impl CsvFormatter {
    /// Escape a field for CSV format.
    pub fn escape_field(field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r')
        {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }
}

// ============================================================
// TSV Formatter
// ============================================================

/// Formatter for TSV (Tab-Separated Values) output.
///
/// The TSV formatter produces tabular output that:
/// - Has a header row
/// - Uses tabs as delimiters
/// - Properly escapes special characters
/// - Is compatible with data processing tools
pub struct TsvFormatter;

impl Formatter for TsvFormatter {
    fn name() -> &'static str {
        "tsv"
    }

    fn format() -> OutputFormat {
        OutputFormat::Tsv
    }
}

impl TsvFormatter {
    /// Escape a field for TSV format.
    pub fn escape_field(field: &str) -> String {
        if field.contains('\t') || field.contains('\n') || field.contains('\r') {
            // TSV doesn't have a standard escaping mechanism, replace problematic chars
            field
                .replace('\t', "\\t")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
        } else {
            field.to_string()
        }
    }
}

// ============================================================
// Agent Formatter
// ============================================================

/// Formatter for AI agent-optimized output.
///
/// The agent formatter produces output that:
/// - Is optimized for AI consumption
/// - Uses structured markdown-like format
/// - Includes metadata sections
/// - Highlights key information
pub struct AgentFormatter;

impl Formatter for AgentFormatter {
    fn name() -> &'static str {
        "agent"
    }

    fn format() -> OutputFormat {
        OutputFormat::Agent
    }
}

impl AgentFormatter {
    /// Format a section header.
    pub fn section_header(title: &str) -> String {
        format!("## {}\n", title)
    }

    /// Format a list item with optional label.
    pub fn list_item(item: &str, label: Option<&str>) -> String {
        match label {
            Some(l) => format!("- {} [{}]\n", item, l),
            None => format!("- {}\n", item),
        }
    }

    /// Format a key-value item with optional label.
    pub fn key_value_item(key: &str, value: &str, label: Option<&str>) -> String {
        match label {
            Some(l) => format!("- {} [{}]: {}\n", key, l, value),
            None => format!("- {}: {}\n", key, value),
        }
    }
}

// ============================================================
// Raw Formatter
// ============================================================

/// Formatter for raw, unprocessed output.
///
/// The raw formatter produces output that:
/// - Is minimally processed
/// - Preserves original formatting
/// - Is useful for debugging
/// - Can be piped to other tools
pub struct RawFormatter;

impl Formatter for RawFormatter {
    fn name() -> &'static str {
        "raw"
    }

    fn format() -> OutputFormat {
        OutputFormat::Raw
    }
}

impl RawFormatter {
    /// Format a simple list of items (one per line).
    pub fn format_list(items: &[impl AsRef<str>]) -> String {
        items
            .iter()
            .map(|s| format!("{}\n", s.as_ref()))
            .collect()
    }

    /// Format a simple message/status line (just key and value).
    pub fn format_message(key: &str, value: &str) -> String {
        format!("{}: {}\n", key, value)
    }

    /// Format a count summary line.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::RawFormatter;
    /// let output = RawFormatter::format_counts(&[("passed", 10), ("failed", 2)]);
    /// assert_eq!(output, "passed=10 failed=2\n");
    /// ```
    pub fn format_counts(counts: &[(&str, usize)]) -> String {
        let parts: Vec<String> = counts
            .iter()
            .filter(|(_, c)| *c > 0)
            .map(|(name, count)| format!("{}={}", name, count))
            .collect();
        if parts.is_empty() {
            String::new()
        } else {
            format!("{}\n", parts.join(" "))
        }
    }

    /// Format a section header with an optional count.
    pub fn format_section_header(name: &str, count: Option<usize>) -> String {
        match count {
            Some(c) => format!("{} ({})\n", name, c),
            None => format!("{}\n", name),
        }
    }

    /// Format a list item (status and path, no indentation).
    pub fn format_item(status: &str, path: &str) -> String {
        format!("{} {}\n", status, path)
    }

    /// Format a list item with rename info.
    pub fn format_item_renamed(status: &str, old_path: &str, new_path: &str) -> String {
        format!("{} {} -> {}\n", status, old_path, new_path)
    }

    /// Format a test result summary.
    pub fn format_test_summary(passed: usize, failed: usize, skipped: usize, duration_ms: u64) -> String {
        let mut parts = Vec::new();
        if passed > 0 {
            parts.push(format!("passed={}", passed));
        }
        if failed > 0 {
            parts.push(format!("failed={}", failed));
        }
        if skipped > 0 {
            parts.push(format!("skipped={}", skipped));
        }

        let mut output = String::new();
        if !parts.is_empty() {
            output.push_str(&format!("{}\n", parts.join(" ")));
        }
        output.push_str(&format!("{}\n", format_duration(duration_ms)));
        output
    }

    /// Format a success/failure indicator.
    pub fn format_status(success: bool) -> &'static str {
        if success {
            "passed\n"
        } else {
            "failed\n"
        }
    }

    /// Format a list of failing tests.
    pub fn format_failures(failures: &[String]) -> String {
        let mut output = String::new();
        for failure in failures {
            output.push_str(&format!("{}\n", failure));
        }
        output
    }

    /// Format log level counts.
    pub fn format_log_levels(error: usize, warn: usize, info: usize, debug: usize) -> String {
        let mut parts = Vec::new();
        if error > 0 {
            parts.push(format!("error={}", error));
        }
        if warn > 0 {
            parts.push(format!("warn={}", warn));
        }
        if info > 0 {
            parts.push(format!("info={}", info));
        }
        if debug > 0 {
            parts.push(format!("debug={}", debug));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("{}\n", parts.join(" "))
        }
    }

    /// Format a grep match line (preserves original format).
    pub fn format_grep_match(file: &str, line: Option<usize>, content: &str) -> String {
        match line {
            Some(l) => format!("{}:{}:{}\n", file, l, content.trim()),
            None => format!("{}:{}\n", file, content.trim()),
        }
    }

    /// Format a grep file header.
    pub fn format_grep_file(file: &str, match_count: usize) -> String {
        format!("{} ({})\n", file, match_count)
    }

    /// Format a diff file entry.
    pub fn format_diff_file(path: &str, change_type: &str, additions: usize, deletions: usize) -> String {
        format!("{} {} +{} -{}\n", change_type, path, additions, deletions)
    }

    /// Format a diff summary.
    pub fn format_diff_summary(files_changed: usize, insertions: usize, deletions: usize) -> String {
        format!(
            "{} files +{} -{}\n",
            files_changed, insertions, deletions
        )
    }

    /// Format a clean state indicator.
    pub fn format_clean() -> String {
        "clean\n".to_string()
    }

    /// Format a dirty state indicator with counts.
    pub fn format_dirty(staged: usize, unstaged: usize, untracked: usize, unmerged: usize) -> String {
        format!(
            "dirty staged={} unstaged={} untracked={} unmerged={}\n",
            staged, unstaged, untracked, unmerged
        )
    }

    /// Format branch info with ahead/behind.
    pub fn format_branch_with_tracking(branch: &str, ahead: usize, behind: usize) -> String {
        let mut tracking = String::new();
        if ahead > 0 {
            tracking.push_str(&format!("ahead {}", ahead));
        }
        if behind > 0 {
            if !tracking.is_empty() {
                tracking.push_str(", ");
            }
            tracking.push_str(&format!("behind {}", behind));
        }
        if tracking.is_empty() {
            format!("{}\n", branch)
        } else {
            format!("{} ({})\n", branch, tracking)
        }
    }

    /// Format an empty result.
    pub fn format_empty() -> String {
        String::new()
    }

    /// Format a truncation warning.
    pub fn format_truncated(shown: usize, total: usize) -> String {
        format!("... {}/{}\n", shown, total)
    }

    /// Format a key-value pair.
    pub fn format_key_value(key: &str, value: &str) -> String {
        format!("{} {}\n", key, value)
    }

    /// Format raw output preserving the original content.
    pub fn format_raw(content: &str) -> String {
        if content.is_empty() {
            String::new()
        } else if content.ends_with('\n') {
            content.to_string()
        } else {
            format!("{}\n", content)
        }
    }
}

// ============================================================
// Format Selection
// ============================================================

/// Select the appropriate formatter for the given output format.
///
/// This is a convenience function for dispatching to the right formatter
/// based on the output format.
pub fn select_formatter(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Json => JsonFormatter::name(),
        OutputFormat::Csv => CsvFormatter::name(),
        OutputFormat::Tsv => TsvFormatter::name(),
        OutputFormat::Agent => AgentFormatter::name(),
        OutputFormat::Compact => CompactFormatter::name(),
        OutputFormat::Raw => RawFormatter::name(),
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatter_names() {
        assert_eq!(CompactFormatter::name(), "compact");
        assert_eq!(JsonFormatter::name(), "json");
        assert_eq!(CsvFormatter::name(), "csv");
        assert_eq!(TsvFormatter::name(), "tsv");
        assert_eq!(AgentFormatter::name(), "agent");
        assert_eq!(RawFormatter::name(), "raw");
    }

    #[test]
    fn test_formatter_output_formats() {
        assert_eq!(CompactFormatter::format(), OutputFormat::Compact);
        assert_eq!(JsonFormatter::format(), OutputFormat::Json);
        assert_eq!(CsvFormatter::format(), OutputFormat::Csv);
        assert_eq!(TsvFormatter::format(), OutputFormat::Tsv);
        assert_eq!(AgentFormatter::format(), OutputFormat::Agent);
        assert_eq!(RawFormatter::format(), OutputFormat::Raw);
    }

    // ============================================================
    // CompactFormatter Tests
    // ============================================================

    #[test]
    fn test_compact_format_message() {
        assert_eq!(
            CompactFormatter::format_message("branch", "main"),
            "branch: main\n"
        );
    }

    #[test]
    fn test_compact_format_counts() {
        let output = CompactFormatter::format_counts("counts", &[("passed", 10), ("failed", 2)]);
        assert_eq!(output, "counts: passed=10 failed=2\n");

        // Zero counts should be filtered out
        let output = CompactFormatter::format_counts("counts", &[("passed", 0), ("failed", 2)]);
        assert_eq!(output, "counts: failed=2\n");

        // All zeros should return empty string
        let output = CompactFormatter::format_counts("counts", &[("passed", 0), ("failed", 0)]);
        assert!(output.is_empty());
    }

    #[test]
    fn test_compact_format_section_header() {
        assert_eq!(
            CompactFormatter::format_section_header("staged", Some(3)),
            "staged (3):\n"
        );
        assert_eq!(
            CompactFormatter::format_section_header("files", None),
            "files:\n"
        );
    }

    #[test]
    fn test_compact_format_item() {
        assert_eq!(
            CompactFormatter::format_item("M", "src/main.rs"),
            "  M src/main.rs\n"
        );
    }

    #[test]
    fn test_compact_format_item_renamed() {
        assert_eq!(
            CompactFormatter::format_item_renamed("R", "old.rs", "new.rs"),
            "  R old.rs -> new.rs\n"
        );
    }

    #[test]
    fn test_compact_format_test_summary() {
        let output = CompactFormatter::format_test_summary(10, 2, 1, 1500);
        assert!(output.contains("tests: passed=10 failed=2 skipped=1"));
        assert!(output.contains("duration: 1.50s"));
    }

    #[test]
    fn test_compact_format_test_summary_only_passed() {
        let output = CompactFormatter::format_test_summary(5, 0, 0, 500);
        assert!(output.contains("tests: passed=5"));
        assert!(!output.contains("failed"));
        assert!(!output.contains("skipped"));
    }

    #[test]
    fn test_compact_format_status() {
        assert_eq!(CompactFormatter::format_status(true), "status: passed\n");
        assert_eq!(CompactFormatter::format_status(false), "status: failed\n");
    }

    #[test]
    fn test_compact_format_failures() {
        let failures = vec!["test_one".to_string(), "test_two".to_string()];
        let output = CompactFormatter::format_failures(&failures);
        assert!(output.contains("failures (2):"));
        assert!(output.contains("test_one"));
        assert!(output.contains("test_two"));
    }

    #[test]
    fn test_compact_format_failures_empty() {
        let failures: Vec<String> = vec![];
        let output = CompactFormatter::format_failures(&failures);
        assert!(output.is_empty());
    }

    #[test]
    fn test_compact_format_log_levels() {
        let output = CompactFormatter::format_log_levels(2, 5, 10, 3);
        assert_eq!(output, "levels: error=2 warn=5 info=10 debug=3\n");
    }

    #[test]
    fn test_compact_format_log_levels_partial() {
        let output = CompactFormatter::format_log_levels(0, 5, 0, 0);
        assert_eq!(output, "levels: warn=5\n");
    }

    #[test]
    fn test_compact_format_log_levels_empty() {
        let output = CompactFormatter::format_log_levels(0, 0, 0, 0);
        assert!(output.is_empty());
    }

    #[test]
    fn test_compact_format_grep_match() {
        let output = CompactFormatter::format_grep_match("src/main.rs", Some(42), "fn main()");
        assert_eq!(output, "src/main.rs:42: fn main()\n");
    }

    #[test]
    fn test_compact_format_grep_match_no_line() {
        let output = CompactFormatter::format_grep_match("src/main.rs", None, "match found");
        assert_eq!(output, "src/main.rs: match found\n");
    }

    #[test]
    fn test_compact_format_grep_file() {
        let output = CompactFormatter::format_grep_file("src/main.rs", 5);
        assert_eq!(output, "src/main.rs (5 matches):\n");
    }

    #[test]
    fn test_compact_format_diff_file() {
        let output = CompactFormatter::format_diff_file("src/main.rs", "M", 10, 5);
        assert_eq!(output, "  M src/main.rs (+10 -5)\n");
    }

    #[test]
    fn test_compact_format_diff_summary() {
        let output = CompactFormatter::format_diff_summary(3, 25, 10);
        assert_eq!(
            output,
            "diff: 3 files changed, 25 insertions, 10 deletions\n"
        );
    }

    #[test]
    fn test_compact_format_clean() {
        assert_eq!(CompactFormatter::format_clean(), "status: clean\n");
    }

    #[test]
    fn test_compact_format_dirty() {
        let output = CompactFormatter::format_dirty(2, 3, 5, 0);
        assert_eq!(
            output,
            "status: dirty (staged=2 unstaged=3 untracked=5 unmerged=0)\n"
        );
    }

    #[test]
    fn test_compact_format_branch_with_tracking() {
        // No tracking
        assert_eq!(
            CompactFormatter::format_branch_with_tracking("main", 0, 0),
            "branch: main\n"
        );

        // Ahead only
        assert_eq!(
            CompactFormatter::format_branch_with_tracking("feature", 3, 0),
            "branch: feature (ahead 3)\n"
        );

        // Behind only
        assert_eq!(
            CompactFormatter::format_branch_with_tracking("feature", 0, 2),
            "branch: feature (behind 2)\n"
        );

        // Both ahead and behind
        assert_eq!(
            CompactFormatter::format_branch_with_tracking("feature", 3, 2),
            "branch: feature (ahead 3, behind 2)\n"
        );
    }

    #[test]
    fn test_compact_format_empty() {
        assert_eq!(CompactFormatter::format_empty(), "(empty)\n");
    }

    #[test]
    fn test_compact_format_truncated() {
        let output = CompactFormatter::format_truncated(10, 50);
        assert_eq!(output, "... showing 10 of 50 total\n");
    }

    // ============================================================
    // Helper Function Tests
    // ============================================================

    #[test]
    fn test_format_count_if_positive() {
        assert_eq!(format_count_if_positive("staged", 0), None);
        assert_eq!(
            format_count_if_positive("staged", 3),
            Some("staged=3".to_string())
        );
    }

    #[test]
    fn test_format_list_with_count() {
        let items = vec!["file1.rs".to_string(), "file2.rs".to_string()];
        let output = format_list_with_count("staged", &items);
        assert!(output.contains("staged (2):"));
        assert!(output.contains("file1.rs"));
        assert!(output.contains("file2.rs"));
    }

    #[test]
    fn test_format_list_with_count_empty() {
        let items: Vec<String> = vec![];
        let output = format_list_with_count("staged", &items);
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_key_value() {
        assert_eq!(format_key_value("branch", "main", None), "branch: main\n");
        assert_eq!(
            format_key_value("status", "M", Some("modified")),
            "status [modified]: M\n"
        );
    }

    #[test]
    fn test_format_line() {
        assert_eq!(format_line("branch", "main"), "branch: main\n");
        assert_eq!(format_line("count", 42), "count: 42\n");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("hi", 3), "hi");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(1500), "1.50s");
        assert_eq!(format_duration(90000), "1m 30s");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500B");
        assert_eq!(format_bytes(1024), "1.00KB");
        assert_eq!(format_bytes(1048576), "1.00MB");
        assert_eq!(format_bytes(1073741824), "1.00GB");
    }

    // ============================================================
    // CSV Formatter Tests
    // ============================================================

    #[test]
    fn test_csv_escape_field() {
        assert_eq!(CsvFormatter::escape_field("simple"), "simple");
        assert_eq!(CsvFormatter::escape_field("with,comma"), "\"with,comma\"");
        assert_eq!(CsvFormatter::escape_field("with\"quote"), "\"with\"\"quote\"");
        assert_eq!(CsvFormatter::escape_field("with\nnewline"), "\"with\nnewline\"");
    }

    // ============================================================
    // TSV Formatter Tests
    // ============================================================

    #[test]
    fn test_tsv_escape_field() {
        assert_eq!(TsvFormatter::escape_field("simple"), "simple");
        assert_eq!(TsvFormatter::escape_field("with\ttab"), "with\\ttab");
        assert_eq!(TsvFormatter::escape_field("with\nnewline"), "with\\nnewline");
    }

    // ============================================================
    // Agent Formatter Tests
    // ============================================================

    #[test]
    fn test_agent_section_header() {
        assert_eq!(AgentFormatter::section_header("Files"), "## Files\n");
    }

    #[test]
    fn test_agent_list_item() {
        assert_eq!(AgentFormatter::list_item("file.rs", None), "- file.rs\n");
        assert_eq!(
            AgentFormatter::list_item("file.rs", Some("modified")),
            "- file.rs [modified]\n"
        );
    }

    #[test]
    fn test_agent_key_value_item() {
        assert_eq!(
            AgentFormatter::key_value_item("branch", "main", None),
            "- branch: main\n"
        );
        assert_eq!(
            AgentFormatter::key_value_item("count", "5", Some("files")),
            "- count [files]: 5\n"
        );
    }

    // ============================================================
    // Raw Formatter Tests
    // ============================================================

    #[test]
    fn test_raw_format_list() {
        let items = vec!["file1.rs", "file2.rs"];
        let output = RawFormatter::format_list(&items);
        assert_eq!(output, "file1.rs\nfile2.rs\n");
    }

    #[test]
    fn test_raw_format_message() {
        assert_eq!(
            RawFormatter::format_message("branch", "main"),
            "branch: main\n"
        );
    }

    #[test]
    fn test_raw_format_counts() {
        let output = RawFormatter::format_counts(&[("passed", 10), ("failed", 2)]);
        assert_eq!(output, "passed=10 failed=2\n");

        // Zero counts should be filtered out
        let output = RawFormatter::format_counts(&[("passed", 0), ("failed", 2)]);
        assert_eq!(output, "failed=2\n");

        // All zeros should return empty string
        let output = RawFormatter::format_counts(&[("passed", 0), ("failed", 0)]);
        assert!(output.is_empty());
    }

    #[test]
    fn test_raw_format_section_header() {
        assert_eq!(
            RawFormatter::format_section_header("staged", Some(3)),
            "staged (3)\n"
        );
        assert_eq!(
            RawFormatter::format_section_header("files", None),
            "files\n"
        );
    }

    #[test]
    fn test_raw_format_item() {
        assert_eq!(
            RawFormatter::format_item("M", "src/main.rs"),
            "M src/main.rs\n"
        );
    }

    #[test]
    fn test_raw_format_item_renamed() {
        assert_eq!(
            RawFormatter::format_item_renamed("R", "old.rs", "new.rs"),
            "R old.rs -> new.rs\n"
        );
    }

    #[test]
    fn test_raw_format_test_summary() {
        let output = RawFormatter::format_test_summary(10, 2, 1, 1500);
        assert!(output.contains("passed=10 failed=2 skipped=1"));
        assert!(output.contains("1.50s"));
    }

    #[test]
    fn test_raw_format_test_summary_only_passed() {
        let output = RawFormatter::format_test_summary(5, 0, 0, 500);
        assert!(output.contains("passed=5"));
        assert!(!output.contains("failed"));
        assert!(!output.contains("skipped"));
    }

    #[test]
    fn test_raw_format_status() {
        assert_eq!(RawFormatter::format_status(true), "passed\n");
        assert_eq!(RawFormatter::format_status(false), "failed\n");
    }

    #[test]
    fn test_raw_format_failures() {
        let failures = vec!["test_one".to_string(), "test_two".to_string()];
        let output = RawFormatter::format_failures(&failures);
        assert!(output.contains("test_one\n"));
        assert!(output.contains("test_two\n"));
    }

    #[test]
    fn test_raw_format_failures_empty() {
        let failures: Vec<String> = vec![];
        let output = RawFormatter::format_failures(&failures);
        assert!(output.is_empty());
    }

    #[test]
    fn test_raw_format_log_levels() {
        let output = RawFormatter::format_log_levels(2, 5, 10, 3);
        assert_eq!(output, "error=2 warn=5 info=10 debug=3\n");
    }

    #[test]
    fn test_raw_format_log_levels_partial() {
        let output = RawFormatter::format_log_levels(0, 5, 0, 0);
        assert_eq!(output, "warn=5\n");
    }

    #[test]
    fn test_raw_format_log_levels_empty() {
        let output = RawFormatter::format_log_levels(0, 0, 0, 0);
        assert!(output.is_empty());
    }

    #[test]
    fn test_raw_format_grep_match() {
        let output = RawFormatter::format_grep_match("src/main.rs", Some(42), "fn main()");
        assert_eq!(output, "src/main.rs:42:fn main()\n");
    }

    #[test]
    fn test_raw_format_grep_match_no_line() {
        let output = RawFormatter::format_grep_match("src/main.rs", None, "match found");
        assert_eq!(output, "src/main.rs:match found\n");
    }

    #[test]
    fn test_raw_format_grep_file() {
        let output = RawFormatter::format_grep_file("src/main.rs", 5);
        assert_eq!(output, "src/main.rs (5)\n");
    }

    #[test]
    fn test_raw_format_diff_file() {
        let output = RawFormatter::format_diff_file("src/main.rs", "M", 10, 5);
        assert_eq!(output, "M src/main.rs +10 -5\n");
    }

    #[test]
    fn test_raw_format_diff_summary() {
        let output = RawFormatter::format_diff_summary(3, 25, 10);
        assert_eq!(output, "3 files +25 -10\n");
    }

    #[test]
    fn test_raw_format_clean() {
        assert_eq!(RawFormatter::format_clean(), "clean\n");
    }

    #[test]
    fn test_raw_format_dirty() {
        let output = RawFormatter::format_dirty(2, 3, 5, 0);
        assert_eq!(
            output,
            "dirty staged=2 unstaged=3 untracked=5 unmerged=0\n"
        );
    }

    #[test]
    fn test_raw_format_branch_with_tracking() {
        // No tracking
        assert_eq!(
            RawFormatter::format_branch_with_tracking("main", 0, 0),
            "main\n"
        );

        // Ahead only
        assert_eq!(
            RawFormatter::format_branch_with_tracking("feature", 3, 0),
            "feature (ahead 3)\n"
        );

        // Behind only
        assert_eq!(
            RawFormatter::format_branch_with_tracking("feature", 0, 2),
            "feature (behind 2)\n"
        );

        // Both ahead and behind
        assert_eq!(
            RawFormatter::format_branch_with_tracking("feature", 3, 2),
            "feature (ahead 3, behind 2)\n"
        );
    }

    #[test]
    fn test_raw_format_empty() {
        assert_eq!(RawFormatter::format_empty(), "");
    }

    #[test]
    fn test_raw_format_truncated() {
        let output = RawFormatter::format_truncated(10, 50);
        assert_eq!(output, "... 10/50\n");
    }

    #[test]
    fn test_raw_format_key_value() {
        assert_eq!(RawFormatter::format_key_value("branch", "main"), "branch main\n");
    }

    #[test]
    fn test_raw_format_raw() {
        // With existing newline
        assert_eq!(RawFormatter::format_raw("content\n"), "content\n");
        // Without newline
        assert_eq!(RawFormatter::format_raw("content"), "content\n");
        // Empty
        assert_eq!(RawFormatter::format_raw(""), "");
    }

    // ============================================================
    // Format Selection Tests
    // ============================================================

    #[test]
    fn test_select_formatter() {
        assert_eq!(select_formatter(OutputFormat::Compact), "compact");
        assert_eq!(select_formatter(OutputFormat::Json), "json");
        assert_eq!(select_formatter(OutputFormat::Csv), "csv");
        assert_eq!(select_formatter(OutputFormat::Tsv), "tsv");
        assert_eq!(select_formatter(OutputFormat::Agent), "agent");
        assert_eq!(select_formatter(OutputFormat::Raw), "raw");
    }
}
