use super::*;

#[test]
fn test_formatter_names() {
    assert_eq!(CompactFormatter::name(), "compact");
    assert_eq!(JsonFormatter::name(), "json");
    assert_eq!(CsvFormatter::name(), "csv");
    assert_eq!(TsvFormatter::name(), "tsv");
    assert_eq!(AgentFormatter::name(), "agent");
    assert_eq!(RawFormatter::name(), "raw");
}

#[test]
fn test_formatter_output_formats() {
    assert_eq!(CompactFormatter::format(), OutputFormat::Compact);
    assert_eq!(JsonFormatter::format(), OutputFormat::Json);
    assert_eq!(CsvFormatter::format(), OutputFormat::Csv);
    assert_eq!(TsvFormatter::format(), OutputFormat::Tsv);
    assert_eq!(AgentFormatter::format(), OutputFormat::Agent);
    assert_eq!(RawFormatter::format(), OutputFormat::Raw);
}

// ============================================================
// CompactFormatter Tests
// ============================================================

#[test]
fn test_compact_format_message() {
    assert_eq!(
        CompactFormatter::format_message("branch", "main"),
        "branch: main\n"
    );
}

#[test]
fn test_compact_format_counts() {
    let output = CompactFormatter::format_counts("counts", &[("passed", 10), ("failed", 2)]);
    assert_eq!(output, "counts: passed=10 failed=2\n");

    // Zero counts should be filtered out
    let output = CompactFormatter::format_counts("counts", &[("passed", 0), ("failed", 2)]);
    assert_eq!(output, "counts: failed=2\n");

    // All zeros should return empty string
    let output = CompactFormatter::format_counts("counts", &[("passed", 0), ("failed", 0)]);
    assert!(output.is_empty());
}

#[test]
fn test_compact_format_section_header() {
    assert_eq!(
        CompactFormatter::format_section_header("staged", Some(3)),
        "staged (3):\n"
    );
    assert_eq!(
        CompactFormatter::format_section_header("files", None),
        "files:\n"
    );
}

#[test]
fn test_compact_format_item() {
    assert_eq!(
        CompactFormatter::format_item("M", "src/main.rs"),
        "  M src/main.rs\n"
    );
}

#[test]
fn test_compact_format_item_renamed() {
    assert_eq!(
        CompactFormatter::format_item_renamed("R", "old.rs", "new.rs"),
        "  R old.rs -> new.rs\n"
    );
}

#[test]
fn test_compact_format_test_summary() {
    let output = CompactFormatter::format_test_summary(10, 2, 1, 1500);
    assert!(output.contains("tests: passed=10 failed=2 skipped=1"));
    assert!(output.contains("duration: 1.50s"));
}

#[test]
fn test_compact_format_test_summary_only_passed() {
    let output = CompactFormatter::format_test_summary(5, 0, 0, 500);
    assert!(output.contains("tests: passed=5"));
    assert!(!output.contains("failed"));
    assert!(!output.contains("skipped"));
}

#[test]
fn test_compact_format_status() {
    assert_eq!(CompactFormatter::format_status(true), "status: passed\n");
    assert_eq!(CompactFormatter::format_status(false), "status: failed\n");
}

#[test]
fn test_compact_format_failures() {
    let failures = vec!["test_one".to_string(), "test_two".to_string()];
    let output = CompactFormatter::format_failures(&failures);
    assert!(output.contains("failures (2):"));
    assert!(output.contains("test_one"));
    assert!(output.contains("test_two"));
}

#[test]
fn test_compact_format_failures_empty() {
    let failures: Vec<String> = vec![];
    let output = CompactFormatter::format_failures(&failures);
    assert!(output.is_empty());
}

#[test]
fn test_compact_format_log_levels() {
    let output = CompactFormatter::format_log_levels(2, 5, 10, 3);
    assert_eq!(output, "levels: error=2 warn=5 info=10 debug=3\n");
}

#[test]
fn test_compact_format_log_levels_partial() {
    let output = CompactFormatter::format_log_levels(0, 5, 0, 0);
    assert_eq!(output, "levels: warn=5\n");
}

#[test]
fn test_compact_format_log_levels_empty() {
    let output = CompactFormatter::format_log_levels(0, 0, 0, 0);
    assert!(output.is_empty());
}

#[test]
fn test_compact_format_grep_match() {
    let output = CompactFormatter::format_grep_match("src/main.rs", Some(42), "fn main()");
    assert_eq!(output, "src/main.rs:42: fn main()\n");
}

#[test]
fn test_compact_format_grep_match_no_line() {
    let output = CompactFormatter::format_grep_match("src/main.rs", None, "match found");
    assert_eq!(output, "src/main.rs: match found\n");
}

#[test]
fn test_compact_format_grep_file() {
    let output = CompactFormatter::format_grep_file("src/main.rs", 5);
    assert_eq!(output, "src/main.rs (5 matches):\n");
}

#[test]
fn test_compact_format_diff_file() {
    let output = CompactFormatter::format_diff_file("src/main.rs", "M", 10, 5);
    assert_eq!(output, "  M src/main.rs (+10 -5)\n");
}

#[test]
fn test_compact_format_diff_summary() {
    let output = CompactFormatter::format_diff_summary(3, 25, 10);
    assert_eq!(
        output,
        "diff: 3 files changed, 25 insertions, 10 deletions\n"
    );
}

#[test]
fn test_compact_format_clean() {
    assert_eq!(CompactFormatter::format_clean(), "status: clean\n");
}

#[test]
fn test_compact_format_dirty() {
    let output = CompactFormatter::format_dirty(2, 3, 5, 0);
    assert_eq!(
        output,
        "status: dirty (staged=2 unstaged=3 untracked=5 unmerged=0)\n"
    );
}

#[test]
fn test_compact_format_branch_with_tracking() {
    // No tracking
    assert_eq!(
        CompactFormatter::format_branch_with_tracking("main", 0, 0),
        "branch: main\n"
    );

    // Ahead only
    assert_eq!(
        CompactFormatter::format_branch_with_tracking("feature", 3, 0),
        "branch: feature (ahead 3)\n"
    );

    // Behind only
    assert_eq!(
        CompactFormatter::format_branch_with_tracking("feature", 0, 2),
        "branch: feature (behind 2)\n"
    );

    // Both ahead and behind
    assert_eq!(
        CompactFormatter::format_branch_with_tracking("feature", 3, 2),
        "branch: feature (ahead 3, behind 2)\n"
    );
}

#[test]
fn test_compact_format_empty() {
    assert_eq!(CompactFormatter::format_empty(), "(empty)\n");
}

#[test]
fn test_compact_format_truncated() {
    let output = CompactFormatter::format_truncated(10, 50);
    assert_eq!(output, "... showing 10 of 50 total\n");
}

// ============================================================
// CompactFormatter Schema Formatting Tests
// ============================================================

