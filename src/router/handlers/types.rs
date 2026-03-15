//! Shared data structures and types for command handlers.
//!
//! Contains all parsed output types for git, ls, find, grep, test runners,
//! and log parsers, plus the CommandHandler trait.

use std::collections::HashMap;
use super::common::{CommandContext, CommandResult};

// ============================================================
// Git Status Data Structures
// ============================================================

/// Section of git status output being parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitStatusSection {
    /// Not in any specific section.
    None,
    /// Staged changes section.
    Staged,
    /// Unstaged changes section.
    Unstaged,
    /// Untracked files section.
    Untracked,
    /// Unmerged paths section.
    Unmerged,
}

/// A single file entry in git status.
#[derive(Debug, Clone, Default)]
pub(crate) struct GitStatusEntry {
    /// Status code (e.g., "M", "A", "D", "??").
    pub(crate) status: String,
    /// Path to the file.
    pub(crate) path: String,
    /// Original path for renamed files.
    pub(crate) new_path: Option<String>,
}

/// Parsed git status output.
#[derive(Debug, Clone, Default)]
pub(crate) struct GitStatus {
    /// Current branch name.
    pub(crate) branch: String,
    /// Whether the working tree is clean.
    pub(crate) is_clean: bool,
    /// Number of commits ahead of upstream.
    pub(crate) ahead: Option<usize>,
    /// Number of commits behind upstream.
    pub(crate) behind: Option<usize>,
    /// Staged changes (to be committed).
    pub(crate) staged: Vec<GitStatusEntry>,
    /// Unstaged changes (not staged for commit).
    pub(crate) unstaged: Vec<GitStatusEntry>,
    /// Untracked files.
    pub(crate) untracked: Vec<GitStatusEntry>,
    /// Unmerged paths (merge conflicts).
    pub(crate) unmerged: Vec<GitStatusEntry>,
    /// Number of staged files.
    pub(crate) staged_count: usize,
    /// Number of unstaged files.
    pub(crate) unstaged_count: usize,
    /// Number of untracked files.
    pub(crate) untracked_count: usize,
    /// Number of unmerged files.
    pub(crate) unmerged_count: usize,
}

// ============================================================
// Git Diff Data Structures
// ============================================================

/// A single file entry in git diff output.
#[derive(Debug, Clone, Default)]
pub(crate) struct GitDiffEntry {
    /// Path to the file (new path for renamed files).
    pub(crate) path: String,
    /// Original path for renamed files.
    pub(crate) new_path: Option<String>,
    /// Change type (M=modified, A=added, D=deleted, R=renamed, C=copied).
    pub(crate) change_type: String,
    /// Number of lines added.
    pub(crate) additions: usize,
    /// Number of lines deleted.
    pub(crate) deletions: usize,
    /// Binary file flag.
    pub(crate) is_binary: bool,
}

/// Parsed git diff output.
#[derive(Debug, Clone, Default)]
pub(crate) struct GitDiff {
    /// List of file entries (limited if truncated).
    pub(crate) files: Vec<GitDiffEntry>,
    /// Total lines added across all files.
    pub(crate) total_additions: usize,
    /// Total lines deleted across all files.
    pub(crate) total_deletions: usize,
    /// Whether the diff is empty.
    pub(crate) is_empty: bool,
    /// Whether the output was truncated.
    pub(crate) is_truncated: bool,
    /// Total number of files available before truncation.
    pub(crate) total_files: usize,
    /// Number of files shown after truncation.
    pub(crate) files_shown: usize,
}

// ============================================================
// LS Data Structures
// ============================================================

/// Entry type for ls output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LsEntryType {
    /// Regular file.
    File,
    /// Directory.
    Directory,
    /// Symbolic link.
    Symlink,
    /// Block device.
    BlockDevice,
    /// Character device.
    CharDevice,
    /// Socket.
    Socket,
    /// Pipe (FIFO).
    Pipe,
    /// Unknown or other type.
    Other,
}

impl Default for LsEntryType {
    fn default() -> Self {
        LsEntryType::File
    }
}

