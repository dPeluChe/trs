//! Git status, git diff, and ls test fixtures module.
//!
//! This module provides access to various git status, git diff, and ls output fixtures
//! for testing the parsers.

use std::path::PathBuf;

/// Returns the path to the fixtures directory.
pub fn fixtures_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures");
    path
}

/// Loads a fixture file by name and returns its contents.
///
/// # Panics
///
/// Panics if the fixture file cannot be read.
pub fn load_fixture(name: &str) -> String {
    let path = fixtures_dir().join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture '{}': {}", name, e))
}

// ============================================================
// Clean Status Fixtures
// ============================================================

/// Returns a clean git status output (no changes).
pub fn git_status_clean() -> String {
    load_fixture("git_status_clean.txt")
}

/// Returns empty git status output.
pub fn git_status_empty() -> String {
    load_fixture("git_status_empty.txt")
}

/// Returns whitespace-only git status output.
pub fn git_status_whitespace_only() -> String {
    load_fixture("git_status_whitespace_only.txt")
}

// ============================================================
// Staged Changes Fixtures
// ============================================================

/// Returns git status with staged changes only.
pub fn git_status_staged() -> String {
    load_fixture("git_status_staged.txt")
}

/// Returns git status with staged renamed files.
pub fn git_status_renamed() -> String {
    load_fixture("git_status_renamed.txt")
}

/// Returns git status with staged copied files.
pub fn git_status_copied() -> String {
    load_fixture("git_status_copied.txt")
}

// ============================================================
// Unstaged Changes Fixtures
// ============================================================

/// Returns git status with unstaged changes only.
pub fn git_status_unstaged() -> String {
    load_fixture("git_status_unstaged.txt")
}

/// Returns git status with typechange (symlink to file).
pub fn git_status_typechange() -> String {
    load_fixture("git_status_typechange.txt")
}

// ============================================================
// Untracked Files Fixtures
// ============================================================

/// Returns git status with untracked files only.
pub fn git_status_untracked() -> String {
    load_fixture("git_status_untracked.txt")
}

// ============================================================
// Mixed Changes Fixtures
// ============================================================

/// Returns git status with staged, unstaged, and untracked changes.
pub fn git_status_mixed() -> String {
    load_fixture("git_status_mixed.txt")
}

/// Returns git status with all possible status codes.
pub fn git_status_all_status_codes() -> String {
    load_fixture("git_status_all_status_codes.txt")
}

// ============================================================
// Branch Status Fixtures
// ============================================================

/// Returns git status where branch is ahead of remote.
pub fn git_status_ahead() -> String {
    load_fixture("git_status_ahead.txt")
}

/// Returns git status where branch is behind remote.
pub fn git_status_behind() -> String {
    load_fixture("git_status_behind.txt")
}

/// Returns git status where branch has diverged from remote.
pub fn git_status_diverged() -> String {
    load_fixture("git_status_diverged.txt")
}

/// Returns git status in detached HEAD state.
pub fn git_status_detached() -> String {
    load_fixture("git_status_detached.txt")
}

/// Returns git status for initial commit (no upstream).
pub fn git_status_no_branch() -> String {
    load_fixture("git_status_no_branch.txt")
}

// ============================================================
// Conflict Fixtures
// ============================================================

/// Returns git status with merge conflicts.
pub fn git_status_conflict() -> String {
    load_fixture("git_status_conflict.txt")
}

// ============================================================
// Porcelain Format Fixtures
// ============================================================

/// Returns git status in porcelain format (v1).
pub fn git_status_porcelain() -> String {
    load_fixture("git_status_porcelain.txt")
}

/// Returns git status in porcelain format (v2).
pub fn git_status_porcelain_v2() -> String {
    load_fixture("git_status_porcelain_v2.txt")
}

// ============================================================
// Localized Fixtures
// ============================================================

/// Returns git status in Spanish (clean).
pub fn git_status_spanish_clean() -> String {
    load_fixture("git_status_spanish_clean.txt")
}

/// Returns git status in Spanish (with changes).
#[allow(dead_code)]
pub fn git_status_spanish_staged() -> String {
    load_fixture("git_status_spanish_staged.txt")
}

/// Returns git status in German (clean).
pub fn git_status_german_clean() -> String {
    load_fixture("git_status_german_clean.txt")
}

// ============================================================
// Edge Cases
// ============================================================

/// Returns git status with very long file paths.
pub fn git_status_long_paths() -> String {
    load_fixture("git_status_long_paths.txt")
}

// ============================================================
// Git Diff - Empty/Clean Fixtures
// ============================================================

/// Returns empty git diff output.
pub fn git_diff_empty() -> String {
    load_fixture("git_diff_empty.txt")
}

// ============================================================
// Git Diff - Basic Change Type Fixtures
// ============================================================

/// Returns git diff with a modified file.
pub fn git_diff_modified() -> String {
    load_fixture("git_diff_modified.txt")
}

/// Returns git diff with a new file added.
pub fn git_diff_added() -> String {
    load_fixture("git_diff_added.txt")
}

/// Returns git diff with a deleted file.
pub fn git_diff_deleted() -> String {
    load_fixture("git_diff_deleted.txt")
}

/// Returns git diff with a renamed file.
pub fn git_diff_renamed() -> String {
    load_fixture("git_diff_renamed.txt")
}

/// Returns git diff with a copied file.
pub fn git_diff_copied() -> String {
    load_fixture("git_diff_copied.txt")
}

// ============================================================
// Git Diff - Binary Files
// ============================================================

/// Returns git diff with a binary file.
pub fn git_diff_binary() -> String {
    load_fixture("git_diff_binary.txt")
}

// ============================================================
// Git Diff - Multiple Files
// ============================================================

/// Returns git diff with multiple files (modified, added, deleted).
pub fn git_diff_multiple() -> String {
    load_fixture("git_diff_multiple.txt")
}

/// Returns git diff with mixed changes (multiple types).
pub fn git_diff_mixed() -> String {
    load_fixture("git_diff_mixed.txt")
}

// ============================================================
// Git Diff - Edge Cases
// ============================================================

/// Returns git diff with many files (for testing truncation).
pub fn git_diff_large() -> String {
    load_fixture("git_diff_large.txt")
}

/// Returns git diff with very long file paths.
pub fn git_diff_long_paths() -> String {
    load_fixture("git_diff_long_paths.txt")
}

// ============================================================
// Ls - Empty/Clean Fixtures
// ============================================================

/// Returns empty ls output.
pub fn ls_empty() -> String {
    load_fixture("ls_empty.txt")
}

// ============================================================
// Ls - Simple Format Fixtures
// ============================================================

/// Returns simple ls output (just filenames).
pub fn ls_simple() -> String {
    load_fixture("ls_simple.txt")
}

/// Returns ls output with directories.
pub fn ls_with_directories() -> String {
    load_fixture("ls_with_directories.txt")
}

/// Returns ls output with hidden files.
pub fn ls_with_hidden() -> String {
    load_fixture("ls_with_hidden.txt")
}

// ============================================================
// Ls - Long Format Fixtures
// ============================================================

/// Returns ls -l output (long format).
pub fn ls_long_format() -> String {
    load_fixture("ls_long_format.txt")
}

/// Returns ls -l output with symlinks.
pub fn ls_long_format_with_symlinks() -> String {
    load_fixture("ls_long_format_with_symlinks.txt")
}

/// Returns ls -l output with broken symlinks.
pub fn ls_broken_symlink() -> String {
    load_fixture("ls_broken_symlink.txt")
}

// ============================================================
// Ls - Error Fixtures
// ============================================================

/// Returns ls output with permission denied errors.
pub fn ls_permission_denied() -> String {
    load_fixture("ls_permission_denied.txt")
}

