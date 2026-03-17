//! TSV formatter for tab-separated output.

use crate::OutputFormat;
use super::Formatter;

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
    pub fn format_message(key: &str, value: &str) -> String {
        format!(
            "{}\n{}\n",
            Self::escape_field(key),
            Self::escape_field(value)
        )
    }

    /// Format a key-value pair as TSV with header row.
    pub fn format_key_value(key: &str, value: &str) -> String {
        format!(
            "{}\n{}\n",
            Self::escape_field(key),
            Self::escape_field(value)
        )
    }

    /// Format multiple key-value pairs as TSV with headers.
    pub fn format_object(pairs: &[(&str, &str)]) -> String {
        let headers: Vec<String> = pairs.iter().map(|(k, _)| Self::escape_field(k)).collect();
        let values: Vec<String> = pairs.iter().map(|(_, v)| Self::escape_field(v)).collect();
        format!("{}\n{}\n", headers.join("\t"), values.join("\t"))
    }

    /// Format a count summary as TSV with header.
    pub fn format_counts(counts: &[(&str, usize)]) -> String {
        let headers: Vec<String> = counts
            .iter()
            .map(|(name, _)| Self::escape_field(name))
            .collect();
        let values: Vec<String> = counts.iter().map(|(_, count)| count.to_string()).collect();
        format!("{}\n{}\n", headers.join("\t"), values.join("\t"))
    }

    /// Format a section with items as TSV with headers.
    pub fn format_section(status_col: &str, path_col: &str, items: &[(&str, &str)]) -> String {
        let mut output = format!("{}\n", Self::format_header(&[status_col, path_col]).trim());
        for (status, path) in items {
            output.push_str(&format!("{}\n", Self::format_row(&[status, path]).trim()));
        }
        output
    }

    /// Format a list item with status and path as TSV.
    pub fn format_item(status: &str, path: &str) -> String {
        format!("{}\n", Self::format_row(&[status, path]).trim())
    }

    /// Format a list item with rename info as TSV.
    pub fn format_item_renamed(status: &str, old_path: &str, new_path: &str) -> String {
        format!(
            "{}\n",
            Self::format_row(&[status, new_path, old_path]).trim()
        )
    }

    /// Format a test result summary as TSV with header.
    pub fn format_test_summary(
        passed: usize,
        failed: usize,
        skipped: usize,
        duration_ms: u64,
    ) -> String {
        format!(
            "passed\tfailed\tskipped\ttotal\tduration_ms\n{}\t{}\t{}\t{}\t{}\n",
            passed,
            failed,
            skipped,
            passed + failed + skipped,
            duration_ms
        )
    }

    /// Format a success/failure status as TSV with header.
    pub fn format_status(success: bool) -> String {
        format!("success\n{}\n", success)
    }

    /// Format a list of failing tests as TSV with header.
    pub fn format_failures(failures: &[String]) -> String {
        let mut output = String::from("failure\n");
        for failure in failures {
            output.push_str(&format!("{}\n", Self::escape_field(failure)));
        }
        output
    }

    /// Format log level counts as TSV with header.
    pub fn format_log_levels(error: usize, warn: usize, info: usize, debug: usize) -> String {
        format!(
            "error\twarn\tinfo\tdebug\ttotal\n{}\t{}\t{}\t{}\t{}\n",
            error,
            warn,
            info,
            debug,
            error + warn + info + debug
        )
    }

    /// Format a grep match as TSV with header.
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
    pub fn format_grep_file(file: &str, match_count: usize) -> String {
        format!(
            "file\tmatch_count\n{}\t{}\n",
            Self::escape_field(file),
            match_count
        )
    }

    /// Format a diff file entry as TSV with header.
    pub fn format_diff_file(
        path: &str,
        change_type: &str,
        additions: usize,
        deletions: usize,
    ) -> String {
        format!(
            "path\tchange_type\tadditions\tdeletions\n{}\t{}\t{}\t{}\n",
            Self::escape_field(path),
            change_type,
            additions,
            deletions
        )
    }

    /// Format a diff summary as TSV with header.
    pub fn format_diff_summary(
        files_changed: usize,
        insertions: usize,
        deletions: usize,
    ) -> String {
        format!(
            "files_changed\tinsertions\tdeletions\n{}\t{}\t{}\n",
            files_changed, insertions, deletions
        )
    }

    /// Format a clean state indicator as TSV with header.
    pub fn format_clean() -> String {
        "is_clean\ntrue\n".to_string()
    }

    /// Format a dirty state indicator with counts as TSV with header.
    pub fn format_dirty(
        staged: usize,
        unstaged: usize,
        untracked: usize,
        unmerged: usize,
    ) -> String {
        format!(
            "is_clean\tstaged\tunstaged\tuntracked\tunmerged\nfalse\t{}\t{}\t{}\t{}\n",
            staged, unstaged, untracked, unmerged
        )
    }

    /// Format branch info with ahead/behind as TSV with header.
    pub fn format_branch_with_tracking(branch: &str, ahead: usize, behind: usize) -> String {
        format!(
            "branch\tahead\tbehind\n{}\t{}\t{}\n",
            Self::escape_field(branch),
            ahead,
            behind
        )
    }

    /// Format an empty result as TSV with header.
    pub fn format_empty() -> String {
        "empty\ntrue\n".to_string()
    }

    /// Format a truncation warning as TSV with header.
    pub fn format_truncated(shown: usize, total: usize) -> String {
        format!("is_truncated\tshown\ttotal\ntrue\t{}\t{}\n", shown, total)
    }

    /// Format an error message as TSV with header.
    pub fn format_error(message: &str) -> String {
        format!("error\tmessage\ntrue\t{}\n", Self::escape_field(message))
    }

    /// Format an error with exit code as TSV with header.
    pub fn format_error_with_code(message: &str, exit_code: i32) -> String {
        format!(
            "error\tmessage\texit_code\ntrue\t{}\t{}\n",
            Self::escape_field(message),
            exit_code
        )
    }

    /// Format a not-implemented message as TSV with header.
    pub fn format_not_implemented(message: &str) -> String {
        format!(
            "not_implemented\tmessage\ntrue\t{}\n",
            Self::escape_field(message)
        )
    }

    /// Format a command result as TSV with header.
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
    pub fn format_list(items: &[impl AsRef<str>]) -> String {
        let mut output = String::from("item\n");
        for item in items {
            output.push_str(&format!("{}\n", Self::escape_field(item.as_ref())));
        }
        output
    }

    /// Format a count as TSV with header.
    pub fn format_count(count: usize) -> String {
        format!("count\n{}\n", count)
    }

    /// Format a boolean flag as TSV with header.
    pub fn format_flag(name: &str, value: bool) -> String {
        format!("{}\n{}\n", Self::escape_field(name), value)
    }

    /// Format items with multiple columns as TSV with custom headers.
    pub fn format_table(headers: &[&str], rows: &[Vec<&str>]) -> String {
        let mut output = format!("{}\n", Self::format_header(headers).trim());
        for row in rows {
            output.push_str(&format!("{}\n", Self::format_row(row).trim()));
        }
        output
    }

    // ============================================================
    // Schema Formatting Methods
    // ============================================================

    /// Format a GitStatusSchema into TSV output.
    pub fn format_git_status(status: &crate::schema::GitStatusSchema) -> String {
        let mut output = String::new();

        output.push_str("branch\tis_clean\tahead\tbehind\tstaged\tunstaged\tuntracked\tunmerged\n");

        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            Self::escape_field(&status.branch),
            status.is_clean,
            status.ahead.unwrap_or(0),
            status.behind.unwrap_or(0),
            status.counts.staged,
            status.counts.unstaged,
            status.counts.untracked,
            status.counts.unmerged
        ));

        if !status.staged.is_empty()
            || !status.unstaged.is_empty()
            || !status.untracked.is_empty()
            || !status.unmerged.is_empty()
        {
            output.push('\n');
            output.push_str("section\tstatus\tpath\told_path\n");

            for entry in &status.staged {
                output.push_str(&format!(
                    "staged\t{}\t{}\t{}\n",
                    Self::escape_field(&entry.status),
                    Self::escape_field(&entry.path),
                    entry
                        .old_path
                        .as_deref()
                        .map(|p| Self::escape_field(p))
                        .unwrap_or_default()
                ));
            }

            for entry in &status.unstaged {
                output.push_str(&format!(
                    "unstaged\t{}\t{}\t{}\n",
                    Self::escape_field(&entry.status),
                    Self::escape_field(&entry.path),
                    entry
                        .old_path
                        .as_deref()
                        .map(|p| Self::escape_field(p))
                        .unwrap_or_default()
                ));
            }

            for entry in &status.untracked {
                output.push_str(&format!(
                    "untracked\t{}\t{}\t{}\n",
                    Self::escape_field(&entry.status),
                    Self::escape_field(&entry.path),
                    entry
                        .old_path
                        .as_deref()
                        .map(|p| Self::escape_field(p))
                        .unwrap_or_default()
                ));
            }

            for entry in &status.unmerged {
                output.push_str(&format!(
                    "unmerged\t{}\t{}\t{}\n",
                    Self::escape_field(&entry.status),
                    Self::escape_field(&entry.path),
                    entry
                        .old_path
                        .as_deref()
                        .map(|p| Self::escape_field(p))
                        .unwrap_or_default()
                ));
            }
        }

        output
    }

    /// Format a GitDiffSchema into TSV output.
    pub fn format_git_diff(diff: &crate::schema::GitDiffSchema) -> String {
        if diff.is_empty {
            return "is_empty\ntrue\n".to_string();
        }

        let mut output = String::new();

        output.push_str("total_files\ttotal_additions\ttotal_deletions\tis_truncated\n");
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            diff.counts.total_files, diff.total_additions, diff.total_deletions, diff.is_truncated
        ));

        if !diff.files.is_empty() {
            output.push('\n');
            output.push_str("path\told_path\tchange_type\tadditions\tdeletions\tis_binary\n");

            for file in &diff.files {
                output.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\n",
                    Self::escape_field(&file.path),
                    file.old_path
                        .as_deref()
                        .map(|p| Self::escape_field(p))
                        .unwrap_or_default(),
                    Self::escape_field(&file.change_type),
                    file.additions,
                    file.deletions,
                    file.is_binary
                ));
            }
        }

        output
    }

    /// Format a LsOutputSchema into TSV output.
    pub fn format_ls(ls: &crate::schema::LsOutputSchema) -> String {
        if ls.is_empty {
            return Self::format_empty();
        }

        let mut output = String::new();

        output.push_str("total\tdirectories\tfiles\tsymlinks\thidden\tgenerated\n");
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            ls.counts.total,
            ls.counts.directories,
            ls.counts.files,
            ls.counts.symlinks,
            ls.counts.hidden,
            ls.counts.generated
        ));

        if !ls.entries.is_empty() {
            output.push('\n');
            output.push_str("name\ttype\tis_hidden\tis_symlink\tsymlink_target\tis_broken\n");

            for entry in &ls.entries {
                let type_str = match entry.entry_type {
                    crate::schema::LsEntryType::File => "file",
                    crate::schema::LsEntryType::Directory => "directory",
                    crate::schema::LsEntryType::Symlink => "symlink",
                    crate::schema::LsEntryType::BlockDevice => "block_device",
                    crate::schema::LsEntryType::CharDevice => "char_device",
                    crate::schema::LsEntryType::Socket => "socket",
                    crate::schema::LsEntryType::Pipe => "pipe",
                    crate::schema::LsEntryType::Other => "other",
                };
                output.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\n",
                    Self::escape_field(&entry.name),
                    type_str,
                    entry.is_hidden,
                    entry.is_symlink,
                    entry
                        .symlink_target
                        .as_deref()
                        .map(|t| Self::escape_field(t))
                        .unwrap_or_default(),
                    entry.is_broken_symlink
                ));
            }
        }

        if !ls.errors.is_empty() {
            output.push('\n');
            output.push_str("error_path\terror_message\n");
            for error in &ls.errors {
                output.push_str(&format!(
                    "{}\t{}\n",
                    Self::escape_field(&error.path),
                    Self::escape_field(&error.message)
                ));
            }
        }

        output
    }

    /// Format a GrepOutputSchema into TSV output.
    pub fn format_grep(grep: &crate::schema::GrepOutputSchema) -> String {
        if grep.is_empty {
            return "is_empty\ntrue\n".to_string();
        }

        let mut output = String::new();

        output.push_str("files\tmatches\ttotal_files\tis_truncated\n");
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            grep.counts.files, grep.counts.matches, grep.counts.total_files, grep.is_truncated
        ));

        output.push('\n');
        output.push_str("file\tline_number\tcolumn\tcontent\tis_context\n");

        for file in &grep.files {
            for m in &file.matches {
                output.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\n",
                    Self::escape_field(&file.path),
                    m.line_number.unwrap_or(0),
                    m.column.unwrap_or(0),
                    Self::escape_field(m.line.trim()),
                    m.is_context
                ));
            }
        }

        output
    }

    /// Format a FindOutputSchema into TSV output.
    pub fn format_find(find: &crate::schema::FindOutputSchema) -> String {
        if find.is_empty {
            return "is_empty\ntrue\n".to_string();
        }

        let mut output = String::new();

        output.push_str("total\tdirectories\tfiles\n");
        output.push_str(&format!(
            "{}\t{}\t{}\n",
            find.counts.total, find.counts.directories, find.counts.files
        ));

        output.push('\n');
        output.push_str("path\tis_directory\tis_hidden\textension\tdepth\n");

        for entry in &find.entries {
            output.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\n",
                Self::escape_field(&entry.path),
                entry.is_directory,
                entry.is_hidden,
                entry.extension.as_deref().unwrap_or(""),
                entry.depth
            ));
        }

        if !find.errors.is_empty() {
            output.push('\n');
            output.push_str("error_path\terror_message\n");
            for error in &find.errors {
                output.push_str(&format!(
                    "{}\t{}\n",
                    Self::escape_field(&error.path),
                    Self::escape_field(&error.message)
                ));
            }
        }

        output
    }

    /// Format a TestOutputSchema into TSV output.
    pub fn format_test_output(test: &crate::schema::TestOutputSchema) -> String {
        if test.is_empty {
            return "is_empty\ntrue\n".to_string();
        }

        let mut output = String::new();

        output.push_str("runner\tsuccess\ttotal\tpassed\tfailed\tskipped\tduration_ms\n");
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            test.runner,
            test.success,
            test.summary.total,
            test.summary.passed,
            test.summary.failed,
            test.summary.skipped,
            test.summary.duration_ms.unwrap_or(0)
        ));

        output.push('\n');
        output.push_str("suite_file\ttest_name\tstatus\tduration_ms\terror_message\n");

        for suite in &test.test_suites {
            for t in &suite.tests {
                let status_str = match t.status {
                    crate::schema::TestStatus::Passed => "passed",
                    crate::schema::TestStatus::Failed => "failed",
                    crate::schema::TestStatus::Skipped => "skipped",
                    crate::schema::TestStatus::XFailed => "xfailed",
                    crate::schema::TestStatus::XPassed => "xpassed",
                    crate::schema::TestStatus::Error => "error",
                    crate::schema::TestStatus::Todo => "todo",
                };
                output.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\n",
                    Self::escape_field(&suite.file),
                    Self::escape_field(&t.name),
                    status_str,
                    t.duration_ms.unwrap_or(0),
                    t.error_message
                        .as_deref()
                        .map(|e| Self::escape_field(e))
                        .unwrap_or_default()
                ));
            }
        }

        output
    }

    /// Format a LogsOutputSchema into TSV output.
    pub fn format_logs(logs: &crate::schema::LogsOutputSchema) -> String {
        if logs.is_empty {
            return "is_empty\ntrue\n".to_string();
        }

        let mut output = String::new();

        output.push_str("total_lines\tdebug\tinfo\twarning\terror\tfatal\tunknown\n");
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            logs.counts.total_lines,
            logs.counts.debug,
            logs.counts.info,
            logs.counts.warning,
            logs.counts.error,
            logs.counts.fatal,
            logs.counts.unknown
        ));

        if !logs.entries.is_empty() {
            output.push('\n');
            output.push_str("line_number\tlevel\ttimestamp\tsource\tmessage\n");

            for entry in &logs.entries {
                let level_str = match entry.level {
                    crate::schema::LogLevel::Debug => "debug",
                    crate::schema::LogLevel::Info => "info",
                    crate::schema::LogLevel::Warning => "warning",
                    crate::schema::LogLevel::Error => "error",
                    crate::schema::LogLevel::Fatal => "fatal",
                    crate::schema::LogLevel::Unknown => "unknown",
                };
                output.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\n",
                    entry.line_number,
                    level_str,
                    entry.timestamp.as_deref().unwrap_or(""),
                    entry.source.as_deref().unwrap_or(""),
                    Self::escape_field(&entry.message)
                ));
            }
        }

        if !logs.recent_critical.is_empty() {
            output.push('\n');
            output.push_str("critical_line_number\tcritical_level\tcritical_message\n");
            for entry in &logs.recent_critical {
                let level_str = match entry.level {
                    crate::schema::LogLevel::Error => "error",
                    crate::schema::LogLevel::Fatal => "fatal",
                    _ => "critical",
                };
                output.push_str(&format!(
                    "{}\t{}\t{}\n",
                    entry.line_number,
                    level_str,
                    Self::escape_field(&entry.message)
                ));
            }
        }

        output
    }

    /// Format a RepositoryStateSchema into TSV output.
    pub fn format_repository_state(state: &crate::schema::RepositoryStateSchema) -> String {
        if !state.is_git_repo {
            return "is_git_repo\nfalse\n".to_string();
        }

        let mut output = String::new();

        output.push_str(
            "is_git_repo\tis_clean\tis_detached\tbranch\tstaged\tunstaged\tuntracked\tunmerged\n",
        );
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            state.is_git_repo,
            state.is_clean,
            state.is_detached,
            state.branch.as_deref().unwrap_or(""),
            state.counts.staged,
            state.counts.unstaged,
            state.counts.untracked,
            state.counts.unmerged
        ));

        output
    }

    /// Format a ProcessOutputSchema into TSV output.
    pub fn format_process(process: &crate::schema::ProcessOutputSchema) -> String {
        let mut output = String::new();

        output.push_str("command\targs\texit_code\tduration_ms\ttimed_out\tsuccess\n");
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            Self::escape_field(&process.command),
            Self::escape_field(&process.args.join(" ")),
            process.exit_code.unwrap_or(-1),
            process.duration_ms,
            process.timed_out,
            process.success
        ));

        if !process.stdout.is_empty() {
            output.push('\n');
            output.push_str("stdout\n");
            output.push_str(&Self::escape_field(&process.stdout));
            output.push('\n');
        }

        if !process.stderr.is_empty() {
            output.push('\n');
            output.push_str("stderr\n");
            output.push_str(&Self::escape_field(&process.stderr));
            output.push('\n');
        }

        output
    }

    /// Format an ErrorSchema into TSV output.
    pub fn format_error_schema(error: &crate::schema::ErrorSchema) -> String {
        format!(
            "error\tmessage\terror_type\texit_code\ntrue\t{}\t{}\t{}\n",
            Self::escape_field(&error.message),
            error.error_type.as_deref().unwrap_or(""),
            error.exit_code.unwrap_or(-1)
        )
    }
}