/// A single entry in ls output.
#[derive(Debug, Clone, Default)]
pub(crate) struct LsEntry {
    /// Name of the file or directory.
    pub(crate) name: String,
    /// Type of entry (file, directory, etc.).
    pub(crate) entry_type: LsEntryType,
    /// Whether this is a hidden file (starts with .).
    pub(crate) is_hidden: bool,
    /// File size in bytes (if available).
    #[allow(dead_code)]
    pub(crate) size: Option<u64>,
    /// File permissions (if available).
    #[allow(dead_code)]
    pub(crate) permissions: Option<String>,
    /// Number of hard links (if available).
    pub(crate) links: Option<u64>,
    /// Owner user name (if available).
    pub(crate) owner: Option<String>,
    /// Owner group name (if available).
    pub(crate) group: Option<String>,
    /// Last modification time (if available).
    pub(crate) modified: Option<String>,
    /// Symlink target (if this is a symlink).
    pub(crate) symlink_target: Option<String>,
    /// Whether the symlink is broken (target doesn't exist).
    pub(crate) is_broken_symlink: bool,
}

// ============================================================
// Find Data Structures
// ============================================================

/// A single entry in find output.
#[derive(Debug, Clone, Default)]
pub(crate) struct FindEntry {
    /// Path to the file or directory.
    pub(crate) path: String,
    /// Whether this is a directory.
    pub(crate) is_directory: bool,
    /// Whether this is a hidden file/directory.
    pub(crate) is_hidden: bool,
    /// File extension (if available).
    pub(crate) extension: Option<String>,
    /// Depth of the path (number of path separators).
    pub(crate) depth: usize,
}

/// A permission denied or error entry from find output.
#[derive(Debug, Clone, Default)]
pub(crate) struct FindError {
    /// The path that was denied access.
    pub(crate) path: String,
    /// The error message.
    pub(crate) message: String,
}

/// Parsed find output.
#[derive(Debug, Clone, Default)]
pub(crate) struct FindOutput {
    /// List of all entries.
    pub(crate) entries: Vec<FindEntry>,
    /// Directory paths.
    pub(crate) directories: Vec<String>,
    /// File paths.
    pub(crate) files: Vec<String>,
    /// Hidden entries.
    pub(crate) hidden: Vec<String>,
    /// File extensions with counts.
    pub(crate) extensions: HashMap<String, usize>,
    /// Permission denied or error entries.
    pub(crate) errors: Vec<FindError>,
    /// Total count of entries (excluding errors).
    pub(crate) total_count: usize,
    /// Whether the output is empty.
    pub(crate) is_empty: bool,
}

// ============================================================
// Grep Data Structures
// ============================================================

/// A single match in grep output.
#[derive(Debug, Clone, Default)]
pub(crate) struct GrepMatch {
    /// Line number (if available with -n flag).
    pub(crate) line_number: Option<usize>,
    /// Column number (if available with --column flag).
    pub(crate) column: Option<usize>,
    /// The matched line content.
    pub(crate) line: String,
    /// Whether this is a context line (not a direct match).
    pub(crate) is_context: bool,
    /// Short excerpt of the matched text.
    pub(crate) excerpt: Option<String>,
}

/// A file with grep matches.
#[derive(Debug, Clone, Default)]
pub(crate) struct GrepFile {
    /// Path to the file.
    pub(crate) path: String,
    /// List of matches in this file.
    pub(crate) matches: Vec<GrepMatch>,
}

/// Parsed grep output.
#[derive(Debug, Clone, Default)]
pub(crate) struct GrepOutput {
    /// List of files with matches (limited if truncated).
    pub(crate) files: Vec<GrepFile>,
    /// Total number of files with matches.
    pub(crate) file_count: usize,
    /// Total number of matches across all files.
    pub(crate) match_count: usize,
    /// Whether the output is empty (no matches).
    pub(crate) is_empty: bool,
    /// Whether the output was truncated.
    pub(crate) is_truncated: bool,
    /// Total number of files available before truncation.
    pub(crate) total_files: usize,
    /// Total number of matches available before truncation.
    pub(crate) total_matches: usize,
    /// Number of files shown after truncation.
    pub(crate) files_shown: usize,
    /// Number of matches shown after truncation.
    pub(crate) matches_shown: usize,
    /// Total bytes of all matched lines (original output size).
    pub(crate) input_bytes: usize,
}

// ============================================================
// Test Output Data Structures (pytest)
// ============================================================

/// Status of a single test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TestStatus {
    /// Test passed.
    Passed,
    /// Test failed.
    Failed,
    /// Test was skipped.
    Skipped,
    /// Test expected to fail (xfail).
    XFailed,
    /// Test expected to fail but passed (xpass).
    XPassed,
    /// Test encountered an error.
    Error,
}

