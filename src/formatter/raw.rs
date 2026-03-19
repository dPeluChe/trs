//! Raw formatter for minimal output.

use super::helpers::format_duration;
use super::Formatter;
use crate::OutputFormat;

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
        items.iter().map(|s| format!("{}\n", s.as_ref())).collect()
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
    pub fn format_test_summary(
        passed: usize,
        failed: usize,
        skipped: usize,
        duration_ms: u64,
    ) -> String {
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
    pub fn format_diff_file(
        path: &str,
        change_type: &str,
        additions: usize,
        deletions: usize,
    ) -> String {
        format!("{} {} +{} -{}\n", change_type, path, additions, deletions)
    }

    /// Format a diff summary.
    pub fn format_diff_summary(
        files_changed: usize,
        insertions: usize,
        deletions: usize,
    ) -> String {
        format!("{} files +{} -{}\n", files_changed, insertions, deletions)
    }

    /// Format a clean state indicator.
    pub fn format_clean() -> String {
        "clean\n".to_string()
    }

    /// Format a dirty state indicator with counts.
    pub fn format_dirty(
        staged: usize,
        unstaged: usize,
        untracked: usize,
        unmerged: usize,
    ) -> String {
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
