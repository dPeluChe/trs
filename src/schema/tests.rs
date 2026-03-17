#[cfg(test)]
mod tests {
    use crate::schema::*;
    #[allow(unused_imports)]
    use serde::Serialize;

    // ============================================================
    // Schema Version Tests
    // ============================================================

    #[test]
    fn test_schema_version() {
        let version = SchemaVersion::new("test_type");
        assert_eq!(version.version, SCHEMA_VERSION);
        assert_eq!(version.schema_type, "test_type");
    }

    #[test]
    fn test_schema_version_serialization() {
        let version = SchemaVersion::new("git_status");
        let json = serde_json::to_string(&version).unwrap();
        assert!(json.contains("\"version\":\"1.0.0\""));
        assert!(json.contains("\"type\":\"git_status\""));
    }

    // ============================================================
    // Git Status Schema Tests
    // ============================================================

    #[test]
    fn test_git_status_schema_new() {
        let schema = GitStatusSchema::new("main");
        assert_eq!(schema.branch, "main");
        assert!(schema.is_clean);
        assert!(schema.staged.is_empty());
        assert!(schema.unstaged.is_empty());
        assert!(schema.untracked.is_empty());
        assert!(schema.unmerged.is_empty());
    }

    #[test]
    fn test_git_status_schema_serialization() {
        let schema = GitStatusSchema::new("main");
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("\"branch\":\"main\""));
        assert!(json.contains("\"is_clean\":true"));
        assert!(json.contains("\"type\":\"git_status\""));
    }

    #[test]
    fn test_git_file_entry_new() {
        let entry = GitFileEntry::new("M", "src/main.rs");
        assert_eq!(entry.status, "M");
        assert_eq!(entry.path, "src/main.rs");
        assert!(entry.old_path.is_none());
    }

    #[test]
    fn test_git_file_entry_renamed() {
        let entry = GitFileEntry::renamed("R", "old.rs", "new.rs");
        assert_eq!(entry.status, "R");
        assert_eq!(entry.path, "new.rs");
        assert_eq!(entry.old_path, Some("old.rs".to_string()));
    }

    #[test]
    fn test_git_status_counts_default() {
        let counts = GitStatusCounts::default();
        assert_eq!(counts.staged, 0);
        assert_eq!(counts.unstaged, 0);
        assert_eq!(counts.untracked, 0);
        assert_eq!(counts.unmerged, 0);
    }

    // ============================================================
    // Git Diff Schema Tests
    // ============================================================

    #[test]
    fn test_git_diff_schema_new() {
        let schema = GitDiffSchema::new();
        assert!(schema.is_empty);
        assert!(!schema.is_truncated);
        assert!(schema.files.is_empty());
    }

    #[test]
    fn test_git_diff_entry_new() {
        let entry = GitDiffEntry::new("src/main.rs", "M");
        assert_eq!(entry.path, "src/main.rs");
        assert_eq!(entry.change_type, "M");
        assert!(entry.old_path.is_none());
    }

    // ============================================================
    // Repository State Schema Tests
    // ============================================================

    #[test]
    fn test_repository_state_schema_new() {
        let schema = RepositoryStateSchema::new();
        assert!(schema.is_git_repo);
        assert!(schema.is_clean);
        assert!(!schema.is_detached);
        assert!(schema.branch.is_none());
    }

    // ============================================================
    // LS Output Schema Tests
    // ============================================================

    #[test]
    fn test_ls_output_schema_new() {
        let schema = LsOutputSchema::new();
        assert!(schema.is_empty);
        assert!(schema.entries.is_empty());
        assert!(schema.directories.is_empty());
        assert!(schema.files.is_empty());
    }

    #[test]
    fn test_ls_entry_new() {
        let entry = LsEntry::new("src", LsEntryType::Directory);
        assert_eq!(entry.name, "src");
        assert_eq!(entry.entry_type, LsEntryType::Directory);
        assert!(!entry.is_hidden);
    }

    #[test]
    fn test_ls_entry_hidden() {
        let entry = LsEntry::new(".gitignore", LsEntryType::File);
        assert!(entry.is_hidden);
    }

    #[test]
    fn test_ls_entry_type_serialization() {
        let entry = LsEntry::new("link", LsEntryType::Symlink);
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"type\":\"symlink\""));
    }

    // ============================================================
    // Find Output Schema Tests
    // ============================================================

    #[test]
    fn test_find_output_schema_new() {
        let schema = FindOutputSchema::new();
        assert!(schema.is_empty);
        assert!(schema.entries.is_empty());
        assert!(schema.directories.is_empty());
        assert!(schema.files.is_empty());
    }

    #[test]
    fn test_find_entry_new() {
        let entry = FindEntry::new("./src/main.rs");
        assert_eq!(entry.path, "./src/main.rs");
        assert!(!entry.is_directory);
    }

    #[test]
    fn test_find_entry_hidden_detection() {
        let entry = FindEntry::new("./.git/config");
        assert!(entry.is_hidden);
    }

    // ============================================================
    // Grep Output Schema Tests
    // ============================================================

    #[test]
    fn test_grep_output_schema_new() {
        let schema = GrepOutputSchema::new();
        assert!(schema.is_empty);
        assert!(!schema.is_truncated);
        assert!(schema.files.is_empty());
    }

    #[test]
    fn test_grep_file_new() {
        let file = GrepFile::new("src/main.rs");
        assert_eq!(file.path, "src/main.rs");
        assert!(file.matches.is_empty());
    }

    #[test]
    fn test_grep_match_new() {
        let m = GrepMatch::new("fn main() {");
        assert_eq!(m.line, "fn main() {");
        assert!(m.line_number.is_none());
        assert!(!m.is_context);
    }

    // ============================================================
    // Replace Output Schema Tests
    // ============================================================

    #[test]
    fn test_replace_output_schema_new() {
        let schema = ReplaceOutputSchema::new("old", "new", false);
        assert!(!schema.dry_run);
        assert_eq!(schema.search_pattern, "old");
        assert_eq!(schema.replacement, "new");
        assert!(schema.files.is_empty());
        assert_eq!(schema.counts.files_affected, 0);
        assert_eq!(schema.counts.total_replacements, 0);
    }

    #[test]
    fn test_replace_output_schema_dry_run() {
        let schema = ReplaceOutputSchema::new("pattern", "replacement", true);
        assert!(schema.dry_run);
    }

    #[test]
    fn test_replace_file_new() {
        let file = ReplaceFile::new("src/main.rs");
        assert_eq!(file.path, "src/main.rs");
        assert!(file.matches.is_empty());
    }

    #[test]
    fn test_replace_match_new() {
        let m = ReplaceMatch::new(10, "old_function()", "new_function()");
        assert_eq!(m.line_number, 10);
        assert_eq!(m.original, "old_function()");
        assert_eq!(m.replaced, "new_function()");
    }

    #[test]
    fn test_replace_output_schema_serialization() {
        let schema = ReplaceOutputSchema::new("old", "new", true);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("\"dry_run\":true"));
        assert!(json.contains("\"search_pattern\":\"old\""));
        assert!(json.contains("\"replacement\":\"new\""));
        assert!(json.contains("\"type\":\"replace_output\""));
    }

    #[test]
    fn test_replace_counts_default() {
        let counts = ReplaceCounts::default();
        assert_eq!(counts.files_affected, 0);
        assert_eq!(counts.total_replacements, 0);
    }

    #[test]
    fn test_replace_output_schema_with_files() {
        let mut file = ReplaceFile::new("test.rs");
        file.matches.push(ReplaceMatch::new(1, "old", "new"));

        let schema = ReplaceOutputSchema::new("old", "new", false)
            .with_files(vec![file])
            .with_counts(ReplaceCounts {
                files_affected: 1,
                total_replacements: 1,
            });

        assert_eq!(schema.files.len(), 1);
        assert_eq!(schema.files[0].path, "test.rs");
        assert_eq!(schema.counts.files_affected, 1);
        assert_eq!(schema.counts.total_replacements, 1);
    }

    #[test]
    fn test_replace_output_round_trip() {
        let mut file = ReplaceFile::new("src/lib.rs");
        file.matches.push(ReplaceMatch::new(5, "foo", "bar"));
        file.matches.push(ReplaceMatch::new(10, "foo", "bar"));

        let original = ReplaceOutputSchema::new("foo", "bar", true)
            .with_files(vec![file])
            .with_counts(ReplaceCounts {
                files_affected: 1,
                total_replacements: 2,
            });

        let json = serde_json::to_string(&original).unwrap();
        let parsed: ReplaceOutputSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    // ============================================================
    // Test Output Schema Tests
    // ============================================================

    #[test]
    fn test_test_output_schema_new() {
        let schema = TestOutputSchema::new(TestRunnerType::Pytest);
        assert_eq!(schema.runner, TestRunnerType::Pytest);
        assert!(schema.is_empty);
        assert!(schema.success);
        assert!(schema.test_suites.is_empty());
    }

    #[test]
    fn test_test_suite_new() {
        let suite = TestSuite::new("tests/test_main.py");
        assert_eq!(suite.file, "tests/test_main.py");
        assert!(suite.passed);
        assert!(suite.tests.is_empty());
    }

    #[test]
    fn test_test_result_new() {
        let result = TestResult::new("test_example", TestStatus::Passed);
        assert_eq!(result.name, "test_example");
        assert_eq!(result.status, TestStatus::Passed);
    }

    #[test]
    fn test_test_runner_type_display() {
        assert_eq!(TestRunnerType::Pytest.to_string(), "pytest");
        assert_eq!(TestRunnerType::Jest.to_string(), "jest");
        assert_eq!(TestRunnerType::Vitest.to_string(), "vitest");
        assert_eq!(TestRunnerType::Npm.to_string(), "npm");
        assert_eq!(TestRunnerType::Pnpm.to_string(), "pnpm");
        assert_eq!(TestRunnerType::Bun.to_string(), "bun");
    }

    #[test]
    fn test_test_status_serialization() {
        let result = TestResult::new("test", TestStatus::Passed);
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"status\":\"passed\""));
    }

    // ============================================================
    // Logs Output Schema Tests
    // ============================================================

    #[test]
    fn test_logs_output_schema_new() {
        let schema = LogsOutputSchema::new();
        assert!(schema.is_empty);
        assert!(schema.entries.is_empty());
        assert!(schema.recent_critical.is_empty());
    }

    #[test]
    fn test_log_entry_new() {
        let entry = LogEntry::new("[INFO] Application started", 1);
        assert_eq!(entry.line, "[INFO] Application started");
        assert_eq!(entry.line_number, 1);
        assert_eq!(entry.level, LogLevel::Unknown);
    }

    #[test]
    fn test_log_level_serialization() {
        let entry = LogEntry {
            line: "test".to_string(),
            level: LogLevel::Error,
            timestamp: None,
            source: None,
            message: "test".to_string(),
            line_number: 1,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"level\":\"error\""));
    }

    // ============================================================
    // Process Output Schema Tests
    // ============================================================

    #[test]
    fn test_process_output_schema_new() {
        let schema = ProcessOutputSchema::new("echo");
        assert_eq!(schema.command, "echo");
        assert!(schema.args.is_empty());
        assert!(schema.stdout.is_empty());
        assert!(schema.stderr.is_empty());
        assert!(schema.exit_code.is_none());
        assert!(schema.success);
    }

    #[test]
    fn test_process_output_schema_serialization() {
        let schema = ProcessOutputSchema::new("echo");
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("\"command\":\"echo\""));
        assert!(json.contains("\"type\":\"process_output\""));
    }

    // ============================================================
    // Error Schema Tests
    // ============================================================

    #[test]
    fn test_error_schema_new() {
        let error = ErrorSchema::new("Something went wrong");
        assert!(error.error);
        assert_eq!(error.message, "Something went wrong");
        assert!(error.error_type.is_none());
    }

    #[test]
    fn test_error_schema_with_type() {
        let error = ErrorSchema::with_type("Command not found", "command_not_found");
        assert!(error.error);
        assert_eq!(error.message, "Command not found");
        assert_eq!(error.error_type, Some("command_not_found".to_string()));
    }

    #[test]
    fn test_error_schema_serialization() {
        let error = ErrorSchema::new("Test error");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"error\":true"));
        assert!(json.contains("\"message\":\"Test error\""));
        assert!(json.contains("\"type\":\"error\""));
    }

    // ============================================================
    // Deserialization Tests
    // ============================================================

    #[test]
    fn test_git_status_schema_deserialization() {
        let json = r#"{
            "schema": {"version": "1.0.0", "type": "git_status"},
            "branch": "main",
            "is_clean": true,
            "staged": [],
            "unstaged": [],
            "untracked": [],
            "unmerged": [],
            "counts": {"staged": 0, "unstaged": 0, "untracked": 0, "unmerged": 0}
        }"#;
        let schema: GitStatusSchema = serde_json::from_str(json).unwrap();
        assert_eq!(schema.branch, "main");
        assert!(schema.is_clean);
    }

    #[test]
    fn test_ls_entry_type_deserialization() {
        let json =
            r#"{"name": "src", "type": "directory", "is_hidden": false, "is_symlink": false}"#;
        let entry: LsEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.name, "src");
        assert_eq!(entry.entry_type, LsEntryType::Directory);
    }

    #[test]
    fn test_test_status_deserialization() {
        let json = r#"{"name": "test", "status": "failed"}"#;
        let result: TestResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.status, TestStatus::Failed);
    }

    // ============================================================
    // Round-trip Tests
    // ============================================================

    #[test]
    fn test_git_status_round_trip() {
        let original = GitStatusSchema::new("feature/test");
        let json = serde_json::to_string(&original).unwrap();
        let parsed: GitStatusSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_ls_output_round_trip() {
        let mut original = LsOutputSchema::new();
        original.is_empty = false;
        original
            .entries
            .push(LsEntry::new("src", LsEntryType::Directory));
        original.directories.push("src".to_string());
        original.counts.total = 1;
        original.counts.directories = 1;

        let json = serde_json::to_string(&original).unwrap();
        let parsed: LsOutputSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_test_output_round_trip() {
        let original = TestOutputSchema::new(TestRunnerType::Jest);
        let json = serde_json::to_string(&original).unwrap();
        let parsed: TestOutputSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }
}
