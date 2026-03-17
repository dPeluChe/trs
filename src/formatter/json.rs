//! JSON formatter for structured output.

use crate::OutputFormat;
use super::Formatter;

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
    pub fn format_test_summary(
        passed: usize,
        failed: usize,
        skipped: usize,
        duration_ms: u64,
    ) -> String {
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
    pub fn format_diff_file(
        path: &str,
        change_type: &str,
        additions: usize,
        deletions: usize,
    ) -> String {
        serde_json::json!({
            "path": path,
            "change_type": change_type,
            "additions": additions,
            "deletions": deletions
        })
        .to_string()
    }

    /// Format a diff summary as JSON.
    pub fn format_diff_summary(
        files_changed: usize,
        insertions: usize,
        deletions: usize,
    ) -> String {
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
    pub fn format_dirty(
        staged: usize,
        unstaged: usize,
        untracked: usize,
        unmerged: usize,
    ) -> String {
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