#[test]
fn test_compact_format_git_status_clean() {
    use crate::schema::{GitStatusCounts, GitStatusSchema};
    let mut status = GitStatusSchema::new("main");
    status.is_clean = true;
    status.counts = GitStatusCounts::default();
    let output = CompactFormatter::format_git_status(&status);
    assert!(output.contains("branch: main"));
    assert!(output.contains("status: clean"));
}

#[test]
fn test_compact_format_git_status_dirty() {
    use crate::schema::{GitFileEntry, GitStatusCounts, GitStatusSchema};
    let mut status = GitStatusSchema::new("feature");
    status.is_clean = false;
    status.ahead = Some(3);
    status.behind = Some(1);
    status.staged.push(GitFileEntry::new("M", "src/main.rs"));
    status.unstaged.push(GitFileEntry::new("M", "src/lib.rs"));
    status
        .untracked
        .push(GitFileEntry::new("??", "new_file.txt"));
    status.counts = GitStatusCounts {
        staged: 1,
        unstaged: 1,
        untracked: 1,
        unmerged: 0,
    };
    let output = CompactFormatter::format_git_status(&status);
    assert!(output.contains("branch: feature (ahead 3, behind 1)"));
    assert!(output.contains("counts: staged=1 unstaged=1 untracked=1"));
    assert!(output.contains("staged (1):"));
    assert!(output.contains("unstaged (1):"));
    assert!(output.contains("untracked (1):"));
}

#[test]
fn test_compact_format_git_status_renamed() {
    use crate::schema::{GitFileEntry, GitStatusSchema};
    let mut status = GitStatusSchema::new("main");
    status.is_clean = false;
    status
        .staged
        .push(GitFileEntry::renamed("R", "old.rs", "new.rs"));
    status.counts.staged = 1;
    let output = CompactFormatter::format_git_status(&status);
    assert!(output.contains("R old.rs -> new.rs"));
}

#[test]
fn test_compact_format_git_diff_empty() {
    use crate::schema::GitDiffSchema;
    let diff = GitDiffSchema::new();
    let output = CompactFormatter::format_git_diff(&diff);
    assert!(output.contains("diff: empty"));
}

#[test]
fn test_compact_format_git_diff_with_files() {
    use crate::schema::{GitDiffCounts, GitDiffEntry, GitDiffSchema};
    let mut diff = GitDiffSchema::new();
    diff.is_empty = false;
    let mut entry = GitDiffEntry::new("src/main.rs", "M");
    entry.additions = 10;
    entry.deletions = 5;
    diff.files.push(entry);
    diff.total_additions = 10;
    diff.total_deletions = 5;
    diff.counts = GitDiffCounts {
        total_files: 1,
        files_shown: 1,
    };
    let output = CompactFormatter::format_git_diff(&diff);
    assert!(output.contains("M src/main.rs (+10 -5)"));
    assert!(output.contains("diff: 1 files changed, 10 insertions, 5 deletions"));
}

#[test]
fn test_compact_format_git_diff_truncated() {
    use crate::schema::{GitDiffCounts, GitDiffEntry, GitDiffSchema};
    let mut diff = GitDiffSchema::new();
    diff.is_empty = false;
    diff.is_truncated = true;
    let mut entry = GitDiffEntry::new("src/main.rs", "M");
    entry.additions = 10;
    entry.deletions = 5;
    diff.files.push(entry);
    diff.total_additions = 10;
    diff.total_deletions = 5;
    diff.counts = GitDiffCounts {
        total_files: 10,
        files_shown: 1,
    };
    let output = CompactFormatter::format_git_diff(&diff);
    assert!(output.contains("... showing 1 of 10 total"));
}

#[test]
fn test_compact_format_ls_empty() {
    use crate::schema::LsOutputSchema;
    let ls = LsOutputSchema::new();
    let output = CompactFormatter::format_ls(&ls);
    assert!(output.contains("(empty)"));
}

#[test]
fn test_compact_format_ls_with_entries() {
    use crate::schema::{LsCounts, LsOutputSchema};
    let mut ls = LsOutputSchema::new();
    ls.is_empty = false;
    ls.directories.push("src".to_string());
    ls.files.push("main.rs".to_string());
    ls.hidden.push(".gitignore".to_string());
    ls.counts = LsCounts {
        total: 3,
        directories: 1,
        files: 1,
        symlinks: 0,
        hidden: 1,
        generated: 0,
    };
    let output = CompactFormatter::format_ls(&ls);
    assert!(output.contains("directories (1):"));
    assert!(output.contains("files (1):"));
    assert!(output.contains("hidden (1):"));
}

#[test]
fn test_compact_format_ls_with_symlinks() {
    use crate::schema::{LsCounts, LsEntry, LsEntryType, LsOutputSchema};
    let mut ls = LsOutputSchema::new();
    ls.is_empty = false;
    let mut entry = LsEntry::new("link", LsEntryType::Symlink);
    entry.symlink_target = Some("target".to_string());
    entry.is_broken_symlink = false;
    ls.entries.push(entry);
    ls.symlinks.push("link".to_string());
    ls.counts = LsCounts {
        total: 1,
        directories: 0,
        files: 0,
        symlinks: 1,
        hidden: 0,
        generated: 0,
    };
    let output = CompactFormatter::format_ls(&ls);
    assert!(output.contains("symlinks (1):"));
    assert!(output.contains("link -> target"));
}

#[test]
fn test_compact_format_ls_broken_symlink() {
    use crate::schema::{LsCounts, LsEntry, LsEntryType, LsOutputSchema};
    let mut ls = LsOutputSchema::new();
    ls.is_empty = false;
    let mut entry = LsEntry::new("broken_link", LsEntryType::Symlink);
    entry.symlink_target = Some("missing".to_string());
    entry.is_broken_symlink = true;
    ls.entries.push(entry);
    ls.symlinks.push("broken_link".to_string());
    ls.counts = LsCounts {
        total: 1,
        directories: 0,
        files: 0,
        symlinks: 1,
        hidden: 0,
        generated: 0,
    };
    let output = CompactFormatter::format_ls(&ls);
    assert!(output.contains("[broken]"));
}

#[test]
fn test_compact_format_grep_empty() {
    use crate::schema::GrepOutputSchema;
    let grep = GrepOutputSchema::new();
    let output = CompactFormatter::format_grep(&grep);
    assert!(output.contains("grep: no matches"));
}

#[test]
fn test_compact_format_grep_with_matches() {
    use crate::schema::{GrepCounts, GrepFile, GrepMatch, GrepOutputSchema};
    let mut grep = GrepOutputSchema::new();
    grep.is_empty = false;
    let mut file = GrepFile::new("src/main.rs");
    let mut m = GrepMatch::new("fn main()");
    m.line_number = Some(10);
    file.matches.push(m);
    grep.files.push(file);
    grep.counts = GrepCounts {
        files: 1,
        matches: 1,
        total_files: 1,
        total_matches: 1,
        files_shown: 1,
        matches_shown: 1,
    };
    let output = CompactFormatter::format_grep(&grep);
    assert!(output.contains("matches: 1 files, 1 results"));
    assert!(output.contains("src/main.rs (1 matches)"));
    assert!(output.contains("10: fn main()"));
}