// ============================================================
// Ls - Mixed Fixtures
// ============================================================

/// Returns ls -l output with mixed content (dirs, files, symlinks, hidden, errors).
pub fn ls_mixed() -> String {
    load_fixture("ls_mixed.txt")
}

/// Returns ls output with generated directories (node_modules, target, etc.).
pub fn ls_generated_dirs() -> String {
    load_fixture("ls_generated_dirs.txt")
}

// ============================================================
// Ls - Edge Cases
// ============================================================

/// Returns ls output with special characters in filenames.
pub fn ls_special_chars() -> String {
    load_fixture("ls_special_chars.txt")
}

/// Returns ls output with long file paths.
pub fn ls_long_paths() -> String {
    load_fixture("ls_long_paths.txt")
}

// ============================================================
// Grep - Empty/Clean Fixtures
// ============================================================

/// Returns empty grep output.
pub fn grep_empty() -> String {
    load_fixture("grep_empty.txt")
}

// ============================================================
// Grep - Simple Format Fixtures
// ============================================================

/// Returns simple grep output (single match).
pub fn grep_simple() -> String {
    load_fixture("grep_simple.txt")
}

/// Returns grep output with multiple matches in a single file.
pub fn grep_single_file_multiple_matches() -> String {
    load_fixture("grep_single_file_multiple_matches.txt")
}

/// Returns grep output with matches in multiple files.
pub fn grep_multiple_files() -> String {
    load_fixture("grep_multiple_files.txt")
}

// ============================================================
// Grep - With Column Numbers
// ============================================================

/// Returns grep output with column numbers (path:line:col:content).
pub fn grep_with_column() -> String {
    load_fixture("grep_with_column.txt")
}

/// Returns grep output without line numbers (path:content).
pub fn grep_without_line_numbers() -> String {
    load_fixture("grep_without_line_numbers.txt")
}

// ============================================================
// Grep - Binary Files
// ============================================================

/// Returns grep output with a binary file match.
pub fn grep_binary_file() -> String {
    load_fixture("grep_binary_file.txt")
}

// ============================================================
// Grep - Context Lines
// ============================================================

/// Returns grep output with context lines (before and after).
pub fn grep_context_lines() -> String {
    load_fixture("grep_context_lines.txt")
}

/// Returns grep output with context lines before matches.
pub fn grep_context_before() -> String {
    load_fixture("grep_context_before.txt")
}

/// Returns grep output with context lines after matches.
pub fn grep_context_after() -> String {
    load_fixture("grep_context_after.txt")
}

// ============================================================
// Grep - Edge Cases
// ============================================================

/// Returns grep output with long file paths.
pub fn grep_long_paths() -> String {
    load_fixture("grep_long_paths.txt")
}

/// Returns grep output with special characters in filenames.
pub fn grep_special_chars() -> String {
    load_fixture("grep_special_chars.txt")
}

/// Returns grep output with colon in content.
pub fn grep_with_colon_in_content() -> String {
    load_fixture("grep_with_colon_in_content.txt")
}

// ============================================================
// Grep - Large Output
// ============================================================

/// Returns grep output with many files and matches (for testing truncation).
pub fn grep_large() -> String {
    load_fixture("grep_large.txt")
}

// ============================================================
// Grep - Mixed Fixtures
// ============================================================

/// Returns grep output with mixed content (multiple files, context, binary).
pub fn grep_mixed() -> String {
    load_fixture("grep_mixed.txt")
}

// ============================================================
// Grep - Ripgrep Heading Format
// ============================================================

/// Returns grep output in ripgrep heading format (--heading).
pub fn grep_ripgrep_heading() -> String {
    load_fixture("grep_ripgrep_heading.txt")
}

// ============================================================
// Pytest - Empty/Clean Fixtures
// ============================================================

/// Returns empty pytest output.
pub fn pytest_empty() -> String {
    load_fixture("pytest_empty.txt")
}

// ============================================================
// Pytest - Basic Fixtures
// ============================================================

/// Returns pytest output with a single passed test.
pub fn pytest_single_passed() -> String {
    load_fixture("pytest_single_passed.txt")
}

/// Returns pytest output with a single failed test.
pub fn pytest_single_failed() -> String {
    load_fixture("pytest_single_failed.txt")
}

/// Returns pytest output with mixed results (passed, failed, skipped).
pub fn pytest_mixed() -> String {
    load_fixture("pytest_mixed.txt")
}

// ============================================================
// Pytest - Error Fixtures
// ============================================================

/// Returns pytest output with an error (e.g., fixture setup error).
pub fn pytest_with_error() -> String {
    load_fixture("pytest_with_error.txt")
}

// ============================================================
// Pytest - XFail/XPASS Fixtures
// ============================================================

/// Returns pytest output with xfail and xpass results.
pub fn pytest_with_xfail() -> String {
    load_fixture("pytest_with_xfail.txt")
}

// ============================================================
// Pytest - All Passed/Failed Fixtures
// ============================================================

/// Returns pytest output with all tests passed.
pub fn pytest_all_passed() -> String {
    load_fixture("pytest_all_passed.txt")
}

/// Returns pytest output with all tests failed.
pub fn pytest_all_failed() -> String {
    load_fixture("pytest_all_failed.txt")
}

// ============================================================
// Pytest - Edge Cases
// ============================================================

/// Returns pytest output with header info (platform, rootdir, etc.).
pub fn pytest_with_header() -> String {
    load_fixture("pytest_with_header.txt")
}

/// Returns pytest output with many test files (for testing truncation).
pub fn pytest_large() -> String {
    load_fixture("pytest_large.txt")
}

/// Returns pytest output with long file paths.
pub fn pytest_long_paths() -> String {
    load_fixture("pytest_long_paths.txt")
}

// ============================================================
// Jest - Empty/Clean Fixtures
// ============================================================

/// Returns empty jest output.
pub fn jest_empty() -> String {
    load_fixture("jest_empty.txt")
}

// ============================================================
// Jest - Basic Fixtures
// ============================================================

/// Returns jest output with a single passed test suite.
pub fn jest_single_suite_passed() -> String {
    load_fixture("jest_single_suite_passed.txt")
}

/// Returns jest output with a single failed test suite.
pub fn jest_single_suite_failed() -> String {
    load_fixture("jest_single_suite_failed.txt")
}

/// Returns jest output with mixed results (passed and failed suites).
pub fn jest_mixed() -> String {
    load_fixture("jest_mixed.txt")
}

// ============================================================
// Jest - All Passed/Failed Fixtures
// ============================================================

/// Returns jest output with all tests passed.
pub fn jest_all_passed() -> String {
    load_fixture("jest_all_passed.txt")
}

/// Returns jest output with all tests failed.
pub fn jest_all_failed() -> String {
    load_fixture("jest_all_failed.txt")
}

// ============================================================
// Jest - Skipped/Todo Fixtures
// ============================================================

/// Returns jest output with skipped tests.
pub fn jest_with_skipped() -> String {
    load_fixture("jest_with_skipped.txt")
}

/// Returns jest output with todo tests.
pub fn jest_with_todo() -> String {
    load_fixture("jest_with_todo.txt")
}

// ============================================================
// Jest - Edge Cases
// ============================================================

/// Returns jest output with multiple test suites.
pub fn jest_multiple_suites() -> String {
    load_fixture("jest_multiple_suites.txt")
}

/// Returns jest output with many test files (for testing truncation).
pub fn jest_large() -> String {
    load_fixture("jest_large.txt")
}

/// Returns jest output with nested describe blocks.
pub fn jest_with_nested_describe() -> String {
    load_fixture("jest_with_nested_describe.txt")
}

// ============================================================
// Vitest - Empty/Clean Fixtures
// ============================================================

/// Returns empty vitest output.
pub fn vitest_empty() -> String {
    load_fixture("vitest_empty.txt")
}

