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

}