#[test]
fn test_compact_format_grep_truncated() {
    use crate::schema::{GrepCounts, GrepFile, GrepMatch, GrepOutputSchema};
    let mut grep = GrepOutputSchema::new();
    grep.is_empty = false;
    grep.is_truncated = true;
    let mut file = GrepFile::new("src/main.rs");
    let mut m = GrepMatch::new("fn main()");
    m.line_number = Some(10);
    file.matches.push(m);
    grep.files.push(file);
    grep.counts = GrepCounts {
        files: 1,
        matches: 1,
        total_files: 5,
        total_matches: 10,
        files_shown: 1,
        matches_shown: 1,
    };
    let output = CompactFormatter::format_grep(&grep);
    assert!(output.contains("... showing 1 of 5 total"));
}

#[test]
fn test_compact_format_find_empty() {
    use crate::schema::FindOutputSchema;
    let find = FindOutputSchema::new();
    let output = CompactFormatter::format_find(&find);
    assert!(output.contains("find: no results"));
}

#[test]
fn test_compact_format_find_with_entries() {
    use crate::schema::{FindCounts, FindOutputSchema};
    let mut find = FindOutputSchema::new();
    find.is_empty = false;
    find.directories.push("./src".to_string());
    find.files.push("./main.rs".to_string());
    find.counts = FindCounts {
        total: 2,
        directories: 1,
        files: 1,
    };
    let output = CompactFormatter::format_find(&find);
    assert!(output.contains("find: 2 entries (1 dirs, 1 files)"));
    assert!(output.contains("directories (1):"));
    assert!(output.contains("files (1):"));
}

#[test]
fn test_compact_format_test_output_empty() {
    use crate::schema::{TestOutputSchema, TestRunnerType};
    let test = TestOutputSchema::new(TestRunnerType::Pytest);
    let output = CompactFormatter::format_test_output(&test);
    assert!(output.contains("tests: no results"));
}

#[test]
fn test_compact_format_test_output_passing() {
    use crate::schema::{TestOutputSchema, TestRunnerType, TestSummary};
    let mut test = TestOutputSchema::new(TestRunnerType::Pytest);
    test.is_empty = false;
    test.success = true;
    test.summary = TestSummary {
        total: 10,
        passed: 10,
        failed: 0,
        skipped: 0,
        xfailed: 0,
        xpassed: 0,
        errors: 0,
        todo: 0,
        suites_passed: 1,
        suites_failed: 0,
        suites_total: 1,
        duration_ms: Some(500),
    };
    let output = CompactFormatter::format_test_output(&test);
    assert!(output.contains("PASS: 10 tests"));
    assert!(output.contains("duration: 500ms"));
}

#[test]
fn test_compact_format_test_output_failing() {
    use crate::schema::{
        TestOutputSchema, TestResult, TestRunnerType, TestStatus, TestSuite, TestSummary,
    };
    let mut test = TestOutputSchema::new(TestRunnerType::Pytest);
    test.is_empty = false;
    test.success = false;
    test.summary = TestSummary {
        total: 10,
        passed: 8,
        failed: 2,
        skipped: 0,
        xfailed: 0,
        xpassed: 0,
        errors: 0,
        todo: 0,
        suites_passed: 0,
        suites_failed: 1,
        suites_total: 1,
        duration_ms: Some(500),
    };
    let mut suite = TestSuite::new("tests/test_main.py");
    suite.passed = false;
    suite
        .tests
        .push(TestResult::new("test_one", TestStatus::Failed));
    suite
        .tests
        .push(TestResult::new("test_two", TestStatus::Passed));
    test.test_suites.push(suite);
    let output = CompactFormatter::format_test_output(&test);
    assert!(output.contains("FAIL: 8 passed, 2 failed"));
    assert!(output.contains("FAIL: test_one"));
}

#[test]
fn test_compact_format_logs_empty() {
    use crate::schema::LogsOutputSchema;
    let logs = LogsOutputSchema::new();
    let output = CompactFormatter::format_logs(&logs);
    assert!(output.contains("logs: empty"));
}

#[test]
fn test_compact_format_logs_with_entries() {
    use crate::schema::{LogCounts, LogsOutputSchema};
    let mut logs = LogsOutputSchema::new();
    logs.is_empty = false;
    logs.counts = LogCounts {
        total_lines: 10,
        debug: 2,
        info: 5,
        warning: 2,
        error: 1,
        fatal: 0,
        unknown: 0,
    };
    let output = CompactFormatter::format_logs(&logs);
    assert!(output.contains("lines: 10"));
    assert!(output.contains("levels: error=1 warn=2 info=5 debug=2"));
}

#[test]
fn test_compact_format_logs_with_critical() {
    use crate::schema::{LogCounts, LogEntry, LogLevel, LogsOutputSchema};
    let mut logs = LogsOutputSchema::new();
    logs.is_empty = false;
    logs.counts = LogCounts {
        total_lines: 3,
        debug: 0,
        info: 1,
        warning: 0,
        error: 2,
        fatal: 0,
        unknown: 0,
    };
    let mut entry = LogEntry::new("[ERROR] Something failed", 2);
    entry.level = LogLevel::Error;
    entry.message = "Something failed".to_string();
    logs.recent_critical.push(entry);
    let output = CompactFormatter::format_logs(&logs);
    assert!(output.contains("recent critical"));
}

#[test]
fn test_compact_format_repository_state_not_git() {
    use crate::schema::RepositoryStateSchema;
    let mut state = RepositoryStateSchema::new();
    state.is_git_repo = false;
    let output = CompactFormatter::format_repository_state(&state);
    assert!(output.contains("error: not a git repository"));
}

#[test]
fn test_compact_format_repository_state_clean() {
    use crate::schema::{GitStatusCounts, RepositoryStateSchema};
    let mut state = RepositoryStateSchema::new();
    state.branch = Some("main".to_string());
    state.is_clean = true;
    state.counts = GitStatusCounts::default();
    let output = CompactFormatter::format_repository_state(&state);
    assert!(output.contains("branch: main"));
    assert!(output.contains("status: clean"));
}

#[test]
fn test_compact_format_repository_state_dirty() {
    use crate::schema::{GitStatusCounts, RepositoryStateSchema};
    let mut state = RepositoryStateSchema::new();
    state.branch = Some("feature".to_string());
    state.is_clean = false;
    state.is_detached = false;
    state.counts = GitStatusCounts {
        staged: 1,
        unstaged: 2,
        untracked: 3,
        unmerged: 0,
    };
    let output = CompactFormatter::format_repository_state(&state);
    assert!(output.contains("branch: feature"));
    assert!(output.contains("status: dirty"));
}

#[test]
fn test_compact_format_repository_state_detached() {
    use crate::schema::{GitStatusCounts, RepositoryStateSchema};
    let mut state = RepositoryStateSchema::new();
    state.branch = Some("abc123".to_string());
    state.is_detached = true;
    state.is_clean = true;
    state.counts = GitStatusCounts::default();
    let output = CompactFormatter::format_repository_state(&state);
    assert!(output.contains("(detached)"));
}