// ============================================================
// Vitest - Basic Fixtures
// ============================================================

/// Returns vitest output with a single passed test.
pub fn vitest_single_passed() -> String {
    load_fixture("vitest_single_passed.txt")
}

/// Returns vitest output with a single failed test.
pub fn vitest_single_failed() -> String {
    load_fixture("vitest_single_failed.txt")
}

/// Returns vitest output with mixed results (passed and failed).
pub fn vitest_mixed() -> String {
    load_fixture("vitest_mixed.txt")
}

// ============================================================
// Vitest - All Passed/Failed Fixtures
// ============================================================

/// Returns vitest output with all tests passed.
pub fn vitest_all_passed() -> String {
    load_fixture("vitest_all_passed.txt")
}

/// Returns vitest output with all tests failed.
pub fn vitest_all_failed() -> String {
    load_fixture("vitest_all_failed.txt")
}

// ============================================================
// Vitest - Skipped/Todo Fixtures
// ============================================================

/// Returns vitest output with skipped tests.
pub fn vitest_with_skipped() -> String {
    load_fixture("vitest_with_skipped.txt")
}

/// Returns vitest output with mixed passed and skipped tests.
pub fn vitest_mixed_skipped() -> String {
    load_fixture("vitest_mixed_skipped.txt")
}

/// Returns vitest output with todo tests.
pub fn vitest_with_todo() -> String {
    load_fixture("vitest_with_todo.txt")
}

// ============================================================
// Vitest - Edge Cases
// ============================================================

/// Returns vitest output with many test files (for testing truncation).
pub fn vitest_large() -> String {
    load_fixture("vitest_large.txt")
}

// ============================================================
// NPM Test - Empty/Clean Fixtures
// ============================================================

/// Returns empty npm test output.
pub fn npm_test_empty() -> String {
    load_fixture("npm_test_empty.txt")
}

// ============================================================
// NPM Test - Basic Fixtures
// ============================================================

/// Returns npm test output with a single passed test.
pub fn npm_test_single_passed() -> String {
    load_fixture("npm_test_single_passed.txt")
}

/// Returns npm test output with a single failed test.
pub fn npm_test_single_failed() -> String {
    load_fixture("npm_test_single_failed.txt")
}

/// Returns npm test output with mixed results (passed, failed, skipped).
pub fn npm_test_mixed() -> String {
    load_fixture("npm_test_mixed.txt")
}

// ============================================================
// NPM Test - All Passed/Failed Fixtures
// ============================================================

/// Returns npm test output with all tests passed.
pub fn npm_test_all_passed() -> String {
    load_fixture("npm_test_all_passed.txt")
}

/// Returns npm test output with all tests failed.
pub fn npm_test_all_failed() -> String {
    load_fixture("npm_test_all_failed.txt")
}

// ============================================================
// NPM Test - Skipped/Todo Fixtures
// ============================================================

/// Returns npm test output with skipped tests.
pub fn npm_test_with_skipped() -> String {
    load_fixture("npm_test_with_skipped.txt")
}

/// Returns npm test output with todo tests.
pub fn npm_test_with_todo() -> String {
    load_fixture("npm_test_with_todo.txt")
}

// ============================================================
// NPM Test - Edge Cases
// ============================================================

/// Returns npm test output with multiple test suites.
pub fn npm_test_multiple_suites() -> String {
    load_fixture("npm_test_multiple_suites.txt")
}

/// Returns npm test output with many test files (for testing truncation).
pub fn npm_test_large() -> String {
    load_fixture("npm_test_large.txt")
}

/// Returns npm test output with header (npm script info).
pub fn npm_test_with_header() -> String {
    load_fixture("npm_test_with_header.txt")
}

// ============================================================
// PNPM Test - Empty/Clean Fixtures
// ============================================================

/// Returns empty pnpm test output.
pub fn pnpm_test_empty() -> String {
    load_fixture("pnpm_test_empty.txt")
}

// ============================================================
// PNPM Test - Basic Fixtures
// ============================================================

/// Returns pnpm test output with a single passed test.
pub fn pnpm_test_single_passed() -> String {
    load_fixture("pnpm_test_single_passed.txt")
}

/// Returns pnpm test output with a single failed test.
pub fn pnpm_test_single_failed() -> String {
    load_fixture("pnpm_test_single_failed.txt")
}

/// Returns pnpm test output with mixed results (passed, failed, skipped).
pub fn pnpm_test_mixed() -> String {
    load_fixture("pnpm_test_mixed.txt")
}

// ============================================================
// PNPM Test - All Passed/Failed Fixtures
// ============================================================

/// Returns pnpm test output with all tests passed.
pub fn pnpm_test_all_passed() -> String {
    load_fixture("pnpm_test_all_passed.txt")
}

/// Returns pnpm test output with all tests failed.
pub fn pnpm_test_all_failed() -> String {
    load_fixture("pnpm_test_all_failed.txt")
}

// ============================================================
// PNPM Test - Skipped/Todo Fixtures
// ============================================================

/// Returns pnpm test output with skipped tests.
pub fn pnpm_test_with_skipped() -> String {
    load_fixture("pnpm_test_with_skipped.txt")
}

/// Returns pnpm test output with todo tests.
pub fn pnpm_test_with_todo() -> String {
    load_fixture("pnpm_test_with_todo.txt")
}

// ============================================================
// PNPM Test - Edge Cases
// ============================================================

/// Returns pnpm test output with many test files (for testing truncation).
pub fn pnpm_test_large() -> String {
    load_fixture("pnpm_test_large.txt")
}

/// Returns pnpm test output with header (pnpm version info).
pub fn pnpm_test_with_header() -> String {
    load_fixture("pnpm_test_with_header.txt")
}

// ============================================================
// Bun Test - Empty/Clean Fixtures
// ============================================================

/// Returns empty bun test output.
pub fn bun_test_empty() -> String {
    load_fixture("bun_test_empty.txt")
}

// ============================================================
// Bun Test - Basic Fixtures
// ============================================================

/// Returns bun test output with a single passed test.
pub fn bun_test_single_passed() -> String {
    load_fixture("bun_test_single_passed.txt")
}

/// Returns bun test output with a single failed test.
pub fn bun_test_single_failed() -> String {
    load_fixture("bun_test_single_failed.txt")
}

/// Returns bun test output with mixed results (passed and failed).
pub fn bun_test_mixed() -> String {
    load_fixture("bun_test_mixed.txt")
}

// ============================================================
// Bun Test - All Passed/Failed Fixtures
// ============================================================

/// Returns bun test output with all tests passed.
pub fn bun_test_all_passed() -> String {
    load_fixture("bun_test_all_passed.txt")
}

/// Returns bun test output with all tests failed.
pub fn bun_test_all_failed() -> String {
    load_fixture("bun_test_all_failed.txt")
}

// ============================================================
// Bun Test - Skipped/Todo Fixtures
// ============================================================

/// Returns bun test output with skipped tests.
pub fn bun_test_with_skipped() -> String {
    load_fixture("bun_test_with_skipped.txt")
}

/// Returns bun test output with todo tests.
pub fn bun_test_with_todo() -> String {
    load_fixture("bun_test_with_todo.txt")
}

// ============================================================
// Bun Test - Edge Cases
// ============================================================

/// Returns bun test output with multiple test suites.
pub fn bun_test_multiple_suites() -> String {
    load_fixture("bun_test_multiple_suites.txt")
}

/// Returns bun test output with many test files (for testing truncation).
pub fn bun_test_large() -> String {
    load_fixture("bun_test_large.txt")
}

/// Returns bun test output in non-TTY format.
pub fn bun_test_non_tty() -> String {
    load_fixture("bun_test_non_tty.txt")
}

