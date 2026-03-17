//! Compact formatter for human-readable output.

use crate::OutputFormat;
use super::Formatter;
use super::helpers::{truncate, format_duration};

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

    // ============================================================
    // Schema Formatting Methods
    // ============================================================

    /// Format a GitStatusSchema into compact output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CompactFormatter;
    /// use tars_cli::schema::{GitStatusSchema, GitFileEntry};
    /// let mut status = GitStatusSchema::new("main");
    /// status.is_clean = true;
    /// let output = CompactFormatter::format_git_status(&status);
    /// assert!(output.contains("branch: main"));
    /// assert!(output.contains("status: clean"));
    /// ```
    pub fn format_git_status(status: &crate::schema::GitStatusSchema) -> String {
        let mut output = String::new();

        // Branch info
        if !status.branch.is_empty() {
            output.push_str(&Self::format_branch_with_tracking(
                &status.branch,
                status.ahead.unwrap_or(0),
                status.behind.unwrap_or(0),
            ));
        }

        // Clean state
        if status.is_clean {
            output.push_str(&Self::format_clean());
            return output;
        }

        // Summary line with counts
        output.push_str(&Self::format_counts(
            "counts",
            &[
                ("staged", status.counts.staged),
                ("unstaged", status.counts.unstaged),
                ("untracked", status.counts.untracked),
                ("unmerged", status.counts.unmerged),
            ],
        ));

        // Staged changes
        if !status.staged.is_empty() {
            output.push_str(&Self::format_section_header(
                "staged",
                Some(status.staged.len()),
            ));
            for entry in &status.staged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&Self::format_item_renamed(
                        &entry.status,
                        old_path,
                        &entry.path,
                    ));
                } else {
                    output.push_str(&Self::format_item(&entry.status, &entry.path));
                }
            }
        }

        // Unstaged changes
        if !status.unstaged.is_empty() {
            output.push_str(&Self::format_section_header(
                "unstaged",
                Some(status.unstaged.len()),
            ));
            for entry in &status.unstaged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&Self::format_item_renamed(
                        &entry.status,
                        old_path,
                        &entry.path,
                    ));
                } else {
                    output.push_str(&Self::format_item(&entry.status, &entry.path));
                }
            }
        }

        // Untracked files
        if !status.untracked.is_empty() {
            output.push_str(&Self::format_section_header(
                "untracked",
                Some(status.untracked.len()),
            ));
            for entry in &status.untracked {
                output.push_str(&Self::format_item(&entry.status, &entry.path));
            }
        }

        // Unmerged files
        if !status.unmerged.is_empty() {
            output.push_str(&Self::format_section_header(
                "unmerged",
                Some(status.unmerged.len()),
            ));
            for entry in &status.unmerged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&Self::format_item_renamed(
                        &entry.status,
                        old_path,
                        &entry.path,
                    ));
                } else {
                    output.push_str(&Self::format_item(&entry.status, &entry.path));
                }
            }
        }

        output
    }

    /// Format a GitDiffSchema into compact output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CompactFormatter;
    /// use tars_cli::schema::GitDiffSchema;
    /// let diff = GitDiffSchema::new();
    /// let output = CompactFormatter::format_git_diff(&diff);
    /// assert!(output.contains("diff: empty"));
    /// ```
    pub fn format_git_diff(diff: &crate::schema::GitDiffSchema) -> String {
        if diff.is_empty {
            return "diff: empty\n".to_string();
        }

        let mut output = String::new();

        // List files with changes
        for file in &diff.files {
            output.push_str(&Self::format_diff_file(
                &file.path,
                &file.change_type,
                file.additions,
                file.deletions,
            ));
        }

        // Summary
        output.push_str(&Self::format_diff_summary(
            diff.counts.total_files,
            diff.total_additions,
            diff.total_deletions,
        ));

        // Truncation warning if needed
        if diff.is_truncated {
            output.push_str(&Self::format_truncated(
                diff.counts.files_shown,
                diff.counts.total_files,
            ));
        }

        output
    }

    /// Format a LsOutputSchema into compact output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CompactFormatter;
    /// use tars_cli::schema::{LsOutputSchema, LsEntry, LsEntryType};
    /// let mut ls = LsOutputSchema::new();
    /// ls.is_empty = false;
    /// ls.directories.push("src".to_string());
    /// ls.counts.directories = 1;
    /// ls.counts.total = 1;
    /// let output = CompactFormatter::format_ls(&ls);
    /// assert!(output.contains("directories (1)"));
    /// ```
    pub fn format_ls(ls: &crate::schema::LsOutputSchema) -> String {
        if ls.is_empty {
            return Self::format_empty();
        }

        let mut output = String::new();

        // Directories
        if !ls.directories.is_empty() {
            output.push_str(&Self::format_section_header(
                "directories",
                Some(ls.directories.len()),
            ));
            for dir in &ls.directories {
                output.push_str(&format!("  {}\n", dir));
            }
        }

        // Files
        if !ls.files.is_empty() {
            output.push_str(&Self::format_section_header("files", Some(ls.files.len())));
            for file in &ls.files {
                output.push_str(&format!("  {}\n", file));
            }
        }

        // Symlinks
        if !ls.symlinks.is_empty() {
            output.push_str(&Self::format_section_header(
                "symlinks",
                Some(ls.symlinks.len()),
            ));
            for symlink in &ls.symlinks {
                // Find the entry to get the target
                if let Some(entry) = ls.entries.iter().find(|e| &e.name == symlink) {
                    if let Some(ref target) = entry.symlink_target {
                        if entry.is_broken_symlink {
                            output.push_str(&format!("  {} -> {} [broken]\n", symlink, target));
                        } else {
                            output.push_str(&format!("  {} -> {}\n", symlink, target));
                        }
                    } else {
                        output.push_str(&format!("  {}\n", symlink));
                    }
                } else {
                    output.push_str(&format!("  {}\n", symlink));
                }
            }
        }

        // Hidden files
        if !ls.hidden.is_empty() {
            output.push_str(&Self::format_section_header(
                "hidden",
                Some(ls.hidden.len()),
            ));
            for hidden in &ls.hidden {
                output.push_str(&format!("  {}\n", hidden));
            }
        }

        // Generated directories
        if !ls.generated.is_empty() {
            output.push_str(&Self::format_section_header(
                "generated",
                Some(ls.generated.len()),
            ));
            for gen in &ls.generated {
                output.push_str(&format!("  {}\n", gen));
            }
        }

        // Errors
        if !ls.errors.is_empty() {
            output.push_str(&Self::format_section_header(
                "errors",
                Some(ls.errors.len()),
            ));
            for error in &ls.errors {
                output.push_str(&format!("  {}: {}\n", error.path, error.message));
            }
        }

        output
    }

    /// Format a GrepOutputSchema into compact output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CompactFormatter;
    /// use tars_cli::schema::{GrepOutputSchema, GrepFile, GrepMatch};
    /// let mut grep = GrepOutputSchema::new();
    /// grep.is_empty = false;
    /// let mut file = GrepFile::new("src/main.rs");
    /// file.matches.push(GrepMatch::new("fn main()"));
    /// grep.files.push(file);
    /// grep.counts.files = 1;
    /// grep.counts.matches = 1;
    /// let output = CompactFormatter::format_grep(&grep);
    /// assert!(output.contains("src/main.rs (1)"));
    /// ```
    pub fn format_grep(grep: &crate::schema::GrepOutputSchema) -> String {
        if grep.is_empty {
            return "grep: no matches\n".to_string();
        }

        let mut output = String::new();

        // Summary header
        output.push_str(&format!(
            "matches: {} files, {} results\n",
            grep.counts.files, grep.counts.matches
        ));

        // Files with matches
        for file in &grep.files {
            output.push_str(&Self::format_grep_file(&file.path, file.matches.len()));
            for m in &file.matches {
                if m.is_context {
                    // Context lines shown with ...
                    if let Some(line) = m.line_number {
                        output.push_str(&format!("  {} ...\n", line));
                    }
                } else if let Some(line) = m.line_number {
                    output.push_str(&format!("  {}: {}\n", line, truncate(m.line.trim(), 80)));
                } else {
                    output.push_str(&format!("  {}\n", truncate(m.line.trim(), 80)));
                }
            }
        }

        // Truncation warning
        if grep.is_truncated {
            output.push_str(&Self::format_truncated(
                grep.counts.files_shown,
                grep.counts.total_files,
            ));
        }

        output
    }

    /// Format a FindOutputSchema into compact output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CompactFormatter;
    /// use tars_cli::schema::{FindOutputSchema, FindEntry};
    /// let mut find = FindOutputSchema::new();
    /// find.is_empty = false;
    /// find.files.push("./src/main.rs".to_string());
    /// find.counts.files = 1;
    /// find.counts.total = 1;
    /// let output = CompactFormatter::format_find(&find);
    /// assert!(output.contains("find: 1 entries"));
    /// ```
    pub fn format_find(find: &crate::schema::FindOutputSchema) -> String {
        if find.is_empty {
            return "find: no results\n".to_string();
        }

        let mut output = String::new();

        // Summary header
        output.push_str(&format!(
            "find: {} entries ({} dirs, {} files)\n",
            find.counts.total, find.counts.directories, find.counts.files
        ));

        // Directories
        if !find.directories.is_empty() {
            output.push_str(&Self::format_section_header(
                "directories",
                Some(find.directories.len()),
            ));
            for dir in &find.directories {
                output.push_str(&format!("  {}\n", dir));
            }
        }

        // Files
        if !find.files.is_empty() {
            output.push_str(&Self::format_section_header(
                "files",
                Some(find.files.len()),
            ));
            for file in &find.files {
                output.push_str(&format!("  {}\n", file));
            }
        }

        // Errors
        if !find.errors.is_empty() {
            output.push_str(&Self::format_section_header(
                "errors",
                Some(find.errors.len()),
            ));
            for error in &find.errors {
                output.push_str(&format!("  {}: {}\n", error.path, error.message));
            }
        }

        output
    }

    /// Format a TestOutputSchema into compact output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CompactFormatter;
    /// use tars_cli::schema::{TestOutputSchema, TestRunnerType};
    /// let mut test = TestOutputSchema::new(TestRunnerType::Pytest);
    /// test.is_empty = false;
    /// test.summary.passed = 10;
    /// test.summary.failed = 0;
    /// test.summary.total = 10;
    /// let output = CompactFormatter::format_test_output(&test);
    /// assert!(output.contains("PASS"));
    /// ```
    pub fn format_test_output(test: &crate::schema::TestOutputSchema) -> String {
        if test.is_empty {
            return "tests: no results\n".to_string();
        }

        let mut output = String::new();

        // Status indicator
        if test.success {
            output.push_str(&format!("PASS: {} tests", test.summary.passed));
            if test.summary.skipped > 0 {
                output.push_str(&format!(", {} skipped", test.summary.skipped));
            }
            output.push('\n');
        } else {
            output.push_str(&format!(
                "FAIL: {} passed, {} failed",
                test.summary.passed, test.summary.failed
            ));
            if test.summary.skipped > 0 {
                output.push_str(&format!(", {} skipped", test.summary.skipped));
            }
            output.push('\n');

            // Show failing tests
            for suite in &test.test_suites {
                if !suite.passed {
                    for t in &suite.tests {
                        if t.status == crate::schema::TestStatus::Failed {
                            output.push_str(&format!("  FAIL: {}\n", t.name));
                        }
                    }
                }
            }
        }

        // Duration
        if let Some(duration) = test.summary.duration_ms {
            output.push_str(&format!("duration: {}\n", format_duration(duration)));
        }

        output
    }

    /// Format a LogsOutputSchema into compact output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CompactFormatter;
    /// use tars_cli::schema::{LogsOutputSchema, LogEntry, LogLevel};
    /// let mut logs = LogsOutputSchema::new();
    /// logs.is_empty = false;
    /// logs.counts.total_lines = 10;
    /// logs.counts.info = 8;
    /// logs.counts.error = 2;
    /// let output = CompactFormatter::format_logs(&logs);
    /// assert!(output.contains("lines: 10"));
    /// ```
    pub fn format_logs(logs: &crate::schema::LogsOutputSchema) -> String {
        if logs.is_empty {
            return "logs: empty\n".to_string();
        }

        let mut output = String::new();

        // Summary
        output.push_str(&format!("lines: {}\n", logs.counts.total_lines));

        // Level counts
        let level_str = Self::format_log_levels(
            logs.counts.error,
            logs.counts.warning,
            logs.counts.info,
            logs.counts.debug,
        );
        if !level_str.is_empty() {
            output.push_str(&level_str);
        }

        // Repeated lines summary
        if !logs.repeated_lines.is_empty() {
            output.push_str("repeated:\n");
            for repeated in &logs.repeated_lines {
                output.push_str(&format!(
                    "  {}-{} {} [x{}]\n",
                    repeated.first_line, repeated.last_line, repeated.line, repeated.count
                ));
            }
        }

        // Recent critical lines
        if !logs.recent_critical.is_empty() {
            output.push_str(&format!(
                "recent critical ({} of {}):\n",
                logs.recent_critical.len(),
                logs.counts.error + logs.counts.fatal
            ));
            for entry in &logs.recent_critical {
                let level_short = match entry.level {
                    crate::schema::LogLevel::Error => "[E]",
                    crate::schema::LogLevel::Fatal => "[F]",
                    _ => "[!]",
                };
                output.push_str(&format!(
                    "  {} {}: {}\n",
                    entry.line_number,
                    level_short,
                    truncate(&entry.message, 60)
                ));
            }
        }

        output
    }

    /// Format a RepositoryStateSchema into compact output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CompactFormatter;
    /// use tars_cli::schema::RepositoryStateSchema;
    /// let mut state = RepositoryStateSchema::new();
    /// state.branch = Some("main".to_string());
    /// let output = CompactFormatter::format_repository_state(&state);
    /// assert!(output.contains("branch: main"));
    /// ```
    pub fn format_repository_state(state: &crate::schema::RepositoryStateSchema) -> String {
        if !state.is_git_repo {
            return "error: not a git repository\n".to_string();
        }

        let mut output = String::new();

        // Branch info
        if let Some(ref branch) = state.branch {
            if state.is_detached {
                output.push_str(&format!("branch: {} (detached)\n", branch));
            } else {
                output.push_str(&format!("branch: {}\n", branch));
            }
        }

        // Status
        if state.is_clean {
            output.push_str(&Self::format_clean());
        } else {
            output.push_str(&Self::format_dirty(
                state.counts.staged,
                state.counts.unstaged,
                state.counts.untracked,
                state.counts.unmerged,
            ));
        }

        output
    }

    /// Format a ProcessOutputSchema into compact output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CompactFormatter;
    /// use tars_cli::schema::ProcessOutputSchema;
    /// let mut proc = ProcessOutputSchema::new("echo");
    /// proc.stdout = "hello\n".to_string();
    /// proc.success = true;
    /// let output = CompactFormatter::format_process(&proc);
    /// assert!(output.contains("hello"));
    /// ```
    pub fn format_process(process: &crate::schema::ProcessOutputSchema) -> String {
        let mut output = String::new();

        // For successful commands, just show stdout
        if process.success {
            output.push_str(&process.stdout);
            if !process.stderr.is_empty() {
                output.push_str(&format!("stderr: {}", process.stderr));
            }
        } else {
            // For failures, show error info
            output.push_str(&format!("command: {}\n", process.command));
            if let Some(code) = process.exit_code {
                output.push_str(&format!("exit_code: {}\n", code));
            }
            if !process.stderr.is_empty() {
                output.push_str(&format!("stderr: {}", process.stderr));
            }
            if !process.stdout.is_empty() {
                output.push_str(&format!("stdout: {}", process.stdout));
            }
        }

        output
    }

    /// Format an ErrorSchema into compact output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CompactFormatter;
    /// use tars_cli::schema::ErrorSchema;
    /// let error = ErrorSchema::new("Something went wrong");
    /// let output = CompactFormatter::format_error_schema(&error);
    /// assert!(output.contains("error: Something went wrong"));
    /// ```
    pub fn format_error_schema(error: &crate::schema::ErrorSchema) -> String {
        let mut output = format!("error: {}\n", error.message);
        if let Some(ref code) = error.exit_code {
            output.push_str(&format!("exit_code: {}\n", code));
        }
        output
    }
}
