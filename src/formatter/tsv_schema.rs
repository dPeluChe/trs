//! Schema formatting methods for the TSV formatter.

use super::TsvFormatter;

#[allow(dead_code)]
impl TsvFormatter {
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
