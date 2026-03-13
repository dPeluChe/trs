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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct CompactFormatter;

impl Formatter for CompactFormatter {
    fn name() -> &'static str {
        "compact"
    }

    fn format() -> OutputFormat {
        OutputFormat::Compact
    }
}

#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn format_count_if_positive(label: &str, count: usize) -> Option<String> {
    if count > 0 {
        Some(format!("{}={}", label, count))
    } else {
        None
    }
}

/// Format a list of items with a header and count.
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn format_key_value(key: &str, value: &str, label: Option<&str>) -> String {
    match label {
        Some(l) => format!("{} [{}]: {}\n", key, l, value),
        None => format!("{}: {}\n", key, value),
    }
}

/// Format a simple key-value line.
#[allow(dead_code)]
pub fn format_line(key: &str, value: impl std::fmt::Display) -> String {
    format!("{}: {}\n", key, value)
}

/// Truncate a string to a maximum length with ellipsis.
#[allow(dead_code)]
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Format a duration in human-readable form.
#[allow(dead_code)]
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
#[allow(dead_code)]
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
///
/// # Example Output
///
/// ```json
/// {"branch": "main", "is_clean": true}
/// ```
///
/// Or for dirty state:
///
/// ```json
/// {"branch": "feature/new-thing", "is_clean": false, "staged_count": 2, "unstaged_count": 3, "untracked_count": 5, "unmerged_count": 0}
/// ```
#[allow(dead_code)]
pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn name() -> &'static str {
        "json"
    }

    fn format() -> OutputFormat {
        OutputFormat::Json
    }
}

#[allow(dead_code)]
impl JsonFormatter {
    /// Format a simple message/status as JSON.
    pub fn format_message(key: &str, value: &str) -> String {
        serde_json::json!({
            key: value
        })
        .to_string()
    }

    /// Format a key-value pair as JSON.
    pub fn format_key_value(key: &str, value: impl serde::Serialize) -> String {
        serde_json::json!({
            key: value
        })
        .to_string()
    }

    /// Format multiple key-value pairs as JSON.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::JsonFormatter;
    /// use serde_json::json;
    /// let output = JsonFormatter::format_object(&[
    ///     ("branch", json!("main")),
    ///     ("is_clean", json!(true)),
    ///     ("count", json!(5)),
    /// ]);
    /// let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    /// assert_eq!(json["branch"], "main");
    /// assert_eq!(json["is_clean"], true);
    /// assert_eq!(json["count"], 5);
    /// ```
    pub fn format_object(pairs: &[(&str, serde_json::Value)]) -> String {
        let mut map = serde_json::Map::new();
        for (key, value) in pairs {
            map.insert(key.to_string(), value.clone());
        }
        serde_json::Value::Object(map).to_string()
    }

    /// Format a count summary as JSON.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::JsonFormatter;
    /// let output = JsonFormatter::format_counts(&[("passed", 10), ("failed", 2)]);
    /// let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    /// assert_eq!(json["passed"], 10);
    /// assert_eq!(json["failed"], 2);
    /// ```
    pub fn format_counts(counts: &[(&str, usize)]) -> String {
        let mut map = serde_json::Map::new();
        for (name, count) in counts {
            map.insert(name.to_string(), serde_json::json!(*count));
        }
        serde_json::Value::Object(map).to_string()
    }

    /// Format a section with items as JSON.
    pub fn format_section(name: &str, items: &[impl serde::Serialize]) -> String {
        serde_json::json!({
            name: items
        })
        .to_string()
    }

    /// Format a list item with status and path as JSON.
    pub fn format_item(status: &str, path: &str) -> String {
        serde_json::json!({
            "status": status,
            "path": path
        })
        .to_string()
    }

    /// Format a list item with rename info as JSON.
    pub fn format_item_renamed(status: &str, old_path: &str, new_path: &str) -> String {
        serde_json::json!({
            "status": status,
            "path": new_path,
            "old_path": old_path
        })
        .to_string()
    }

    /// Format a test result summary as JSON.
    pub fn format_test_summary(passed: usize, failed: usize, skipped: usize, duration_ms: u64) -> String {
        serde_json::json!({
            "passed": passed,
            "failed": failed,
            "skipped": skipped,
            "total": passed + failed + skipped,
            "duration_ms": duration_ms
        })
        .to_string()
    }

    /// Format a success/failure status as JSON.
    pub fn format_status(success: bool) -> String {
        serde_json::json!({
            "success": success
        })
        .to_string()
    }

    /// Format a list of failing tests as JSON.
    pub fn format_failures(failures: &[String]) -> String {
        serde_json::json!({
            "failures": failures,
            "count": failures.len()
        })
        .to_string()
    }

    /// Format log level counts as JSON.
    pub fn format_log_levels(error: usize, warn: usize, info: usize, debug: usize) -> String {
        serde_json::json!({
            "error": error,
            "warn": warn,
            "info": info,
            "debug": debug,
            "total": error + warn + info + debug
        })
        .to_string()
    }

    /// Format a grep match as JSON.
    pub fn format_grep_match(file: &str, line: Option<usize>, content: &str) -> String {
        serde_json::json!({
            "file": file,
            "line": line,
            "content": content.trim()
        })
        .to_string()
    }

    /// Format a grep file with matches as JSON.
    pub fn format_grep_file(file: &str, match_count: usize) -> String {
        serde_json::json!({
            "file": file,
            "match_count": match_count
        })
        .to_string()
    }

    /// Format a diff file entry as JSON.
    pub fn format_diff_file(path: &str, change_type: &str, additions: usize, deletions: usize) -> String {
        serde_json::json!({
            "path": path,
            "change_type": change_type,
            "additions": additions,
            "deletions": deletions
        })
        .to_string()
    }

    /// Format a diff summary as JSON.
    pub fn format_diff_summary(files_changed: usize, insertions: usize, deletions: usize) -> String {
        serde_json::json!({
            "files_changed": files_changed,
            "insertions": insertions,
            "deletions": deletions
        })
        .to_string()
    }

    /// Format a clean state indicator as JSON.
    pub fn format_clean() -> String {
        serde_json::json!({
            "is_clean": true
        })
        .to_string()
    }

    /// Format a dirty state indicator with counts as JSON.
    pub fn format_dirty(staged: usize, unstaged: usize, untracked: usize, unmerged: usize) -> String {
        serde_json::json!({
            "is_clean": false,
            "staged": staged,
            "unstaged": unstaged,
            "untracked": untracked,
            "unmerged": unmerged
        })
        .to_string()
    }

    /// Format branch info with ahead/behind as JSON.
    pub fn format_branch_with_tracking(branch: &str, ahead: usize, behind: usize) -> String {
        serde_json::json!({
            "branch": branch,
            "ahead": ahead,
            "behind": behind
        })
        .to_string()
    }

    /// Format an empty result as JSON.
    pub fn format_empty() -> String {
        serde_json::json!({
            "empty": true
        })
        .to_string()
    }

    /// Format a truncation warning as JSON.
    pub fn format_truncated(shown: usize, total: usize) -> String {
        serde_json::json!({
            "is_truncated": true,
            "shown": shown,
            "total": total
        })
        .to_string()
    }

    /// Format an error message as JSON.
    pub fn format_error(message: &str) -> String {
        serde_json::json!({
            "error": true,
            "message": message
        })
        .to_string()
    }

    /// Format an error with exit code as JSON.
    pub fn format_error_with_code(message: &str, exit_code: i32) -> String {
        serde_json::json!({
            "error": true,
            "message": message,
            "exit_code": exit_code
        })
        .to_string()
    }

    /// Format a not-implemented message as JSON.
    pub fn format_not_implemented(message: &str) -> String {
        serde_json::json!({
            "not_implemented": true,
            "message": message
        })
        .to_string()
    }

    /// Format a command result as JSON.
    pub fn format_command_result(
        command: &str,
        args: &[String],
        stdout: &str,
        stderr: &str,
        exit_code: i32,
        duration_ms: u64,
    ) -> String {
        serde_json::json!({
            "command": command,
            "args": args,
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": exit_code,
            "duration_ms": duration_ms
        })
        .to_string()
    }

    /// Format a list of strings as JSON array.
    pub fn format_list(items: &[impl AsRef<str>]) -> String {
        serde_json::json!(items.iter().map(|s| s.as_ref()).collect::<Vec<_>>()).to_string()
    }

    /// Format a count as JSON.
    pub fn format_count(count: usize) -> String {
        serde_json::json!({ "count": count }).to_string()
    }

    /// Format a boolean flag as JSON.
    pub fn format_flag(name: &str, value: bool) -> String {
        serde_json::json!({ name: value }).to_string()
    }

    /// Format an array of objects as JSON.
    pub fn format_array<T: serde::Serialize>(items: &[T]) -> String {
        serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string())
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
#[allow(dead_code)]
pub struct CsvFormatter;

impl Formatter for CsvFormatter {
    fn name() -> &'static str {
        "csv"
    }

    fn format() -> OutputFormat {
        OutputFormat::Csv
    }
}