/// A single test result.
#[derive(Debug, Clone)]
pub(crate) struct TestResult {
    /// Full test name (module::test_name or file::test_name).
    pub(crate) name: String,
    /// Status of the test.
    pub(crate) status: TestStatus,
    /// Duration in seconds (if available).
    pub(crate) duration: Option<f64>,
    /// File path (if available).
    pub(crate) file: Option<String>,
    /// Line number (if available).
    pub(crate) line: Option<usize>,
    /// Error message (for failed tests).
    pub(crate) error_message: Option<String>,
}

/// Summary of test results.
#[derive(Debug, Clone, Default)]
pub(crate) struct TestSummary {
    /// Number of passed tests.
    pub(crate) passed: usize,
    /// Number of failed tests.
    pub(crate) failed: usize,
    /// Number of skipped tests.
    pub(crate) skipped: usize,
    /// Number of xfailed tests.
    pub(crate) xfailed: usize,
    /// Number of xpassed tests.
    pub(crate) xpassed: usize,
    /// Number of error tests.
    pub(crate) errors: usize,
    /// Total number of tests.
    pub(crate) total: usize,
    /// Total duration in seconds.
    pub(crate) duration: Option<f64>,
}

/// Parsed pytest output.
#[derive(Debug, Clone, Default)]
pub(crate) struct PytestOutput {
    /// List of test results.
    pub(crate) tests: Vec<TestResult>,
    /// Summary statistics.
    pub(crate) summary: TestSummary,
    /// Whether all tests passed.
    pub(crate) success: bool,
    /// Whether the output is empty.
    pub(crate) is_empty: bool,
    /// Collection rootdir (if available).
    pub(crate) rootdir: Option<String>,
    /// Platform info (if available).
    pub(crate) platform: Option<String>,
    /// Python version (if available).
    pub(crate) python_version: Option<String>,
    /// Pytest version (if available).
    pub(crate) pytest_version: Option<String>,
}

/// Parsed Jest output.
#[derive(Debug, Clone, Default)]
pub(crate) struct JestOutput {
    /// List of test suites.
    pub(crate) test_suites: Vec<JestTestSuite>,
    /// Summary statistics.
    pub(crate) summary: JestSummary,
    /// Whether all tests passed.
    pub(crate) success: bool,
    /// Whether the output is empty.
    pub(crate) is_empty: bool,
    /// Jest version (if available).
    pub(crate) jest_version: Option<String>,
    /// Test path pattern (if available).
    pub(crate) test_path_pattern: Option<String>,
}

/// A Jest test suite result.
#[derive(Debug, Clone)]
pub(crate) struct JestTestSuite {
    /// Test file path.
    pub(crate) file: String,
    /// Whether the suite passed.
    pub(crate) passed: bool,
    /// Execution time in seconds.
    pub(crate) duration: Option<f64>,
    /// List of test results in this suite.
    pub(crate) tests: Vec<JestTest>,
}

/// A single Jest test result.
#[derive(Debug, Clone)]
pub(crate) struct JestTest {
    /// Full test name (ancestor titles + test name).
    pub(crate) name: String,
    /// Test name only.
    pub(crate) test_name: String,
    /// Ancestor titles (describe blocks).
    pub(crate) ancestors: Vec<String>,
    /// Status of the test.
    pub(crate) status: JestTestStatus,
    /// Duration in seconds (if available).
    pub(crate) duration: Option<f64>,
    /// Error message (for failed tests).
    pub(crate) error_message: Option<String>,
}

/// Status of a Jest test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JestTestStatus {
    /// Test passed.
    Passed,
    /// Test failed.
    Failed,
    /// Test was skipped.
    Skipped,
    /// Test was todo.
    Todo,
}

/// Jest summary statistics.
#[derive(Debug, Clone, Default)]
pub(crate) struct JestSummary {
    /// Number of test suites passed.
    pub(crate) suites_passed: usize,
    /// Number of test suites failed.
    pub(crate) suites_failed: usize,
    /// Number of test suites total.
    pub(crate) suites_total: usize,
    /// Number of tests passed.
    pub(crate) tests_passed: usize,
    /// Number of tests failed.
    pub(crate) tests_failed: usize,
    /// Number of tests skipped.
    pub(crate) tests_skipped: usize,
    /// Number of tests todo.
    pub(crate) tests_todo: usize,
    /// Number of tests total.
    pub(crate) tests_total: usize,
    /// Number of snapshots updated/added/removed.
    pub(crate) snapshots: Option<usize>,
    /// Total duration in seconds.
    pub(crate) duration: Option<f64>,
}