#[test]
fn test_compact_format_process_success() {
    use crate::schema::ProcessOutputSchema;
    let mut proc = ProcessOutputSchema::new("echo");
    proc.stdout = "hello\n".to_string();
    proc.success = true;
    let output = CompactFormatter::format_process(&proc);
    assert!(output.contains("hello"));
}

#[test]
fn test_compact_format_process_failure() {
    use crate::schema::ProcessOutputSchema;
    let mut proc = ProcessOutputSchema::new("false");
    proc.exit_code = Some(1);
    proc.success = false;
    proc.stderr = "error message\n".to_string();
    let output = CompactFormatter::format_process(&proc);
    assert!(output.contains("command: false"));
    assert!(output.contains("exit_code: 1"));
}

#[test]
fn test_compact_format_error_schema() {
    use crate::schema::ErrorSchema;
    let error = ErrorSchema::new("Something went wrong");
    let output = CompactFormatter::format_error_schema(&error);
    assert!(output.contains("error: Something went wrong"));
}

#[test]
fn test_compact_format_error_schema_with_code() {
    use crate::schema::ErrorSchema;
    let mut error = ErrorSchema::new("Command failed");
    error.exit_code = Some(1);
    let output = CompactFormatter::format_error_schema(&error);
    assert!(output.contains("error: Command failed"));
    assert!(output.contains("exit_code: 1"));
}

// ============================================================
// Helper Function Tests
// ============================================================

#[test]
fn test_format_count_if_positive() {
    assert_eq!(format_count_if_positive("staged", 0), None);
    assert_eq!(
        format_count_if_positive("staged", 3),
        Some("staged=3".to_string())
    );
}

#[test]
fn test_format_list_with_count() {
    let items = vec!["file1.rs".to_string(), "file2.rs".to_string()];
    let output = format_list_with_count("staged", &items);
    assert!(output.contains("staged (2):"));
    assert!(output.contains("file1.rs"));
    assert!(output.contains("file2.rs"));
}

#[test]
fn test_format_list_with_count_empty() {
    let items: Vec<String> = vec![];
    let output = format_list_with_count("staged", &items);
    assert!(output.is_empty());
}

#[test]
fn test_format_key_value() {
    assert_eq!(format_key_value("branch", "main", None), "branch: main\n");
    assert_eq!(
        format_key_value("status", "M", Some("modified")),
        "status [modified]: M\n"
    );
}

#[test]
fn test_format_line() {
    assert_eq!(format_line("branch", "main"), "branch: main\n");
    assert_eq!(format_line("count", 42), "count: 42\n");
}

#[test]
fn test_truncate() {
    assert_eq!(truncate("hello", 10), "hello");
    assert_eq!(truncate("hello world", 8), "hello...");
    assert_eq!(truncate("hi", 3), "hi");
}

#[test]
fn test_format_duration() {
    assert_eq!(format_duration(500), "500ms");
    assert_eq!(format_duration(1500), "1.50s");
    assert_eq!(format_duration(90000), "1m 30s");
}

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(500), "500B");
    assert_eq!(format_bytes(1024), "1.00KB");
    assert_eq!(format_bytes(1048576), "1.00MB");
    assert_eq!(format_bytes(1073741824), "1.00GB");
}

// ============================================================
// JSON Formatter Tests
// ============================================================

#[test]
fn test_json_format_message() {
    let output = JsonFormatter::format_message("branch", "main");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "main");
}

#[test]
fn test_json_format_key_value() {
    let output = JsonFormatter::format_key_value("count", 42);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["count"], 42);
}

#[test]
fn test_json_format_object() {
    let output = JsonFormatter::format_object(&[
        ("branch", serde_json::json!("main")),
        ("is_clean", serde_json::json!(true)),
        ("count", serde_json::json!(5)),
    ]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "main");
    assert_eq!(json["is_clean"], true);
    assert_eq!(json["count"], 5);
}

#[test]
fn test_json_format_counts() {
    let output = JsonFormatter::format_counts(&[("passed", 10), ("failed", 2)]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["passed"], 10);
    assert_eq!(json["failed"], 2);
}

#[test]
fn test_json_format_counts_with_zeros() {
    let output = JsonFormatter::format_counts(&[("passed", 0), ("failed", 2)]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["passed"], 0);
    assert_eq!(json["failed"], 2);
}

#[test]
fn test_json_format_section() {
    let items = vec!["file1.rs", "file2.rs"];
    let output = JsonFormatter::format_section("staged", &items);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(json["staged"].is_array());
    assert_eq!(json["staged"][0], "file1.rs");
    assert_eq!(json["staged"][1], "file2.rs");
}

#[test]
fn test_json_format_item() {
    let output = JsonFormatter::format_item("M", "src/main.rs");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["status"], "M");
    assert_eq!(json["path"], "src/main.rs");
}

#[test]
fn test_json_format_item_renamed() {
    let output = JsonFormatter::format_item_renamed("R", "old.rs", "new.rs");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["status"], "R");
    assert_eq!(json["path"], "new.rs");
    assert_eq!(json["old_path"], "old.rs");
}

#[test]
fn test_json_format_test_summary() {
    let output = JsonFormatter::format_test_summary(10, 2, 1, 1500);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["passed"], 10);
    assert_eq!(json["failed"], 2);
    assert_eq!(json["skipped"], 1);
    assert_eq!(json["total"], 13);
    assert_eq!(json["duration_ms"], 1500);
}

#[test]
fn test_json_format_status() {
    let success_output = JsonFormatter::format_status(true);
    let success_json: serde_json::Value = serde_json::from_str(&success_output).unwrap();
    assert_eq!(success_json["success"], true);

    let failure_output = JsonFormatter::format_status(false);
    let failure_json: serde_json::Value = serde_json::from_str(&failure_output).unwrap();
    assert_eq!(failure_json["success"], false);
}

#[test]
fn test_json_format_failures() {
    let failures = vec!["test_one".to_string(), "test_two".to_string()];
    let output = JsonFormatter::format_failures(&failures);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(json["failures"].is_array());
    assert_eq!(json["count"], 2);
}

#[test]
fn test_json_format_failures_empty() {
    let failures: Vec<String> = vec![];
    let output = JsonFormatter::format_failures(&failures);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(json["failures"].is_array());
    assert_eq!(json["count"], 0);
}

#[test]
fn test_json_format_log_levels() {
    let output = JsonFormatter::format_log_levels(2, 5, 10, 3);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["error"], 2);
    assert_eq!(json["warn"], 5);
    assert_eq!(json["info"], 10);
    assert_eq!(json["debug"], 3);
    assert_eq!(json["total"], 20);
}

#[test]
fn test_json_format_log_levels_with_zeros() {
    let output = JsonFormatter::format_log_levels(0, 5, 0, 0);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["error"], 0);
    assert_eq!(json["warn"], 5);
    assert_eq!(json["total"], 5);
}