// ============================================================
// Logs - Empty/Clean Fixtures
// ============================================================

/// Returns empty logs output.
pub fn logs_empty() -> String {
    load_fixture("logs_empty.txt")
}

// ============================================================
// Logs - Simple Format Fixtures
// ============================================================

/// Returns simple log output with all levels.
pub fn logs_simple() -> String {
    load_fixture("logs_simple.txt")
}

/// Returns logs with ISO 8601 timestamps.
pub fn logs_iso8601_timestamp() -> String {
    load_fixture("logs_iso8601_timestamp.txt")
}

/// Returns logs in syslog format.
pub fn logs_syslog_format() -> String {
    load_fixture("logs_syslog_format.txt")
}

// ============================================================
// Logs - All Levels Fixtures
// ============================================================

/// Returns logs with all log levels represented.
pub fn logs_all_levels() -> String {
    load_fixture("logs_all_levels.txt")
}

/// Returns logs with only error-level entries.
pub fn logs_errors_only() -> String {
    load_fixture("logs_errors_only.txt")
}

// ============================================================
// Logs - Exception/Stack Trace Fixtures
// ============================================================

/// Returns logs with Java and Python exceptions/stack traces.
pub fn logs_with_exceptions() -> String {
    load_fixture("logs_with_exceptions.txt")
}

/// Returns logs with multiline error messages and stack traces.
pub fn logs_multiline_error() -> String {
    load_fixture("logs_multiline_error.txt")
}

// ============================================================
// Logs - Mixed Format Fixtures
// ============================================================

/// Returns logs with mixed format styles (timestamps, levels).
pub fn logs_mixed_format() -> String {
    load_fixture("logs_mixed_format.txt")
}

/// Returns logs with colon-level format (LEVEL: message).
pub fn logs_colon_format() -> String {
    load_fixture("logs_colon_format.txt")
}

/// Returns logs with pipe-level format (|LEVEL| message).
pub fn logs_pipe_format() -> String {
    load_fixture("logs_pipe_format.txt")
}

// ============================================================
// Logs - Edge Cases
// ============================================================

/// Returns logs with repeated lines (for testing deduplication).
pub fn logs_repeated_lines() -> String {
    load_fixture("logs_repeated_lines.txt")
}

/// Returns logs with many entries (for testing truncation).
pub fn logs_large() -> String {
    load_fixture("logs_large.txt")
}

/// Returns logs in JSON format.
pub fn logs_json_format() -> String {
    load_fixture("logs_json_format.txt")
}

/// Returns logs with source/logger names.
pub fn logs_with_source() -> String {
    load_fixture("logs_with_source.txt")
}

/// Returns application startup/shutdown logs.
pub fn logs_application() -> String {
    load_fixture("logs_application.txt")
}