/// Parsed Vitest output.
#[derive(Debug, Clone, Default)]
pub(crate) struct VitestOutput {
    /// List of test suites.
    pub(crate) test_suites: Vec<VitestTestSuite>,
    /// Summary statistics.
    pub(crate) summary: VitestSummary,
    /// Whether all tests passed.
    pub(crate) success: bool,
    /// Whether the output is empty.
    pub(crate) is_empty: bool,
    /// Vitest version (if available).
    pub(crate) vitest_version: Option<String>,
}

/// A Vitest test suite result.
#[derive(Debug, Clone)]
pub(crate) struct VitestTestSuite {
    /// Test file path.
    pub(crate) file: String,
    /// Whether the suite passed.
    pub(crate) passed: bool,
    /// Execution time in seconds.
    pub(crate) duration: Option<f64>,
    /// Number of tests in suite.
    pub(crate) test_count: Option<usize>,
    /// Number of skipped tests in suite.
    pub(crate) skipped_count: Option<usize>,
    /// List of test results in this suite.
    pub(crate) tests: Vec<VitestTest>,
}

/// A single Vitest test result.
#[derive(Debug, Clone)]
pub(crate) struct VitestTest {
    /// Full test name (ancestor titles + test name).
    pub(crate) name: String,
    /// Test name only.
    pub(crate) test_name: String,
    /// Ancestor titles (describe blocks).
    pub(crate) ancestors: Vec<String>,
    /// Status of the test.
    pub(crate) status: VitestTestStatus,
    /// Duration in seconds (if available).
    pub(crate) duration: Option<f64>,
    /// Error message (for failed tests).
    pub(crate) error_message: Option<String>,
}

/// Status of a Vitest test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VitestTestStatus {
    /// Test passed.
    Passed,
    /// Test failed.
    Failed,
    /// Test was skipped.
    Skipped,
    /// Test was todo.
    Todo,
}

/// Vitest summary statistics.
#[derive(Debug, Clone, Default)]
pub(crate) struct VitestSummary {
    /// Number of test suites passed.
    pub(crate) suites_passed: usize,
    /// Number of test suites failed.
    pub(crate) suites_failed: usize,
    /// Number of test suites total.
    pub(crate) suites_total: usize,
    /// Number of tests passed.
    pub(crate) tests_passed: usize,
    /// Number of tests failed.
    pub(crate) tests_failed: usize,
    /// Number of tests skipped.
    pub(crate) tests_skipped: usize,
    /// Number of tests todo.
    pub(crate) tests_todo: usize,
    /// Number of tests total.
    pub(crate) tests_total: usize,
    /// Total duration in seconds.
    pub(crate) duration: Option<f64>,
    /// Start time (if available).
    pub(crate) start_at: Option<String>,
}

/// Helper struct for parsing vitest suite headers.
pub(crate) struct VitestSuiteInfo {
    pub(crate) file: String,
    pub(crate) passed: bool,
    pub(crate) duration: Option<f64>,
    pub(crate) test_count: Option<usize>,
    pub(crate) skipped_count: Option<usize>,
}

// ============================================================
// NPM Test (Node.js built-in test runner) Parser
// ============================================================

/// Parsed npm test output (Node.js built-in test runner with spec reporter).
#[derive(Debug, Clone, Default)]
pub(crate) struct NpmTestOutput {
    /// List of test suites (files).
    pub(crate) test_suites: Vec<NpmTestSuite>,
    /// Summary statistics.
    pub(crate) summary: NpmTestSummary,
    /// Whether all tests passed.
    pub(crate) success: bool,
    /// Whether the output is empty.
    pub(crate) is_empty: bool,
    /// Node.js version (if available).
    pub(crate) node_version: Option<String>,
}

/// A npm test suite result (a test file).
#[derive(Debug, Clone)]
pub(crate) struct NpmTestSuite {
    /// Test file path.
    pub(crate) file: String,
    /// Whether the suite passed.
    pub(crate) passed: bool,
    /// Execution time in seconds.
    pub(crate) duration: Option<f64>,
    /// List of test results in this suite.
    pub(crate) tests: Vec<NpmTest>,
}

