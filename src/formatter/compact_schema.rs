//! Schema formatting methods for the Compact formatter.

use super::CompactFormatter;
use super::helpers::{truncate, format_duration};

#[allow(dead_code)]
impl CompactFormatter {
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
