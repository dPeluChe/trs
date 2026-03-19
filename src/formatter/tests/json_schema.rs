use super::*;

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