/// A single npm test result.
#[derive(Debug, Clone)]
pub(crate) struct NpmTest {
    /// Full test name (including nested structure).
    pub(crate) name: String,
    /// Test name only (last part).
    pub(crate) test_name: String,
    /// Ancestor names (describe/nested test blocks).
    pub(crate) ancestors: Vec<String>,
    /// Status of the test.
    pub(crate) status: NpmTestStatus,
    /// Duration in seconds (if available).
    pub(crate) duration: Option<f64>,
    /// Error message (for failed tests).
    pub(crate) error_message: Option<String>,
}

/// Status of a npm test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NpmTestStatus {
    /// Test passed.
    Passed,
    /// Test failed.
    Failed,
    /// Test was skipped.
    Skipped,
    /// Test was todo.
    Todo,
}

/// npm test summary statistics.
#[derive(Debug, Clone, Default)]
pub(crate) struct NpmTestSummary {
    /// Number of test suites passed.
    pub(crate) suites_passed: usize,
    /// Number of test suites failed.
    pub(crate) suites_failed: usize,
    /// Number of test suites skipped.
    pub(crate) suites_skipped: usize,
    /// Number of test suites total.
    pub(crate) suites_total: usize,
    /// Number of tests passed.
    pub(crate) tests_passed: usize,
    /// Number of tests failed.
    pub(crate) tests_failed: usize,
    /// Number of tests skipped.
    pub(crate) tests_skipped: usize,
    /// Number of tests todo.
    pub(crate) tests_todo: usize,
    /// Number of tests total.
    pub(crate) tests_total: usize,
    /// Total duration in seconds.
    pub(crate) duration: Option<f64>,
}

// ============================================================
// PNPM Test (Node.js built-in test runner via pnpm) Parser
// ============================================================

/// Parsed pnpm test output (Node.js built-in test runner with spec reporter).
/// pnpm test uses the same output format as npm test since it runs the same
/// test runner defined in package.json scripts.test.
#[derive(Debug, Clone, Default)]
pub(crate) struct PnpmTestOutput {
    /// List of test suites (files).
    pub(crate) test_suites: Vec<PnpmTestSuite>,
    /// Summary statistics.
    pub(crate) summary: PnpmTestSummary,
    /// Whether all tests passed.
    pub(crate) success: bool,
    /// Whether the output is empty.
    pub(crate) is_empty: bool,
    /// pnpm version (if available).
    pub(crate) pnpm_version: Option<String>,
}

/// A pnpm test suite result (a test file).
#[derive(Debug, Clone)]
pub(crate) struct PnpmTestSuite {
    /// Test file path.
    pub(crate) file: String,
    /// Whether the suite passed.
    pub(crate) passed: bool,
    /// Execution time in seconds.
    pub(crate) duration: Option<f64>,
    /// List of test results in this suite.
    pub(crate) tests: Vec<PnpmTest>,
}

/// A single pnpm test result.
#[derive(Debug, Clone)]
pub(crate) struct PnpmTest {
    /// Full test name (including nested structure).
    pub(crate) name: String,
    /// Test name only (last part).
    pub(crate) test_name: String,
    /// Ancestor names (describe/nested test blocks).
    pub(crate) ancestors: Vec<String>,
    /// Status of the test.
    pub(crate) status: PnpmTestStatus,
    /// Duration in seconds (if available).
    pub(crate) duration: Option<f64>,
    /// Error message (for failed tests).
    pub(crate) error_message: Option<String>,
}

/// Status of a pnpm test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PnpmTestStatus {
    /// Test passed.
    Passed,
    /// Test failed.
    Failed,
    /// Test was skipped.
    Skipped,
    /// Test was todo.
    Todo,
}

/// pnpm test summary statistics.
#[derive(Debug, Clone, Default)]
pub(crate) struct PnpmTestSummary {
    /// Number of test suites passed.
    pub(crate) suites_passed: usize,
    /// Number of test suites failed.
    pub(crate) suites_failed: usize,
    /// Number of test suites skipped.
    pub(crate) suites_skipped: usize,
    /// Number of test suites total.
    pub(crate) suites_total: usize,
    /// Number of tests passed.
    pub(crate) tests_passed: usize,
    /// Number of tests failed.
    pub(crate) tests_failed: usize,
    /// Number of tests skipped.
    pub(crate) tests_skipped: usize,
    /// Number of tests todo.
    pub(crate) tests_todo: usize,
    /// Number of tests total.
    pub(crate) tests_total: usize,
    /// Total duration in seconds.
    pub(crate) duration: Option<f64>,
}