#[allow(dead_code)]
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

    /// Format a CSV header row from field names.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_header(&["branch", "is_clean", "count"]);
    /// assert_eq!(output, "branch,is_clean,count\n");
    /// ```
    pub fn format_header(fields: &[&str]) -> String {
        let escaped: Vec<String> = fields.iter().map(|f| Self::escape_field(f)).collect();
        format!("{}\n", escaped.join(","))
    }

    /// Format a CSV data row from field values.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_row(&["main", "true", "5"]);
    /// assert_eq!(output, "main,true,5\n");
    /// ```
    pub fn format_row(values: &[&str]) -> String {
        let escaped: Vec<String> = values.iter().map(|v| Self::escape_field(v)).collect();
        format!("{}\n", escaped.join(","))
    }

    /// Format a simple message/status as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_message("key", "value");
    /// assert_eq!(output, "key\nvalue\n");
    /// ```
    pub fn format_message(key: &str, value: &str) -> String {
        format!("{}\n{}\n", Self::escape_field(key), Self::escape_field(value))
    }

    /// Format a key-value pair as CSV with header row.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_key_value("branch", "main");
    /// assert_eq!(output, "branch\nmain\n");
    /// ```
    pub fn format_key_value(key: &str, value: &str) -> String {
        format!("{}\n{}\n", Self::escape_field(key), Self::escape_field(value))
    }

    /// Format multiple key-value pairs as CSV with headers.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_object(&[("branch", "main"), ("is_clean", "true"), ("count", "5")]);
    /// assert!(output.contains("branch,is_clean,count"));
    /// assert!(output.contains("main,true,5"));
    /// ```
    pub fn format_object(pairs: &[(&str, &str)]) -> String {
        let headers: Vec<String> = pairs.iter().map(|(k, _)| Self::escape_field(k)).collect();
        let values: Vec<String> = pairs.iter().map(|(_, v)| Self::escape_field(v)).collect();
        format!("{}\n{}\n", headers.join(","), values.join(","))
    }

    /// Format a count summary as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_counts(&[("passed", 10), ("failed", 2)]);
    /// assert!(output.contains("passed,failed"));
    /// assert!(output.contains("10,2"));
    /// ```
    pub fn format_counts(counts: &[(&str, usize)]) -> String {
        let headers: Vec<String> = counts.iter().map(|(name, _)| Self::escape_field(name)).collect();
        let values: Vec<String> = counts.iter().map(|(_, count)| count.to_string()).collect();
        format!("{}\n{}\n", headers.join(","), values.join(","))
    }

    /// Format a section with items as CSV with headers.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_section("status", "path", &[("M", "src/main.rs"), ("A", "src/new.rs")]);
    /// assert!(output.contains("status,path"));
    /// assert!(output.contains("M,src/main.rs"));
    /// assert!(output.contains("A,src/new.rs"));
    /// ```
    pub fn format_section(status_col: &str, path_col: &str, items: &[(&str, &str)]) -> String {
        let mut output = format!("{}\n", Self::format_header(&[status_col, path_col]).trim());
        for (status, path) in items {
            output.push_str(&format!("{}\n", Self::format_row(&[status, path]).trim()));
        }
        output
    }

    /// Format a list item with status and path as CSV.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_item("M", "src/main.rs");
    /// assert_eq!(output, "M,src/main.rs\n");
    /// ```
    pub fn format_item(status: &str, path: &str) -> String {
        format!("{}\n", Self::format_row(&[status, path]).trim())
    }

    /// Format a list item with rename info as CSV.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_item_renamed("R", "old.rs", "new.rs");
    /// assert_eq!(output, "R,new.rs,old.rs\n");
    /// ```
    pub fn format_item_renamed(status: &str, old_path: &str, new_path: &str) -> String {
        format!("{}\n", Self::format_row(&[status, new_path, old_path]).trim())
    }

    /// Format a test result summary as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_test_summary(10, 2, 1, 1500);
    /// assert!(output.contains("passed,failed,skipped,total,duration_ms"));
    /// assert!(output.contains("10,2,1,13,1500"));
    /// ```
    pub fn format_test_summary(passed: usize, failed: usize, skipped: usize, duration_ms: u64) -> String {
        format!(
            "passed,failed,skipped,total,duration_ms\n{},{},{},{},{}\n",
            passed, failed, skipped, passed + failed + skipped, duration_ms
        )
    }

    /// Format a success/failure status as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_status(true);
    /// assert_eq!(output, "success\ntrue\n");
    /// ```
    pub fn format_status(success: bool) -> String {
        format!("success\n{}\n", success)
    }

    /// Format a list of failing tests as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_failures(&["test_one".to_string(), "test_two".to_string()]);
    /// assert!(output.contains("failure"));
    /// assert!(output.contains("test_one"));
    /// assert!(output.contains("test_two"));
    /// ```
    pub fn format_failures(failures: &[String]) -> String {
        let mut output = String::from("failure\n");
        for failure in failures {
            output.push_str(&format!("{}\n", Self::escape_field(failure)));
        }
        output
    }

    /// Format log level counts as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_log_levels(2, 5, 10, 3);
    /// assert!(output.contains("error,warn,info,debug,total"));
    /// assert!(output.contains("2,5,10,3,20"));
    /// ```
    pub fn format_log_levels(error: usize, warn: usize, info: usize, debug: usize) -> String {
        format!(
            "error,warn,info,debug,total\n{},{},{},{},{}\n",
            error, warn, info, debug, error + warn + info + debug
        )
    }

    /// Format a grep match as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_grep_match("src/main.rs", Some(42), "fn main()");
    /// assert!(output.contains("file,line,content"));
    /// assert!(output.contains("src/main.rs,42,fn main()"));
    /// ```
    pub fn format_grep_match(file: &str, line: Option<usize>, content: &str) -> String {
        match line {
            Some(l) => format!(
                "file,line,content\n{},{},{}\n",
                Self::escape_field(file),
                l,
                Self::escape_field(content.trim())
            ),
            None => format!(
                "file,line,content\n{},{},{}\n",
                Self::escape_field(file),
                "",
                Self::escape_field(content.trim())
            ),
        }
    }

    /// Format a grep file with match count as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_grep_file("src/main.rs", 5);
    /// assert_eq!(output, "file,match_count\nsrc/main.rs,5\n");
    /// ```
    pub fn format_grep_file(file: &str, match_count: usize) -> String {
        format!(
            "file,match_count\n{},{}\n",
            Self::escape_field(file),
            match_count
        )
    }

    /// Format a diff file entry as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_diff_file("src/main.rs", "M", 10, 5);
    /// assert_eq!(output, "path,change_type,additions,deletions\nsrc/main.rs,M,10,5\n");
    /// ```
    pub fn format_diff_file(path: &str, change_type: &str, additions: usize, deletions: usize) -> String {
        format!(
            "path,change_type,additions,deletions\n{},{},{},{}\n",
            Self::escape_field(path),
            change_type,
            additions,
            deletions
        )
    }

    /// Format a diff summary as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_diff_summary(3, 25, 10);
    /// assert_eq!(output, "files_changed,insertions,deletions\n3,25,10\n");
    /// ```
    pub fn format_diff_summary(files_changed: usize, insertions: usize, deletions: usize) -> String {
        format!(
            "files_changed,insertions,deletions\n{},{},{}\n",
            files_changed, insertions, deletions
        )
    }

    /// Format a clean state indicator as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_clean();
    /// assert_eq!(output, "is_clean\ntrue\n");
    /// ```
    pub fn format_clean() -> String {
        "is_clean\ntrue\n".to_string()
    }

    /// Format a dirty state indicator with counts as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_dirty(2, 3, 5, 0);
    /// assert_eq!(output, "is_clean,staged,unstaged,untracked,unmerged\nfalse,2,3,5,0\n");
    /// ```
    pub fn format_dirty(staged: usize, unstaged: usize, untracked: usize, unmerged: usize) -> String {
        format!(
            "is_clean,staged,unstaged,untracked,unmerged\nfalse,{},{},{},{}\n",
            staged, unstaged, untracked, unmerged
        )
    }

    /// Format branch info with ahead/behind as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_branch_with_tracking("main", 3, 2);
    /// assert_eq!(output, "branch,ahead,behind\nmain,3,2\n");
    /// ```
    pub fn format_branch_with_tracking(branch: &str, ahead: usize, behind: usize) -> String {
        format!(
            "branch,ahead,behind\n{},{},{}\n",
            Self::escape_field(branch),
            ahead,
            behind
        )
    }

    /// Format an empty result as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_empty();
    /// assert_eq!(output, "empty\ntrue\n");
    /// ```
    pub fn format_empty() -> String {
        "empty\ntrue\n".to_string()
    }

    /// Format a truncation warning as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_truncated(10, 50);
    /// assert_eq!(output, "is_truncated,shown,total\ntrue,10,50\n");
    /// ```
    pub fn format_truncated(shown: usize, total: usize) -> String {
        format!("is_truncated,shown,total\ntrue,{},{}\n", shown, total)
    }

    /// Format an error message as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_error("Something went wrong");
    /// assert!(output.contains("error,message"));
    /// assert!(output.contains("true,Something went wrong"));
    /// ```
    pub fn format_error(message: &str) -> String {
        format!(
            "error,message\ntrue,{}\n",
            Self::escape_field(message)
        )
    }

    /// Format an error with exit code as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_error_with_code("Command failed", 1);
    /// assert_eq!(output, "error,message,exit_code\ntrue,Command failed,1\n");
    /// ```
    pub fn format_error_with_code(message: &str, exit_code: i32) -> String {
        format!(
            "error,message,exit_code\ntrue,{},{}\n",
            Self::escape_field(message),
            exit_code
        )
    }

    /// Format a not-implemented message as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_not_implemented("Feature X");
    /// assert!(output.contains("not_implemented,message"));
    /// assert!(output.contains("true,Feature X"));
    /// ```
    pub fn format_not_implemented(message: &str) -> String {
        format!(
            "not_implemented,message\ntrue,{}\n",
            Self::escape_field(message)
        )
    }

    /// Format a command result as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_command_result("echo", &["hello".to_string(), "world".to_string()], "hello world\n", "", 0, 10);
    /// assert!(output.contains("command,args,stdout,stderr,exit_code,duration_ms"));
    /// ```
    pub fn format_command_result(
        command: &str,
        args: &[String],
        stdout: &str,
        stderr: &str,
        exit_code: i32,
        duration_ms: u64,
    ) -> String {
        let args_str = args.join(" ");
        format!(
            "command,args,stdout,stderr,exit_code,duration_ms\n{},{},{},{},{},{}\n",
            Self::escape_field(command),
            Self::escape_field(&args_str),
            Self::escape_field(stdout),
            Self::escape_field(stderr),
            exit_code,
            duration_ms
        )
    }

    /// Format a list of strings as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_list(&["file1.rs", "file2.rs"]);
    /// assert_eq!(output, "item\nfile1.rs\nfile2.rs\n");
    /// ```
    pub fn format_list(items: &[impl AsRef<str>]) -> String {
        let mut output = String::from("item\n");
        for item in items {
            output.push_str(&format!("{}\n", Self::escape_field(item.as_ref())));
        }
        output
    }

    /// Format a count as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_count(42);
    /// assert_eq!(output, "count\n42\n");
    /// ```
    pub fn format_count(count: usize) -> String {
        format!("count\n{}\n", count)
    }

    /// Format a boolean flag as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_flag("is_clean", true);
    /// assert_eq!(output, "is_clean\ntrue\n");
    /// ```
    pub fn format_flag(name: &str, value: bool) -> String {
        format!("{}\n{}\n", Self::escape_field(name), value)
    }

    /// Format items with multiple columns as CSV with custom headers.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let items = vec![
    ///     vec!["file1.rs", "M", "10"],
    ///     vec!["file2.rs", "A", "5"],
    /// ];
    /// let output = CsvFormatter::format_table(&["path", "status", "lines"], &items);
    /// assert!(output.contains("path,status,lines"));
    /// assert!(output.contains("file1.rs,M,10"));
    /// assert!(output.contains("file2.rs,A,5"));
    /// ```
    pub fn format_table(headers: &[&str], rows: &[Vec<&str>]) -> String {
        let mut output = format!("{}\n", Self::format_header(headers).trim());
        for row in rows {
            output.push_str(&format!("{}\n", Self::format_row(row).trim()));
        }
        output
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
#[allow(dead_code)]
pub struct TsvFormatter;

impl Formatter for TsvFormatter {
    fn name() -> &'static str {
        "tsv"
    }

    fn format() -> OutputFormat {
        OutputFormat::Tsv
    }
}

