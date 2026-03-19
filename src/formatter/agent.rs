//! Agent formatter for AI-optimized output.

use super::helpers::format_duration;
use super::Formatter;
use crate::OutputFormat;

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
    pub fn section_header(title: &str) -> String {
        format!("## {}\n", title)
    }

    /// Format a subsection header.
    pub fn subsection_header(title: &str) -> String {
        format!("### {}\n", title)
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

    /// Format a simple message/status line.
    pub fn format_message(key: &str, value: &str) -> String {
        format!("- {}: {}\n", key, value)
    }

    /// Format a count summary line.
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
    pub fn format_section_header(name: &str, count: Option<usize>) -> String {
        match count {
            Some(c) => format!("## {} ({})\n", name, c),
            None => format!("## {}\n", name),
        }
    }

    /// Format an indented list item.
    pub fn format_item(status: &str, path: &str) -> String {
        format!("  - [{}] {}\n", status, path)
    }

    /// Format an indented list item with rename info.
    pub fn format_item_renamed(status: &str, old_path: &str, new_path: &str) -> String {
        format!("  - [{}] {} -> {}\n", status, old_path, new_path)
    }

    /// Format a test result summary.
    pub fn format_test_summary(
        passed: usize,
        failed: usize,
        skipped: usize,
        duration_ms: u64,
    ) -> String {
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
    pub fn format_status(success: bool) -> String {
        format!("- status: {}\n", if success { "passed" } else { "failed" })
    }

    /// Format a list of failing tests.
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
    pub fn format_grep_file(file: &str, match_count: usize) -> String {
        format!("### {} ({} matches)\n", file, match_count)
    }

    /// Format a diff file entry.
    pub fn format_diff_file(
        path: &str,
        change_type: &str,
        additions: usize,
        deletions: usize,
    ) -> String {
        format!(
            "- [{}] {} (+{} -{})\n",
            change_type, path, additions, deletions
        )
    }

    /// Format a diff summary.
    pub fn format_diff_summary(
        files_changed: usize,
        insertions: usize,
        deletions: usize,
    ) -> String {
        let mut output = String::new();
        output.push_str("## Diff Summary\n");
        output.push_str(&format!("- files changed: {}\n", files_changed));
        output.push_str(&format!("- insertions: {}\n", insertions));
        output.push_str(&format!("- deletions: {}\n", deletions));
        output
    }

    /// Format a clean state indicator.
    pub fn format_clean() -> String {
        "- status: clean\n".to_string()
    }

    /// Format a dirty state indicator with counts.
    pub fn format_dirty(
        staged: usize,
        unstaged: usize,
        untracked: usize,
        unmerged: usize,
    ) -> String {
        let mut output = String::new();
        output.push_str("- status: dirty\n");
        output.push_str(&format!("- staged: {}\n", staged));
        output.push_str(&format!("- unstaged: {}\n", unstaged));
        output.push_str(&format!("- untracked: {}\n", untracked));
        output.push_str(&format!("- unmerged: {}\n", unmerged));
        output
    }

    /// Format branch info with ahead/behind.
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
    pub fn format_empty() -> String {
        "- result: empty\n".to_string()
    }

    /// Format a truncation warning.
    pub fn format_truncated(shown: usize, total: usize) -> String {
        format!("- truncated: showing {} of {}\n", shown, total)
    }

    /// Format an error message.
    pub fn format_error(message: &str) -> String {
        format!("- error: {}\n", message)
    }

    /// Format an error with exit code.
    pub fn format_error_with_code(message: &str, exit_code: i32) -> String {
        format!("- error: {}\n- exit_code: {}\n", message, exit_code)
    }

    /// Format a not-implemented message.
    pub fn format_not_implemented(message: &str) -> String {
        format!("- not_implemented: {}\n", message)
    }

    /// Format a command result.
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
    pub fn format_list(items: &[impl AsRef<str>]) -> String {
        items
            .iter()
            .map(|s| format!("- {}\n", s.as_ref()))
            .collect()
    }

    /// Format a count.
    pub fn format_count(count: usize) -> String {
        format!("- count: {}\n", count)
    }

    /// Format a boolean flag.
    pub fn format_flag(name: &str, value: bool) -> String {
        format!("- {}: {}\n", name, value)
    }

    /// Format an array of objects.
    pub fn format_array(items: &[impl AsRef<str>]) -> String {
        Self::format_list(items)
    }

    /// Format a table with headers.
    pub fn format_table(headers: &[&str], rows: &[Vec<&str>]) -> String {
        let mut output = String::new();

        output.push_str(&format!("| {} |\n", headers.join(" | ")));

        output.push_str(&format!(
            "| {} |\n",
            headers
                .iter()
                .map(|_| "---")
                .collect::<Vec<_>>()
                .join(" | ")
        ));

        for row in rows {
            output.push_str(&format!("| {} |\n", row.join(" | ")));
        }

        output
    }

    /// Format a key-value pair.
    pub fn format_key_value(key: &str, value: &str) -> String {
        format!("- {}: {}\n", key, value)
    }

    /// Format a metadata block.
    pub fn format_metadata(items: &[(&str, &str)]) -> String {
        let mut output = String::new();
        output.push_str("## Metadata\n");
        for (key, value) in items {
            output.push_str(&format!("- {}: {}\n", key, value));
        }
        output
    }

    /// Format a code block.
    pub fn format_code_block(code: &str, language: Option<&str>) -> String {
        match language {
            Some(lang) => format!("```{}\n{}\n```\n", lang, code),
            None => format!("```\n{}\n```\n", code),
        }
    }

    /// Format a divider line.
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