// ============================================================
// Bun Test Parser Data Structures
// ============================================================

/// Parsed Bun test output.
///
/// Expected format (default console reporter):
/// ```text
/// test/package-json-lint.test.ts:
/// ✓ test/package.json [0.88ms]
/// ✓ test/js/third_party/grpc-js/package.json [0.18ms]
///
///  4 pass
///  0 fail
///  4 expect() calls
/// Ran 4 tests in 1.44ms
/// ```
///
/// For non-TTY environments (no colors):
/// ```text
/// test/package-json-lint.test.ts:
/// (pass) test/package.json [0.48ms]
/// (fail) test/failing.test.ts
/// (skip) test/skipped.test.ts
/// ```
#[derive(Debug, Clone, Default)]
pub(crate) struct BunTestOutput {
    /// List of test suites (files).
    pub(crate) test_suites: Vec<BunTestSuite>,
    /// Summary statistics.
    pub(crate) summary: BunTestSummary,
    /// Whether all tests passed.
    pub(crate) success: bool,
    /// Whether the output is empty.
    pub(crate) is_empty: bool,
    /// Bun version (if available).
    pub(crate) bun_version: Option<String>,
}

/// A Bun test suite result (a test file).
#[derive(Debug, Clone)]
pub(crate) struct BunTestSuite {
    /// Test file path.
    pub(crate) file: String,
    /// Whether the suite passed.
    pub(crate) passed: bool,
    /// Execution time in seconds.
    pub(crate) duration: Option<f64>,
    /// List of test results in this suite.
    pub(crate) tests: Vec<BunTest>,
}

/// A single Bun test result.
#[derive(Debug, Clone)]
pub(crate) struct BunTest {
    /// Full test name (including nested structure).
    pub(crate) name: String,
    /// Test name only (last part).
    pub(crate) test_name: String,
    /// Ancestor names (describe/nested test blocks).
    pub(crate) ancestors: Vec<String>,
    /// Status of the test.
    pub(crate) status: BunTestStatus,
    /// Duration in seconds (if available).
    pub(crate) duration: Option<f64>,
    /// Error message (for failed tests).
    pub(crate) error_message: Option<String>,
}

/// Status of a Bun test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BunTestStatus {
    /// Test passed.
    Passed,
    /// Test failed.
    Failed,
    /// Test was skipped.
    Skipped,
    /// Test was todo.
    Todo,
}

/// Bun test summary statistics.
#[derive(Debug, Clone, Default)]
pub(crate) struct BunTestSummary {
    /// Number of test suites passed.
    pub(crate) suites_passed: usize,
    /// Number of test suites failed.
    pub(crate) suites_failed: usize,
    /// Number of test suites skipped.
    pub(crate) suites_skipped: usize,
    /// Number of test suites total.
    pub(crate) suites_total: usize,
    /// Number of tests passed.
    pub(crate) tests_passed: usize,
    /// Number of tests failed.
    pub(crate) tests_failed: usize,
    /// Number of tests skipped.
    pub(crate) tests_skipped: usize,
    /// Number of tests todo.
    pub(crate) tests_todo: usize,
    /// Number of tests total.
    pub(crate) tests_total: usize,
    /// Number of expect() calls.
    pub(crate) expect_calls: Option<usize>,
    /// Total duration in seconds.
    pub(crate) duration: Option<f64>,
}

// ============================================================
// Log Stream Parser
// ============================================================

/// Log level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LogLevel {
    /// Debug level.
    Debug,
    /// Info level.
    Info,
    /// Warning level.
    Warning,
    /// Error level.
    Error,
    /// Fatal/Critical level.
    Fatal,
    /// Unknown or unclassified level.
    Unknown,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Unknown
    }
}

/// A single parsed log line.
#[derive(Debug, Clone, Default)]
pub(crate) struct LogEntry {
    /// Original line content.
    pub(crate) line: String,
    /// Detected log level.
    pub(crate) level: LogLevel,
    /// Timestamp (if detected).
    pub(crate) timestamp: Option<String>,
    /// Source/logger name (if detected).
    #[allow(dead_code)]
    pub(crate) source: Option<String>,
    /// Message content (without timestamp/level prefix).
    pub(crate) message: String,
    /// Line number in the input.
    pub(crate) line_number: usize,
}