#[allow(dead_code)]
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

    /// Format a TSV header row from field names.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_header(&["branch", "is_clean", "count"]);
    /// assert_eq!(output, "branch\tis_clean\tcount\n");
    /// ```
    pub fn format_header(fields: &[&str]) -> String {
        let escaped: Vec<String> = fields.iter().map(|f| Self::escape_field(f)).collect();
        format!("{}\n", escaped.join("\t"))
    }

    /// Format a TSV data row from field values.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_row(&["main", "true", "5"]);
    /// assert_eq!(output, "main\ttrue\t5\n");
    /// ```
    pub fn format_row(values: &[&str]) -> String {
        let escaped: Vec<String> = values.iter().map(|v| Self::escape_field(v)).collect();
        format!("{}\n", escaped.join("\t"))
    }

    /// Format a simple message/status as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_message("key", "value");
    /// assert_eq!(output, "key\nvalue\n");
    /// ```
    pub fn format_message(key: &str, value: &str) -> String {
        format!("{}\n{}\n", Self::escape_field(key), Self::escape_field(value))
    }

    /// Format a key-value pair as TSV with header row.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_key_value("branch", "main");
    /// assert_eq!(output, "branch\nmain\n");
    /// ```
    pub fn format_key_value(key: &str, value: &str) -> String {
        format!("{}\n{}\n", Self::escape_field(key), Self::escape_field(value))
    }

    /// Format multiple key-value pairs as TSV with headers.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_object(&[("branch", "main"), ("is_clean", "true"), ("count", "5")]);
    /// assert!(output.contains("branch\tis_clean\tcount"));
    /// assert!(output.contains("main\ttrue\t5"));
    /// ```
    pub fn format_object(pairs: &[(&str, &str)]) -> String {
        let headers: Vec<String> = pairs.iter().map(|(k, _)| Self::escape_field(k)).collect();
        let values: Vec<String> = pairs.iter().map(|(_, v)| Self::escape_field(v)).collect();
        format!("{}\n{}\n", headers.join("\t"), values.join("\t"))
    }

    /// Format a count summary as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_counts(&[("passed", 10), ("failed", 2)]);
    /// assert!(output.contains("passed\tfailed"));
    /// assert!(output.contains("10\t2"));
    /// ```
    pub fn format_counts(counts: &[(&str, usize)]) -> String {
        let headers: Vec<String> = counts.iter().map(|(name, _)| Self::escape_field(name)).collect();
        let values: Vec<String> = counts.iter().map(|(_, count)| count.to_string()).collect();
        format!("{}\n{}\n", headers.join("\t"), values.join("\t"))
    }

    /// Format a section with items as TSV with headers.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_section("status", "path", &[("M", "src/main.rs"), ("A", "src/new.rs")]);
    /// assert!(output.contains("status\tpath"));
    /// assert!(output.contains("M\tsrc/main.rs"));
    /// assert!(output.contains("A\tsrc/new.rs"));
    /// ```
    pub fn format_section(status_col: &str, path_col: &str, items: &[(&str, &str)]) -> String {
        let mut output = format!("{}\n", Self::format_header(&[status_col, path_col]).trim());
        for (status, path) in items {
            output.push_str(&format!("{}\n", Self::format_row(&[status, path]).trim()));
        }
        output
    }

    /// Format a list item with status and path as TSV.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_item("M", "src/main.rs");
    /// assert_eq!(output, "M\tsrc/main.rs\n");
    /// ```
    pub fn format_item(status: &str, path: &str) -> String {
        format!("{}\n", Self::format_row(&[status, path]).trim())
    }

    /// Format a list item with rename info as TSV.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_item_renamed("R", "old.rs", "new.rs");
    /// assert_eq!(output, "R\tnew.rs\told.rs\n");
    /// ```
    pub fn format_item_renamed(status: &str, old_path: &str, new_path: &str) -> String {
        format!("{}\n", Self::format_row(&[status, new_path, old_path]).trim())
    }

    /// Format a test result summary as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_test_summary(10, 2, 1, 1500);
    /// assert!(output.contains("passed\tfailed\tskipped\ttotal\tduration_ms"));
    /// assert!(output.contains("10\t2\t1\t13\t1500"));
    /// ```
    pub fn format_test_summary(passed: usize, failed: usize, skipped: usize, duration_ms: u64) -> String {
        format!(
            "passed\tfailed\tskipped\ttotal\tduration_ms\n{}\t{}\t{}\t{}\t{}\n",
            passed, failed, skipped, passed + failed + skipped, duration_ms
        )
    }

    /// Format a success/failure status as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_status(true);
    /// assert_eq!(output, "success\ntrue\n");
    /// ```
    pub fn format_status(success: bool) -> String {
        format!("success\n{}\n", success)
    }

    /// Format a list of failing tests as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_failures(&["test_one".to_string(), "test_two".to_string()]);
    /// assert!(output.contains("failure"));
    /// assert!(output.contains("test_one"));
    /// assert!(output.contains("test_two"));
    /// ```
    pub fn format_failures(failures: &[String]) -> String {
        let mut output = String::from("failure\n");
        for failure in failures {
            output.push_str(&format!("{}\n", Self::escape_field(failure)));
        }
        output
    }

    /// Format log level counts as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_log_levels(2, 5, 10, 3);
    /// assert!(output.contains("error\twarn\tinfo\tdebug\ttotal"));
    /// assert!(output.contains("2\t5\t10\t3\t20"));
    /// ```
    pub fn format_log_levels(error: usize, warn: usize, info: usize, debug: usize) -> String {
        format!(
            "error\twarn\tinfo\tdebug\ttotal\n{}\t{}\t{}\t{}\t{}\n",
            error, warn, info, debug, error + warn + info + debug
        )
    }

    /// Format a grep match as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_grep_match("src/main.rs", Some(42), "fn main()");
    /// assert!(output.contains("file\tline\tcontent"));
    /// assert!(output.contains("src/main.rs\t42\tfn main()"));
    /// ```
    pub fn format_grep_match(file: &str, line: Option<usize>, content: &str) -> String {
        match line {
            Some(l) => format!(
                "file\tline\tcontent\n{}\t{}\t{}\n",
                Self::escape_field(file),
                l,
                Self::escape_field(content.trim())
            ),
            None => format!(
                "file\tline\tcontent\n{}\t\t{}\n",
                Self::escape_field(file),
                Self::escape_field(content.trim())
            ),
        }
    }

    /// Format a grep file with match count as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_grep_file("src/main.rs", 5);
    /// assert_eq!(output, "file\tmatch_count\nsrc/main.rs\t5\n");
    /// ```
    pub fn format_grep_file(file: &str, match_count: usize) -> String {
        format!(
            "file\tmatch_count\n{}\t{}\n",
            Self::escape_field(file),
            match_count
        )
    }

    /// Format a diff file entry as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_diff_file("src/main.rs", "M", 10, 5);
    /// assert_eq!(output, "path\tchange_type\tadditions\tdeletions\nsrc/main.rs\tM\t10\t5\n");
    /// ```
    pub fn format_diff_file(path: &str, change_type: &str, additions: usize, deletions: usize) -> String {
        format!(
            "path\tchange_type\tadditions\tdeletions\n{}\t{}\t{}\t{}\n",
            Self::escape_field(path),
            change_type,
            additions,
            deletions
        )
    }

    /// Format a diff summary as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_diff_summary(3, 25, 10);
    /// assert_eq!(output, "files_changed\tinsertions\tdeletions\n3\t25\t10\n");
    /// ```
    pub fn format_diff_summary(files_changed: usize, insertions: usize, deletions: usize) -> String {
        format!(
            "files_changed\tinsertions\tdeletions\n{}\t{}\t{}\n",
            files_changed, insertions, deletions
        )
    }

    /// Format a clean state indicator as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_clean();
    /// assert_eq!(output, "is_clean\ntrue\n");
    /// ```
    pub fn format_clean() -> String {
        "is_clean\ntrue\n".to_string()
    }

    /// Format a dirty state indicator with counts as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_dirty(2, 3, 5, 0);
    /// assert_eq!(output, "is_clean\tstaged\tunstaged\tuntracked\tunmerged\nfalse\t2\t3\t5\t0\n");
    /// ```
    pub fn format_dirty(staged: usize, unstaged: usize, untracked: usize, unmerged: usize) -> String {
        format!(
            "is_clean\tstaged\tunstaged\tuntracked\tunmerged\nfalse\t{}\t{}\t{}\t{}\n",
            staged, unstaged, untracked, unmerged
        )
    }

    /// Format branch info with ahead/behind as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_branch_with_tracking("main", 3, 2);
    /// assert_eq!(output, "branch\tahead\tbehind\nmain\t3\t2\n");
    /// ```
    pub fn format_branch_with_tracking(branch: &str, ahead: usize, behind: usize) -> String {
        format!(
            "branch\tahead\tbehind\n{}\t{}\t{}\n",
            Self::escape_field(branch),
            ahead,
            behind
        )
    }

    /// Format an empty result as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_empty();
    /// assert_eq!(output, "empty\ntrue\n");
    /// ```
    pub fn format_empty() -> String {
        "empty\ntrue\n".to_string()
    }

    /// Format a truncation warning as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_truncated(10, 50);
    /// assert_eq!(output, "is_truncated\tshown\ttotal\ntrue\t10\t50\n");
    /// ```
    pub fn format_truncated(shown: usize, total: usize) -> String {
        format!("is_truncated\tshown\ttotal\ntrue\t{}\t{}\n", shown, total)
    }

    /// Format an error message as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_error("Something went wrong");
    /// assert!(output.contains("error\tmessage"));
    /// assert!(output.contains("true\tSomething went wrong"));
    /// ```
    pub fn format_error(message: &str) -> String {
        format!(
            "error\tmessage\ntrue\t{}\n",
            Self::escape_field(message)
        )
    }

    /// Format an error with exit code as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_error_with_code("Command failed", 1);
    /// assert_eq!(output, "error\tmessage\texit_code\ntrue\tCommand failed\t1\n");
    /// ```
    pub fn format_error_with_code(message: &str, exit_code: i32) -> String {
        format!(
            "error\tmessage\texit_code\ntrue\t{}\t{}\n",
            Self::escape_field(message),
            exit_code
        )
    }

    /// Format a not-implemented message as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_not_implemented("Feature X");
    /// assert!(output.contains("not_implemented\tmessage"));
    /// assert!(output.contains("true\tFeature X"));
    /// ```
    pub fn format_not_implemented(message: &str) -> String {
        format!(
            "not_implemented\tmessage\ntrue\t{}\n",
            Self::escape_field(message)
        )
    }

    /// Format a command result as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_command_result("echo", &["hello".to_string(), "world".to_string()], "hello world\n", "", 0, 10);
    /// assert!(output.contains("command\targs\tstdout\tstderr\texit_code\tduration_ms"));
    /// ```
    pub fn format_command_result(
        command: &str,
        args: &[String],
        stdout: &str,
        stderr: &str,
        exit_code: i32,
        duration_ms: u64,
    ) -> String {
        let args_str = args.join(" ");
        format!(
            "command\targs\tstdout\tstderr\texit_code\tduration_ms\n{}\t{}\t{}\t{}\t{}\t{}\n",
            Self::escape_field(command),
            Self::escape_field(&args_str),
            Self::escape_field(stdout),
            Self::escape_field(stderr),
            exit_code,
            duration_ms
        )
    }

    /// Format a list of strings as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_list(&["file1.rs", "file2.rs"]);
    /// assert_eq!(output, "item\nfile1.rs\nfile2.rs\n");
    /// ```
    pub fn format_list(items: &[impl AsRef<str>]) -> String {
        let mut output = String::from("item\n");
        for item in items {
            output.push_str(&format!("{}\n", Self::escape_field(item.as_ref())));
        }
        output
    }

    /// Format a count as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_count(42);
    /// assert_eq!(output, "count\n42\n");
    /// ```
    pub fn format_count(count: usize) -> String {
        format!("count\n{}\n", count)
    }

    /// Format a boolean flag as TSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let output = TsvFormatter::format_flag("is_clean", true);
    /// assert_eq!(output, "is_clean\ntrue\n");
    /// ```
    pub fn format_flag(name: &str, value: bool) -> String {
        format!("{}\n{}\n", Self::escape_field(name), value)
    }

    /// Format items with multiple columns as TSV with custom headers.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::TsvFormatter;
    /// let items = vec![
    ///     vec!["file1.rs", "M", "10"],
    ///     vec!["file2.rs", "A", "5"],
    /// ];
    /// let output = TsvFormatter::format_table(&["path", "status", "lines"], &items);
    /// assert!(output.contains("path\tstatus\tlines"));
    /// assert!(output.contains("file1.rs\tM\t10"));
    /// assert!(output.contains("file2.rs\tA\t5"));
    /// ```
    pub fn format_table(headers: &[&str], rows: &[Vec<&str>]) -> String {
        let mut output = format!("{}\n", Self::format_header(headers).trim());
        for row in rows {
            output.push_str(&format!("{}\n", Self::format_row(row).trim()));
        }
        output
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
/// - Uses concise key-value pairs
/// - Groups related data with headers
#[allow(dead_code)]
pub struct AgentFormatter;

impl Formatter for AgentFormatter {
    fn name() -> &'static str {
        "agent"
    }

    fn format() -> OutputFormat {
        OutputFormat::Agent
    }
}