#[test]
fn test_json_format_grep_match() {
    let output = JsonFormatter::format_grep_match("src/main.rs", Some(42), "fn main()");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["file"], "src/main.rs");
    assert_eq!(json["line"], 42);
    assert_eq!(json["content"], "fn main()");
}

#[test]
fn test_json_format_grep_match_no_line() {
    let output = JsonFormatter::format_grep_match("src/main.rs", None, "match found");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["file"], "src/main.rs");
    assert!(json["line"].is_null());
}

#[test]
fn test_json_format_grep_file() {
    let output = JsonFormatter::format_grep_file("src/main.rs", 5);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["file"], "src/main.rs");
    assert_eq!(json["match_count"], 5);
}

#[test]
fn test_json_format_diff_file() {
    let output = JsonFormatter::format_diff_file("src/main.rs", "M", 10, 5);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["path"], "src/main.rs");
    assert_eq!(json["change_type"], "M");
    assert_eq!(json["additions"], 10);
    assert_eq!(json["deletions"], 5);
}

#[test]
fn test_json_format_diff_summary() {
    let output = JsonFormatter::format_diff_summary(3, 25, 10);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["files_changed"], 3);
    assert_eq!(json["insertions"], 25);
    assert_eq!(json["deletions"], 10);
}

#[test]
fn test_json_format_clean() {
    let output = JsonFormatter::format_clean();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_clean"], true);
}

#[test]
fn test_json_format_dirty() {
    let output = JsonFormatter::format_dirty(2, 3, 5, 0);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_clean"], false);
    assert_eq!(json["staged"], 2);
    assert_eq!(json["unstaged"], 3);
    assert_eq!(json["untracked"], 5);
    assert_eq!(json["unmerged"], 0);
}

#[test]
fn test_json_format_branch_with_tracking() {
    let output = JsonFormatter::format_branch_with_tracking("main", 0, 0);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "main");
    assert_eq!(json["ahead"], 0);
    assert_eq!(json["behind"], 0);

    let output = JsonFormatter::format_branch_with_tracking("feature", 3, 2);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "feature");
    assert_eq!(json["ahead"], 3);
    assert_eq!(json["behind"], 2);
}

#[test]
fn test_json_format_empty() {
    let output = JsonFormatter::format_empty();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["empty"], true);
}

#[test]
fn test_json_format_truncated() {
    let output = JsonFormatter::format_truncated(10, 50);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_truncated"], true);
    assert_eq!(json["shown"], 10);
    assert_eq!(json["total"], 50);
}

#[test]
fn test_json_format_error() {
    let output = JsonFormatter::format_error("Something went wrong");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["error"], true);
    assert_eq!(json["message"], "Something went wrong");
}

#[test]
fn test_json_format_error_with_code() {
    let output = JsonFormatter::format_error_with_code("Command failed", 1);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["error"], true);
    assert_eq!(json["message"], "Command failed");
    assert_eq!(json["exit_code"], 1);
}

#[test]
fn test_json_format_not_implemented() {
    let output = JsonFormatter::format_not_implemented("Feature X");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["not_implemented"], true);
    assert_eq!(json["message"], "Feature X");
}

#[test]
fn test_json_format_command_result() {
    let output = JsonFormatter::format_command_result(
        "echo",
        &["hello".to_string(), "world".to_string()],
        "hello world\n",
        "",
        0,
        10,
    );
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["command"], "echo");
    assert!(json["args"].is_array());
    assert_eq!(json["stdout"], "hello world\n");
    assert_eq!(json["stderr"], "");
    assert_eq!(json["exit_code"], 0);
    assert_eq!(json["duration_ms"], 10);
}

#[test]
fn test_json_format_list() {
    let items = vec!["file1.rs", "file2.rs"];
    let output = JsonFormatter::format_list(&items);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(json.is_array());
    assert_eq!(json[0], "file1.rs");
    assert_eq!(json[1], "file2.rs");
}

#[test]
fn test_json_format_count() {
    let output = JsonFormatter::format_count(42);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["count"], 42);
}

#[test]
fn test_json_format_flag() {
    let output = JsonFormatter::format_flag("is_clean", true);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_clean"], true);
}

#[test]
fn test_json_format_array() {
    #[derive(serde::Serialize)]
    struct Item {
        name: &'static str,
        value: usize,
    }
    let items = vec![
        Item {
            name: "first",
            value: 1,
        },
        Item {
            name: "second",
            value: 2,
        },
    ];
    let output = JsonFormatter::format_array(&items);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(json.is_array());
    assert_eq!(json[0]["name"], "first");
    assert_eq!(json[1]["value"], 2);
}

// ============================================================
// JsonFormatter Schema Formatting Tests
// ============================================================

#[test]
fn test_json_format_git_status_clean() {
    use crate::schema::{GitStatusCounts, GitStatusSchema};
    let mut status = GitStatusSchema::new("main");
    status.is_clean = true;
    status.counts = GitStatusCounts::default();
    let output = JsonFormatter::format_git_status(&status);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "main");
    assert_eq!(json["is_clean"], true);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "git_status");
}

#[test]
fn test_json_format_git_status_dirty() {
    use crate::schema::{GitFileEntry, GitStatusCounts, GitStatusSchema};
    let mut status = GitStatusSchema::new("feature");
    status.is_clean = false;
    status.ahead = Some(3);
    status.behind = Some(1);
    status.staged.push(GitFileEntry::new("M", "src/main.rs"));
    status.unstaged.push(GitFileEntry::new("M", "src/lib.rs"));
    status
        .untracked
        .push(GitFileEntry::new("??", "new_file.txt"));
    status.counts = GitStatusCounts {
        staged: 1,
        unstaged: 1,
        untracked: 1,
        unmerged: 0,
    };
    let output = JsonFormatter::format_git_status(&status);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "feature");
    assert_eq!(json["is_clean"], false);
    assert_eq!(json["ahead"], 3);
    assert_eq!(json["behind"], 1);
    assert!(json["staged"].is_array());
    assert!(json["unstaged"].is_array());
    assert!(json["untracked"].is_array());
    assert_eq!(json["counts"]["staged"], 1);
    assert_eq!(json["counts"]["unstaged"], 1);
    assert_eq!(json["counts"]["untracked"], 1);
}

#[test]
fn test_json_format_git_status_renamed() {
    use crate::schema::{GitFileEntry, GitStatusSchema};
    let mut status = GitStatusSchema::new("main");
    status.is_clean = false;
    status
        .staged
        .push(GitFileEntry::renamed("R", "old.rs", "new.rs"));
    status.counts.staged = 1;
    let output = JsonFormatter::format_git_status(&status);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["staged"][0]["status"], "R");
    assert_eq!(json["staged"][0]["path"], "new.rs");
    assert_eq!(json["staged"][0]["old_path"], "old.rs");
}

#[test]
fn test_json_format_git_diff_empty() {
    use crate::schema::GitDiffSchema;
    let diff = GitDiffSchema::new();
    let output = JsonFormatter::format_git_diff(&diff);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], true);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "git_diff");
}

