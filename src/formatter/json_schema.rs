//! Schema formatting methods for the JSON formatter.

use super::JsonFormatter;

#[allow(dead_code)]
impl JsonFormatter {
    /// Format a GitStatusSchema into JSON output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::JsonFormatter;
    /// use tars_cli::schema::{GitStatusSchema, GitFileEntry};
    /// let mut status = GitStatusSchema::new("main");
    /// status.is_clean = true;
    /// let output = JsonFormatter::format_git_status(&status);
    /// let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    /// assert_eq!(json["branch"], "main");
    /// assert_eq!(json["is_clean"], true);
    /// ```
    pub fn format_git_status(status: &crate::schema::GitStatusSchema) -> String {
        serde_json::to_string_pretty(status).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format a GitDiffSchema into JSON output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::JsonFormatter;
    /// use tars_cli::schema::GitDiffSchema;
    /// let diff = GitDiffSchema::new();
    /// let output = JsonFormatter::format_git_diff(&diff);
    /// let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    /// assert_eq!(json["is_empty"], true);
    /// ```
    pub fn format_git_diff(diff: &crate::schema::GitDiffSchema) -> String {
        serde_json::to_string_pretty(diff).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format a LsOutputSchema into JSON output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::JsonFormatter;
    /// use tars_cli::schema::{LsOutputSchema, LsEntry, LsEntryType};
    /// let mut ls = LsOutputSchema::new();
    /// ls.is_empty = false;
    /// ls.directories.push("src".to_string());
    /// ls.counts.directories = 1;
    /// ls.counts.total = 1;
    /// let output = JsonFormatter::format_ls(&ls);
    /// let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    /// assert_eq!(json["is_empty"], false);
    /// ```
    pub fn format_ls(ls: &crate::schema::LsOutputSchema) -> String {
        serde_json::to_string_pretty(ls).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format a GrepOutputSchema into JSON output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::JsonFormatter;
    /// use tars_cli::schema::{GrepOutputSchema, GrepFile, GrepMatch};
    /// let mut grep = GrepOutputSchema::new();
    /// grep.is_empty = false;
    /// let mut file = GrepFile::new("src/main.rs");
    /// file.matches.push(GrepMatch::new("fn main()"));
    /// grep.files.push(file);
    /// grep.counts.files = 1;
    /// grep.counts.matches = 1;
    /// let output = JsonFormatter::format_grep(&grep);
    /// let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    /// assert_eq!(json["is_empty"], false);
    /// ```
    pub fn format_grep(grep: &crate::schema::GrepOutputSchema) -> String {
        serde_json::to_string_pretty(grep).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format a FindOutputSchema into JSON output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::JsonFormatter;
    /// use tars_cli::schema::{FindOutputSchema, FindEntry};
    /// let mut find = FindOutputSchema::new();
    /// find.is_empty = false;
    /// find.files.push("./src/main.rs".to_string());
    /// find.counts.files = 1;
    /// find.counts.total = 1;
    /// let output = JsonFormatter::format_find(&find);
    /// let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    /// assert_eq!(json["is_empty"], false);
    /// ```
    pub fn format_find(find: &crate::schema::FindOutputSchema) -> String {
        serde_json::to_string_pretty(find).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format a TestOutputSchema into JSON output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::JsonFormatter;
    /// use tars_cli::schema::{TestOutputSchema, TestRunnerType};
    /// let mut test = TestOutputSchema::new(TestRunnerType::Pytest);
    /// test.is_empty = false;
    /// test.summary.passed = 10;
    /// test.summary.failed = 0;
    /// test.summary.total = 10;
    /// let output = JsonFormatter::format_test_output(&test);
    /// let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    /// assert_eq!(json["is_empty"], false);
    /// ```
    pub fn format_test_output(test: &crate::schema::TestOutputSchema) -> String {
        serde_json::to_string_pretty(test).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format a LogsOutputSchema into JSON output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::JsonFormatter;
    /// use tars_cli::schema::{LogsOutputSchema, LogEntry, LogLevel};
    /// let mut logs = LogsOutputSchema::new();
    /// logs.is_empty = false;
    /// logs.counts.total_lines = 10;
    /// logs.counts.info = 8;
    /// logs.counts.error = 2;
    /// let output = JsonFormatter::format_logs(&logs);
    /// let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    /// assert_eq!(json["is_empty"], false);
    /// ```
    pub fn format_logs(logs: &crate::schema::LogsOutputSchema) -> String {
        serde_json::to_string_pretty(logs).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format a RepositoryStateSchema into JSON output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::JsonFormatter;
    /// use tars_cli::schema::RepositoryStateSchema;
    /// let mut state = RepositoryStateSchema::new();
    /// state.branch = Some("main".to_string());
    /// let output = JsonFormatter::format_repository_state(&state);
    /// let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    /// assert_eq!(json["branch"], "main");
    /// ```
    pub fn format_repository_state(state: &crate::schema::RepositoryStateSchema) -> String {
        serde_json::to_string_pretty(state).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format a ProcessOutputSchema into JSON output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::JsonFormatter;
    /// use tars_cli::schema::ProcessOutputSchema;
    /// let mut proc = ProcessOutputSchema::new("echo");
    /// proc.stdout = "hello\n".to_string();
    /// proc.success = true;
    /// let output = JsonFormatter::format_process(&proc);
    /// let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    /// assert_eq!(json["success"], true);
    /// ```
    pub fn format_process(process: &crate::schema::ProcessOutputSchema) -> String {
        serde_json::to_string_pretty(process).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format an ErrorSchema into JSON output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::JsonFormatter;
    /// use tars_cli::schema::ErrorSchema;
    /// let error = ErrorSchema::new("Something went wrong");
    /// let output = JsonFormatter::format_error_schema(&error);
    /// let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    /// assert_eq!(json["message"], "Something went wrong");
    /// ```
    pub fn format_error_schema(error: &crate::schema::ErrorSchema) -> String {
        serde_json::to_string_pretty(error).unwrap_or_else(|_| "{}".to_string())
    }
}