/// Statistics for repeated lines.
#[derive(Debug, Clone)]
pub(crate) struct RepeatedLine {
    /// The repeated line content.
    pub(crate) line: String,
    /// Number of occurrences.
    pub(crate) count: usize,
    /// First occurrence line number.
    pub(crate) first_line: usize,
    /// Last occurrence line number.
    pub(crate) last_line: usize,
}

/// Maximum number of recent critical (ERROR/FATAL) lines to track.
pub(crate) const MAX_RECENT_CRITICAL: usize = 10;

/// Parsed log output.
#[derive(Debug, Clone, Default)]
pub(crate) struct LogsOutput {
    /// All log entries.
    pub(crate) entries: Vec<LogEntry>,
    /// Total line count.
    pub(crate) total_lines: usize,
    /// Count by level.
    pub(crate) debug_count: usize,
    pub(crate) info_count: usize,
    pub(crate) warning_count: usize,
    pub(crate) error_count: usize,
    pub(crate) fatal_count: usize,
    pub(crate) unknown_count: usize,
    /// Repeated lines (collapsed).
    pub(crate) repeated_lines: Vec<RepeatedLine>,
    /// Most recent critical lines (ERROR and FATAL level entries).
    pub(crate) recent_critical: Vec<LogEntry>,
    /// Whether the output is empty.
    pub(crate) is_empty: bool,
}

/// Common generated directory names that are typically build artifacts or dependencies.
pub(crate) const COMMON_GENERATED_DIRS: &[&str] = &[
    // JavaScript/TypeScript
    "node_modules",
    "dist",
    "build",
    "out",
    ".next",
    ".nuxt",
    ".output",
    // Python
    "__pycache__",
    ".venv",
    "venv",
    "env",
    ".tox",
    ".nox",
    "htmlcov",
    ".eggs",
    "eggs",
    "sdist",
    "wheelhouse",
    // Rust
    "target",
    // Java/Kotlin
    "target", // Maven
    "build",  // Gradle
    "out",    // IntelliJ
    ".gradle",
    // Go
    "vendor",
    // Ruby
    "vendor",
    ".bundle",
    // PHP
    "vendor",
    // .NET/C#
    "bin",
    "obj",
    // Swift/Objective-C
    "DerivedData",
    "Pods",
    ".build",
    // Elixir/Erlang
    "_build",
    "deps",
    // Haskell
    "dist-newstyle",
    ".stack-work",
    // Scala
    ".bloop",
    ".metals",
    // Docker
    ".docker",
    // Cache directories
    ".cache",
    ".npm",
    ".yarn",
    ".pnpm-store",
    // IDE/Editor
    ".idea",
    ".vscode",
    ".vs",
    // Misc
    "tmp",
    "temp",
];

/// Check if a directory name is a common generated directory.
pub(crate) fn is_generated_directory(name: &str) -> bool {
    // Strip trailing slash if present (common in ls output)
    let name = name.strip_suffix('/').unwrap_or(name);
    let name_lower = name.to_lowercase();
    COMMON_GENERATED_DIRS.contains(&name_lower.as_str())
}

/// A permission denied or error entry.
#[derive(Debug, Clone, Default)]
pub(crate) struct LsError {
    /// The path that was denied access.
    pub(crate) path: String,
    /// The error message.
    pub(crate) message: String,
}

/// Parsed ls output.
#[derive(Debug, Clone, Default)]
pub(crate) struct LsOutput {
    /// List of all entries.
    pub(crate) entries: Vec<LsEntry>,
    /// Directory entries.
    pub(crate) directories: Vec<LsEntry>,
    /// File entries.
    pub(crate) files: Vec<LsEntry>,
    /// Symlink entries.
    pub(crate) symlinks: Vec<LsEntry>,
    /// Hidden entries.
    pub(crate) hidden: Vec<LsEntry>,
    /// Generated directory entries (build artifacts, dependencies, etc.).
    pub(crate) generated: Vec<LsEntry>,
    /// Permission denied or error entries.
    pub(crate) errors: Vec<LsError>,
    /// Total count of entries (excluding errors).
    pub(crate) total_count: usize,
    /// Whether the output is empty.
    pub(crate) is_empty: bool,
}

// ============================================================
// CommandHandler Trait
// ============================================================

/// Trait for command handlers that parse and reduce command output.
pub trait CommandHandler {
    type Input;
    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult;
}
