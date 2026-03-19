//! CSV formatter for comma-separated output.

use super::Formatter;
use crate::OutputFormat;

/// Formatter for CSV (Comma-Separated Values) output.
///
/// The CSV formatter produces tabular output that:
/// - Has a header row
/// - Uses commas as delimiters
/// - Properly escapes special characters
/// - Is compatible with spreadsheet tools
#[allow(dead_code)]
pub struct CsvFormatter;

impl Formatter for CsvFormatter {
    fn name() -> &'static str {
        "csv"
    }

    fn format() -> OutputFormat {
        OutputFormat::Csv
    }
}

#[allow(dead_code)]
impl CsvFormatter {
    /// Escape a field for CSV format.
    pub fn escape_field(field: &str) -> String {
        if field.contains(',')
            || field.contains('"')
            || field.contains('\n')
            || field.contains('\r')
        {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }

    /// Format a CSV header row from field names.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_header(&["branch", "is_clean", "count"]);
    /// assert_eq!(output, "branch,is_clean,count\n");
    /// ```
    pub fn format_header(fields: &[&str]) -> String {
        let escaped: Vec<String> = fields.iter().map(|f| Self::escape_field(f)).collect();
        format!("{}\n", escaped.join(","))
    }

    /// Format a CSV data row from field values.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_row(&["main", "true", "5"]);
    /// assert_eq!(output, "main,true,5\n");
    /// ```
    pub fn format_row(values: &[&str]) -> String {
        let escaped: Vec<String> = values.iter().map(|v| Self::escape_field(v)).collect();
        format!("{}\n", escaped.join(","))
    }

    /// Format a simple message/status as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_message("key", "value");
    /// assert_eq!(output, "key\nvalue\n");
    /// ```
    pub fn format_message(key: &str, value: &str) -> String {
        format!(
            "{}\n{}\n",
            Self::escape_field(key),
            Self::escape_field(value)
        )
    }

    /// Format a key-value pair as CSV with header row.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_key_value("branch", "main");
    /// assert_eq!(output, "branch\nmain\n");
    /// ```
    pub fn format_key_value(key: &str, value: &str) -> String {
        format!(
            "{}\n{}\n",
            Self::escape_field(key),
            Self::escape_field(value)
        )
    }

    /// Format multiple key-value pairs as CSV with headers.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_object(&[("branch", "main"), ("is_clean", "true"), ("count", "5")]);
    /// assert!(output.contains("branch,is_clean,count"));
    /// assert!(output.contains("main,true,5"));
    /// ```
    pub fn format_object(pairs: &[(&str, &str)]) -> String {
        let headers: Vec<String> = pairs.iter().map(|(k, _)| Self::escape_field(k)).collect();
        let values: Vec<String> = pairs.iter().map(|(_, v)| Self::escape_field(v)).collect();
        format!("{}\n{}\n", headers.join(","), values.join(","))
    }

    /// Format a count summary as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_counts(&[("passed", 10), ("failed", 2)]);
    /// assert!(output.contains("passed,failed"));
    /// assert!(output.contains("10,2"));
    /// ```
    pub fn format_counts(counts: &[(&str, usize)]) -> String {
        let headers: Vec<String> = counts
            .iter()
            .map(|(name, _)| Self::escape_field(name))
            .collect();
        let values: Vec<String> = counts.iter().map(|(_, count)| count.to_string()).collect();
        format!("{}\n{}\n", headers.join(","), values.join(","))
    }

    /// Format a section with items as CSV with headers.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_section("status", "path", &[("M", "src/main.rs"), ("A", "src/new.rs")]);
    /// assert!(output.contains("status,path"));
    /// assert!(output.contains("M,src/main.rs"));
    /// assert!(output.contains("A,src/new.rs"));
    /// ```
    pub fn format_section(status_col: &str, path_col: &str, items: &[(&str, &str)]) -> String {
        let mut output = format!("{}\n", Self::format_header(&[status_col, path_col]).trim());
        for (status, path) in items {
            output.push_str(&format!("{}\n", Self::format_row(&[status, path]).trim()));
        }
        output
    }

    /// Format a list item with status and path as CSV.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_item("M", "src/main.rs");
    /// assert_eq!(output, "M,src/main.rs\n");
    /// ```
    pub fn format_item(status: &str, path: &str) -> String {
        format!("{}\n", Self::format_row(&[status, path]).trim())
    }

    /// Format a list item with rename info as CSV.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_item_renamed("R", "old.rs", "new.rs");
    /// assert_eq!(output, "R,new.rs,old.rs\n");
    /// ```
    pub fn format_item_renamed(status: &str, old_path: &str, new_path: &str) -> String {
        format!(
            "{}\n",
            Self::format_row(&[status, new_path, old_path]).trim()
        )
    }

    /// Format a test result summary as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_test_summary(10, 2, 1, 1500);
    /// assert!(output.contains("passed,failed,skipped,total,duration_ms"));
    /// assert!(output.contains("10,2,1,13,1500"));
    /// ```
    pub fn format_test_summary(
        passed: usize,
        failed: usize,
        skipped: usize,
        duration_ms: u64,
    ) -> String {
        format!(
            "passed,failed,skipped,total,duration_ms\n{},{},{},{},{}\n",
            passed,
            failed,
            skipped,
            passed + failed + skipped,
            duration_ms
        )
    }

    /// Format a success/failure status as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_status(true);
    /// assert_eq!(output, "success\ntrue\n");
    /// ```
    pub fn format_status(success: bool) -> String {
        format!("success\n{}\n", success)
    }

    /// Format a list of failing tests as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_failures(&["test_one".to_string(), "test_two".to_string()]);
    /// assert!(output.contains("failure"));
    /// assert!(output.contains("test_one"));
    /// assert!(output.contains("test_two"));
    /// ```
    pub fn format_failures(failures: &[String]) -> String {
        let mut output = String::from("failure\n");
        for failure in failures {
            output.push_str(&format!("{}\n", Self::escape_field(failure)));
        }
        output
    }

    /// Format log level counts as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_log_levels(2, 5, 10, 3);
    /// assert!(output.contains("error,warn,info,debug,total"));
    /// assert!(output.contains("2,5,10,3,20"));
    /// ```
    pub fn format_log_levels(error: usize, warn: usize, info: usize, debug: usize) -> String {
        format!(
            "error,warn,info,debug,total\n{},{},{},{},{}\n",
            error,
            warn,
            info,
            debug,
            error + warn + info + debug
        )
    }

    /// Format a grep match as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_grep_match("src/main.rs", Some(42), "fn main()");
    /// assert!(output.contains("file,line,content"));
    /// assert!(output.contains("src/main.rs,42,fn main()"));
    /// ```
    pub fn format_grep_match(file: &str, line: Option<usize>, content: &str) -> String {
        match line {
            Some(l) => format!(
                "file,line,content\n{},{},{}\n",
                Self::escape_field(file),
                l,
                Self::escape_field(content.trim())
            ),
            None => format!(
                "file,line,content\n{},{},{}\n",
                Self::escape_field(file),
                "",
                Self::escape_field(content.trim())
            ),
        }
    }

    /// Format a grep file with match count as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_grep_file("src/main.rs", 5);
    /// assert_eq!(output, "file,match_count\nsrc/main.rs,5\n");
    /// ```
    pub fn format_grep_file(file: &str, match_count: usize) -> String {
        format!(
            "file,match_count\n{},{}\n",
            Self::escape_field(file),
            match_count
        )
    }

    /// Format a diff file entry as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_diff_file("src/main.rs", "M", 10, 5);
    /// assert_eq!(output, "path,change_type,additions,deletions\nsrc/main.rs,M,10,5\n");
    /// ```
    pub fn format_diff_file(
        path: &str,
        change_type: &str,
        additions: usize,
        deletions: usize,
    ) -> String {
        format!(
            "path,change_type,additions,deletions\n{},{},{},{}\n",
            Self::escape_field(path),
            change_type,
            additions,
            deletions
        )
    }

    /// Format a diff summary as CSV with header.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CsvFormatter;
    /// let output = CsvFormatter::format_diff_summary(3, 25, 10);
    /// assert_eq!(output, "files_changed,insertions,deletions\n3,25,10\n");
    /// ```
    pub fn format_diff_summary(
        files_changed: usize,
        insertions: usize,
        deletions: usize,
    ) -> String {
        format!(
            "files_changed,insertions,deletions\n{},{},{}\n",
            files_changed, insertions, deletions
        )
    }

    /// Format a clean state indicator as CSV with header.
    pub fn format_clean() -> String {
        "is_clean\ntrue\n".to_string()
    }

    /// Format a dirty state indicator with counts as CSV with header.
    pub fn format_dirty(
        staged: usize,
        unstaged: usize,
        untracked: usize,
        unmerged: usize,
    ) -> String {
        format!(
            "is_clean,staged,unstaged,untracked,unmerged\nfalse,{},{},{},{}\n",
            staged, unstaged, untracked, unmerged
        )
    }

    /// Format branch info with ahead/behind as CSV with header.
    pub fn format_branch_with_tracking(branch: &str, ahead: usize, behind: usize) -> String {
        format!(
            "branch,ahead,behind\n{},{},{}\n",
            Self::escape_field(branch),
            ahead,
            behind
        )
    }

    /// Format an empty result as CSV with header.
    pub fn format_empty() -> String {
        "empty\ntrue\n".to_string()
    }

    /// Format a truncation warning as CSV with header.
    pub fn format_truncated(shown: usize, total: usize) -> String {
        format!("is_truncated,shown,total\ntrue,{},{}\n", shown, total)
    }

    /// Format an error message as CSV with header.
    pub fn format_error(message: &str) -> String {
        format!("error,message\ntrue,{}\n", Self::escape_field(message))
    }

    /// Format an error with exit code as CSV with header.
    pub fn format_error_with_code(message: &str, exit_code: i32) -> String {
        format!(
            "error,message,exit_code\ntrue,{},{}\n",
            Self::escape_field(message),
            exit_code
        )
    }

    /// Format a not-implemented message as CSV with header.
    pub fn format_not_implemented(message: &str) -> String {
        format!(
            "not_implemented,message\ntrue,{}\n",
            Self::escape_field(message)
        )
    }

    /// Format a command result as CSV with header.
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
            "command,args,stdout,stderr,exit_code,duration_ms\n{},{},{},{},{},{}\n",
            Self::escape_field(command),
            Self::escape_field(&args_str),
            Self::escape_field(stdout),
            Self::escape_field(stderr),
            exit_code,
            duration_ms
        )
    }

    /// Format a list of strings as CSV with header.
    pub fn format_list(items: &[impl AsRef<str>]) -> String {
        let mut output = String::from("item\n");
        for item in items {
            output.push_str(&format!("{}\n", Self::escape_field(item.as_ref())));
        }
        output
    }

    /// Format a count as CSV with header.
    pub fn format_count(count: usize) -> String {
        format!("count\n{}\n", count)
    }

    /// Format a boolean flag as CSV with header.
    pub fn format_flag(name: &str, value: bool) -> String {
        format!("{}\n{}\n", Self::escape_field(name), value)
    }

    /// Format items with multiple columns as CSV with custom headers.
    pub fn format_table(headers: &[&str], rows: &[Vec<&str>]) -> String {
        let mut output = format!("{}\n", Self::format_header(headers).trim());
        for row in rows {
            output.push_str(&format!("{}\n", Self::format_row(row).trim()));
        }
        output
    }
}
