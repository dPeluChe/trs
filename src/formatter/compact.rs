//! Compact formatter for human-readable output.

use super::helpers::format_duration;
use super::Formatter;
use crate::OutputFormat;

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
    pub fn format_diff_file(
        path: &str,
        change_type: &str,
        additions: usize,
        deletions: usize,
    ) -> String {
        format!(
            "  {} {} (+{} -{})\n",
            change_type, path, additions, deletions
        )
    }

    /// Format a diff summary.
    pub fn format_diff_summary(
        files_changed: usize,
        insertions: usize,
        deletions: usize,
    ) -> String {
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
    pub fn format_dirty(
        staged: usize,
        unstaged: usize,
        untracked: usize,
        unmerged: usize,
    ) -> String {
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