/// Returns HTTP access logs (Apache/nginx style).
pub fn logs_access() -> String {
    load_fixture("logs_access.txt")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixtures_dir_exists() {
        let dir = fixtures_dir();
        assert!(dir.exists(), "Fixtures directory should exist: {:?}", dir);
    }

    #[test]
    fn test_load_fixture_clean() {
        let content = git_status_clean();
        assert!(content.contains("On branch main"));
        assert!(content.contains("working tree clean"));
    }

    #[test]
    fn test_load_fixture_staged() {
        let content = git_status_staged();
        assert!(content.contains("Changes to be committed"));
        assert!(content.contains("modified:"));
        assert!(content.contains("new file:"));
        assert!(content.contains("deleted:"));
    }

    #[test]
    fn test_load_fixture_unstaged() {
        let content = git_status_unstaged();
        assert!(content.contains("Changes not staged for commit"));
        assert!(content.contains("modified:"));
        assert!(content.contains("deleted:"));
    }

    #[test]
    fn test_load_fixture_untracked() {
        let content = git_status_untracked();
        assert!(content.contains("Untracked files"));
        assert!(content.contains("new_feature.rs"));
    }

    #[test]
    fn test_load_fixture_mixed() {
        let content = git_status_mixed();
        assert!(content.contains("Changes to be committed"));
        assert!(content.contains("Changes not staged for commit"));
        assert!(content.contains("Untracked files"));
    }

    #[test]
    fn test_load_fixture_ahead() {
        let content = git_status_ahead();
        assert!(content.contains("ahead of"));
        assert!(content.contains("by 3 commits"));
    }

    #[test]
    fn test_load_fixture_behind() {
        let content = git_status_behind();
        assert!(content.contains("behind"));
        assert!(content.contains("by 5 commits"));
    }

    #[test]
    fn test_load_fixture_diverged() {
        let content = git_status_diverged();
        assert!(content.contains("diverged"));
        assert!(content.contains("3 and 5 different commits"));
    }

    #[test]
    fn test_load_fixture_detached() {
        let content = git_status_detached();
        assert!(content.contains("HEAD detached at"));
    }

    #[test]
    fn test_load_fixture_renamed() {
        let content = git_status_renamed();
        assert!(content.contains("renamed:"));
        assert!(content.contains("->"));
    }

    #[test]
    fn test_load_fixture_conflict() {
        let content = git_status_conflict();
        assert!(content.contains("Unmerged paths"));
        assert!(content.contains("both modified:"));
        assert!(content.contains("both added:"));
    }

    #[test]
    fn test_load_fixture_porcelain() {
        let content = git_status_porcelain();
        assert!(content.contains(" M "));
        assert!(content.contains("A  "));
        assert!(content.contains("?? "));
    }

    #[test]
    fn test_load_fixture_porcelain_v2() {
        let content = git_status_porcelain_v2();
        assert!(content.contains("# branch.head"));
        assert!(content.contains("# branch.ab"));
    }

    #[test]
    fn test_load_fixture_copied() {
        let content = git_status_copied();
        assert!(content.contains("copied:"));
    }

    #[test]
    fn test_load_fixture_typechange() {
        let content = git_status_typechange();
        assert!(content.contains("typechange:"));
    }

    #[test]
    fn test_load_fixture_spanish_clean() {
        let content = git_status_spanish_clean();
        assert!(content.contains("En la rama"));
        assert!(content.contains("árbol de trabajo limpio"));
    }

    #[test]
    fn test_load_fixture_german_clean() {
        let content = git_status_german_clean();
        assert!(content.contains("Auf Branch"));
        assert!(content.contains("Arbeitsverzeichnis unverändert"));
    }

    #[test]
    fn test_load_fixture_empty() {
        let content = git_status_empty();
        assert!(content.is_empty());
    }

    #[test]
    fn test_load_fixture_whitespace_only() {
        let content = git_status_whitespace_only();
        assert!(content.trim().is_empty());
    }

    #[test]
    fn test_load_fixture_no_branch() {
        let content = git_status_no_branch();
        assert!(content.contains("Initial commit"));
    }

    #[test]
    fn test_load_fixture_long_paths() {
        let content = git_status_long_paths();
        assert!(content.contains("very/deeply/nested"));
        assert!(content.contains("path/with spaces"));
    }

    #[test]
    fn test_load_fixture_all_status_codes() {
        let content = git_status_all_status_codes();
        assert!(content.contains("new file:"));
        assert!(content.contains("modified:"));
        assert!(content.contains("deleted:"));
        assert!(content.contains("renamed:"));
        assert!(content.contains("copied:"));
        assert!(content.contains("typechange:"));
        assert!(content.contains("both modified:"));
    }

    // ============================================================
    // Git Diff Fixture Tests
    // ============================================================

    #[test]
    fn test_load_git_diff_empty() {
        let content = git_diff_empty();
        assert!(content.trim().is_empty());
    }

    #[test]
    fn test_load_git_diff_modified() {
        let content = git_diff_modified();
        assert!(content.contains("diff --git"));
        assert!(content.contains("src/main.rs"));
        assert!(content.contains("let x = 1"));
        assert!(content.contains("let y = 2"));
    }

    #[test]
    fn test_load_git_diff_added() {
        let content = git_diff_added();
        assert!(content.contains("diff --git"));
        assert!(content.contains("new file mode"));
        assert!(content.contains("src/utils.rs"));
        assert!(content.contains("+pub fn helper()"));
    }

    #[test]
    fn test_load_git_diff_deleted() {
        let content = git_diff_deleted();
        assert!(content.contains("diff --git"));
        assert!(content.contains("deleted file mode"));
        assert!(content.contains("src/deprecated.rs"));
        assert!(content.contains("-pub fn old_function()"));
    }

    #[test]
    fn test_load_git_diff_renamed() {
        let content = git_diff_renamed();
        assert!(content.contains("diff --git"));
        assert!(content.contains("rename from"));
        assert!(content.contains("rename to"));
        assert!(content.contains("src/old_name.rs"));
        assert!(content.contains("src/new_name.rs"));
    }

    #[test]
    fn test_load_git_diff_copied() {
        let content = git_diff_copied();
        assert!(content.contains("diff --git"));
        assert!(content.contains("copy from"));
        assert!(content.contains("copy to"));
        assert!(content.contains("src/template.rs"));
        assert!(content.contains("src/implementation.rs"));
    }

    #[test]
    fn test_load_git_diff_binary() {
        let content = git_diff_binary();
        assert!(content.contains("diff --git"));
        assert!(content.contains("Binary files"));
        assert!(content.contains("differ"));
        assert!(content.contains("assets/image.png"));
    }

    #[test]
    fn test_load_git_diff_multiple() {
        let content = git_diff_multiple();
        assert!(content.contains("src/main.rs"));
        assert!(content.contains("src/utils.rs"));
        assert!(content.contains("src/old.rs"));
        // Check for different change types
        assert!(content.contains("new file mode"));
        assert!(content.contains("deleted file mode"));
    }

    #[test]
    fn test_load_git_diff_mixed() {
        let content = git_diff_mixed();
        // Check for multiple files
        assert!(content.contains("src/main.rs"));
        assert!(content.contains("src/lib.rs"));
        assert!(content.contains("src/utils.rs"));
        assert!(content.contains("src/deprecated.rs"));
        assert!(content.contains("assets/logo.png"));
        // Check for binary diff
        assert!(content.contains("Binary files"));
    }

    #[test]
    fn test_load_git_diff_large() {
        let content = git_diff_large();
        // Check that we have 10 files
        assert!(content.contains("src/file01.rs"));
        assert!(content.contains("src/file10.rs"));
        // All should be new files
        let new_file_count = content.matches("new file mode").count();
        assert_eq!(new_file_count, 10);
    }

    #[test]
    fn test_load_git_diff_long_paths() {
        let content = git_diff_long_paths();
        assert!(content.contains("very/deeply/nested"));
        assert!(content.contains("file with spaces"));
        assert!(content.contains("special chars"));
    }

    // ============================================================
    // Ls Fixture Tests
    // ============================================================

    #[test]
    fn test_load_ls_empty() {
        let content = ls_empty();
        assert!(content.is_empty());
    }

    #[test]
    fn test_load_ls_simple() {
        let content = ls_simple();
        assert!(content.contains("src"));
        assert!(content.contains("Cargo.toml"));
        assert!(content.contains("README.md"));
    }

    #[test]
    fn test_load_ls_with_directories() {
        let content = ls_with_directories();
        assert!(content.contains("src"));
        assert!(content.contains("tests"));
        assert!(content.contains("target"));
        assert!(content.contains("node_modules"));
    }

    #[test]
    fn test_load_ls_with_hidden() {
        let content = ls_with_hidden();
        assert!(content.contains(".git"));
        assert!(content.contains(".gitignore"));
        assert!(content.contains(".cargo"));
        assert!(content.contains(".hidden_file"));
    }

    #[test]
    fn test_load_ls_long_format() {
        let content = ls_long_format();
        assert!(content.contains("total 32"));
        assert!(content.contains("drwxr-xr-x"));
        assert!(content.contains("-rw-r--r--"));
    }

    #[test]
    fn test_load_ls_long_format_with_symlinks() {
        let content = ls_long_format_with_symlinks();
        assert!(content.contains("lrwxr-xr-x"));
        assert!(content.contains("->"));
        assert!(content.contains("link_to_src"));
        assert!(content.contains("link_to_file"));
    }

    #[test]
    fn test_load_ls_broken_symlink() {
        let content = ls_broken_symlink();
        assert!(content.contains("broken_link"));
        assert!(content.contains("old_link"));
        assert!(content.contains("->"));
    }

    #[test]
    fn test_load_ls_permission_denied() {
        let content = ls_permission_denied();
        assert!(content.contains("ls:"));
        assert!(content.contains("Permission denied"));
        assert!(content.contains("No such file or directory"));
    }

    #[test]
    fn test_load_ls_mixed() {
        let content = ls_mixed();
        assert!(content.contains("total 48"));
        assert!(content.contains("drwxr-xr-x"));
        assert!(content.contains("lrwxr-xr-x"));
        assert!(content.contains("-rw-r--r--"));
        assert!(content.contains(".git"));
        assert!(content.contains("ls:"));
        assert!(content.contains("Permission denied"));
    }

    #[test]
    fn test_load_ls_generated_dirs() {
        let content = ls_generated_dirs();
        assert!(content.contains("node_modules"));
        assert!(content.contains("target"));
        assert!(content.contains("dist"));
        assert!(content.contains("build"));
        assert!(content.contains("__pycache__"));
    }

    #[test]
    fn test_load_ls_special_chars() {
        let content = ls_special_chars();
        assert!(content.contains("file with spaces.txt"));
        assert!(content.contains("special[1].txt"));
        assert!(content.contains("bracket(2).txt"));
        assert!(content.contains("unicode_ñame.txt"));
    }

    #[test]
    fn test_load_ls_long_paths() {
        let content = ls_long_paths();
        assert!(content.contains("very/deeply/nested"));
        assert!(content.contains("another/long/path"));
        assert!(content.contains("project/submodule/src"));
    }

    // ============================================================
    // Grep Fixture Tests
    // ============================================================

    #[test]
    fn test_load_grep_empty() {
        let content = grep_empty();
        assert!(content.is_empty());
    }

    #[test]
    fn test_load_grep_simple() {
        let content = grep_simple();
        assert!(content.contains("src/main.rs"));
        assert!(content.contains("fn main()"));
        assert!(content.contains(":42:"));
    }

    #[test]
    fn test_load_grep_single_file_multiple_matches() {
        let content = grep_single_file_multiple_matches();
        assert!(content.contains("src/main.rs"));
        assert!(content.contains("fn init()"));
        assert!(content.contains("fn process()"));
        assert!(content.contains("fn main()"));
        assert!(content.contains("fn cleanup()"));
    }

    #[test]
    fn test_load_grep_multiple_files() {
        let content = grep_multiple_files();
        assert!(content.contains("src/main.rs"));
        assert!(content.contains("src/lib.rs"));
        assert!(content.contains("src/utils.rs"));
    }

    #[test]
    fn test_load_grep_with_column() {
        let content = grep_with_column();
        assert!(content.contains("src/main.rs:42:10:"));
        assert!(content.contains("src/lib.rs:15:5:"));
    }

    #[test]
    fn test_load_grep_without_line_numbers() {
        let content = grep_without_line_numbers();
        assert!(content.contains("src/main.rs:fn main()"));
        assert!(content.contains("src/lib.rs:pub fn init()"));
    }

    #[test]
    fn test_load_grep_binary_file() {
        let content = grep_binary_file();
        assert!(content.contains("Binary file"));
        assert!(content.contains("matches"));
    }

    #[test]
    fn test_load_grep_context_lines() {
        let content = grep_context_lines();
        assert!(content.contains("src/main.rs:41:fn main()"));
        assert!(content.contains("src/main.rs-39-"));
        assert!(content.contains("src/main.rs-43-"));
    }

    #[test]
    fn test_load_grep_context_before() {
        let content = grep_context_before();
        assert!(content.contains("src/main.rs:41:fn main()"));
        assert!(content.contains("src/main.rs-39-"));
        assert!(content.contains("src/main.rs-40-"));
    }

    #[test]
    fn test_load_grep_context_after() {
        let content = grep_context_after();
        assert!(content.contains("src/main.rs:41:fn main()"));
        assert!(content.contains("src/main.rs-42-"));
        assert!(content.contains("src/main.rs-43-"));
    }

    #[test]
    fn test_load_grep_long_paths() {
        let content = grep_long_paths();
        assert!(content.contains("very/deeply/nested"));
        assert!(content.contains("another/long/path"));
        assert!(content.contains("project/submodule/src"));
    }

    #[test]
    fn test_load_grep_special_chars() {
        let content = grep_special_chars();
        assert!(content.contains("file with spaces.rs"));
        assert!(content.contains("special[1].rs"));
        assert!(content.contains("bracket(2).rs"));
        assert!(content.contains("unicode_ñame.rs"));
    }

    #[test]
    fn test_load_grep_with_colon_in_content() {
        let content = grep_with_colon_in_content();
        assert!(content.contains("url: \"https://example.com/api\""));
        assert!(content.contains("host: \"localhost:8080\""));
        assert!(content.contains("Parse format: key:value"));
    }

    #[test]
    fn test_load_grep_large() {
        let content = grep_large();
        // Check that we have 55 files
        assert!(content.contains("src/file01.rs"));
        assert!(content.contains("src/file55.rs"));
        // Check multiple matches in file01
        assert!(content.contains("fn function_01()"));
        assert!(content.contains("fn function_25()"));
    }

    #[test]
    fn test_load_grep_mixed() {
        let content = grep_mixed();
        assert!(content.contains("src/main.rs"));
        assert!(content.contains("src/lib.rs"));
        assert!(content.contains("src/utils.rs"));
        assert!(content.contains("Binary file"));
        assert!(content.contains("very/deep"));
    }

    #[test]
    fn test_load_grep_ripgrep_heading() {
        let content = grep_ripgrep_heading();
        assert!(content.contains("src/main.rs"));
        assert!(content.contains("src/lib.rs"));
        assert!(content.contains("src/utils.rs"));
        // In heading format, line numbers don't have file prefix
        assert!(content.contains("42:fn main()"));
        assert!(content.contains("15:pub fn init()"));
    }

    // ============================================================
    // Pytest Fixture Tests
    // ============================================================

    #[test]
    fn test_load_pytest_empty() {
        let content = pytest_empty();
        assert!(content.is_empty());
    }

    #[test]
    fn test_load_pytest_single_passed() {
        let content = pytest_single_passed();
        assert!(content.contains("test_add PASSED"));
        assert!(content.contains("1 passed"));
    }

    #[test]
    fn test_load_pytest_single_failed() {
        let content = pytest_single_failed();
        assert!(content.contains("test_divide_by_zero FAILED"));
        assert!(content.contains("ZeroDivisionError"));
        assert!(content.contains("1 failed"));
    }

    #[test]
    fn test_load_pytest_mixed() {
        let content = pytest_mixed();
        assert!(content.contains("PASSED"));
        assert!(content.contains("FAILED"));
        assert!(content.contains("SKIPPED"));
        assert!(content.contains("5 passed, 1 failed, 1 skipped"));
    }

    #[test]
    fn test_load_pytest_with_error() {
        let content = pytest_with_error();
        assert!(content.contains("ERROR"));
        assert!(content.contains("RuntimeError"));
        assert!(content.contains("1 passed, 1 error"));
    }

    #[test]
    fn test_load_pytest_with_xfail() {
        let content = pytest_with_xfail();
        assert!(content.contains("XFAIL"));
        assert!(content.contains("XPASS"));
        assert!(content.contains("SKIPPED"));
        assert!(content.contains("xfailed"));
        assert!(content.contains("xpassed"));
    }

    #[test]
    fn test_load_pytest_all_passed() {
        let content = pytest_all_passed();
        assert!(content.contains("test_add PASSED"));
        assert!(content.contains("test_subtract PASSED"));
        assert!(content.contains("test_multiply PASSED"));
        assert!(content.contains("test_divide PASSED"));
        assert!(content.contains("4 passed"));
    }

    #[test]
    fn test_load_pytest_all_failed() {
        let content = pytest_all_failed();
        assert!(content.contains("test_one FAILED"));
        assert!(content.contains("test_two FAILED"));
        assert!(content.contains("test_three FAILED"));
        assert!(content.contains("3 failed"));
    }

    #[test]
    fn test_load_pytest_with_header() {
        let content = pytest_with_header();
        assert!(content.contains("platform"));
        assert!(content.contains("pytest-8.0.0"));
        assert!(content.contains("rootdir:"));
        assert!(content.contains("3 passed"));
    }

    #[test]
    fn test_load_pytest_large() {
        let content = pytest_large();
        assert!(content.contains("test_file01.py"));
        assert!(content.contains("test_file10.py"));
        assert!(content.contains("20 passed"));
    }

    #[test]
    fn test_load_pytest_long_paths() {
        let content = pytest_long_paths();
        assert!(content.contains("very/deeply/nested"));
        assert!(content.contains("unit/services/authentication"));
        assert!(content.contains("3 passed"));
    }

    // ============================================================
    // Jest Fixture Tests
    // ============================================================

    #[test]
    fn test_load_jest_empty() {
        let content = jest_empty();
        assert!(content.is_empty());
    }

    #[test]
    fn test_load_jest_single_suite_passed() {
        let content = jest_single_suite_passed();
        assert!(content.contains("PASS"));
        assert!(content.contains("src/utils.test.js"));
        assert!(content.contains("✓"));
        assert!(content.contains("1 passed"));
    }

    #[test]
    fn test_load_jest_single_suite_failed() {
        let content = jest_single_suite_failed();
        assert!(content.contains("FAIL"));
        assert!(content.contains("✕"));
        assert!(content.contains("expect(received).toBe(expected)"));
        assert!(content.contains("1 failed"));
    }

    #[test]
    fn test_load_jest_mixed() {
        let content = jest_mixed();
        assert!(content.contains("PASS"));
        assert!(content.contains("FAIL"));
        assert!(content.contains("✓"));
        assert!(content.contains("✕"));
        assert!(content.contains("○ skipped"));
        assert!(content.contains("1 passed, 1 failed"));
    }

    #[test]
    fn test_load_jest_all_passed() {
        let content = jest_all_passed();
        assert!(content.contains("PASS"));
        assert!(content.contains("6 passed"));
        assert!(content.contains("2 passed, 2 total"));
    }

    #[test]
    fn test_load_jest_all_failed() {
        let content = jest_all_failed();
        assert!(content.contains("FAIL"));
        assert!(content.contains("2 failed"));
    }

    #[test]
    fn test_load_jest_with_skipped() {
        let content = jest_with_skipped();
        assert!(content.contains("○ skipped"));
        assert!(content.contains("2 skipped"));
    }

    #[test]
    fn test_load_jest_with_todo() {
        let content = jest_with_todo();
        assert!(content.contains("○ todo"));
        assert!(content.contains("1 todo"));
    }

    #[test]
    fn test_load_jest_multiple_suites() {
        let content = jest_multiple_suites();
        assert!(content.contains("Button.test.js"));
        assert!(content.contains("Input.test.js"));
        assert!(content.contains("Modal.test.js"));
        assert!(content.contains("3 passed"));
        assert!(content.contains("7 passed"));
    }

    #[test]
    fn test_load_jest_large() {
        let content = jest_large();
        assert!(content.contains("test01.test.js"));
        assert!(content.contains("test10.test.js"));
        assert!(content.contains("10 passed"));
        assert!(content.contains("20 passed"));
    }

    #[test]
    fn test_load_jest_with_nested_describe() {
        let content = jest_with_nested_describe();
        assert!(content.contains("describe block"));
        assert!(content.contains("nested describe"));
        assert!(content.contains("alternative delimiter"));
    }

    // ============================================================
    // Vitest Fixture Tests
    // ============================================================

    #[test]
    fn test_load_vitest_empty() {
        let content = vitest_empty();
        assert!(content.is_empty());
    }

    #[test]
    fn test_load_vitest_single_passed() {
        let content = vitest_single_passed();
        assert!(content.contains("✓"));
        assert!(content.contains("1 passed"));
    }

    #[test]
    fn test_load_vitest_single_failed() {
        let content = vitest_single_failed();
        assert!(content.contains("❯"));
        assert!(content.contains("AssertionError"));
        assert!(content.contains("1 failed"));
    }

    #[test]
    fn test_load_vitest_mixed() {
        let content = vitest_mixed();
        assert!(content.contains("✓"));
        assert!(content.contains("❯"));
        assert!(content.contains("1 passed, 1 failed"));
    }

    #[test]
    fn test_load_vitest_all_passed() {
        let content = vitest_all_passed();
        assert!(content.contains("✓"));
        assert!(content.contains("3 passed (3)"));
        assert!(content.contains("9 passed (9)"));
    }

    #[test]
    fn test_load_vitest_all_failed() {
        let content = vitest_all_failed();
        assert!(content.contains("❯"));
        assert!(content.contains("2 failed"));
    }

    #[test]
    fn test_load_vitest_with_skipped() {
        let content = vitest_with_skipped();
        assert!(content.contains("1 skipped"));
    }

    #[test]
    fn test_load_vitest_mixed_skipped() {
        let content = vitest_mixed_skipped();
        assert!(content.contains("10 passed"));
        assert!(content.contains("3 skipped"));
    }

    #[test]
    fn test_load_vitest_with_todo() {
        let content = vitest_with_todo();
        assert!(content.contains("1 todo"));
    }

    #[test]
    fn test_load_vitest_large() {
        let content = vitest_large();
        assert!(content.contains("test01.test.ts"));
        assert!(content.contains("test10.test.ts"));
        assert!(content.contains("10 passed"));
        assert!(content.contains("20 passed"));
    }

    // ============================================================
    // NPM Test Fixture Tests
    // ============================================================

    #[test]
    fn test_load_npm_test_empty() {
        let content = npm_test_empty();
        assert!(content.is_empty());
    }

    #[test]
    fn test_load_npm_test_single_passed() {
        let content = npm_test_single_passed();
        assert!(content.contains("✔"));
        assert!(content.contains("1 passed"));
    }

    #[test]
    fn test_load_npm_test_single_failed() {
        let content = npm_test_single_failed();
        assert!(content.contains("✖"));
        assert!(content.contains("AssertionError"));
        assert!(content.contains("1 failed"));
    }

    #[test]
    fn test_load_npm_test_mixed() {
        let content = npm_test_mixed();
        assert!(content.contains("✔"));
        assert!(content.contains("✖"));
        assert!(content.contains("# SKIP"));
    }

    #[test]
    fn test_load_npm_test_all_passed() {
        let content = npm_test_all_passed();
        assert!(content.contains("✔"));
        assert!(content.contains("5 passed"));
    }

    #[test]
    fn test_load_npm_test_all_failed() {
        let content = npm_test_all_failed();
        assert!(content.contains("✖"));
        assert!(content.contains("2 failed"));
    }

    #[test]
    fn test_load_npm_test_with_skipped() {
        let content = npm_test_with_skipped();
        assert!(content.contains("# SKIP"));
        assert!(content.contains("2 passed"));
    }

    #[test]
    fn test_load_npm_test_with_todo() {
        let content = npm_test_with_todo();
        assert!(content.contains("# TODO"));
        assert!(content.contains("# SKIP"));
    }

    #[test]
    fn test_load_npm_test_multiple_suites() {
        let content = npm_test_multiple_suites();
        assert!(content.contains("Button.test.js"));
        assert!(content.contains("Input.test.js"));
        assert!(content.contains("Modal.test.js"));
        assert!(content.contains("7 passed"));
    }

    #[test]
    fn test_load_npm_test_large() {
        let content = npm_test_large();
        assert!(content.contains("test01.test.js"));
        assert!(content.contains("test10.test.js"));
        assert!(content.contains("20 passed"));
    }

    #[test]
    fn test_load_npm_test_with_header() {
        let content = npm_test_with_header();
        assert!(content.contains("> project@1.0.0 test"));
        assert!(content.contains("> node --test"));
    }

    // ============================================================
    // PNPM Test Fixture Tests
    // ============================================================

    #[test]
    fn test_load_pnpm_test_empty() {
        let content = pnpm_test_empty();
        assert!(content.is_empty());
    }

    #[test]
    fn test_load_pnpm_test_single_passed() {
        let content = pnpm_test_single_passed();
        assert!(content.contains("✔"));
        assert!(content.contains("1 passed"));
    }

    #[test]
    fn test_load_pnpm_test_single_failed() {
        let content = pnpm_test_single_failed();
        assert!(content.contains("✖"));
        assert!(content.contains("AssertionError"));
        assert!(content.contains("1 failed"));
    }

    #[test]
    fn test_load_pnpm_test_mixed() {
        let content = pnpm_test_mixed();
        assert!(content.contains("✔"));
        assert!(content.contains("✖"));
        assert!(content.contains("# SKIP"));
    }

    #[test]
    fn test_load_pnpm_test_all_passed() {
        let content = pnpm_test_all_passed();
        assert!(content.contains("✔"));
        assert!(content.contains("5 passed"));
    }

    #[test]
    fn test_load_pnpm_test_all_failed() {
        let content = pnpm_test_all_failed();
        assert!(content.contains("✖"));
        assert!(content.contains("2 failed"));
    }

    #[test]
    fn test_load_pnpm_test_with_skipped() {
        let content = pnpm_test_with_skipped();
        assert!(content.contains("# SKIP"));
        assert!(content.contains("2 passed"));
    }

    #[test]
    fn test_load_pnpm_test_with_todo() {
        let content = pnpm_test_with_todo();
        assert!(content.contains("# TODO"));
        assert!(content.contains("# SKIP"));
    }

    #[test]
    fn test_load_pnpm_test_large() {
        let content = pnpm_test_large();
        assert!(content.contains("test01.test.js"));
        assert!(content.contains("test10.test.js"));
        assert!(content.contains("20 passed"));
    }

    #[test]
    fn test_load_pnpm_test_with_header() {
        let content = pnpm_test_with_header();
        assert!(content.contains("pnpm: 9.0.0"));
    }

    // ============================================================
    // Bun Test Fixture Tests
    // ============================================================

    #[test]
    fn test_load_bun_test_empty() {
        let content = bun_test_empty();
        assert!(content.is_empty());
    }

    #[test]
    fn test_load_bun_test_single_passed() {
        let content = bun_test_single_passed();
        assert!(content.contains("✓"));
        assert!(content.contains("1 pass"));
    }

    #[test]
    fn test_load_bun_test_single_failed() {
        let content = bun_test_single_failed();
        assert!(content.contains("✗"));
        assert!(content.contains("AssertionError"));
        assert!(content.contains("1 fail"));
    }

    #[test]
    fn test_load_bun_test_mixed() {
        let content = bun_test_mixed();
        assert!(content.contains("✓"));
        assert!(content.contains("✗"));
        assert!(content.contains("3 pass"));
        assert!(content.contains("1 fail"));
    }

    #[test]
    fn test_load_bun_test_all_passed() {
        let content = bun_test_all_passed();
        assert!(content.contains("✓"));
        assert!(content.contains("5 pass"));
        assert!(content.contains("0 fail"));
    }

    #[test]
    fn test_load_bun_test_all_failed() {
        let content = bun_test_all_failed();
        assert!(content.contains("✗"));
        assert!(content.contains("0 pass"));
        assert!(content.contains("2 fail"));
    }

    #[test]
    fn test_load_bun_test_with_skipped() {
        let content = bun_test_with_skipped();
        assert!(content.contains("(skip)"));
        assert!(content.contains("2 skip"));
    }

    #[test]
    fn test_load_bun_test_with_todo() {
        let content = bun_test_with_todo();
        assert!(content.contains("(todo)"));
        assert!(content.contains("(skip)"));
        assert!(content.contains("1 todo"));
    }

    #[test]
    fn test_load_bun_test_multiple_suites() {
        let content = bun_test_multiple_suites();
        assert!(content.contains("Button.test.js"));
        assert!(content.contains("Input.test.js"));
        assert!(content.contains("Modal.test.js"));
        assert!(content.contains("7 pass"));
    }

    #[test]
    fn test_load_bun_test_large() {
        let content = bun_test_large();
        assert!(content.contains("test01.test.js"));
        assert!(content.contains("test10.test.js"));
        assert!(content.contains("20 pass"));
    }

    #[test]
    fn test_load_bun_test_non_tty() {
        let content = bun_test_non_tty();
        assert!(content.contains("(pass)"));
        assert!(content.contains("2 pass"));
    }

    // ============================================================
    // Logs Fixture Tests
    // ============================================================

    #[test]
    fn test_load_logs_empty() {
        let content = logs_empty();
        assert!(content.is_empty());
    }

    #[test]
    fn test_load_logs_simple() {
        let content = logs_simple();
        assert!(content.contains("[INFO]"));
        assert!(content.contains("[DEBUG]"));
        assert!(content.contains("[WARN]"));
        assert!(content.contains("[ERROR]"));
        assert!(content.contains("[FATAL]"));
    }

    #[test]
    fn test_load_logs_iso8601_timestamp() {
        let content = logs_iso8601_timestamp();
        assert!(content.contains("2024-01-15T10:30:00"));
        assert!(content.contains("[INFO]"));
        assert!(content.contains("[ERROR]"));
    }

    #[test]
    fn test_load_logs_syslog_format() {
        let content = logs_syslog_format();
        assert!(content.contains("Jan 15 10:30:00"));
        assert!(content.contains("server01"));
        assert!(content.contains("INFO"));
        assert!(content.contains("ERROR"));
        assert!(content.contains("CRITICAL"));
    }

    #[test]
    fn test_load_logs_all_levels() {
        let content = logs_all_levels();
        assert!(content.contains("[TRACE]"));
        assert!(content.contains("[DEBUG]"));
        assert!(content.contains("[VERBOSE]"));
        assert!(content.contains("[INFO]"));
        assert!(content.contains("[NOTICE]"));
        assert!(content.contains("[WARN]"));
        assert!(content.contains("[WARNING]"));
        assert!(content.contains("[ERROR]"));
        assert!(content.contains("[CRITICAL]"));
        assert!(content.contains("[FATAL]"));
        assert!(content.contains("[PANIC]"));
    }

    #[test]
    fn test_load_logs_errors_only() {
        let content = logs_errors_only();
        assert!(content.contains("[ERROR]"));
        assert!(content.contains("ERROR:"));
        assert!(content.contains("Exception"));
        assert!(content.contains("FAILED"));
        assert!(content.contains("STACK TRACE"));
        assert!(content.contains("Connection refused"));
        assert!(content.contains("ACCESS DENIED"));
    }

    #[test]
    fn test_load_logs_with_exceptions() {
        let content = logs_with_exceptions();
        assert!(content.contains("java.lang.NullPointerException"));
        assert!(content.contains("Traceback"));
        assert!(content.contains("ConnectionError"));
        assert!(content.contains("PANIC"));
        assert!(content.contains("Backtrace"));
    }

    #[test]
    fn test_load_logs_mixed_format() {
        let content = logs_mixed_format();
        assert!(content.contains("2024-01-15 10:30:00"));
        assert!(content.contains("2024/01/15"));
        assert!(content.contains("|INFO|"));
        assert!(content.contains("[WARN]"));
        assert!(content.contains("WARNING:"));
    }

    #[test]
    fn test_load_logs_repeated_lines() {
        let content = logs_repeated_lines();
        assert!(content.contains("[DEBUG] Processing request"));
        assert!(content.contains("[WARN] Cache miss"));
        assert!(content.contains("[ERROR] Connection failed"));
    }

    #[test]
    fn test_load_logs_large() {
        let content = logs_large();
        assert!(content.contains("[INFO] Application starting"));
        assert!(content.contains("[DEBUG] Initializing module"));
        assert!(content.contains("[ERROR] Database connection pool exhausted"));
        assert!(content.contains("[FATAL] Out of memory error"));
        assert!(content.contains("Shutdown complete"));
    }

    #[test]
    fn test_load_logs_json_format() {
        let content = logs_json_format();
        assert!(content.contains("\"level\": \"INFO\""));
        assert!(content.contains("\"level\": \"ERROR\""));
        assert!(content.contains("\"level\": \"FATAL\""));
        assert!(content.contains("\"message\""));
        assert!(content.contains("\"timestamp\""));
    }

    #[test]
    fn test_load_logs_with_source() {
        let content = logs_with_source();
        assert!(content.contains("[auth]"));
        assert!(content.contains("[database]"));
        assert!(content.contains("[api]"));
        assert!(content.contains("[cache]"));
        assert!(content.contains("[payment]"));
    }

    #[test]
    fn test_load_logs_application() {
        let content = logs_application();
        assert!(content.contains("Starting MyApp"));
        assert!(content.contains("Database connection established"));
        assert!(content.contains("Server ready"));
        assert!(content.contains("Graceful shutdown"));
    }

    #[test]
    fn test_load_logs_access() {
        let content = logs_access();
        assert!(content.contains("192.168.1.100"));
        assert!(content.contains("GET /api/health"));
        assert!(content.contains("HTTP/1.1"));
        assert!(content.contains("200"));
        assert!(content.contains("404"));
        assert!(content.contains("500"));
    }

    #[test]
    fn test_load_logs_colon_format() {
        let content = logs_colon_format();
        assert!(content.contains("DEBUG:"));
        assert!(content.contains("INFO:"));
        assert!(content.contains("WARNING:"));
        assert!(content.contains("ERROR:"));
        assert!(content.contains("CRITICAL:"));
        assert!(content.contains("FATAL:"));
    }

    #[test]
    fn test_load_logs_pipe_format() {
        let content = logs_pipe_format();
        assert!(content.contains("|DEBUG|"));
        assert!(content.contains("|INFO|"));
        assert!(content.contains("|WARN|"));
        assert!(content.contains("|ERROR|"));
        assert!(content.contains("|CRIT|"));
        assert!(content.contains("|FATAL|"));
    }

    #[test]
    fn test_load_logs_multiline_error() {
        let content = logs_multiline_error();
        assert!(content.contains("java.lang.IllegalArgumentException"));
        assert!(content.contains("Caused by:"));
        assert!(content.contains("Traceback"));
        assert!(content.contains("json.decoder.JSONDecodeError"));
    }
}