#[allow(dead_code)]
impl AgentFormatter {
    /// Format a section header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::section_header("Files");
    /// assert_eq!(output, "## Files\n");
    /// ```
    pub fn section_header(title: &str) -> String {
        format!("## {}\n", title)
    }

    /// Format a subsection header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::subsection_header("Details");
    /// assert_eq!(output, "### Details\n");
    /// ```
    pub fn subsection_header(title: &str) -> String {
        format!("### {}\n", title)
    }

    /// Format a list item with optional label.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::list_item("file.rs", None);
    /// assert_eq!(output, "- file.rs\n");
    /// let output = AgentFormatter::list_item("file.rs", Some("modified"));
    /// assert_eq!(output, "- file.rs [modified]\n");
    /// ```
    pub fn list_item(item: &str, label: Option<&str>) -> String {
        match label {
            Some(l) => format!("- {} [{}]\n", item, l),
            None => format!("- {}\n", item),
        }
    }

    /// Format a key-value item with optional label.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::key_value_item("branch", "main", None);
    /// assert_eq!(output, "- branch: main\n");
    /// let output = AgentFormatter::key_value_item("count", "5", Some("files"));
    /// assert_eq!(output, "- count [files]: 5\n");
    /// ```
    pub fn key_value_item(key: &str, value: &str, label: Option<&str>) -> String {
        match label {
            Some(l) => format!("- {} [{}]: {}\n", key, l, value),
            None => format!("- {}: {}\n", key, value),
        }
    }

    /// Format a simple message/status line.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_message("branch", "main");
    /// assert_eq!(output, "- branch: main\n");
    /// ```
    pub fn format_message(key: &str, value: &str) -> String {
        format!("- {}: {}\n", key, value)
    }

    /// Format a count summary line.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_counts("counts", &[("passed", 10), ("failed", 2)]);
    /// assert_eq!(output, "- counts: passed=10 failed=2\n");
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
            format!("- {}: {}\n", label, parts.join(" "))
        }
    }

    /// Format a section header with an optional count.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_section_header("staged", Some(3));
    /// assert_eq!(output, "## staged (3)\n");
    /// let output = AgentFormatter::format_section_header("files", None);
    /// assert_eq!(output, "## files\n");
    /// ```
    pub fn format_section_header(name: &str, count: Option<usize>) -> String {
        match count {
            Some(c) => format!("## {} ({})\n", name, c),
            None => format!("## {}\n", name),
        }
    }

    /// Format an indented list item.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_item("M", "src/main.rs");
    /// assert_eq!(output, "  - [M] src/main.rs\n");
    /// ```
    pub fn format_item(status: &str, path: &str) -> String {
        format!("  - [{}] {}\n", status, path)
    }

    /// Format an indented list item with rename info.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_item_renamed("R", "old.rs", "new.rs");
    /// assert_eq!(output, "  - [R] old.rs -> new.rs\n");
    /// ```
    pub fn format_item_renamed(status: &str, old_path: &str, new_path: &str) -> String {
        format!("  - [{}] {} -> {}\n", status, old_path, new_path)
    }

    /// Format a test result summary.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_test_summary(10, 2, 1, 1500);
    /// assert!(output.contains("passed: 10"));
    /// assert!(output.contains("failed: 2"));
    /// assert!(output.contains("duration: 1.50s"));
    /// ```
    pub fn format_test_summary(passed: usize, failed: usize, skipped: usize, duration_ms: u64) -> String {
        let mut output = String::new();
        output.push_str("## Test Results\n");
        output.push_str(&format!("- passed: {}\n", passed));
        output.push_str(&format!("- failed: {}\n", failed));
        output.push_str(&format!("- skipped: {}\n", skipped));
        output.push_str(&format!("- total: {}\n", passed + failed + skipped));
        output.push_str(&format!("- duration: {}\n", format_duration(duration_ms)));
        output
    }

    /// Format a success/failure indicator.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_status(true);
    /// assert_eq!(output, "- status: passed\n");
    /// let output = AgentFormatter::format_status(false);
    /// assert_eq!(output, "- status: failed\n");
    /// ```
    pub fn format_status(success: bool) -> String {
        format!("- status: {}\n", if success { "passed" } else { "failed" })
    }

    /// Format a list of failing tests.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let failures = vec!["test_one".to_string(), "test_two".to_string()];
    /// let output = AgentFormatter::format_failures(&failures);
    /// assert!(output.contains("## Failures"));
    /// assert!(output.contains("test_one"));
    /// assert!(output.contains("test_two"));
    /// ```
    pub fn format_failures(failures: &[String]) -> String {
        let mut output = String::new();
        if !failures.is_empty() {
            output.push_str(&format!("## Failures ({})\n", failures.len()));
            for failure in failures {
                output.push_str(&format!("- {}\n", failure));
            }
        }
        output
    }

    /// Format log level counts.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_log_levels(2, 5, 10, 3);
    /// assert!(output.contains("error: 2"));
    /// assert!(output.contains("warn: 5"));
    /// assert!(output.contains("info: 10"));
    /// assert!(output.contains("debug: 3"));
    /// ```
    pub fn format_log_levels(error: usize, warn: usize, info: usize, debug: usize) -> String {
        let mut output = String::new();
        output.push_str("## Log Levels\n");
        output.push_str(&format!("- error: {}\n", error));
        output.push_str(&format!("- warn: {}\n", warn));
        output.push_str(&format!("- info: {}\n", info));
        output.push_str(&format!("- debug: {}\n", debug));
        output.push_str(&format!("- total: {}\n", error + warn + info + debug));
        output
    }

    /// Format a grep match line.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_grep_match("src/main.rs", Some(42), "fn main()");
    /// assert!(output.contains("file: src/main.rs"));
    /// assert!(output.contains("line: 42"));
    /// assert!(output.contains("content: fn main()"));
    /// ```
    pub fn format_grep_match(file: &str, line: Option<usize>, content: &str) -> String {
        let mut output = String::new();
        output.push_str(&format!("- file: {}\n", file));
        if let Some(l) = line {
            output.push_str(&format!("  line: {}\n", l));
        }
        output.push_str(&format!("  content: {}\n", content.trim()));
        output
    }

    /// Format a grep file header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_grep_file("src/main.rs", 5);
    /// assert_eq!(output, "### src/main.rs (5 matches)\n");
    /// ```
    pub fn format_grep_file(file: &str, match_count: usize) -> String {
        format!("### {} ({} matches)\n", file, match_count)
    }

    /// Format a diff file entry.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_diff_file("src/main.rs", "M", 10, 5);
    /// assert!(output.contains("[M] src/main.rs"));
    /// assert!(output.contains("added: 10"));
    /// assert!(output.contains("removed: 5"));
    /// ```
    pub fn format_diff_file(path: &str, change_type: &str, additions: usize, deletions: usize) -> String {
        format!("- [{}] {} (+{} -{})\n", change_type, path, additions, deletions)
    }

    /// Format a diff summary.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_diff_summary(3, 25, 10);
    /// assert!(output.contains("files changed: 3"));
    /// assert!(output.contains("insertions: 25"));
    /// assert!(output.contains("deletions: 10"));
    /// ```
    pub fn format_diff_summary(files_changed: usize, insertions: usize, deletions: usize) -> String {
        let mut output = String::new();
        output.push_str("## Diff Summary\n");
        output.push_str(&format!("- files changed: {}\n", files_changed));
        output.push_str(&format!("- insertions: {}\n", insertions));
        output.push_str(&format!("- deletions: {}\n", deletions));
        output
    }

    /// Format a clean state indicator.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_clean();
    /// assert_eq!(output, "- status: clean\n");
    /// ```
    pub fn format_clean() -> String {
        "- status: clean\n".to_string()
    }

    /// Format a dirty state indicator with counts.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_dirty(2, 3, 5, 0);
    /// assert!(output.contains("status: dirty"));
    /// assert!(output.contains("staged: 2"));
    /// assert!(output.contains("unstaged: 3"));
    /// ```
    pub fn format_dirty(staged: usize, unstaged: usize, untracked: usize, unmerged: usize) -> String {
        let mut output = String::new();
        output.push_str("- status: dirty\n");
        output.push_str(&format!("- staged: {}\n", staged));
        output.push_str(&format!("- unstaged: {}\n", unstaged));
        output.push_str(&format!("- untracked: {}\n", untracked));
        output.push_str(&format!("- unmerged: {}\n", unmerged));
        output
    }

    /// Format branch info with ahead/behind.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_branch_with_tracking("main", 3, 2);
    /// assert!(output.contains("branch: main"));
    /// assert!(output.contains("ahead: 3"));
    /// assert!(output.contains("behind: 2"));
    /// ```
    pub fn format_branch_with_tracking(branch: &str, ahead: usize, behind: usize) -> String {
        let mut output = String::new();
        output.push_str(&format!("- branch: {}\n", branch));
        if ahead > 0 || behind > 0 {
            output.push_str(&format!("- ahead: {}\n", ahead));
            output.push_str(&format!("- behind: {}\n", behind));
        }
        output
    }

    /// Format an empty result.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_empty();
    /// assert_eq!(output, "- result: empty\n");
    /// ```
    pub fn format_empty() -> String {
        "- result: empty\n".to_string()
    }

    /// Format a truncation warning.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_truncated(10, 50);
    /// assert!(output.contains("truncated: true"));
    /// assert!(output.contains("shown: 10"));
    /// assert!(output.contains("total: 50"));
    /// ```
    pub fn format_truncated(shown: usize, total: usize) -> String {
        format!("- truncated: showing {} of {}\n", shown, total)
    }

    /// Format an error message.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_error("Something went wrong");
    /// assert!(output.contains("error: Something went wrong"));
    /// ```
    pub fn format_error(message: &str) -> String {
        format!("- error: {}\n", message)
    }

    /// Format an error with exit code.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_error_with_code("Command failed", 1);
    /// assert!(output.contains("error: Command failed"));
    /// assert!(output.contains("exit_code: 1"));
    /// ```
    pub fn format_error_with_code(message: &str, exit_code: i32) -> String {
        format!("- error: {}\n- exit_code: {}\n", message, exit_code)
    }

    /// Format a not-implemented message.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_not_implemented("Feature X");
    /// assert!(output.contains("not_implemented: Feature X"));
    /// ```
    pub fn format_not_implemented(message: &str) -> String {
        format!("- not_implemented: {}\n", message)
    }

    /// Format a command result.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_command_result(
    ///     "echo",
    ///     &["hello".to_string(), "world".to_string()],
    ///     "hello world\n",
    ///     "",
    ///     0,
    ///     10,
    /// );
    /// assert!(output.contains("command: echo"));
    /// assert!(output.contains("exit_code: 0"));
    /// assert!(output.contains("duration_ms: 10"));
    /// ```
    pub fn format_command_result(
        command: &str,
        args: &[String],
        stdout: &str,
        stderr: &str,
        exit_code: i32,
        duration_ms: u64,
    ) -> String {
        let mut output = String::new();
        output.push_str("## Command Result\n");
        output.push_str(&format!("- command: {}\n", command));
        if !args.is_empty() {
            output.push_str(&format!("- args: {}\n", args.join(" ")));
        }
        output.push_str(&format!("- exit_code: {}\n", exit_code));
        output.push_str(&format!("- duration_ms: {}\n", duration_ms));
        if !stdout.is_empty() {
            output.push_str("### stdout\n");
            output.push_str(&format!("```\n{}```\n", stdout));
        }
        if !stderr.is_empty() {
            output.push_str("### stderr\n");
            output.push_str(&format!("```\n{}```\n", stderr));
        }
        output
    }

    /// Format a list of strings.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_list(&["file1.rs", "file2.rs"]);
    /// assert!(output.contains("- file1.rs"));
    /// assert!(output.contains("- file2.rs"));
    /// ```
    pub fn format_list(items: &[impl AsRef<str>]) -> String {
        items
            .iter()
            .map(|s| format!("- {}\n", s.as_ref()))
            .collect()
    }

    /// Format a count.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_count(42);
    /// assert_eq!(output, "- count: 42\n");
    /// ```
    pub fn format_count(count: usize) -> String {
        format!("- count: {}\n", count)
    }

    /// Format a boolean flag.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_flag("is_clean", true);
    /// assert_eq!(output, "- is_clean: true\n");
    /// ```
    pub fn format_flag(name: &str, value: bool) -> String {
        format!("- {}: {}\n", name, value)
    }

    /// Format an array of objects.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let items = vec!["item1", "item2", "item3"];
    /// let output = AgentFormatter::format_array(&items);
    /// assert!(output.contains("- item1"));
    /// assert!(output.contains("- item2"));
    /// assert!(output.contains("- item3"));
    /// ```
    pub fn format_array(items: &[impl AsRef<str>]) -> String {
        Self::format_list(items)
    }

    /// Format a table with headers.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let items = vec![
    ///     vec!["file1.rs", "M", "10"],
    ///     vec!["file2.rs", "A", "5"],
    /// ];
    /// let output = AgentFormatter::format_table(&["path", "status", "lines"], &items);
    /// assert!(output.contains("| path | status | lines |"));
    /// assert!(output.contains("| file1.rs | M | 10 |"));
    /// ```
    pub fn format_table(headers: &[&str], rows: &[Vec<&str>]) -> String {
        let mut output = String::new();

        // Header row
        output.push_str(&format!("| {} |\n", headers.join(" | ")));

        // Separator row
        output.push_str(&format!("| {} |\n", headers.iter().map(|_| "---").collect::<Vec<_>>().join(" | ")));

        // Data rows
        for row in rows {
            output.push_str(&format!("| {} |\n", row.join(" | ")));
        }

        output
    }

    /// Format a key-value pair.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_key_value("branch", "main");
    /// assert_eq!(output, "- branch: main\n");
    /// ```
    pub fn format_key_value(key: &str, value: &str) -> String {
        format!("- {}: {}\n", key, value)
    }

    /// Format a metadata block.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_metadata(&[
    ///     ("branch", "main"),
    ///     ("is_clean", "true"),
    /// ]);
    /// assert!(output.contains("## Metadata"));
    /// assert!(output.contains("branch: main"));
    /// assert!(output.contains("is_clean: true"));
    /// ```
    pub fn format_metadata(items: &[(&str, &str)]) -> String {
        let mut output = String::new();
        output.push_str("## Metadata\n");
        for (key, value) in items {
            output.push_str(&format!("- {}: {}\n", key, value));
        }
        output
    }

    /// Format a code block.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_code_block("fn main() {}", Some("rust"));
    /// assert!(output.contains("```rust"));
    /// assert!(output.contains("fn main() {}"));
    /// assert!(output.contains("```"));
    /// ```
    pub fn format_code_block(code: &str, language: Option<&str>) -> String {
        match language {
            Some(lang) => format!("```{}\n{}\n```\n", lang, code),
            None => format!("```\n{}\n```\n", code),
        }
    }

    /// Format a divider line.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::AgentFormatter;
    /// let output = AgentFormatter::format_divider();
    /// assert_eq!(output, "---\n");
    /// ```
    pub fn format_divider() -> String {
        "---\n".to_string()
    }

    /// Format a bold text.
    pub fn format_bold(text: &str) -> String {
        format!("**{}**", text)
    }

    /// Format an italic text.
    pub fn format_italic(text: &str) -> String {
        format!("*{}*", text)
    }

    /// Format a code inline.
    pub fn format_code_inline(text: &str) -> String {
        format!("`{}`", text)
    }

    /// Format a link.
    pub fn format_link(text: &str, url: &str) -> String {
        format!("[{}]({})", text, url)
    }

    /// Start a new output document.
    pub fn start_document(title: &str) -> String {
        format!("# {}\n\n", title)
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
#[allow(dead_code)]
pub struct RawFormatter;

impl Formatter for RawFormatter {
    fn name() -> &'static str {
        "raw"
    }

    fn format() -> OutputFormat {
        OutputFormat::Raw
    }
}