#[test]
fn test_json_format_git_diff_with_files() {
    use crate::schema::{GitDiffCounts, GitDiffEntry, GitDiffSchema};
    let mut diff = GitDiffSchema::new();
    diff.is_empty = false;
    let mut entry = GitDiffEntry::new("src/main.rs", "M");
    entry.additions = 10;
    entry.deletions = 5;
    diff.files.push(entry);
    diff.total_additions = 10;
    diff.total_deletions = 5;
    diff.counts = GitDiffCounts { total_files: 1, files_shown: 1 };
    let output = JsonFormatter::format_git_diff(&diff);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], false);
    assert!(json["files"].is_array());
    assert_eq!(json["files"][0]["path"], "src/main.rs");
    assert_eq!(json["files"][0]["change_type"], "M");
    assert_eq!(json["files"][0]["additions"], 10);
    assert_eq!(json["files"][0]["deletions"], 5);
    assert_eq!(json["total_additions"], 10);
    assert_eq!(json["total_deletions"], 5);
}

#[test]
fn test_json_format_git_diff_truncated() {
    use crate::schema::{GitDiffCounts, GitDiffEntry, GitDiffSchema};
    let mut diff = GitDiffSchema::new();
    diff.is_empty = false;
    diff.is_truncated = true;
    let mut entry = GitDiffEntry::new("src/main.rs", "M");
    entry.additions = 10;
    entry.deletions = 5;
    diff.files.push(entry);
    diff.total_additions = 10;
    diff.total_deletions = 5;
    diff.counts = GitDiffCounts { total_files: 10, files_shown: 1 };
    let output = JsonFormatter::format_git_diff(&diff);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_truncated"], true);
    assert_eq!(json["counts"]["total_files"], 10);
    assert_eq!(json["counts"]["files_shown"], 1);
}

#[test]
fn test_json_format_ls_empty() {
    use crate::schema::LsOutputSchema;
    let ls = LsOutputSchema::new();
    let output = JsonFormatter::format_ls(&ls);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], true);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "ls_output");
}

#[test]
fn test_json_format_ls_with_entries() {
    use crate::schema::{LsCounts, LsOutputSchema};
    let mut ls = LsOutputSchema::new();
    ls.is_empty = false;
    ls.directories.push("src".to_string());
    ls.files.push("main.rs".to_string());
    ls.hidden.push(".gitignore".to_string());
    ls.counts = LsCounts { total: 3, directories: 1, files: 1, symlinks: 0, hidden: 1, generated: 0 };
    let output = JsonFormatter::format_ls(&ls);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], false);
    assert!(json["directories"].is_array());
    assert!(json["files"].is_array());
    assert!(json["hidden"].is_array());
    assert_eq!(json["counts"]["directories"], 1);
    assert_eq!(json["counts"]["files"], 1);
    assert_eq!(json["counts"]["hidden"], 1);
}

#[test]
fn test_json_format_ls_with_symlinks() {
    use crate::schema::{LsCounts, LsEntry, LsEntryType, LsOutputSchema};
    let mut ls = LsOutputSchema::new();
    ls.is_empty = false;
    let mut entry = LsEntry::new("link", LsEntryType::Symlink);
    entry.symlink_target = Some("target".to_string());
    entry.is_broken_symlink = false;
    ls.entries.push(entry);
    ls.symlinks.push("link".to_string());
    ls.counts = LsCounts { total: 1, directories: 0, files: 0, symlinks: 1, hidden: 0, generated: 0 };
    let output = JsonFormatter::format_ls(&ls);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(json["symlinks"].is_array());
    assert!(json["entries"][0]["symlink_target"].is_string());
}

#[test]
fn test_json_format_ls_broken_symlink() {
    use crate::schema::{LsCounts, LsEntry, LsEntryType, LsOutputSchema};
    let mut ls = LsOutputSchema::new();
    ls.is_empty = false;
    let mut entry = LsEntry::new("broken_link", LsEntryType::Symlink);
    entry.symlink_target = Some("missing".to_string());
    entry.is_broken_symlink = true;
    ls.entries.push(entry);
    ls.symlinks.push("broken_link".to_string());
    ls.counts = LsCounts { total: 1, directories: 0, files: 0, symlinks: 1, hidden: 0, generated: 0 };
    let output = JsonFormatter::format_ls(&ls);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["entries"][0]["is_broken_symlink"], true);
}

#[test]
fn test_json_format_grep_empty() {
    use crate::schema::GrepOutputSchema;
    let grep = GrepOutputSchema::new();
    let output = JsonFormatter::format_grep(&grep);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], true);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "grep_output");
}

#[test]
fn test_json_format_grep_with_matches() {
    use crate::schema::{GrepCounts, GrepFile, GrepMatch, GrepOutputSchema};
    let mut grep = GrepOutputSchema::new();
    grep.is_empty = false;
    let mut file = GrepFile::new("src/main.rs");
    let mut m = GrepMatch::new("fn main()");
    m.line_number = Some(10);
    file.matches.push(m);
    grep.files.push(file);
    grep.counts = GrepCounts { files: 1, matches: 1, total_files: 1, total_matches: 1, files_shown: 1, matches_shown: 1 };
    let output = JsonFormatter::format_grep(&grep);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], false);
    assert!(json["files"].is_array());
    assert_eq!(json["files"][0]["path"], "src/main.rs");
    assert_eq!(json["files"][0]["matches"][0]["line"], "fn main()");
    assert_eq!(json["files"][0]["matches"][0]["line_number"], 10);
    assert_eq!(json["counts"]["files"], 1);
    assert_eq!(json["counts"]["matches"], 1);
}

#[test]
fn test_json_format_grep_truncated() {
    use crate::schema::{GrepCounts, GrepFile, GrepMatch, GrepOutputSchema};
    let mut grep = GrepOutputSchema::new();
    grep.is_empty = false;
    grep.is_truncated = true;
    let mut file = GrepFile::new("src/main.rs");
    let mut m = GrepMatch::new("fn main()");
    m.line_number = Some(10);
    file.matches.push(m);
    grep.files.push(file);
    grep.counts = GrepCounts { files: 1, matches: 1, total_files: 5, total_matches: 10, files_shown: 1, matches_shown: 1 };
    let output = JsonFormatter::format_grep(&grep);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_truncated"], true);
    assert_eq!(json["counts"]["total_files"], 5);
    assert_eq!(json["counts"]["total_matches"], 10);
}

#[test]
fn test_json_format_find_empty() {
    use crate::schema::FindOutputSchema;
    let find = FindOutputSchema::new();
    let output = JsonFormatter::format_find(&find);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], true);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "find_output");
}

#[test]
fn test_json_format_find_with_entries() {
    use crate::schema::{FindCounts, FindOutputSchema};
    let mut find = FindOutputSchema::new();
    find.is_empty = false;
    find.directories.push("./src".to_string());
    find.files.push("./main.rs".to_string());
    find.counts = FindCounts { total: 2, directories: 1, files: 1 };
    let output = JsonFormatter::format_find(&find);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], false);
    assert!(json["directories"].is_array());
    assert!(json["files"].is_array());
    assert_eq!(json["counts"]["total"], 2);
    assert_eq!(json["counts"]["directories"], 1);
    assert_eq!(json["counts"]["files"], 1);
}

