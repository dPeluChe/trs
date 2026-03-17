//! Agent formatter for AI-optimized output.

use crate::OutputFormat;
use super::Formatter;
use super::helpers::{truncate, format_duration};

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

    // ============================================================
    // Schema Formatting Methods
    // ============================================================

    /// Format a GitStatusSchema into agent-optimized output.
    pub fn format_git_status(status: &crate::schema::GitStatusSchema) -> String {
        let mut output = String::new();
        output.push_str("# Git Status\n\n");

        if !status.branch.is_empty() {
            output.push_str(&format!("- branch: {}\n", status.branch));
            if let Some(ahead) = status.ahead {
                if ahead > 0 {
                    output.push_str(&format!("- ahead: {}\n", ahead));
                }
            }
            if let Some(behind) = status.behind {
                if behind > 0 {
                    output.push_str(&format!("- behind: {}\n", behind));
                }
            }
        }

        if status.is_clean {
            output.push_str("- status: clean\n");
            return output;
        }

        output.push_str("- status: dirty\n");
        output.push_str(&format!("- staged: {}\n", status.counts.staged));
        output.push_str(&format!("- unstaged: {}\n", status.counts.unstaged));
        output.push_str(&format!("- untracked: {}\n", status.counts.untracked));
        output.push_str(&format!("- unmerged: {}\n", status.counts.unmerged));

        if !status.staged.is_empty() {
            output.push_str(&format!("\n## Staged ({})\n", status.staged.len()));
            for entry in &status.staged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&format!(
                        "  - [{}] {} -> {}\n",
                        entry.status, old_path, entry.path
                    ));
                } else {
                    output.push_str(&format!("  - [{}] {}\n", entry.status, entry.path));
                }
            }
        }

        if !status.unstaged.is_empty() {
            output.push_str(&format!("\n## Unstaged ({})\n", status.unstaged.len()));
            for entry in &status.unstaged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&format!(
                        "  - [{}] {} -> {}\n",
                        entry.status, old_path, entry.path
                    ));
                } else {
                    output.push_str(&format!("  - [{}] {}\n", entry.status, entry.path));
                }
            }
        }

        if !status.untracked.is_empty() {
            output.push_str(&format!("\n## Untracked ({})\n", status.untracked.len()));
            for entry in &status.untracked {
                output.push_str(&format!("  - [{}] {}\n", entry.status, entry.path));
            }
        }

        if !status.unmerged.is_empty() {
            output.push_str(&format!("\n## Unmerged ({})\n", status.unmerged.len()));
            for entry in &status.unmerged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&format!(
                        "  - [{}] {} -> {}\n",
                        entry.status, old_path, entry.path
                    ));
                } else {
                    output.push_str(&format!("  - [{}] {}\n", entry.status, entry.path));
                }
            }
        }

        output
    }

    /// Format a GitDiffSchema into agent-optimized output.
    pub fn format_git_diff(diff: &crate::schema::GitDiffSchema) -> String {
        if diff.is_empty {
            return "# Git Diff\n\n- status: empty\n".to_string();
        }

        let mut output = String::new();
        output.push_str("# Git Diff\n\n");

        output.push_str(&format!("- files changed: {}\n", diff.counts.total_files));
        output.push_str(&format!("- insertions: {}\n", diff.total_additions));
        output.push_str(&format!("- deletions: {}\n", diff.total_deletions));

        output.push_str(&format!("\n## Files ({})\n", diff.files.len()));
        for file in &diff.files {
            output.push_str(&format!(
                "- [{}] {} (+{} -{})\n",
                file.change_type, file.path, file.additions, file.deletions
            ));
        }

        if diff.is_truncated {
            output.push_str(&format!(
                "\n- truncated: showing {} of {} files\n",
                diff.counts.files_shown, diff.counts.total_files
            ));
        }

        output
    }

    /// Format a LsOutputSchema into agent-optimized output.
    pub fn format_ls(ls: &crate::schema::LsOutputSchema) -> String {
        if ls.is_empty {
            return "# Directory Listing\n\n- result: empty\n".to_string();
        }

        let mut output = String::new();
        output.push_str("# Directory Listing\n\n");

        output.push_str(&format!("- total: {}\n", ls.counts.total));
        output.push_str(&format!("- directories: {}\n", ls.counts.directories));
        output.push_str(&format!("- files: {}\n", ls.counts.files));
        output.push_str(&format!("- symlinks: {}\n", ls.counts.symlinks));
        if ls.counts.hidden > 0 {
            output.push_str(&format!("- hidden: {}\n", ls.counts.hidden));
        }

        if !ls.directories.is_empty() {
            output.push_str(&format!("\n## Directories ({})\n", ls.directories.len()));
            for dir in &ls.directories {
                output.push_str(&format!("- {}\n", dir));
            }
        }

        if !ls.files.is_empty() {
            output.push_str(&format!("\n## Files ({})\n", ls.files.len()));
            for file in &ls.files {
                output.push_str(&format!("- {}\n", file));
            }
        }

        if !ls.symlinks.is_empty() {
            output.push_str(&format!("\n## Symlinks ({})\n", ls.symlinks.len()));
            for symlink in &ls.symlinks {
                if let Some(entry) = ls.entries.iter().find(|e| &e.name == symlink) {
                    if let Some(ref target) = entry.symlink_target {
                        if entry.is_broken_symlink {
                            output.push_str(&format!("- {} -> {} [broken]\n", symlink, target));
                        } else {
                            output.push_str(&format!("- {} -> {}\n", symlink, target));
                        }
                    } else {
                        output.push_str(&format!("- {}\n", symlink));
                    }
                } else {
                    output.push_str(&format!("- {}\n", symlink));
                }
            }
        }

        if !ls.hidden.is_empty() {
            output.push_str(&format!("\n## Hidden ({})\n", ls.hidden.len()));
            for hidden in &ls.hidden {
                output.push_str(&format!("- {}\n", hidden));
            }
        }

        if !ls.generated.is_empty() {
            output.push_str(&format!("\n## Generated ({})\n", ls.generated.len()));
            for gen in &ls.generated {
                output.push_str(&format!("- {}\n", gen));
            }
        }

        if !ls.errors.is_empty() {
            output.push_str(&format!("\n## Errors ({})\n", ls.errors.len()));
            for error in &ls.errors {
                output.push_str(&format!("- {}: {}\n", error.path, error.message));
            }
        }

        output
    }

    /// Format a GrepOutputSchema into agent-optimized output.
    pub fn format_grep(grep: &crate::schema::GrepOutputSchema) -> String {
        if grep.is_empty {
            return "# Search Results\n\n- result: no matches\n".to_string();
        }

        let mut output = String::new();
        output.push_str("# Search Results\n\n");

        output.push_str(&format!("- files: {}\n", grep.counts.files));
        output.push_str(&format!("- matches: {}\n", grep.counts.matches));

        for file in &grep.files {
            output.push_str(&format!("\n## {}\n", file.path));
            output.push_str(&format!("- matches: {}\n", file.matches.len()));
            for m in &file.matches {
                if m.is_context {
                    if let Some(line) = m.line_number {
                        output.push_str(&format!("  - line {} [context]\n", line));
                    }
                } else if let Some(line) = m.line_number {
                    output.push_str(&format!(
                        "  - line {}: {}\n",
                        line,
                        truncate(m.line.trim(), 80)
                    ));
                } else {
                    output.push_str(&format!("  - {}\n", truncate(m.line.trim(), 80)));
                }
            }
        }

        if grep.is_truncated {
            output.push_str(&format!(
                "\n- truncated: showing {} of {} files\n",
                grep.counts.files_shown, grep.counts.total_files
            ));
        }

        output
    }

    /// Format a FindOutputSchema into agent-optimized output.
    pub fn format_find(find: &crate::schema::FindOutputSchema) -> String {
        if find.is_empty {
            return "# Find Results\n\n- result: no matches\n".to_string();
        }

        let mut output = String::new();
        output.push_str("# Find Results\n\n");

        output.push_str(&format!("- total: {}\n", find.counts.total));
        output.push_str(&format!("- directories: {}\n", find.counts.directories));
        output.push_str(&format!("- files: {}\n", find.counts.files));

        if !find.directories.is_empty() {
            output.push_str(&format!("\n## Directories ({})\n", find.directories.len()));
            for dir in &find.directories {
                output.push_str(&format!("- {}\n", dir));
            }
        }

        if !find.files.is_empty() {
            output.push_str(&format!("\n## Files ({})\n", find.files.len()));
            for file in &find.files {
                output.push_str(&format!("- {}\n", file));
            }
        }

        if !find.hidden.is_empty() {
            output.push_str(&format!("\n## Hidden ({})\n", find.hidden.len()));
            for hidden in &find.hidden {
                output.push_str(&format!("- {}\n", hidden));
            }
        }

        if !find.extensions.is_empty() {
            output.push_str(&format!("\n## Extensions\n"));
            let mut exts: Vec<_> = find.extensions.iter().collect();
            exts.sort_by(|a, b| b.1.cmp(a.1));
            for (ext, count) in exts {
                output.push_str(&format!("- .{}: {}\n", ext, count));
            }
        }

        if !find.errors.is_empty() {
            output.push_str(&format!("\n## Errors ({})\n", find.errors.len()));
            for error in &find.errors {
                output.push_str(&format!("- {}: {}\n", error.path, error.message));
            }
        }

        output
    }

    /// Format a TestOutputSchema into agent-optimized output.
    pub fn format_test_output(test: &crate::schema::TestOutputSchema) -> String {
        if test.is_empty {
            return "# Test Results\n\n- result: no tests\n".to_string();
        }

        let mut output = String::new();
        output.push_str("# Test Results\n\n");

        output.push_str(&format!("- runner: {}\n", test.runner));
        if let Some(ref version) = test.runner_version {
            output.push_str(&format!("- version: {}\n", version));
        }

        output.push_str(&format!(
            "- status: {}\n",
            if test.success { "passed" } else { "failed" }
        ));

        output.push_str(&format!("- total: {}\n", test.summary.total));
        output.push_str(&format!("- passed: {}\n", test.summary.passed));
        output.push_str(&format!("- failed: {}\n", test.summary.failed));
        if test.summary.skipped > 0 {
            output.push_str(&format!("- skipped: {}\n", test.summary.skipped));
        }
        if test.summary.xfailed > 0 {
            output.push_str(&format!("- xfailed: {}\n", test.summary.xfailed));
        }
        if test.summary.xpassed > 0 {
            output.push_str(&format!("- xpassed: {}\n", test.summary.xpassed));
        }
        if test.summary.errors > 0 {
            output.push_str(&format!("- errors: {}\n", test.summary.errors));
        }
        if let Some(duration) = test.summary.duration_ms {
            output.push_str(&format!("- duration: {}\n", format_duration(duration)));
        }

        if test.summary.suites_total > 0 {
            output.push_str(&format!(
                "\n- suites: {} ({} passed, {} failed)\n",
                test.summary.suites_total, test.summary.suites_passed, test.summary.suites_failed
            ));
        }

        if !test.success {
            output.push_str("\n## Failed Tests\n");
            for suite in &test.test_suites {
                if !suite.passed {
                    for t in &suite.tests {
                        if t.status == crate::schema::TestStatus::Failed {
                            output.push_str(&format!("- {}", t.name));
                            if let Some(ref file) = t.file {
                                output.push_str(&format!(" ({})", file));
                                if let Some(line) = t.line {
                                    output.push_str(&format!(":{}", line));
                                }
                            }
                            output.push('\n');
                            if let Some(ref msg) = t.error_message {
                                for line in msg.lines().take(5) {
                                    output.push_str(&format!("  > {}\n", line));
                                }
                            }
                        }
                    }
                }
            }
        }

        output
    }

    /// Format a LogsOutputSchema into agent-optimized output.
    pub fn format_logs(logs: &crate::schema::LogsOutputSchema) -> String {
        if logs.is_empty {
            return "# Log Output\n\n- result: empty\n".to_string();
        }

        let mut output = String::new();
        output.push_str("# Log Output\n\n");

        output.push_str(&format!("- total lines: {}\n", logs.counts.total_lines));

        output.push_str(&format!("- error: {}\n", logs.counts.error));
        output.push_str(&format!("- warning: {}\n", logs.counts.warning));
        output.push_str(&format!("- info: {}\n", logs.counts.info));
        output.push_str(&format!("- debug: {}\n", logs.counts.debug));

        if !logs.repeated_lines.is_empty() {
            output.push_str(&format!(
                "\n## Repeated Lines ({})\n",
                logs.repeated_lines.len()
            ));
            for repeated in &logs.repeated_lines {
                output.push_str(&format!(
                    "- lines {}-{} [x{}]: {}\n",
                    repeated.first_line, repeated.last_line, repeated.count, repeated.line
                ));
            }
        }

        if !logs.recent_critical.is_empty() {
            let critical_count = logs.counts.error + logs.counts.fatal;
            output.push_str(&format!(
                "\n## Recent Critical ({}/{})\n",
                logs.recent_critical.len(),
                critical_count
            ));
            for entry in &logs.recent_critical {
                let level = match entry.level {
                    crate::schema::LogLevel::Error => "ERROR",
                    crate::schema::LogLevel::Fatal => "FATAL",
                    _ => "!",
                };
                output.push_str(&format!(
                    "- line {} [{}]: {}\n",
                    entry.line_number,
                    level,
                    truncate(&entry.message, 80)
                ));
            }
        }

        output
    }

    /// Format a RepositoryStateSchema into agent-optimized output.
    pub fn format_repository_state(state: &crate::schema::RepositoryStateSchema) -> String {
        let mut output = String::new();
        output.push_str("# Repository State\n\n");

        if !state.is_git_repo {
            output.push_str("- is_git_repo: false\n");
            return output;
        }

        output.push_str("- is_git_repo: true\n");

        if let Some(ref branch) = state.branch {
            if state.is_detached {
                output.push_str(&format!("- branch: {} (detached)\n", branch));
            } else {
                output.push_str(&format!("- branch: {}\n", branch));
            }
        }

        if state.is_clean {
            output.push_str("- status: clean\n");
        } else {
            output.push_str("- status: dirty\n");
            output.push_str(&format!("- staged: {}\n", state.counts.staged));
            output.push_str(&format!("- unstaged: {}\n", state.counts.unstaged));
            output.push_str(&format!("- untracked: {}\n", state.counts.untracked));
            output.push_str(&format!("- unmerged: {}\n", state.counts.unmerged));
        }

        output
    }

    /// Format a ProcessOutputSchema into agent-optimized output.
    pub fn format_process(process: &crate::schema::ProcessOutputSchema) -> String {
        let mut output = String::new();
        output.push_str("# Process Output\n\n");

        output.push_str(&format!("- command: {}\n", process.command));
        if !process.args.is_empty() {
            output.push_str(&format!("- args: {}\n", process.args.join(" ")));
        }
        output.push_str(&format!(
            "- status: {}\n",
            if process.success { "success" } else { "failed" }
        ));
        if let Some(code) = process.exit_code {
            output.push_str(&format!("- exit_code: {}\n", code));
        }
        output.push_str(&format!("- duration_ms: {}\n", process.duration_ms));
        if process.timed_out {
            output.push_str("- timed_out: true\n");
        }

        if !process.stdout.is_empty() {
            output.push_str("\n## stdout\n");
            output.push_str(&format!("```\n{}```\n", process.stdout));
        }

        if !process.stderr.is_empty() {
            output.push_str("\n## stderr\n");
            output.push_str(&format!("```\n{}```\n", process.stderr));
        }

        output
    }

    /// Format an ErrorSchema into agent-optimized output.
    pub fn format_error_schema(error: &crate::schema::ErrorSchema) -> String {
        let mut output = String::new();
        output.push_str("# Error\n\n");

        output.push_str(&format!("- message: {}\n", error.message));
        if let Some(ref error_type) = error.error_type {
            output.push_str(&format!("- type: {}\n", error_type));
        }
        if let Some(code) = error.exit_code {
            output.push_str(&format!("- exit_code: {}\n", code));
        }

        if !error.context.is_empty() {
            output.push_str("\n## Context\n");
            for (key, value) in &error.context {
                output.push_str(&format!("- {}: {}\n", key, value));
            }
        }

        output
    }
}