#[allow(dead_code)]
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
#[allow(dead_code)]
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
    // JSON Formatter Tests
    // ============================================================

    #[test]
    fn test_json_format_message() {
        let output = JsonFormatter::format_message("branch", "main");
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["branch"], "main");
    }

    #[test]
    fn test_json_format_key_value() {
        let output = JsonFormatter::format_key_value("count", 42);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["count"], 42);
    }

    #[test]
    fn test_json_format_object() {
        let output = JsonFormatter::format_object(&[
            ("branch", serde_json::json!("main")),
            ("is_clean", serde_json::json!(true)),
            ("count", serde_json::json!(5)),
        ]);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["branch"], "main");
        assert_eq!(json["is_clean"], true);
        assert_eq!(json["count"], 5);
    }

    #[test]
    fn test_json_format_counts() {
        let output = JsonFormatter::format_counts(&[("passed", 10), ("failed", 2)]);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["passed"], 10);
        assert_eq!(json["failed"], 2);
    }

    #[test]
    fn test_json_format_counts_with_zeros() {
        // Unlike compact, JSON includes zero counts
        let output = JsonFormatter::format_counts(&[("passed", 0), ("failed", 2)]);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["passed"], 0);
        assert_eq!(json["failed"], 2);
    }

    #[test]
    fn test_json_format_section() {
        let items = vec!["file1.rs", "file2.rs"];
        let output = JsonFormatter::format_section("staged", &items);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(json["staged"].is_array());
        assert_eq!(json["staged"][0], "file1.rs");
        assert_eq!(json["staged"][1], "file2.rs");
    }

    #[test]
    fn test_json_format_item() {
        let output = JsonFormatter::format_item("M", "src/main.rs");
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["status"], "M");
        assert_eq!(json["path"], "src/main.rs");
    }

    #[test]
    fn test_json_format_item_renamed() {
        let output = JsonFormatter::format_item_renamed("R", "old.rs", "new.rs");
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["status"], "R");
        assert_eq!(json["path"], "new.rs");
        assert_eq!(json["old_path"], "old.rs");
    }

    #[test]
    fn test_json_format_test_summary() {
        let output = JsonFormatter::format_test_summary(10, 2, 1, 1500);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["passed"], 10);
        assert_eq!(json["failed"], 2);
        assert_eq!(json["skipped"], 1);
        assert_eq!(json["total"], 13);
        assert_eq!(json["duration_ms"], 1500);
    }

    #[test]
    fn test_json_format_status() {
        let success_output = JsonFormatter::format_status(true);
        let success_json: serde_json::Value = serde_json::from_str(&success_output).unwrap();
        assert_eq!(success_json["success"], true);

        let failure_output = JsonFormatter::format_status(false);
        let failure_json: serde_json::Value = serde_json::from_str(&failure_output).unwrap();
        assert_eq!(failure_json["success"], false);
    }

    #[test]
    fn test_json_format_failures() {
        let failures = vec!["test_one".to_string(), "test_two".to_string()];
        let output = JsonFormatter::format_failures(&failures);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(json["failures"].is_array());
        assert_eq!(json["count"], 2);
    }

    #[test]
    fn test_json_format_failures_empty() {
        let failures: Vec<String> = vec![];
        let output = JsonFormatter::format_failures(&failures);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(json["failures"].is_array());
        assert_eq!(json["count"], 0);
    }

    #[test]
    fn test_json_format_log_levels() {
        let output = JsonFormatter::format_log_levels(2, 5, 10, 3);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["error"], 2);
        assert_eq!(json["warn"], 5);
        assert_eq!(json["info"], 10);
        assert_eq!(json["debug"], 3);
        assert_eq!(json["total"], 20);
    }

    #[test]
    fn test_json_format_log_levels_with_zeros() {
        let output = JsonFormatter::format_log_levels(0, 5, 0, 0);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["error"], 0);
        assert_eq!(json["warn"], 5);
        assert_eq!(json["total"], 5);
    }

    #[test]
    fn test_json_format_grep_match() {
        let output = JsonFormatter::format_grep_match("src/main.rs", Some(42), "fn main()");
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["file"], "src/main.rs");
        assert_eq!(json["line"], 42);
        assert_eq!(json["content"], "fn main()");
    }

    #[test]
    fn test_json_format_grep_match_no_line() {
        let output = JsonFormatter::format_grep_match("src/main.rs", None, "match found");
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["file"], "src/main.rs");
        assert!(json["line"].is_null());
    }

    #[test]
    fn test_json_format_grep_file() {
        let output = JsonFormatter::format_grep_file("src/main.rs", 5);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["file"], "src/main.rs");
        assert_eq!(json["match_count"], 5);
    }

    #[test]
    fn test_json_format_diff_file() {
        let output = JsonFormatter::format_diff_file("src/main.rs", "M", 10, 5);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["path"], "src/main.rs");
        assert_eq!(json["change_type"], "M");
        assert_eq!(json["additions"], 10);
        assert_eq!(json["deletions"], 5);
    }

    #[test]
    fn test_json_format_diff_summary() {
        let output = JsonFormatter::format_diff_summary(3, 25, 10);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["files_changed"], 3);
        assert_eq!(json["insertions"], 25);
        assert_eq!(json["deletions"], 10);
    }

    #[test]
    fn test_json_format_clean() {
        let output = JsonFormatter::format_clean();
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["is_clean"], true);
    }

    #[test]
    fn test_json_format_dirty() {
        let output = JsonFormatter::format_dirty(2, 3, 5, 0);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["is_clean"], false);
        assert_eq!(json["staged"], 2);
        assert_eq!(json["unstaged"], 3);
        assert_eq!(json["untracked"], 5);
        assert_eq!(json["unmerged"], 0);
    }

    #[test]
    fn test_json_format_branch_with_tracking() {
        // No tracking
        let output = JsonFormatter::format_branch_with_tracking("main", 0, 0);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["branch"], "main");
        assert_eq!(json["ahead"], 0);
        assert_eq!(json["behind"], 0);

        // With tracking
        let output = JsonFormatter::format_branch_with_tracking("feature", 3, 2);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["branch"], "feature");
        assert_eq!(json["ahead"], 3);
        assert_eq!(json["behind"], 2);
    }

    #[test]
    fn test_json_format_empty() {
        let output = JsonFormatter::format_empty();
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["empty"], true);
    }

    #[test]
    fn test_json_format_truncated() {
        let output = JsonFormatter::format_truncated(10, 50);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["is_truncated"], true);
        assert_eq!(json["shown"], 10);
        assert_eq!(json["total"], 50);
    }

    #[test]
    fn test_json_format_error() {
        let output = JsonFormatter::format_error("Something went wrong");
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["error"], true);
        assert_eq!(json["message"], "Something went wrong");
    }

    #[test]
    fn test_json_format_error_with_code() {
        let output = JsonFormatter::format_error_with_code("Command failed", 1);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["error"], true);
        assert_eq!(json["message"], "Command failed");
        assert_eq!(json["exit_code"], 1);
    }

    #[test]
    fn test_json_format_not_implemented() {
        let output = JsonFormatter::format_not_implemented("Feature X");
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["not_implemented"], true);
        assert_eq!(json["message"], "Feature X");
    }

    #[test]
    fn test_json_format_command_result() {
        let output = JsonFormatter::format_command_result(
            "echo",
            &["hello".to_string(), "world".to_string()],
            "hello world\n",
            "",
            0,
            10,
        );
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["command"], "echo");
        assert!(json["args"].is_array());
        assert_eq!(json["stdout"], "hello world\n");
        assert_eq!(json["stderr"], "");
        assert_eq!(json["exit_code"], 0);
        assert_eq!(json["duration_ms"], 10);
    }

    #[test]
    fn test_json_format_list() {
        let items = vec!["file1.rs", "file2.rs"];
        let output = JsonFormatter::format_list(&items);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(json.is_array());
        assert_eq!(json[0], "file1.rs");
        assert_eq!(json[1], "file2.rs");
    }

    #[test]
    fn test_json_format_count() {
        let output = JsonFormatter::format_count(42);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["count"], 42);
    }

    #[test]
    fn test_json_format_flag() {
        let output = JsonFormatter::format_flag("is_clean", true);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["is_clean"], true);
    }

    #[test]
    fn test_json_format_array() {
        #[derive(serde::Serialize)]
        struct Item {
            name: &'static str,
            value: usize,
        }
        let items = vec![
            Item { name: "first", value: 1 },
            Item { name: "second", value: 2 },
        ];
        let output = JsonFormatter::format_array(&items);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(json.is_array());
        assert_eq!(json[0]["name"], "first");
        assert_eq!(json[1]["value"], 2);
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
        assert_eq!(CsvFormatter::escape_field("with\rcarriage"), "\"with\rcarriage\"");
    }

    #[test]
    fn test_csv_format_header() {
        let output = CsvFormatter::format_header(&["branch", "is_clean", "count"]);
        assert_eq!(output, "branch,is_clean,count\n");
    }

    #[test]
    fn test_csv_format_header_with_special_chars() {
        let output = CsvFormatter::format_header(&["branch", "has,comma", "normal"]);
        assert_eq!(output, "branch,\"has,comma\",normal\n");
    }

    #[test]
    fn test_csv_format_row() {
        let output = CsvFormatter::format_row(&["main", "true", "5"]);
        assert_eq!(output, "main,true,5\n");
    }

    #[test]
    fn test_csv_format_row_with_special_chars() {
        let output = CsvFormatter::format_row(&["main", "has,comma", "5"]);
        assert_eq!(output, "main,\"has,comma\",5\n");
    }

    #[test]
    fn test_csv_format_message() {
        let output = CsvFormatter::format_message("branch", "main");
        assert_eq!(output, "branch\nmain\n");
    }

    #[test]
    fn test_csv_format_key_value() {
        let output = CsvFormatter::format_key_value("branch", "main");
        assert_eq!(output, "branch\nmain\n");
    }

    #[test]
    fn test_csv_format_object() {
        let output = CsvFormatter::format_object(&[("branch", "main"), ("is_clean", "true"), ("count", "5")]);
        assert!(output.contains("branch,is_clean,count"));
        assert!(output.contains("main,true,5"));
    }

    #[test]
    fn test_csv_format_counts() {
        let output = CsvFormatter::format_counts(&[("passed", 10), ("failed", 2)]);
        assert!(output.contains("passed,failed"));
        assert!(output.contains("10,2"));
    }

    #[test]
    fn test_csv_format_counts_with_zeros() {
        let output = CsvFormatter::format_counts(&[("passed", 0), ("failed", 2)]);
        assert!(output.contains("passed,failed"));
        assert!(output.contains("0,2"));
    }

    #[test]
    fn test_csv_format_section() {
        let output = CsvFormatter::format_section("status", "path", &[("M", "src/main.rs"), ("A", "src/new.rs")]);
        assert!(output.contains("status,path"));
        assert!(output.contains("M,src/main.rs"));
        assert!(output.contains("A,src/new.rs"));
    }

    #[test]
    fn test_csv_format_item() {
        let output = CsvFormatter::format_item("M", "src/main.rs");
        assert_eq!(output, "M,src/main.rs\n");
    }

    #[test]
    fn test_csv_format_item_renamed() {
        let output = CsvFormatter::format_item_renamed("R", "old.rs", "new.rs");
        assert_eq!(output, "R,new.rs,old.rs\n");
    }

    #[test]
    fn test_csv_format_test_summary() {
        let output = CsvFormatter::format_test_summary(10, 2, 1, 1500);
        assert!(output.contains("passed,failed,skipped,total,duration_ms"));
        assert!(output.contains("10,2,1,13,1500"));
    }

    #[test]
    fn test_csv_format_test_summary_only_passed() {
        let output = CsvFormatter::format_test_summary(5, 0, 0, 500);
        assert!(output.contains("passed,failed,skipped,total,duration_ms"));
        assert!(output.contains("5,0,0,5,500"));
    }

    #[test]
    fn test_csv_format_status() {
        let success_output = CsvFormatter::format_status(true);
        assert_eq!(success_output, "success\ntrue\n");

        let failure_output = CsvFormatter::format_status(false);
        assert_eq!(failure_output, "success\nfalse\n");
    }

    #[test]
    fn test_csv_format_failures() {
        let failures = vec!["test_one".to_string(), "test_two".to_string()];
        let output = CsvFormatter::format_failures(&failures);
        assert!(output.contains("failure"));
        assert!(output.contains("test_one"));
        assert!(output.contains("test_two"));
    }

    #[test]
    fn test_csv_format_failures_empty() {
        let failures: Vec<String> = vec![];
        let output = CsvFormatter::format_failures(&failures);
        assert_eq!(output, "failure\n");
    }

    #[test]
    fn test_csv_format_log_levels() {
        let output = CsvFormatter::format_log_levels(2, 5, 10, 3);
        assert!(output.contains("error,warn,info,debug,total"));
        assert!(output.contains("2,5,10,3,20"));
    }

    #[test]
    fn test_csv_format_log_levels_with_zeros() {
        let output = CsvFormatter::format_log_levels(0, 5, 0, 0);
        assert!(output.contains("error,warn,info,debug,total"));
        assert!(output.contains("0,5,0,0,5"));
    }

    #[test]
    fn test_csv_format_grep_match() {
        let output = CsvFormatter::format_grep_match("src/main.rs", Some(42), "fn main()");
        assert!(output.contains("file,line,content"));
        assert!(output.contains("src/main.rs,42,fn main()"));
    }

    #[test]
    fn test_csv_format_grep_match_no_line() {
        let output = CsvFormatter::format_grep_match("src/main.rs", None, "match found");
        assert!(output.contains("file,line,content"));
        assert!(output.contains("src/main.rs,,match found"));
    }

    #[test]
    fn test_csv_format_grep_file() {
        let output = CsvFormatter::format_grep_file("src/main.rs", 5);
        assert_eq!(output, "file,match_count\nsrc/main.rs,5\n");
    }

    #[test]
    fn test_csv_format_diff_file() {
        let output = CsvFormatter::format_diff_file("src/main.rs", "M", 10, 5);
        assert_eq!(output, "path,change_type,additions,deletions\nsrc/main.rs,M,10,5\n");
    }

    #[test]
    fn test_csv_format_diff_summary() {
        let output = CsvFormatter::format_diff_summary(3, 25, 10);
        assert_eq!(output, "files_changed,insertions,deletions\n3,25,10\n");
    }

    #[test]
    fn test_csv_format_clean() {
        let output = CsvFormatter::format_clean();
        assert_eq!(output, "is_clean\ntrue\n");
    }

    #[test]
    fn test_csv_format_dirty() {
        let output = CsvFormatter::format_dirty(2, 3, 5, 0);
        assert_eq!(output, "is_clean,staged,unstaged,untracked,unmerged\nfalse,2,3,5,0\n");
    }

    #[test]
    fn test_csv_format_branch_with_tracking() {
        // No tracking
        let output = CsvFormatter::format_branch_with_tracking("main", 0, 0);
        assert_eq!(output, "branch,ahead,behind\nmain,0,0\n");

        // With tracking
        let output = CsvFormatter::format_branch_with_tracking("feature", 3, 2);
        assert_eq!(output, "branch,ahead,behind\nfeature,3,2\n");
    }

    #[test]
    fn test_csv_format_empty() {
        let output = CsvFormatter::format_empty();
        assert_eq!(output, "empty\ntrue\n");
    }

    #[test]
    fn test_csv_format_truncated() {
        let output = CsvFormatter::format_truncated(10, 50);
        assert_eq!(output, "is_truncated,shown,total\ntrue,10,50\n");
    }

    #[test]
    fn test_csv_format_error() {
        let output = CsvFormatter::format_error("Something went wrong");
        assert!(output.contains("error,message"));
        assert!(output.contains("true,Something went wrong"));
    }

    #[test]
    fn test_csv_format_error_with_code() {
        let output = CsvFormatter::format_error_with_code("Command failed", 1);
        assert_eq!(output, "error,message,exit_code\ntrue,Command failed,1\n");
    }

    #[test]
    fn test_csv_format_not_implemented() {
        let output = CsvFormatter::format_not_implemented("Feature X");
        assert!(output.contains("not_implemented,message"));
        assert!(output.contains("true,Feature X"));
    }

    #[test]
    fn test_csv_format_command_result() {
        let output = CsvFormatter::format_command_result(
            "echo",
            &["hello".to_string(), "world".to_string()],
            "hello world\n",
            "",
            0,
            10,
        );
        assert!(output.contains("command,args,stdout,stderr,exit_code,duration_ms"));
        assert!(output.contains("echo"));
        assert!(output.contains("hello world"));
    }

    #[test]
    fn test_csv_format_list() {
        let items = vec!["file1.rs", "file2.rs"];
        let output = CsvFormatter::format_list(&items);
        assert_eq!(output, "item\nfile1.rs\nfile2.rs\n");
    }

    #[test]
    fn test_csv_format_list_empty() {
        let items: Vec<&str> = vec![];
        let output = CsvFormatter::format_list(&items);
        assert_eq!(output, "item\n");
    }

    #[test]
    fn test_csv_format_count() {
        let output = CsvFormatter::format_count(42);
        assert_eq!(output, "count\n42\n");
    }

    #[test]
    fn test_csv_format_flag() {
        let output = CsvFormatter::format_flag("is_clean", true);
        assert_eq!(output, "is_clean\ntrue\n");

        let output = CsvFormatter::format_flag("is_clean", false);
        assert_eq!(output, "is_clean\nfalse\n");
    }

    #[test]
    fn test_csv_format_table() {
        let items = vec![
            vec!["file1.rs", "M", "10"],
            vec!["file2.rs", "A", "5"],
        ];
        let output = CsvFormatter::format_table(&["path", "status", "lines"], &items);
        assert!(output.contains("path,status,lines"));
        assert!(output.contains("file1.rs,M,10"));
        assert!(output.contains("file2.rs,A,5"));
    }

    #[test]
    fn test_csv_format_table_empty() {
        let items: Vec<Vec<&str>> = vec![];
        let output = CsvFormatter::format_table(&["path", "status"], &items);
        assert_eq!(output, "path,status\n");
    }

    #[test]
    fn test_csv_format_table_with_special_chars() {
        let items = vec![
            vec!["file,with,commas.rs", "M", "10"],
            vec!["file\"with\"quotes.rs", "A", "5"],
        ];
        let output = CsvFormatter::format_table(&["path", "status", "lines"], &items);
        assert!(output.contains("\"file,with,commas.rs\""));
        assert!(output.contains("\"file\"\"with\"\"quotes.rs\""));
    }

    // ============================================================
    // TSV Formatter Tests
    // ============================================================

    #[test]
    fn test_tsv_escape_field() {
        assert_eq!(TsvFormatter::escape_field("simple"), "simple");
        assert_eq!(TsvFormatter::escape_field("with\ttab"), "with\\ttab");
        assert_eq!(TsvFormatter::escape_field("with\nnewline"), "with\\nnewline");
        assert_eq!(TsvFormatter::escape_field("with\rcarriage"), "with\\rcarriage");
    }

    #[test]
    fn test_tsv_format_header() {
        let output = TsvFormatter::format_header(&["branch", "is_clean", "count"]);
        assert_eq!(output, "branch\tis_clean\tcount\n");
    }

    #[test]
    fn test_tsv_format_header_with_special_chars() {
        let output = TsvFormatter::format_header(&["branch", "has\ttab", "normal"]);
        assert_eq!(output, "branch\thas\\ttab\tnormal\n");
    }

    #[test]
    fn test_tsv_format_row() {
        let output = TsvFormatter::format_row(&["main", "true", "5"]);
        assert_eq!(output, "main\ttrue\t5\n");
    }

    #[test]
    fn test_tsv_format_row_with_special_chars() {
        let output = TsvFormatter::format_row(&["main", "has\ttab", "5"]);
        assert_eq!(output, "main\thas\\ttab\t5\n");
    }

    #[test]
    fn test_tsv_format_message() {
        let output = TsvFormatter::format_message("branch", "main");
        assert_eq!(output, "branch\nmain\n");
    }

    #[test]
    fn test_tsv_format_key_value() {
        let output = TsvFormatter::format_key_value("branch", "main");
        assert_eq!(output, "branch\nmain\n");
    }

    #[test]
    fn test_tsv_format_object() {
        let output = TsvFormatter::format_object(&[("branch", "main"), ("is_clean", "true"), ("count", "5")]);
        assert!(output.contains("branch\tis_clean\tcount"));
        assert!(output.contains("main\ttrue\t5"));
    }

    #[test]
    fn test_tsv_format_counts() {
        let output = TsvFormatter::format_counts(&[("passed", 10), ("failed", 2)]);
        assert!(output.contains("passed\tfailed"));
        assert!(output.contains("10\t2"));
    }

    #[test]
    fn test_tsv_format_counts_with_zeros() {
        let output = TsvFormatter::format_counts(&[("passed", 0), ("failed", 2)]);
        assert!(output.contains("passed\tfailed"));
        assert!(output.contains("0\t2"));
    }

    #[test]
    fn test_tsv_format_section() {
        let output = TsvFormatter::format_section("status", "path", &[("M", "src/main.rs"), ("A", "src/new.rs")]);
        assert!(output.contains("status\tpath"));
        assert!(output.contains("M\tsrc/main.rs"));
        assert!(output.contains("A\tsrc/new.rs"));
    }

    #[test]
    fn test_tsv_format_item() {
        let output = TsvFormatter::format_item("M", "src/main.rs");
        assert_eq!(output, "M\tsrc/main.rs\n");
    }

    #[test]
    fn test_tsv_format_item_renamed() {
        let output = TsvFormatter::format_item_renamed("R", "old.rs", "new.rs");
        assert_eq!(output, "R\tnew.rs\told.rs\n");
    }

    #[test]
    fn test_tsv_format_test_summary() {
        let output = TsvFormatter::format_test_summary(10, 2, 1, 1500);
        assert!(output.contains("passed\tfailed\tskipped\ttotal\tduration_ms"));
        assert!(output.contains("10\t2\t1\t13\t1500"));
    }

    #[test]
    fn test_tsv_format_test_summary_only_passed() {
        let output = TsvFormatter::format_test_summary(5, 0, 0, 500);
        assert!(output.contains("passed\tfailed\tskipped\ttotal\tduration_ms"));
        assert!(output.contains("5\t0\t0\t5\t500"));
    }

    #[test]
    fn test_tsv_format_status() {
        let success_output = TsvFormatter::format_status(true);
        assert_eq!(success_output, "success\ntrue\n");

        let failure_output = TsvFormatter::format_status(false);
        assert_eq!(failure_output, "success\nfalse\n");
    }

    #[test]
    fn test_tsv_format_failures() {
        let failures = vec!["test_one".to_string(), "test_two".to_string()];
        let output = TsvFormatter::format_failures(&failures);
        assert!(output.contains("failure"));
        assert!(output.contains("test_one"));
        assert!(output.contains("test_two"));
    }

    #[test]
    fn test_tsv_format_failures_empty() {
        let failures: Vec<String> = vec![];
        let output = TsvFormatter::format_failures(&failures);
        assert_eq!(output, "failure\n");
    }

    #[test]
    fn test_tsv_format_log_levels() {
        let output = TsvFormatter::format_log_levels(2, 5, 10, 3);
        assert!(output.contains("error\twarn\tinfo\tdebug\ttotal"));
        assert!(output.contains("2\t5\t10\t3\t20"));
    }

    #[test]
    fn test_tsv_format_log_levels_with_zeros() {
        let output = TsvFormatter::format_log_levels(0, 5, 0, 0);
        assert!(output.contains("error\twarn\tinfo\tdebug\ttotal"));
        assert!(output.contains("0\t5\t0\t0\t5"));
    }

    #[test]
    fn test_tsv_format_grep_match() {
        let output = TsvFormatter::format_grep_match("src/main.rs", Some(42), "fn main()");
        assert!(output.contains("file\tline\tcontent"));
        assert!(output.contains("src/main.rs\t42\tfn main()"));
    }

    #[test]
    fn test_tsv_format_grep_match_no_line() {
        let output = TsvFormatter::format_grep_match("src/main.rs", None, "match found");
        assert!(output.contains("file\tline\tcontent"));
        assert!(output.contains("src/main.rs\t\tmatch found"));
    }

    #[test]
    fn test_tsv_format_grep_file() {
        let output = TsvFormatter::format_grep_file("src/main.rs", 5);
        assert_eq!(output, "file\tmatch_count\nsrc/main.rs\t5\n");
    }

    #[test]
    fn test_tsv_format_diff_file() {
        let output = TsvFormatter::format_diff_file("src/main.rs", "M", 10, 5);
        assert_eq!(output, "path\tchange_type\tadditions\tdeletions\nsrc/main.rs\tM\t10\t5\n");
    }

    #[test]
    fn test_tsv_format_diff_summary() {
        let output = TsvFormatter::format_diff_summary(3, 25, 10);
        assert_eq!(output, "files_changed\tinsertions\tdeletions\n3\t25\t10\n");
    }

    #[test]
    fn test_tsv_format_clean() {
        let output = TsvFormatter::format_clean();
        assert_eq!(output, "is_clean\ntrue\n");
    }

    #[test]
    fn test_tsv_format_dirty() {
        let output = TsvFormatter::format_dirty(2, 3, 5, 0);
        assert_eq!(output, "is_clean\tstaged\tunstaged\tuntracked\tunmerged\nfalse\t2\t3\t5\t0\n");
    }

    #[test]
    fn test_tsv_format_branch_with_tracking() {
        // No tracking
        let output = TsvFormatter::format_branch_with_tracking("main", 0, 0);
        assert_eq!(output, "branch\tahead\tbehind\nmain\t0\t0\n");

        // With tracking
        let output = TsvFormatter::format_branch_with_tracking("feature", 3, 2);
        assert_eq!(output, "branch\tahead\tbehind\nfeature\t3\t2\n");
    }

    #[test]
    fn test_tsv_format_empty() {
        let output = TsvFormatter::format_empty();
        assert_eq!(output, "empty\ntrue\n");
    }

    #[test]
    fn test_tsv_format_truncated() {
        let output = TsvFormatter::format_truncated(10, 50);
        assert_eq!(output, "is_truncated\tshown\ttotal\ntrue\t10\t50\n");
    }

    #[test]
    fn test_tsv_format_error() {
        let output = TsvFormatter::format_error("Something went wrong");
        assert!(output.contains("error\tmessage"));
        assert!(output.contains("true\tSomething went wrong"));
    }

    #[test]
    fn test_tsv_format_error_with_code() {
        let output = TsvFormatter::format_error_with_code("Command failed", 1);
        assert_eq!(output, "error\tmessage\texit_code\ntrue\tCommand failed\t1\n");
    }

    #[test]
    fn test_tsv_format_not_implemented() {
        let output = TsvFormatter::format_not_implemented("Feature X");
        assert!(output.contains("not_implemented\tmessage"));
        assert!(output.contains("true\tFeature X"));
    }

    #[test]
    fn test_tsv_format_command_result() {
        let output = TsvFormatter::format_command_result(
            "echo",
            &["hello".to_string(), "world".to_string()],
            "hello world\n",
            "",
            0,
            10,
        );
        assert!(output.contains("command\targs\tstdout\tstderr\texit_code\tduration_ms"));
        assert!(output.contains("echo"));
        assert!(output.contains("hello world"));
    }

    #[test]
    fn test_tsv_format_list() {
        let items = vec!["file1.rs", "file2.rs"];
        let output = TsvFormatter::format_list(&items);
        assert_eq!(output, "item\nfile1.rs\nfile2.rs\n");
    }

    #[test]
    fn test_tsv_format_list_empty() {
        let items: Vec<&str> = vec![];
        let output = TsvFormatter::format_list(&items);
        assert_eq!(output, "item\n");
    }

    #[test]
    fn test_tsv_format_count() {
        let output = TsvFormatter::format_count(42);
        assert_eq!(output, "count\n42\n");
    }

    #[test]
    fn test_tsv_format_flag() {
        let output = TsvFormatter::format_flag("is_clean", true);
        assert_eq!(output, "is_clean\ntrue\n");

        let output = TsvFormatter::format_flag("is_clean", false);
        assert_eq!(output, "is_clean\nfalse\n");
    }

    #[test]
    fn test_tsv_format_table() {
        let items = vec![
            vec!["file1.rs", "M", "10"],
            vec!["file2.rs", "A", "5"],
        ];
        let output = TsvFormatter::format_table(&["path", "status", "lines"], &items);
        assert!(output.contains("path\tstatus\tlines"));
        assert!(output.contains("file1.rs\tM\t10"));
        assert!(output.contains("file2.rs\tA\t5"));
    }

    #[test]
    fn test_tsv_format_table_empty() {
        let items: Vec<Vec<&str>> = vec![];
        let output = TsvFormatter::format_table(&["path", "status"], &items);
        assert_eq!(output, "path\tstatus\n");
    }

    #[test]
    fn test_tsv_format_table_with_special_chars() {
        let items = vec![
            vec!["file\twith\ttabs.rs", "M", "10"],
            vec!["file\nwith\nnewlines.rs", "A", "5"],
        ];
        let output = TsvFormatter::format_table(&["path", "status", "lines"], &items);
        assert!(output.contains("file\\twith\\ttabs.rs"));
        assert!(output.contains("file\\nwith\\nnewlines.rs"));
    }

    // ============================================================
    // Agent Formatter Tests
    // ============================================================

    #[test]
    fn test_agent_section_header() {
        assert_eq!(AgentFormatter::section_header("Files"), "## Files\n");
    }

    #[test]
    fn test_agent_subsection_header() {
        assert_eq!(AgentFormatter::subsection_header("Details"), "### Details\n");
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

    #[test]
    fn test_agent_format_message() {
        assert_eq!(
            AgentFormatter::format_message("branch", "main"),
            "- branch: main\n"
        );
    }

    #[test]
    fn test_agent_format_counts() {
        let output = AgentFormatter::format_counts("counts", &[("passed", 10), ("failed", 2)]);
        assert_eq!(output, "- counts: passed=10 failed=2\n");

        // Zero counts should be filtered out
        let output = AgentFormatter::format_counts("counts", &[("passed", 0), ("failed", 2)]);
        assert_eq!(output, "- counts: failed=2\n");

        // All zeros should return empty string
        let output = AgentFormatter::format_counts("counts", &[("passed", 0), ("failed", 0)]);
        assert!(output.is_empty());
    }

    #[test]
    fn test_agent_format_section_header() {
        assert_eq!(
            AgentFormatter::format_section_header("staged", Some(3)),
            "## staged (3)\n"
        );
        assert_eq!(
            AgentFormatter::format_section_header("files", None),
            "## files\n"
        );
    }

    #[test]
    fn test_agent_format_item() {
        assert_eq!(
            AgentFormatter::format_item("M", "src/main.rs"),
            "  - [M] src/main.rs\n"
        );
    }

    #[test]
    fn test_agent_format_item_renamed() {
        assert_eq!(
            AgentFormatter::format_item_renamed("R", "old.rs", "new.rs"),
            "  - [R] old.rs -> new.rs\n"
        );
    }

    #[test]
    fn test_agent_format_test_summary() {
        let output = AgentFormatter::format_test_summary(10, 2, 1, 1500);
        assert!(output.contains("## Test Results"));
        assert!(output.contains("- passed: 10"));
        assert!(output.contains("- failed: 2"));
        assert!(output.contains("- skipped: 1"));
        assert!(output.contains("- total: 13"));
        assert!(output.contains("- duration: 1.50s"));
    }

    #[test]
    fn test_agent_format_test_summary_only_passed() {
        let output = AgentFormatter::format_test_summary(5, 0, 0, 500);
        assert!(output.contains("- passed: 5"));
        assert!(output.contains("- failed: 0"));
        assert!(output.contains("- duration: 500ms"));
    }

    #[test]
    fn test_agent_format_status() {
        assert_eq!(AgentFormatter::format_status(true), "- status: passed\n");
        assert_eq!(AgentFormatter::format_status(false), "- status: failed\n");
    }

    #[test]
    fn test_agent_format_failures() {
        let failures = vec!["test_one".to_string(), "test_two".to_string()];
        let output = AgentFormatter::format_failures(&failures);
        assert!(output.contains("## Failures (2)"));
        assert!(output.contains("- test_one"));
        assert!(output.contains("- test_two"));
    }

    #[test]
    fn test_agent_format_failures_empty() {
        let failures: Vec<String> = vec![];
        let output = AgentFormatter::format_failures(&failures);
        assert!(output.is_empty());
    }

    #[test]
    fn test_agent_format_log_levels() {
        let output = AgentFormatter::format_log_levels(2, 5, 10, 3);
        assert!(output.contains("## Log Levels"));
        assert!(output.contains("- error: 2"));
        assert!(output.contains("- warn: 5"));
        assert!(output.contains("- info: 10"));
        assert!(output.contains("- debug: 3"));
        assert!(output.contains("- total: 20"));
    }

    #[test]
    fn test_agent_format_log_levels_with_zeros() {
        let output = AgentFormatter::format_log_levels(0, 5, 0, 0);
        assert!(output.contains("- error: 0"));
        assert!(output.contains("- warn: 5"));
        assert!(output.contains("- total: 5"));
    }

    #[test]
    fn test_agent_format_grep_match() {
        let output = AgentFormatter::format_grep_match("src/main.rs", Some(42), "fn main()");
        assert!(output.contains("- file: src/main.rs"));
        assert!(output.contains("  line: 42"));
        assert!(output.contains("  content: fn main()"));
    }

    #[test]
    fn test_agent_format_grep_match_no_line() {
        let output = AgentFormatter::format_grep_match("src/main.rs", None, "match found");
        assert!(output.contains("- file: src/main.rs"));
        assert!(output.contains("  content: match found"));
        assert!(!output.contains("line:"));
    }

    #[test]
    fn test_agent_format_grep_file() {
        let output = AgentFormatter::format_grep_file("src/main.rs", 5);
        assert_eq!(output, "### src/main.rs (5 matches)\n");
    }

    #[test]
    fn test_agent_format_diff_file() {
        let output = AgentFormatter::format_diff_file("src/main.rs", "M", 10, 5);
        assert_eq!(output, "- [M] src/main.rs (+10 -5)\n");
    }

    #[test]
    fn test_agent_format_diff_summary() {
        let output = AgentFormatter::format_diff_summary(3, 25, 10);
        assert!(output.contains("## Diff Summary"));
        assert!(output.contains("- files changed: 3"));
        assert!(output.contains("- insertions: 25"));
        assert!(output.contains("- deletions: 10"));
    }

    #[test]
    fn test_agent_format_clean() {
        assert_eq!(AgentFormatter::format_clean(), "- status: clean\n");
    }

    #[test]
    fn test_agent_format_dirty() {
        let output = AgentFormatter::format_dirty(2, 3, 5, 0);
        assert!(output.contains("- status: dirty"));
        assert!(output.contains("- staged: 2"));
        assert!(output.contains("- unstaged: 3"));
        assert!(output.contains("- untracked: 5"));
        assert!(output.contains("- unmerged: 0"));
    }

    #[test]
    fn test_agent_format_branch_with_tracking() {
        // No tracking
        let output = AgentFormatter::format_branch_with_tracking("main", 0, 0);
        assert!(output.contains("- branch: main"));
        assert!(!output.contains("- ahead:"));
        assert!(!output.contains("- behind:"));

        // With tracking
        let output = AgentFormatter::format_branch_with_tracking("feature", 3, 2);
        assert!(output.contains("- branch: feature"));
        assert!(output.contains("- ahead: 3"));
        assert!(output.contains("- behind: 2"));
    }

    #[test]
    fn test_agent_format_empty() {
        assert_eq!(AgentFormatter::format_empty(), "- result: empty\n");
    }

    #[test]
    fn test_agent_format_truncated() {
        let output = AgentFormatter::format_truncated(10, 50);
        assert_eq!(output, "- truncated: showing 10 of 50\n");
    }

    #[test]
    fn test_agent_format_error() {
        let output = AgentFormatter::format_error("Something went wrong");
        assert_eq!(output, "- error: Something went wrong\n");
    }

    #[test]
    fn test_agent_format_error_with_code() {
        let output = AgentFormatter::format_error_with_code("Command failed", 1);
        assert!(output.contains("- error: Command failed"));
        assert!(output.contains("- exit_code: 1"));
    }

    #[test]
    fn test_agent_format_not_implemented() {
        let output = AgentFormatter::format_not_implemented("Feature X");
        assert_eq!(output, "- not_implemented: Feature X\n");
    }

    #[test]
    fn test_agent_format_command_result() {
        let output = AgentFormatter::format_command_result(
            "echo",
            &["hello".to_string(), "world".to_string()],
            "hello world\n",
            "",
            0,
            10,
        );
        assert!(output.contains("## Command Result"));
        assert!(output.contains("- command: echo"));
        assert!(output.contains("- args: hello world"));
        assert!(output.contains("- exit_code: 0"));
        assert!(output.contains("- duration_ms: 10"));
        assert!(output.contains("### stdout"));
        assert!(output.contains("hello world"));
    }

    #[test]
    fn test_agent_format_command_result_with_stderr() {
        let output = AgentFormatter::format_command_result(
            "cmd",
            &[],
            "stdout\n",
            "stderr\n",
            1,
            20,
        );
        assert!(output.contains("### stdout"));
        assert!(output.contains("### stderr"));
        assert!(output.contains("stderr"));
        assert!(output.contains("- exit_code: 1"));
    }

    #[test]
    fn test_agent_format_command_result_no_args() {
        let output = AgentFormatter::format_command_result(
            "pwd",
            &[],
            "/home\n",
            "",
            0,
            5,
        );
        assert!(output.contains("- command: pwd"));
        assert!(!output.contains("- args:"));
    }

    #[test]
    fn test_agent_format_list() {
        let items = vec!["file1.rs", "file2.rs"];
        let output = AgentFormatter::format_list(&items);
        assert_eq!(output, "- file1.rs\n- file2.rs\n");
    }

    #[test]
    fn test_agent_format_list_empty() {
        let items: Vec<&str> = vec![];
        let output = AgentFormatter::format_list(&items);
        assert!(output.is_empty());
    }

    #[test]
    fn test_agent_format_count() {
        let output = AgentFormatter::format_count(42);
        assert_eq!(output, "- count: 42\n");
    }

    #[test]
    fn test_agent_format_flag() {
        let output = AgentFormatter::format_flag("is_clean", true);
        assert_eq!(output, "- is_clean: true\n");

        let output = AgentFormatter::format_flag("is_clean", false);
        assert_eq!(output, "- is_clean: false\n");
    }

    #[test]
    fn test_agent_format_array() {
        let items = vec!["item1", "item2", "item3"];
        let output = AgentFormatter::format_array(&items);
        assert!(output.contains("- item1"));
        assert!(output.contains("- item2"));
        assert!(output.contains("- item3"));
    }

    #[test]
    fn test_agent_format_table() {
        let items = vec![
            vec!["file1.rs", "M", "10"],
            vec!["file2.rs", "A", "5"],
        ];
        let output = AgentFormatter::format_table(&["path", "status", "lines"], &items);
        assert!(output.contains("| path | status | lines |"));
        assert!(output.contains("| --- | --- | --- |"));
        assert!(output.contains("| file1.rs | M | 10 |"));
        assert!(output.contains("| file2.rs | A | 5 |"));
    }

    #[test]
    fn test_agent_format_table_empty() {
        let items: Vec<Vec<&str>> = vec![];
        let output = AgentFormatter::format_table(&["path", "status"], &items);
        assert!(output.contains("| path | status |"));
        assert!(output.contains("| --- | --- |"));
    }

    #[test]
    fn test_agent_format_key_value() {
        let output = AgentFormatter::format_key_value("branch", "main");
        assert_eq!(output, "- branch: main\n");
    }

    #[test]
    fn test_agent_format_metadata() {
        let output = AgentFormatter::format_metadata(&[
            ("branch", "main"),
            ("is_clean", "true"),
        ]);
        assert!(output.contains("## Metadata"));
        assert!(output.contains("- branch: main"));
        assert!(output.contains("- is_clean: true"));
    }

    #[test]
    fn test_agent_format_code_block() {
        let output = AgentFormatter::format_code_block("fn main() {}", Some("rust"));
        assert!(output.contains("```rust"));
        assert!(output.contains("fn main() {}"));
        assert!(output.contains("```"));
    }

    #[test]
    fn test_agent_format_code_block_no_language() {
        let output = AgentFormatter::format_code_block("code", None);
        assert!(output.contains("```\ncode\n```"));
        assert!(!output.contains("```rust"));
    }

    #[test]
    fn test_agent_format_divider() {
        assert_eq!(AgentFormatter::format_divider(), "---\n");
    }

    #[test]
    fn test_agent_format_bold() {
        assert_eq!(AgentFormatter::format_bold("text"), "**text**");
    }

    #[test]
    fn test_agent_format_italic() {
        assert_eq!(AgentFormatter::format_italic("text"), "*text*");
    }

    #[test]
    fn test_agent_format_code_inline() {
        assert_eq!(AgentFormatter::format_code_inline("code"), "`code`");
    }

    #[test]
    fn test_agent_format_link() {
        let output = AgentFormatter::format_link("text", "https://example.com");
        assert_eq!(output, "[text](https://example.com)");
    }

    #[test]
    fn test_agent_start_document() {
        let output = AgentFormatter::start_document("Title");
        assert_eq!(output, "# Title\n\n");
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