#[test]
fn test_json_format_test_output_empty() {
    use crate::schema::{TestOutputSchema, TestRunnerType};
    let test = TestOutputSchema::new(TestRunnerType::Pytest);
    let output = JsonFormatter::format_test_output(&test);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], true);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "test_output");
}

#[test]
fn test_json_format_test_output_passing() {
    use crate::schema::{TestOutputSchema, TestRunnerType, TestSummary};
    let mut test = TestOutputSchema::new(TestRunnerType::Pytest);
    test.is_empty = false;
    test.success = true;
    test.summary = TestSummary { total: 10, passed: 10, failed: 0, skipped: 0, xfailed: 0, xpassed: 0, errors: 0, todo: 0, suites_passed: 1, suites_failed: 0, suites_total: 1, duration_ms: Some(500) };
    let output = JsonFormatter::format_test_output(&test);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], false);
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["passed"], 10);
    assert_eq!(json["summary"]["failed"], 0);
    assert_eq!(json["summary"]["total"], 10);
    assert_eq!(json["summary"]["duration_ms"], 500);
}

#[test]
fn test_json_format_test_output_failing() {
    use crate::schema::{TestOutputSchema, TestResult, TestRunnerType, TestStatus, TestSuite, TestSummary};
    let mut test = TestOutputSchema::new(TestRunnerType::Pytest);
    test.is_empty = false;
    test.success = false;
    test.summary = TestSummary { total: 10, passed: 8, failed: 2, skipped: 0, xfailed: 0, xpassed: 0, errors: 0, todo: 0, suites_passed: 0, suites_failed: 1, suites_total: 1, duration_ms: Some(500) };
    let mut suite = TestSuite::new("tests/test_main.py");
    suite.passed = false;
    suite.tests.push(TestResult::new("test_one", TestStatus::Failed));
    suite.tests.push(TestResult::new("test_two", TestStatus::Passed));
    test.test_suites.push(suite);
    let output = JsonFormatter::format_test_output(&test);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["summary"]["passed"], 8);
    assert_eq!(json["summary"]["failed"], 2);
    assert!(json["test_suites"].is_array());
    assert_eq!(json["test_suites"][0]["file"], "tests/test_main.py");
    assert_eq!(json["test_suites"][0]["passed"], false);
}

#[test]
fn test_json_format_logs_empty() {
    use crate::schema::LogsOutputSchema;
    let logs = LogsOutputSchema::new();
    let output = JsonFormatter::format_logs(&logs);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], true);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "logs_output");
}

#[test]
fn test_json_format_logs_with_entries() {
    use crate::schema::{LogCounts, LogsOutputSchema};
    let mut logs = LogsOutputSchema::new();
    logs.is_empty = false;
    logs.counts = LogCounts { total_lines: 10, debug: 2, info: 5, warning: 2, error: 1, fatal: 0, unknown: 0 };
    let output = JsonFormatter::format_logs(&logs);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], false);
    assert_eq!(json["counts"]["total_lines"], 10);
    assert_eq!(json["counts"]["error"], 1);
    assert_eq!(json["counts"]["warning"], 2);
    assert_eq!(json["counts"]["info"], 5);
    assert_eq!(json["counts"]["debug"], 2);
}

#[test]
fn test_json_format_logs_with_critical() {
    use crate::schema::{LogCounts, LogEntry, LogLevel, LogsOutputSchema};
    let mut logs = LogsOutputSchema::new();
    logs.is_empty = false;
    logs.counts = LogCounts { total_lines: 3, debug: 0, info: 1, warning: 0, error: 2, fatal: 0, unknown: 0 };
    let mut entry = LogEntry::new("[ERROR] Something failed", 2);
    entry.level = LogLevel::Error;
    entry.message = "Something failed".to_string();
    logs.recent_critical.push(entry);
    let output = JsonFormatter::format_logs(&logs);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(json["recent_critical"].is_array());
    assert_eq!(json["recent_critical"][0]["message"], "Something failed");
    assert_eq!(json["recent_critical"][0]["level"], "error");
}

#[test]
fn test_json_format_repository_state_not_git() {
    use crate::schema::RepositoryStateSchema;
    let mut state = RepositoryStateSchema::new();
    state.is_git_repo = false;
    let output = JsonFormatter::format_repository_state(&state);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_git_repo"], false);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "repository_state");
}

#[test]
fn test_json_format_repository_state_clean() {
    use crate::schema::{GitStatusCounts, RepositoryStateSchema};
    let mut state = RepositoryStateSchema::new();
    state.branch = Some("main".to_string());
    state.is_clean = true;
    state.counts = GitStatusCounts::default();
    let output = JsonFormatter::format_repository_state(&state);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "main");
    assert_eq!(json["is_clean"], true);
    assert_eq!(json["is_detached"], false);
}

#[test]
fn test_json_format_repository_state_dirty() {
    use crate::schema::{GitStatusCounts, RepositoryStateSchema};
    let mut state = RepositoryStateSchema::new();
    state.branch = Some("feature".to_string());
    state.is_clean = false;
    state.is_detached = false;
    state.counts = GitStatusCounts { staged: 1, unstaged: 2, untracked: 3, unmerged: 0 };
    let output = JsonFormatter::format_repository_state(&state);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "feature");
    assert_eq!(json["is_clean"], false);
    assert_eq!(json["counts"]["staged"], 1);
    assert_eq!(json["counts"]["unstaged"], 2);
    assert_eq!(json["counts"]["untracked"], 3);
}

#[test]
fn test_json_format_repository_state_detached() {
    use crate::schema::{GitStatusCounts, RepositoryStateSchema};
    let mut state = RepositoryStateSchema::new();
    state.branch = Some("abc123".to_string());
    state.is_detached = true;
    state.is_clean = true;
    state.counts = GitStatusCounts::default();
    let output = JsonFormatter::format_repository_state(&state);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "abc123");
    assert_eq!(json["is_detached"], true);
}

#[test]
fn test_json_format_process_success() {
    use crate::schema::ProcessOutputSchema;
    let mut proc = ProcessOutputSchema::new("echo");
    proc.stdout = "hello\n".to_string();
    proc.success = true;
    let output = JsonFormatter::format_process(&proc);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["stdout"], "hello\n");
    assert_eq!(json["command"], "echo");
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "process_output");
}

#[test]
fn test_json_format_process_failure() {
    use crate::schema::ProcessOutputSchema;
    let mut proc = ProcessOutputSchema::new("false");
    proc.exit_code = Some(1);
    proc.success = false;
    proc.stderr = "error message\n".to_string();
    let output = JsonFormatter::format_process(&proc);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["exit_code"], 1);
    assert_eq!(json["stderr"], "error message\n");
}

#[test]
fn test_json_format_error_schema() {
    use crate::schema::ErrorSchema;
    let error = ErrorSchema::new("Something went wrong");
    let output = JsonFormatter::format_error_schema(&error);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["message"], "Something went wrong");
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "error");
}

#[test]
fn test_json_format_error_schema_with_code() {
    use crate::schema::ErrorSchema;
    let mut error = ErrorSchema::new("Command failed");
    error.exit_code = Some(1);
    let output = JsonFormatter::format_error_schema(&error);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["message"], "Command failed");
    assert_eq!(json["exit_code"], 1);
}

// ============================================================
// Format Selection Tests
// ============================================================

#[test]
fn test_select_formatter() {
    assert_eq!(select_formatter(OutputFormat::Compact), "compact");
    assert_eq!(select_formatter(OutputFormat::Json), "json");
    assert_eq!(select_formatter(OutputFormat::Csv), "csv");
    assert_eq!(select_formatter(OutputFormat::Tsv), "tsv");
    assert_eq!(select_formatter(OutputFormat::Agent), "agent");
    assert_eq!(select_formatter(OutputFormat::Raw), "raw");
}

// ============================================================
// Raw Formatter Tests
// ============================================================

#[test]
fn test_raw_format_list() {
    let items = vec!["file1.rs", "file2.rs"];
    let output = RawFormatter::format_list(&items);
    assert_eq!(output, "file1.rs\nfile2.rs\n");
}

#[test]
fn test_raw_format_message() {
    assert_eq!(
        RawFormatter::format_message("branch", "main"),
        "branch: main\n"
    );
}

#[test]
fn test_raw_format_counts() {
    let output = RawFormatter::format_counts(&[("passed", 10), ("failed", 2)]);
    assert_eq!(output, "passed=10 failed=2\n");

    let output = RawFormatter::format_counts(&[("passed", 0), ("failed", 2)]);
    assert_eq!(output, "failed=2\n");

    let output = RawFormatter::format_counts(&[("passed", 0), ("failed", 0)]);
    assert!(output.is_empty());
}

#[test]
fn test_raw_format_section_header() {
    assert_eq!(
        RawFormatter::format_section_header("staged", Some(3)),
        "staged (3)\n"
    );
    assert_eq!(
        RawFormatter::format_section_header("files", None),
        "files\n"
    );
}

#[test]
fn test_raw_format_item() {
    assert_eq!(
        RawFormatter::format_item("M", "src/main.rs"),
        "M src/main.rs\n"
    );
}

#[test]
fn test_raw_format_item_renamed() {
    assert_eq!(
        RawFormatter::format_item_renamed("R", "old.rs", "new.rs"),
        "R old.rs -> new.rs\n"
    );
}

#[test]
fn test_raw_format_test_summary() {
    let output = RawFormatter::format_test_summary(10, 2, 1, 1500);
    assert!(output.contains("passed=10 failed=2 skipped=1"));
    assert!(output.contains("1.50s"));
}

#[test]
fn test_raw_format_test_summary_only_passed() {
    let output = RawFormatter::format_test_summary(5, 0, 0, 500);
    assert!(output.contains("passed=5"));
    assert!(!output.contains("failed"));
    assert!(!output.contains("skipped"));
}

#[test]
fn test_raw_format_status() {
    assert_eq!(RawFormatter::format_status(true), "passed\n");
    assert_eq!(RawFormatter::format_status(false), "failed\n");
}

#[test]
fn test_raw_format_failures() {
    let failures = vec!["test_one".to_string(), "test_two".to_string()];
    let output = RawFormatter::format_failures(&failures);
    assert!(output.contains("test_one\n"));
    assert!(output.contains("test_two\n"));
}

#[test]
fn test_raw_format_failures_empty() {
    let failures: Vec<String> = vec![];
    let output = RawFormatter::format_failures(&failures);
    assert!(output.is_empty());
}

#[test]
fn test_raw_format_log_levels() {
    let output = RawFormatter::format_log_levels(2, 5, 10, 3);
    assert_eq!(output, "error=2 warn=5 info=10 debug=3\n");
}

#[test]
fn test_raw_format_log_levels_partial() {
    let output = RawFormatter::format_log_levels(0, 5, 0, 0);
    assert_eq!(output, "warn=5\n");
}

#[test]
fn test_raw_format_log_levels_empty() {
    let output = RawFormatter::format_log_levels(0, 0, 0, 0);
    assert!(output.is_empty());
}

#[test]
fn test_raw_format_grep_match() {
    let output = RawFormatter::format_grep_match("src/main.rs", Some(42), "fn main()");
    assert_eq!(output, "src/main.rs:42:fn main()\n");
}

#[test]
fn test_raw_format_grep_match_no_line() {
    let output = RawFormatter::format_grep_match("src/main.rs", None, "match found");
    assert_eq!(output, "src/main.rs:match found\n");
}

#[test]
fn test_raw_format_grep_file() {
    let output = RawFormatter::format_grep_file("src/main.rs", 5);
    assert_eq!(output, "src/main.rs (5)\n");
}

#[test]
fn test_raw_format_diff_file() {
    let output = RawFormatter::format_diff_file("src/main.rs", "M", 10, 5);
    assert_eq!(output, "M src/main.rs +10 -5\n");
}

#[test]
fn test_raw_format_diff_summary() {
    let output = RawFormatter::format_diff_summary(3, 25, 10);
    assert_eq!(output, "3 files +25 -10\n");
}

#[test]
fn test_raw_format_clean() {
    assert_eq!(RawFormatter::format_clean(), "clean\n");
}

#[test]
fn test_raw_format_dirty() {
    let output = RawFormatter::format_dirty(2, 3, 5, 0);
    assert_eq!(output, "dirty staged=2 unstaged=3 untracked=5 unmerged=0\n");
}

#[test]
fn test_raw_format_branch_with_tracking() {
    assert_eq!(
        RawFormatter::format_branch_with_tracking("main", 0, 0),
        "main\n"
    );

    assert_eq!(
        RawFormatter::format_branch_with_tracking("feature", 3, 0),
        "feature (ahead 3)\n"
    );

    assert_eq!(
        RawFormatter::format_branch_with_tracking("feature", 0, 2),
        "feature (behind 2)\n"
    );

    assert_eq!(
        RawFormatter::format_branch_with_tracking("feature", 3, 2),
        "feature (ahead 3, behind 2)\n"
    );
}

#[test]
fn test_raw_format_empty() {
    assert_eq!(RawFormatter::format_empty(), "");
}

#[test]
fn test_raw_format_truncated() {
    let output = RawFormatter::format_truncated(10, 50);
    assert_eq!(output, "... 10/50\n");
}

#[test]
fn test_raw_format_key_value() {
    assert_eq!(
        RawFormatter::format_key_value("branch", "main"),
        "branch main\n"
    );
}

#[test]
fn test_raw_format_raw() {
    assert_eq!(RawFormatter::format_raw("content\n"), "content\n");
    assert_eq!(RawFormatter::format_raw("content"), "content\n");
    assert_eq!(RawFormatter::format_raw(""), "");
}
