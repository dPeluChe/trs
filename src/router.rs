//! Command routing system for TARS CLI.
//!
//! This module provides a modular routing system that dispatches CLI commands
//! to their respective handlers. Each command has a dedicated handler that
//! implements the `CommandHandler` trait.

use crate::process::{ProcessBuilder, ProcessError, ProcessOutput};
use crate::{Cli, Commands, OutputFormat, ParseCommands};

/// Strip ANSI escape codes from a string.
///
/// This function handles all common ANSI escape sequence types:
/// - CSI (Control Sequence Introducer): ESC [ ... <final byte>
/// - OSC (Operating System Command): ESC ] ... (BEL or ST)
/// - Simple escape sequences: ESC followed by a single character
/// - Other sequences: ESC (, ESC ), ESC #, etc.
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\x1b' {
            // Skip the escape character
            i += 1;

            if i >= chars.len() {
                break;
            }

            match chars[i] {
                // CSI (Control Sequence Introducer): ESC [ ... <final byte>
                '[' => {
                    i += 1;
                    // Skip parameter and intermediate bytes until we reach a final byte
                    // Final bytes are in range 0x40-0x7E (@A-Z[\]^_`a-z{|}~)
                    while i < chars.len() && !(chars[i] >= '@' && chars[i] <= '~') {
                        i += 1;
                    }
                    if i < chars.len() {
                        i += 1; // Skip the final byte
                    }
                }
                // OSC (Operating System Command): ESC ] ... (BEL or ST)
                ']' => {
                    i += 1;
                    // Skip until we find BEL (0x07) or ST (ESC \)
                    while i < chars.len() {
                        if chars[i] == '\x07' {
                            // Found BEL, skip it and continue
                            i += 1;
                            break;
                        } else if chars[i] == '\x1b' {
                            // Possible ST sequence (ESC \)
                            i += 1;
                            if i < chars.len() && chars[i] == '\\' {
                                i += 1;
                                break;
                            }
                        } else {
                            i += 1;
                        }
                    }
                }
                // Character set selection: ESC (, ESC ), ESC *, ESC + followed by a char
                '(' | ')' | '*' | '+' | '-' | '.' | '/' => {
                    i += 1;
                    // Skip the character set identifier
                    if i < chars.len() {
                        i += 1;
                    }
                }
                // Simple two-character escape sequences and other Fe sequences
                // These include: ESC c (RIS), ESC D (IND), ESC E (NEL), ESC H (HTS), etc.
                _ => {
                    // Skip the character after ESC (it's part of the escape sequence)
                    i += 1;
                }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Sanitize input string by handling control characters.
///
/// This function:
/// - Removes null bytes (0x00)
/// - Replaces other control characters (except newlines and tabs) with spaces
/// - Normalizes multiple consecutive spaces to single space
/// - Preserves valid Unicode characters
fn sanitize_control_chars(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_was_space = false;

    for c in s.chars() {
        match c {
            // Remove null bytes entirely
            '\x00' => continue,
            // Preserve newlines and tabs
            '\n' | '\t' | '\r' => {
                result.push(c);
                prev_was_space = false;
            }
            // Replace other ASCII control characters with space
            c if c.is_ascii_control() => {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            }
            // Keep all other characters (including Unicode)
            c => {
                result.push(c);
                prev_was_space = false;
            }
        }
    }

    result
}

/// Context passed to command handlers containing global CLI options.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// The output format to use for the command result.
    pub format: OutputFormat,
    /// Whether to show execution statistics.
    pub stats: bool,
    /// List of enabled format flags (for warnings/debugging).
    pub enabled_formats: Vec<OutputFormat>,
}

impl CommandContext {
    /// Create a new command context from CLI options.
    pub fn from_cli(cli: &Cli) -> Self {
        Self {
            format: cli.output_format(),
            stats: cli.stats,
            enabled_formats: cli.enabled_format_flags(),
        }
    }

    /// Returns true if multiple format flags were specified.
    pub fn has_conflicting_formats(&self) -> bool {
        self.enabled_formats.len() > 1
    }
}

/// Result type for command handlers.
pub type CommandResult<T = ()> = Result<T, CommandError>;

/// Error type for command execution.
#[derive(Debug, Clone)]
pub enum CommandError {
    /// The command is not yet implemented.
    NotImplemented(String),
    /// An error occurred during execution with an optional exit code.
    ExecutionError {
        message: String,
        exit_code: Option<i32>,
    },
    /// Invalid arguments provided.
    InvalidArguments(String),
    /// I/O error occurred.
    IoError(String),
}

impl CommandError {
    /// Returns the exit code if this error is associated with a non-zero exit.
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            CommandError::ExecutionError { exit_code, .. } => *exit_code,
            _ => None,
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            CommandError::ExecutionError { message, .. } => {
                write!(f, "Execution error: {}", message)
            }
            CommandError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            CommandError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

// ============================================================
// Git Status Data Structures
// ============================================================

/// Section of git status output being parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GitStatusSection {
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
struct GitStatusEntry {
    /// Status code (e.g., "M", "A", "D", "??").
    status: String,
    /// Path to the file.
    path: String,
    /// Original path for renamed files.
    new_path: Option<String>,
}

/// Parsed git status output.
#[derive(Debug, Clone, Default)]
struct GitStatus {
    /// Current branch name.
    branch: String,
    /// Whether the working tree is clean.
    is_clean: bool,
    /// Number of commits ahead of upstream.
    ahead: Option<usize>,
    /// Number of commits behind upstream.
    behind: Option<usize>,
    /// Staged changes (to be committed).
    staged: Vec<GitStatusEntry>,
    /// Unstaged changes (not staged for commit).
    unstaged: Vec<GitStatusEntry>,
    /// Untracked files.
    untracked: Vec<GitStatusEntry>,
    /// Unmerged paths (merge conflicts).
    unmerged: Vec<GitStatusEntry>,
    /// Number of staged files.
    staged_count: usize,
    /// Number of unstaged files.
    unstaged_count: usize,
    /// Number of untracked files.
    untracked_count: usize,
    /// Number of unmerged files.
    unmerged_count: usize,
}

// ============================================================
// Git Diff Data Structures
// ============================================================

/// A single file entry in git diff output.
#[derive(Debug, Clone, Default)]
struct GitDiffEntry {
    /// Path to the file (new path for renamed files).
    path: String,
    /// Original path for renamed files.
    new_path: Option<String>,
    /// Change type (M=modified, A=added, D=deleted, R=renamed, C=copied).
    change_type: String,
    /// Number of lines added.
    additions: usize,
    /// Number of lines deleted.
    deletions: usize,
    /// Binary file flag.
    is_binary: bool,
}

/// Parsed git diff output.
#[derive(Debug, Clone, Default)]
struct GitDiff {
    /// List of file entries (limited if truncated).
    files: Vec<GitDiffEntry>,
    /// Total lines added across all files.
    total_additions: usize,
    /// Total lines deleted across all files.
    total_deletions: usize,
    /// Whether the diff is empty.
    is_empty: bool,
    /// Whether the output was truncated.
    is_truncated: bool,
    /// Total number of files available before truncation.
    total_files: usize,
    /// Number of files shown after truncation.
    files_shown: usize,
}

// ============================================================
// LS Data Structures
// ============================================================

/// Entry type for ls output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LsEntryType {
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
struct LsEntry {
    /// Name of the file or directory.
    name: String,
    /// Type of entry (file, directory, etc.).
    entry_type: LsEntryType,
    /// Whether this is a hidden file (starts with .).
    is_hidden: bool,
    /// File size in bytes (if available).
    size: Option<u64>,
    /// File permissions (if available).
    permissions: Option<String>,
    /// Number of hard links (if available).
    links: Option<u64>,
    /// Owner user name (if available).
    owner: Option<String>,
    /// Owner group name (if available).
    group: Option<String>,
    /// Last modification time (if available).
    modified: Option<String>,
    /// Symlink target (if this is a symlink).
    symlink_target: Option<String>,
    /// Whether the symlink is broken (target doesn't exist).
    is_broken_symlink: bool,
}

// ============================================================
// Find Data Structures
// ============================================================

/// A single entry in find output.
#[derive(Debug, Clone, Default)]
struct FindEntry {
    /// Path to the file or directory.
    path: String,
    /// Whether this is a directory.
    is_directory: bool,
    /// Whether this is a hidden file/directory.
    is_hidden: bool,
    /// File extension (if available).
    extension: Option<String>,
    /// Depth of the path (number of path separators).
    depth: usize,
}

/// A permission denied or error entry from find output.
#[derive(Debug, Clone, Default)]
struct FindError {
    /// The path that was denied access.
    path: String,
    /// The error message.
    message: String,
}

/// Parsed find output.
#[derive(Debug, Clone, Default)]
struct FindOutput {
    /// List of all entries.
    entries: Vec<FindEntry>,
    /// Directory paths.
    directories: Vec<String>,
    /// File paths.
    files: Vec<String>,
    /// Hidden entries.
    hidden: Vec<String>,
    /// File extensions with counts.
    extensions: std::collections::HashMap<String, usize>,
    /// Permission denied or error entries.
    errors: Vec<FindError>,
    /// Total count of entries (excluding errors).
    total_count: usize,
    /// Whether the output is empty.
    is_empty: bool,
}

// ============================================================
// Grep Data Structures
// ============================================================

/// A single match in grep output.
#[derive(Debug, Clone, Default)]
struct GrepMatch {
    /// Line number (if available with -n flag).
    line_number: Option<usize>,
    /// Column number (if available with --column flag).
    column: Option<usize>,
    /// The matched line content.
    line: String,
    /// Whether this is a context line (not a direct match).
    is_context: bool,
    /// Short excerpt of the matched text.
    excerpt: Option<String>,
}

/// A file with grep matches.
#[derive(Debug, Clone, Default)]
struct GrepFile {
    /// Path to the file.
    path: String,
    /// List of matches in this file.
    matches: Vec<GrepMatch>,
}

/// Parsed grep output.
#[derive(Debug, Clone, Default)]
struct GrepOutput {
    /// List of files with matches (limited if truncated).
    files: Vec<GrepFile>,
    /// Total number of files with matches.
    file_count: usize,
    /// Total number of matches across all files.
    match_count: usize,
    /// Whether the output is empty (no matches).
    is_empty: bool,
    /// Whether the output was truncated.
    is_truncated: bool,
    /// Total number of files available before truncation.
    total_files: usize,
    /// Total number of matches available before truncation.
    total_matches: usize,
    /// Number of files shown after truncation.
    files_shown: usize,
    /// Number of matches shown after truncation.
    matches_shown: usize,
}

// ============================================================
// Test Output Data Structures (pytest)
// ============================================================

/// Status of a single test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestStatus {
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
struct TestResult {
    /// Full test name (module::test_name or file::test_name).
    name: String,
    /// Status of the test.
    status: TestStatus,
    /// Duration in seconds (if available).
    duration: Option<f64>,
    /// File path (if available).
    file: Option<String>,
    /// Line number (if available).
    line: Option<usize>,
    /// Error message (for failed tests).
    error_message: Option<String>,
}

/// Summary of test results.
#[derive(Debug, Clone, Default)]
struct TestSummary {
    /// Number of passed tests.
    passed: usize,
    /// Number of failed tests.
    failed: usize,
    /// Number of skipped tests.
    skipped: usize,
    /// Number of xfailed tests.
    xfailed: usize,
    /// Number of xpassed tests.
    xpassed: usize,
    /// Number of error tests.
    errors: usize,
    /// Total number of tests.
    total: usize,
    /// Total duration in seconds.
    duration: Option<f64>,
}

/// Parsed pytest output.
#[derive(Debug, Clone, Default)]
struct PytestOutput {
    /// List of test results.
    tests: Vec<TestResult>,
    /// Summary statistics.
    summary: TestSummary,
    /// Whether all tests passed.
    success: bool,
    /// Whether the output is empty.
    is_empty: bool,
    /// Collection rootdir (if available).
    rootdir: Option<String>,
    /// Platform info (if available).
    platform: Option<String>,
    /// Python version (if available).
    python_version: Option<String>,
    /// Pytest version (if available).
    pytest_version: Option<String>,
}

/// Parsed Jest output.
#[derive(Debug, Clone, Default)]
struct JestOutput {
    /// List of test suites.
    test_suites: Vec<JestTestSuite>,
    /// Summary statistics.
    summary: JestSummary,
    /// Whether all tests passed.
    success: bool,
    /// Whether the output is empty.
    is_empty: bool,
    /// Jest version (if available).
    jest_version: Option<String>,
    /// Test path pattern (if available).
    test_path_pattern: Option<String>,
}

/// A Jest test suite result.
#[derive(Debug, Clone)]
struct JestTestSuite {
    /// Test file path.
    file: String,
    /// Whether the suite passed.
    passed: bool,
    /// Execution time in seconds.
    duration: Option<f64>,
    /// List of test results in this suite.
    tests: Vec<JestTest>,
}

/// A single Jest test result.
#[derive(Debug, Clone)]
struct JestTest {
    /// Full test name (ancestor titles + test name).
    name: String,
    /// Test name only.
    test_name: String,
    /// Ancestor titles (describe blocks).
    ancestors: Vec<String>,
    /// Status of the test.
    status: JestTestStatus,
    /// Duration in seconds (if available).
    duration: Option<f64>,
    /// Error message (for failed tests).
    error_message: Option<String>,
}

/// Status of a Jest test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JestTestStatus {
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
struct JestSummary {
    /// Number of test suites passed.
    suites_passed: usize,
    /// Number of test suites failed.
    suites_failed: usize,
    /// Number of test suites total.
    suites_total: usize,
    /// Number of tests passed.
    tests_passed: usize,
    /// Number of tests failed.
    tests_failed: usize,
    /// Number of tests skipped.
    tests_skipped: usize,
    /// Number of tests todo.
    tests_todo: usize,
    /// Number of tests total.
    tests_total: usize,
    /// Number of snapshots updated/added/removed.
    snapshots: Option<usize>,
    /// Total duration in seconds.
    duration: Option<f64>,
}

/// Parsed Vitest output.
#[derive(Debug, Clone, Default)]
struct VitestOutput {
    /// List of test suites.
    test_suites: Vec<VitestTestSuite>,
    /// Summary statistics.
    summary: VitestSummary,
    /// Whether all tests passed.
    success: bool,
    /// Whether the output is empty.
    is_empty: bool,
    /// Vitest version (if available).
    vitest_version: Option<String>,
}

/// A Vitest test suite result.
#[derive(Debug, Clone)]
struct VitestTestSuite {
    /// Test file path.
    file: String,
    /// Whether the suite passed.
    passed: bool,
    /// Execution time in seconds.
    duration: Option<f64>,
    /// Number of tests in suite.
    test_count: Option<usize>,
    /// Number of skipped tests in suite.
    skipped_count: Option<usize>,
    /// List of test results in this suite.
    tests: Vec<VitestTest>,
}

/// A single Vitest test result.
#[derive(Debug, Clone)]
struct VitestTest {
    /// Full test name (ancestor titles + test name).
    name: String,
    /// Test name only.
    test_name: String,
    /// Ancestor titles (describe blocks).
    ancestors: Vec<String>,
    /// Status of the test.
    status: VitestTestStatus,
    /// Duration in seconds (if available).
    duration: Option<f64>,
    /// Error message (for failed tests).
    error_message: Option<String>,
}

/// Status of a Vitest test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VitestTestStatus {
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
struct VitestSummary {
    /// Number of test suites passed.
    suites_passed: usize,
    /// Number of test suites failed.
    suites_failed: usize,
    /// Number of test suites total.
    suites_total: usize,
    /// Number of tests passed.
    tests_passed: usize,
    /// Number of tests failed.
    tests_failed: usize,
    /// Number of tests skipped.
    tests_skipped: usize,
    /// Number of tests todo.
    tests_todo: usize,
    /// Number of tests total.
    tests_total: usize,
    /// Total duration in seconds.
    duration: Option<f64>,
    /// Start time (if available).
    start_at: Option<String>,
}

/// Helper struct for parsing vitest suite headers.
struct VitestSuiteInfo {
    file: String,
    passed: bool,
    duration: Option<f64>,
    test_count: Option<usize>,
    skipped_count: Option<usize>,
}

// ============================================================
// NPM Test (Node.js built-in test runner) Parser
// ============================================================

/// Parsed npm test output (Node.js built-in test runner with spec reporter).
#[derive(Debug, Clone, Default)]
struct NpmTestOutput {
    /// List of test suites (files).
    test_suites: Vec<NpmTestSuite>,
    /// Summary statistics.
    summary: NpmTestSummary,
    /// Whether all tests passed.
    success: bool,
    /// Whether the output is empty.
    is_empty: bool,
    /// Node.js version (if available).
    node_version: Option<String>,
}

/// A npm test suite result (a test file).
#[derive(Debug, Clone)]
struct NpmTestSuite {
    /// Test file path.
    file: String,
    /// Whether the suite passed.
    passed: bool,
    /// Execution time in seconds.
    duration: Option<f64>,
    /// List of test results in this suite.
    tests: Vec<NpmTest>,
}

/// A single npm test result.
#[derive(Debug, Clone)]
struct NpmTest {
    /// Full test name (including nested structure).
    name: String,
    /// Test name only (last part).
    test_name: String,
    /// Ancestor names (describe/nested test blocks).
    ancestors: Vec<String>,
    /// Status of the test.
    status: NpmTestStatus,
    /// Duration in seconds (if available).
    duration: Option<f64>,
    /// Error message (for failed tests).
    error_message: Option<String>,
}

/// Status of a npm test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NpmTestStatus {
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
struct NpmTestSummary {
    /// Number of test suites passed.
    suites_passed: usize,
    /// Number of test suites failed.
    suites_failed: usize,
    /// Number of test suites skipped.
    suites_skipped: usize,
    /// Number of test suites total.
    suites_total: usize,
    /// Number of tests passed.
    tests_passed: usize,
    /// Number of tests failed.
    tests_failed: usize,
    /// Number of tests skipped.
    tests_skipped: usize,
    /// Number of tests todo.
    tests_todo: usize,
    /// Number of tests total.
    tests_total: usize,
    /// Total duration in seconds.
    duration: Option<f64>,
}

// ============================================================
// PNPM Test (Node.js built-in test runner via pnpm) Parser
// ============================================================

/// Parsed pnpm test output (Node.js built-in test runner with spec reporter).
/// pnpm test uses the same output format as npm test since it runs the same
/// test runner defined in package.json scripts.test.
#[derive(Debug, Clone, Default)]
struct PnpmTestOutput {
    /// List of test suites (files).
    test_suites: Vec<PnpmTestSuite>,
    /// Summary statistics.
    summary: PnpmTestSummary,
    /// Whether all tests passed.
    success: bool,
    /// Whether the output is empty.
    is_empty: bool,
    /// pnpm version (if available).
    pnpm_version: Option<String>,
}

/// A pnpm test suite result (a test file).
#[derive(Debug, Clone)]
struct PnpmTestSuite {
    /// Test file path.
    file: String,
    /// Whether the suite passed.
    passed: bool,
    /// Execution time in seconds.
    duration: Option<f64>,
    /// List of test results in this suite.
    tests: Vec<PnpmTest>,
}

/// A single pnpm test result.
#[derive(Debug, Clone)]
struct PnpmTest {
    /// Full test name (including nested structure).
    name: String,
    /// Test name only (last part).
    test_name: String,
    /// Ancestor names (describe/nested test blocks).
    ancestors: Vec<String>,
    /// Status of the test.
    status: PnpmTestStatus,
    /// Duration in seconds (if available).
    duration: Option<f64>,
    /// Error message (for failed tests).
    error_message: Option<String>,
}

/// Status of a pnpm test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PnpmTestStatus {
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
struct PnpmTestSummary {
    /// Number of test suites passed.
    suites_passed: usize,
    /// Number of test suites failed.
    suites_failed: usize,
    /// Number of test suites skipped.
    suites_skipped: usize,
    /// Number of test suites total.
    suites_total: usize,
    /// Number of tests passed.
    tests_passed: usize,
    /// Number of tests failed.
    tests_failed: usize,
    /// Number of tests skipped.
    tests_skipped: usize,
    /// Number of tests todo.
    tests_todo: usize,
    /// Number of tests total.
    tests_total: usize,
    /// Total duration in seconds.
    duration: Option<f64>,
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
struct BunTestOutput {
    /// List of test suites (files).
    test_suites: Vec<BunTestSuite>,
    /// Summary statistics.
    summary: BunTestSummary,
    /// Whether all tests passed.
    success: bool,
    /// Whether the output is empty.
    is_empty: bool,
    /// Bun version (if available).
    bun_version: Option<String>,
}

/// A Bun test suite result (a test file).
#[derive(Debug, Clone)]
struct BunTestSuite {
    /// Test file path.
    file: String,
    /// Whether the suite passed.
    passed: bool,
    /// Execution time in seconds.
    duration: Option<f64>,
    /// List of test results in this suite.
    tests: Vec<BunTest>,
}

/// A single Bun test result.
#[derive(Debug, Clone)]
struct BunTest {
    /// Full test name (including nested structure).
    name: String,
    /// Test name only (last part).
    test_name: String,
    /// Ancestor names (describe/nested test blocks).
    ancestors: Vec<String>,
    /// Status of the test.
    status: BunTestStatus,
    /// Duration in seconds (if available).
    duration: Option<f64>,
    /// Error message (for failed tests).
    error_message: Option<String>,
}

/// Status of a Bun test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BunTestStatus {
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
struct BunTestSummary {
    /// Number of test suites passed.
    suites_passed: usize,
    /// Number of test suites failed.
    suites_failed: usize,
    /// Number of test suites skipped.
    suites_skipped: usize,
    /// Number of test suites total.
    suites_total: usize,
    /// Number of tests passed.
    tests_passed: usize,
    /// Number of tests failed.
    tests_failed: usize,
    /// Number of tests skipped.
    tests_skipped: usize,
    /// Number of tests todo.
    tests_todo: usize,
    /// Number of tests total.
    tests_total: usize,
    /// Number of expect() calls.
    expect_calls: Option<usize>,
    /// Total duration in seconds.
    duration: Option<f64>,
}

// ============================================================
// Log Stream Parser
// ============================================================

/// Log level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogLevel {
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
struct LogEntry {
    /// Original line content.
    line: String,
    /// Detected log level.
    level: LogLevel,
    /// Timestamp (if detected).
    timestamp: Option<String>,
    /// Source/logger name (if detected).
    source: Option<String>,
    /// Message content (without timestamp/level prefix).
    message: String,
    /// Line number in the input.
    line_number: usize,
}

/// Statistics for repeated lines.
#[derive(Debug, Clone)]
struct RepeatedLine {
    /// The repeated line content.
    line: String,
    /// Number of occurrences.
    count: usize,
    /// First occurrence line number.
    first_line: usize,
    /// Last occurrence line number.
    last_line: usize,
}

/// Maximum number of recent critical (ERROR/FATAL) lines to track.
const MAX_RECENT_CRITICAL: usize = 10;

/// Parsed log output.
#[derive(Debug, Clone, Default)]
struct LogsOutput {
    /// All log entries.
    entries: Vec<LogEntry>,
    /// Total line count.
    total_lines: usize,
    /// Count by level.
    debug_count: usize,
    info_count: usize,
    warning_count: usize,
    error_count: usize,
    fatal_count: usize,
    unknown_count: usize,
    /// Repeated lines (collapsed).
    repeated_lines: Vec<RepeatedLine>,
    /// Most recent critical lines (ERROR and FATAL level entries).
    recent_critical: Vec<LogEntry>,
    /// Whether the output is empty.
    is_empty: bool,
}

/// Common generated directory names that are typically build artifacts or dependencies.
const COMMON_GENERATED_DIRS: &[&str] = &[
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
fn is_generated_directory(name: &str) -> bool {
    // Strip trailing slash if present (common in ls output)
    let name = name.strip_suffix('/').unwrap_or(name);
    let name_lower = name.to_lowercase();
    COMMON_GENERATED_DIRS.contains(&name_lower.as_str())
}

/// A permission denied or error entry.
#[derive(Debug, Clone, Default)]
struct LsError {
    /// The path that was denied access.
    path: String,
    /// The error message.
    message: String,
}

/// Parsed ls output.
#[derive(Debug, Clone, Default)]
struct LsOutput {
    /// List of all entries.
    entries: Vec<LsEntry>,
    /// Directory entries.
    directories: Vec<LsEntry>,
    /// File entries.
    files: Vec<LsEntry>,
    /// Symlink entries.
    symlinks: Vec<LsEntry>,
    /// Hidden entries.
    hidden: Vec<LsEntry>,
    /// Generated directory entries (build artifacts, dependencies, etc.).
    generated: Vec<LsEntry>,
    /// Permission denied or error entries.
    errors: Vec<LsError>,
    /// Total count of entries (excluding errors).
    total_count: usize,
    /// Whether the output is empty.
    is_empty: bool,
}

/// Trait for command handlers.
///
/// Each command in the CLI implements this trait to handle its specific logic.
pub trait CommandHandler {
    /// The input type for this command (the command variant data).
    type Input;

    /// Execute the command with the given input and context.
    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult;
}

/// Handler for the `run` command.
pub struct RunHandler;

impl RunHandler {
    /// Format output based on the specified format.
    fn format_output(output: &ProcessOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => {
                // JSON output includes all fields
                serde_json::json!({
                    "command": output.command,
                    "args": output.args,
                    "stdout": output.stdout,
                    "stderr": output.stderr,
                    "exit_code": output.exit_code,
                    "duration_ms": output.duration.as_millis(),
                    "timed_out": output.timed_out,
                })
                .to_string()
            }
            OutputFormat::Csv => {
                // CSV output with header row
                let mut result = String::new();
                result.push_str("command,args,stdout,stderr,exit_code,duration_ms,timed_out\n");
                let args_str = output.args.join(" ");
                let stdout_escaped = Self::escape_csv_field(&output.stdout);
                let stderr_escaped = Self::escape_csv_field(&output.stderr);
                result.push_str(&format!(
                    "{},{},{},{},{},{},{}\n",
                    output.command,
                    args_str,
                    stdout_escaped,
                    stderr_escaped,
                    output.exit_code.map(|c| c.to_string()).unwrap_or_default(),
                    output.duration.as_millis(),
                    output.timed_out
                ));
                result
            }
            OutputFormat::Tsv => {
                // TSV output with header row
                let mut result = String::new();
                result
                    .push_str("command\targs\tstdout\tstderr\texit_code\tduration_ms\ttimed_out\n");
                let args_str = output.args.join(" ");
                let stdout_escaped = Self::escape_tsv_field(&output.stdout);
                let stderr_escaped = Self::escape_tsv_field(&output.stderr);
                result.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    output.command,
                    args_str,
                    stdout_escaped,
                    stderr_escaped,
                    output.exit_code.map(|c| c.to_string()).unwrap_or_default(),
                    output.duration.as_millis(),
                    output.timed_out
                ));
                result
            }
            OutputFormat::Compact | OutputFormat::Agent => {
                // Compact output shows essential info
                let mut result = String::new();
                if output.has_stdout() {
                    result.push_str(&output.stdout);
                    if !result.ends_with('\n') && !result.is_empty() {
                        result.push('\n');
                    }
                }
                if output.has_stderr() {
                    result.push_str(&output.stderr);
                }
                result
            }
            OutputFormat::Raw => {
                // Raw output: unprocessed stdout and stderr
                let mut result = output.stdout.clone();
                if output.has_stderr() && !output.stderr.is_empty() {
                    result.push_str(&output.stderr);
                }
                result
            }
        }
    }

    /// Escape a field for CSV format.
    fn escape_csv_field(field: &str) -> String {
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

    /// Escape a field for TSV format.
    fn escape_tsv_field(field: &str) -> String {
        // TSV doesn't support tabs in fields; replace with space
        field.replace('\t', " ").replace('\r', "")
    }

    /// Format error message based on format.
    fn format_error(error: &ProcessError, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => serde_json::json!({
                "error": true,
                "message": error.to_string(),
                "exit_code": error.exit_code(),
                "is_timeout": error.is_timeout(),
                "is_command_not_found": error.is_command_not_found(),
                "is_permission_denied": error.is_permission_denied(),
            })
            .to_string(),
            OutputFormat::Raw
            | OutputFormat::Compact
            | OutputFormat::Agent
            | OutputFormat::Csv
            | OutputFormat::Tsv => format!("Error: {}", error),
        }
    }
}

impl CommandHandler for RunHandler {
    type Input = RunInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        // Build and execute the process
        let mut builder = ProcessBuilder::new(&input.command)
            .args(&input.args)
            .capture_stdout(input.capture_stdout)
            .capture_stderr(input.capture_stderr)
            .capture_exit_code(input.capture_exit_code)
            .capture_duration(input.capture_duration);

        // Add timeout if specified
        if let Some(timeout) = input.timeout {
            builder = builder.timeout(std::time::Duration::from_secs(timeout));
        }

        let result = builder.run();

        match result {
            Ok(output) => {
                // Print stats if requested
                if ctx.stats {
                    eprintln!("Stats:");
                    eprintln!("  Command: {} {:?}", output.command, output.args);
                    eprintln!("  Exit code: {:?}", output.exit_code);
                    eprintln!("  Duration: {:.2}s", output.duration.as_secs_f64());
                    eprintln!("  Stdout bytes: {}", output.stdout.len());
                    eprintln!("  Stderr bytes: {}", output.stderr.len());
                }

                // Format and print output
                let formatted = Self::format_output(&output, ctx.format);
                print!("{}", formatted);

                // Propagate exit code (only if we captured it)
                if input.capture_exit_code && !output.success() {
                    return Err(CommandError::ExecutionError {
                        message: format!("Command exited with code {}", output.code()),
                        exit_code: output.exit_code,
                    });
                }

                Ok(())
            }
            Err(error) => {
                // Print stats if requested
                if ctx.stats {
                    eprintln!("Stats:");
                    eprintln!("  Command failed: {}", error);
                }

                // Return appropriate error type (error printing is handled by Router::execute_and_print)
                Err(match &error {
                    ProcessError::CommandNotFound { command } => CommandError::ExecutionError {
                        message: format!("Command not found: {}", command),
                        exit_code: Some(127), // Standard "command not found" exit code
                    },
                    ProcessError::PermissionDenied { command } => CommandError::ExecutionError {
                        message: format!("Permission denied: {}", command),
                        exit_code: Some(126), // Standard "permission denied" exit code
                    },
                    ProcessError::Timeout {
                        command, duration, ..
                    } => CommandError::ExecutionError {
                        message: format!(
                            "Command '{}' timed out after {:.2}s",
                            command,
                            duration.as_secs_f64()
                        ),
                        exit_code: Some(124), // Standard timeout exit code
                    },
                    ProcessError::NonZeroExit { output } => CommandError::ExecutionError {
                        message: format!("Command exited with code {}", output.code()),
                        exit_code: output.exit_code,
                    },
                    ProcessError::IoError { message, .. } => CommandError::IoError(message.clone()),
                    ProcessError::SpawnFailed { message, .. } => CommandError::ExecutionError {
                        message: message.clone(),
                        exit_code: None,
                    },
                })
            }
        }
    }
}

/// Input data for the `run` command.
#[derive(Debug, Clone)]
pub struct RunInput {
    pub command: String,
    pub args: Vec<String>,
    pub capture_stdout: bool,
    pub capture_stderr: bool,
    pub capture_exit_code: bool,
    pub capture_duration: bool,
    /// Optional timeout in seconds
    pub timeout: Option<u64>,
}

impl From<(&String, &Vec<String>, bool, bool, bool, bool, Option<u64>)> for RunInput {
    fn from(
        (
            command,
            args,
            capture_stdout,
            capture_stderr,
            capture_exit_code,
            capture_duration,
            timeout,
        ): (&String, &Vec<String>, bool, bool, bool, bool, Option<u64>),
    ) -> Self {
        Self {
            command: command.clone(),
            args: args.clone(),
            capture_stdout,
            capture_stderr,
            capture_exit_code,
            capture_duration,
            timeout,
        }
    }
}

/// Handler for the `search` command.
pub struct SearchHandler;

impl SearchHandler {
    /// Default directories to ignore during search.
    const DEFAULT_IGNORE_DIRS: &'static [&'static str] = &[
        ".git",
        "node_modules",
        "target",
        "dist",
        "build",
        ".cache",
        "__pycache__",
        ".venv",
        "venv",
        ".idea",
        ".vscode",
        "vendor",
        "bundle",
        ".tox",
        ".mypy_cache",
        ".pytest_cache",
        "coverage",
        ".next",
        ".nuxt",
    ];

    /// Default maximum number of files to show in output before truncation.
    const DEFAULT_MAX_FILES: usize = 50;

    /// Execute high-performance search using ripgrep crates.
    fn execute_search(&self, input: &SearchInput) -> CommandResult<GrepOutput> {
        use grep::matcher::Matcher;
        use grep::regex::RegexMatcher;
        use grep::searcher::Searcher;
        use grep::searcher::SearcherBuilder;
        use grep::searcher::Sink;
        use ignore::WalkBuilder;
        use std::sync::{Arc, Mutex};

        // Build the regex matcher
        let matcher = if input.ignore_case {
            RegexMatcher::new(&format!("(?i){}", input.query))
        } else {
            RegexMatcher::new(&input.query)
        }
        .map_err(|e| CommandError::ExecutionError {
            message: format!("Invalid regex pattern '{}': {}", input.query, e),
            exit_code: Some(2),
        })?;

        /// A single match result with column information.
        #[derive(Debug, Clone)]
        struct MatchResult {
            line_number: usize,
            column: usize,
            line: String,
            excerpt: String,
            is_context: bool,
        }

        /// Custom sink to capture match positions and excerpts.
        struct MatchSink {
            matches: Vec<MatchResult>,
            matcher: RegexMatcher,
        }

        impl MatchSink {
            fn new(matcher: RegexMatcher) -> Self {
                Self {
                    matches: Vec::new(),
                    matcher,
                }
            }
        }

        impl Sink for MatchSink {
            type Error = std::io::Error;

            fn matched(
                &mut self,
                _searcher: &Searcher,
                mat: &grep::searcher::SinkMatch<'_>,
            ) -> Result<bool, Self::Error> {
                let line_number = mat.line_number().unwrap_or(0) as usize;
                let line_bytes = mat.bytes();
                let line_str = String::from_utf8_lossy(line_bytes);
                let line = line_str.to_string();

                // Find the column position and extract the excerpt
                let (column, excerpt) = if let Ok(Some(m)) = self.matcher.find(line_bytes) {
                    let col = m.start();
                    let excerpt_bytes = &line_bytes[m.start()..m.end()];
                    let excerpt_str = String::from_utf8_lossy(excerpt_bytes);
                    // Calculate character column (not byte offset) for display
                    let char_col =
                        String::from_utf8_lossy(&line_bytes[..col.min(line_bytes.len())])
                            .chars()
                            .count();
                    (char_col + 1, excerpt_str.to_string()) // 1-indexed
                } else {
                    (1, String::new())
                };

                self.matches.push(MatchResult {
                    line_number,
                    column,
                    line: line.trim_end().to_string(),
                    excerpt,
                    is_context: false,
                });
                Ok(true)
            }

            fn context(
                &mut self,
                _searcher: &Searcher,
                ctx: &grep::searcher::SinkContext<'_>,
            ) -> Result<bool, Self::Error> {
                let line_number = ctx.line_number().unwrap_or(0) as usize;
                let line_bytes = ctx.bytes();
                let line_str = String::from_utf8_lossy(line_bytes);

                self.matches.push(MatchResult {
                    line_number,
                    column: 0,
                    line: line_str.trim_end().to_string(),
                    excerpt: String::new(),
                    is_context: true,
                });
                Ok(true)
            }
        }

        // Shared state for collecting matches per file
        let file_matches: Arc<Mutex<Vec<(String, Vec<MatchResult>)>>> =
            Arc::new(Mutex::new(Vec::new()));

        // Build the directory walker with ignore rules
        let mut walk_builder = WalkBuilder::new(&input.path);

        // Add custom ignore patterns for common directories
        for dir in Self::DEFAULT_IGNORE_DIRS {
            walk_builder.add_ignore(format!("!{}/", dir));
        }

        // Configure walker
        walk_builder
            .hidden(false) // Don't skip hidden files by default
            .git_ignore(true) // Respect .gitignore
            .ignore(true) // Respect .ignore files
            .follow_links(false); // Don't follow symlinks

        // Search each file
        for entry_result in walk_builder.build() {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue, // Skip files we can't access
            };

            // Skip directories
            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }

            // Skip files that don't match the extension filter
            if let Some(ref ext) = input.extension {
                let path_ext = entry.path().extension().and_then(|e| e.to_str());
                if path_ext != Some(ext) {
                    continue;
                }
            }

            let path = entry.path().to_string_lossy().to_string();

            // Create a new searcher for each file
            let mut searcher_builder = SearcherBuilder::new();
            searcher_builder.line_number(true);

            // Configure context if requested
            if let Some(ctx) = input.context {
                searcher_builder.before_context(ctx);
                searcher_builder.after_context(ctx);
            }

            let mut searcher = searcher_builder.build();

            // Create sink with matcher clone
            let mut sink = MatchSink::new(matcher.clone());

            // Search with our custom sink
            let search_result = searcher.search_path(&matcher, entry.path(), &mut sink);

            // Ignore search errors (binary files, permission issues, etc.)
            if search_result.is_ok() && !sink.matches.is_empty() {
                file_matches.lock().unwrap().push((path, sink.matches));
            }
        }

        // Collect results into GrepOutput
        let collected = Arc::try_unwrap(file_matches)
            .expect("All references should be dropped")
            .into_inner()
            .expect("Mutex should not be poisoned");

        // Convert to GrepFile structures with excerpts
        let mut files: Vec<GrepFile> = collected
            .into_iter()
            .map(|(path, match_results)| {
                let grep_matches: Vec<GrepMatch> = match_results
                    .into_iter()
                    .map(|mr| GrepMatch {
                        line_number: Some(mr.line_number),
                        column: if mr.is_context { None } else { Some(mr.column) },
                        line: mr.line,
                        is_context: mr.is_context,
                        excerpt: if mr.excerpt.is_empty() || mr.is_context {
                            None
                        } else {
                            Some(mr.excerpt)
                        },
                    })
                    .collect();
                GrepFile {
                    path,
                    matches: grep_matches,
                }
            })
            .collect();

        // Sort files by path
        files.sort_by(|a, b| a.path.cmp(&b.path));

        // Calculate counts
        let file_count = files.len();
        let match_count: usize = files.iter().map(|f| f.matches.len()).sum();

        // Apply truncation
        let max_files = input.limit.unwrap_or(Self::DEFAULT_MAX_FILES);
        let is_truncated = files.len() > max_files;

        let total_files = files.len();
        let total_matches = match_count;

        // Truncate files if needed
        if files.len() > max_files {
            files.truncate(max_files);
        }

        let files_shown = files.len();
        let matches_shown: usize = files.iter().map(|f| f.matches.len()).sum();

        Ok(GrepOutput {
            files,
            file_count,
            match_count,
            is_empty: file_count == 0,
            is_truncated,
            total_files,
            total_matches,
            files_shown,
            matches_shown,
        })
    }

    /// Format search output for display.
    fn format_output(grep_output: &GrepOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_json(grep_output),
            OutputFormat::Csv => Self::format_csv(grep_output),
            OutputFormat::Tsv => Self::format_tsv(grep_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_compact(grep_output),
            OutputFormat::Raw => Self::format_raw(grep_output),
        }
    }

    /// Format search output as JSON using the schema.
    fn format_json(grep_output: &GrepOutput) -> String {
        use crate::schema::{
            GrepCounts, GrepFile as SchemaGrepFile, GrepMatch as SchemaGrepMatch, GrepOutputSchema,
        };

        let mut schema = GrepOutputSchema::new();
        schema.is_empty = grep_output.is_empty;
        schema.is_truncated = grep_output.is_truncated;

        // Convert internal GrepFile to schema GrepFile
        schema.files = grep_output
            .files
            .iter()
            .map(|f| SchemaGrepFile {
                path: f.path.clone(),
                matches: f
                    .matches
                    .iter()
                    .map(|m| SchemaGrepMatch {
                        line_number: m.line_number,
                        column: m.column,
                        line: m.line.clone(),
                        is_context: m.is_context,
                        excerpt: m.excerpt.clone(),
                    })
                    .collect(),
            })
            .collect();

        schema.counts = GrepCounts {
            files: grep_output.file_count,
            matches: grep_output.match_count,
            total_files: grep_output.total_files,
            total_matches: grep_output.total_matches,
            files_shown: grep_output.files_shown,
            matches_shown: grep_output.matches_shown,
        };

        serde_json::to_string_pretty(&schema).unwrap_or_else(|e| {
            serde_json::json!({"error": format!("Failed to serialize: {}", e)}).to_string()
        })
    }

    /// Format search output as CSV.
    fn format_csv(grep_output: &GrepOutput) -> String {
        ParseHandler::format_grep_csv(grep_output)
    }

    /// Format search output as TSV.
    fn format_tsv(grep_output: &GrepOutput) -> String {
        ParseHandler::format_grep_tsv(grep_output)
    }

    /// Format search output in compact format.
    fn format_compact(grep_output: &GrepOutput) -> String {
        ParseHandler::format_grep_compact(grep_output)
    }

    /// Format search output as raw (ripgrep output).
    fn format_raw(grep_output: &GrepOutput) -> String {
        ParseHandler::format_grep_raw(grep_output)
    }
}

impl CommandHandler for SearchHandler {
    type Input = SearchInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Execute the search
        let grep_output = self.execute_search(input)?;

        // Format and print the output
        let output = Self::format_output(&grep_output, ctx.format);
        print!("{}", output);

        Ok(())
    }
}

/// Input data for the `search` command.
#[derive(Debug, Clone)]
pub struct SearchInput {
    pub path: std::path::PathBuf,
    pub query: String,
    pub extension: Option<String>,
    pub ignore_case: bool,
    pub context: Option<usize>,
    pub limit: Option<usize>,
}

/// A single replacement result.
#[derive(Debug, Clone)]
struct Replacement {
    line_number: usize,
    original: String,
    replaced: String,
}

/// Handler for the `replace` command.
pub struct ReplaceHandler;

impl ReplaceHandler {
    /// Default directories to ignore during replace.
    const DEFAULT_IGNORE_DIRS: &'static [&'static str] = &[
        ".git",
        "node_modules",
        "target",
        "dist",
        "build",
        ".cache",
        "__pycache__",
        ".venv",
        "venv",
        ".idea",
        ".vscode",
        "vendor",
        "bundle",
        ".tox",
        ".mypy_cache",
        ".pytest_cache",
        "coverage",
        ".next",
        ".nuxt",
    ];

    /// Execute search and replace using ripgrep crates.
    fn execute_replace(
        &self,
        input: &ReplaceInput,
    ) -> CommandResult<Vec<(String, Vec<Replacement>)>> {
        use grep::matcher::Matcher;
        use grep::regex::RegexMatcher;
        use ignore::WalkBuilder;

        // Build the regex matcher
        let matcher =
            RegexMatcher::new(&input.search).map_err(|e| CommandError::ExecutionError {
                message: format!("Invalid regex pattern '{}': {}", input.search, e),
                exit_code: Some(2),
            })?;

        // Shared state for collecting replacements per file
        let mut file_replacements: Vec<(String, Vec<Replacement>)> = Vec::new();

        // Build the directory walker with ignore rules
        let mut walk_builder = WalkBuilder::new(&input.path);

        // Add custom ignore patterns for common directories
        for dir in Self::DEFAULT_IGNORE_DIRS {
            walk_builder.add_ignore(format!("!{}/", dir));
        }

        // Configure walker
        walk_builder
            .hidden(false)
            .git_ignore(true)
            .ignore(true)
            .follow_links(false);

        // Process each file
        for entry_result in walk_builder.build() {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Skip directories
            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }

            // Skip files that don't match the extension filter
            if let Some(ref ext) = input.extension {
                let path_ext = entry.path().extension().and_then(|e| e.to_str());
                if path_ext != Some(ext) {
                    continue;
                }
            }

            let path = entry.path().to_string_lossy().to_string();

            // Read file content
            let content = match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue, // Skip files we can't read
            };

            // Find all matches in this file
            let lines: Vec<&str> = content.lines().collect();
            let mut replacements: Vec<Replacement> = Vec::new();
            let mut modified_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            let mut has_changes = false;

            for (line_idx, line) in lines.iter().enumerate() {
                let line_bytes = line.as_bytes();

                // Find all matches in this line
                let mut offset = 0usize;
                let mut modified_line = line.to_string();
                let mut line_changed = false;

                while let Ok(Some(m)) = matcher.find_at(line_bytes, offset) {
                    let start = m.start();
                    let end = m.end();

                    // Perform the replacement
                    modified_line.replace_range(start..end, &input.replace);
                    line_changed = true;
                    has_changes = true;

                    // Move past this match
                    offset = end;
                    if offset >= line_bytes.len() {
                        break;
                    }
                }

                if line_changed {
                    replacements.push(Replacement {
                        line_number: line_idx + 1, // 1-indexed
                        original: line.to_string(),
                        replaced: modified_line.clone(),
                    });
                    modified_lines[line_idx] = modified_line;
                }
            }

            // If there are changes and not dry run, write the file back
            if has_changes {
                if !input.dry_run {
                    let new_content = modified_lines.join("\n");
                    if let Err(e) = std::fs::write(entry.path(), new_content) {
                        eprintln!("Warning: Failed to write {}: {}", path, e);
                        continue;
                    }
                }
                file_replacements.push((path, replacements));
            }
        }

        Ok(file_replacements)
    }

    /// Format replace output based on the specified format.
    fn format_output(
        replacements: &[(String, Vec<Replacement>)],
        input: &ReplaceInput,
        format: OutputFormat,
    ) -> String {
        match format {
            OutputFormat::Json => Self::format_json(replacements, input),
            OutputFormat::Csv => Self::format_csv(replacements, input),
            OutputFormat::Tsv => Self::format_tsv(replacements, input),
            OutputFormat::Compact | OutputFormat::Agent => {
                Self::format_compact(replacements, input)
            }
            OutputFormat::Raw => Self::format_raw(replacements, input),
        }
    }

    /// Format replace output as JSON using the schema.
    fn format_json(replacements: &[(String, Vec<Replacement>)], input: &ReplaceInput) -> String {
        use crate::schema::{ReplaceCounts, ReplaceFile, ReplaceMatch, ReplaceOutputSchema};

        let files: Vec<ReplaceFile> = replacements
            .iter()
            .map(|(path, reps)| {
                let matches: Vec<ReplaceMatch> = reps
                    .iter()
                    .map(|r| ReplaceMatch::new(r.line_number, &r.original, &r.replaced))
                    .collect();
                ReplaceFile {
                    path: path.clone(),
                    matches,
                }
            })
            .collect();

        let total_replacements: usize = files.iter().map(|f| f.matches.len()).sum();

        let schema = ReplaceOutputSchema::new(&input.search, &input.replace, input.dry_run)
            .with_files(files)
            .with_counts(ReplaceCounts {
                files_affected: replacements.len(),
                total_replacements,
            });

        serde_json::to_string_pretty(&schema).unwrap_or_else(|e| {
            serde_json::json!({"error": format!("Failed to serialize: {}", e)}).to_string()
        })
    }

    /// Format replace output as CSV.
    fn format_csv(replacements: &[(String, Vec<Replacement>)], input: &ReplaceInput) -> String {
        let mut result = String::new();
        result.push_str("file,line_number,original,replaced\n");

        for (path, reps) in replacements {
            for rep in reps {
                let original_escaped = Self::escape_csv_field(&rep.original);
                let replaced_escaped = Self::escape_csv_field(&rep.replaced);
                result.push_str(&format!(
                    "{},{},{},{}\n",
                    path, rep.line_number, original_escaped, replaced_escaped
                ));
            }
        }

        // Add summary at the end
        let total_replacements: usize = replacements.iter().map(|(_, r)| r.len()).sum();
        result.push_str(&format!(
            "\n# Summary: {} files, {} replacements (dry_run: {})\n",
            replacements.len(),
            total_replacements,
            input.dry_run
        ));

        result
    }

    /// Format replace output as TSV.
    fn format_tsv(replacements: &[(String, Vec<Replacement>)], input: &ReplaceInput) -> String {
        let mut result = String::new();
        result.push_str("file\tline_number\toriginal\treplaced\n");

        for (path, reps) in replacements {
            for rep in reps {
                let original_escaped = Self::escape_tsv_field(&rep.original);
                let replaced_escaped = Self::escape_tsv_field(&rep.replaced);
                result.push_str(&format!(
                    "{}\t{}\t{}\t{}\n",
                    path, rep.line_number, original_escaped, replaced_escaped
                ));
            }
        }

        let total_replacements: usize = replacements.iter().map(|(_, r)| r.len()).sum();
        result.push_str(&format!(
            "\n# Summary: {} files, {} replacements (dry_run: {})\n",
            replacements.len(),
            total_replacements,
            input.dry_run
        ));

        result
    }

    /// Format replace output in compact format.
    fn format_compact(replacements: &[(String, Vec<Replacement>)], input: &ReplaceInput) -> String {
        let mut result = String::new();

        if replacements.is_empty() {
            if input.dry_run {
                result.push_str("No matches found.\n");
            } else {
                result.push_str("No changes made.\n");
            }
            return result;
        }

        let total_replacements: usize = replacements.iter().map(|(_, r)| r.len()).sum();

        if input.dry_run {
            result.push_str(&format!(
                "Preview: {} replacements in {} files\n\n",
                total_replacements,
                replacements.len()
            ));
        } else {
            result.push_str(&format!(
                "Replaced {} occurrences in {} files\n\n",
                total_replacements,
                replacements.len()
            ));
        }

        for (path, reps) in replacements {
            result.push_str(&format!("{}:\n", path));
            for rep in reps {
                result.push_str(&format!(
                    "  {}:{}\n",
                    rep.line_number,
                    Self::truncate_line(&rep.replaced, 80)
                ));
            }
            result.push('\n');
        }

        result
    }

    /// Format replace output as raw.
    fn format_raw(replacements: &[(String, Vec<Replacement>)], input: &ReplaceInput) -> String {
        let mut result = String::new();

        for (path, reps) in replacements {
            for rep in reps {
                result.push_str(&format!(
                    "{}:{}: {} -> {}\n",
                    path, rep.line_number, rep.original, rep.replaced
                ));
            }
        }

        let total_replacements: usize = replacements.iter().map(|(_, r)| r.len()).sum();
        result.push_str(&format!(
            "\nSummary: {} files, {} replacements (dry_run: {})\n",
            replacements.len(),
            total_replacements,
            input.dry_run
        ));

        result
    }

    /// Truncate a line to a maximum length.
    fn truncate_line(line: &str, max_len: usize) -> String {
        if line.len() <= max_len {
            line.to_string()
        } else {
            format!("{}...", &line[..max_len.saturating_sub(3)])
        }
    }

    /// Escape a field for CSV format.
    fn escape_csv_field(field: &str) -> String {
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

    /// Escape a field for TSV format.
    fn escape_tsv_field(field: &str) -> String {
        field
            .replace('\t', "\\t")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    }

    /// Format replacement count for output (just the number).
    fn format_count(count: usize, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => serde_json::json!({ "count": count }).to_string(),
            OutputFormat::Raw | OutputFormat::Compact | OutputFormat::Agent => {
                format!("{}\n", count)
            }
            OutputFormat::Csv | OutputFormat::Tsv => {
                format!("count\n{}\n", count)
            }
        }
    }
}

impl CommandHandler for ReplaceHandler {
    type Input = ReplaceInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Execute the replace
        let replacements = self.execute_replace(input)?;

        // If count flag is specified, output only the count
        if input.count {
            let total_replacements: usize = replacements.iter().map(|(_, r)| r.len()).sum();
            let output = Self::format_count(total_replacements, ctx.format);
            print!("{}", output);
            return Ok(());
        }

        // Format and print the output
        let output = Self::format_output(&replacements, input, ctx.format);
        print!("{}", output);

        Ok(())
    }
}

/// Input data for the `replace` command.
#[derive(Debug, Clone)]
pub struct ReplaceInput {
    pub path: std::path::PathBuf,
    pub search: String,
    pub replace: String,
    pub extension: Option<String>,
    pub dry_run: bool,
    pub count: bool,
}

/// Handler for the `tail` command.
pub struct TailHandler;

/// A single line in tail output.
#[derive(Debug, Clone)]
struct TailLine {
    /// Line number (1-indexed).
    line_number: usize,
    /// The line content.
    line: String,
    /// Whether this line is an error line.
    is_error: bool,
}

/// Parsed tail output.
#[derive(Debug, Clone)]
struct TailOutput {
    /// The file being tailed.
    file: std::path::PathBuf,
    /// List of lines.
    lines: Vec<TailLine>,
    /// Total lines read.
    total_lines: usize,
    /// Lines shown (after filtering).
    lines_shown: usize,
    /// Whether filtering is active.
    filtering_errors: bool,
}

impl TailHandler {
    /// Read the last N lines from a file.
    fn read_tail_lines(&self, input: &TailInput) -> CommandResult<TailOutput> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        // Check if file exists
        if !input.file.exists() {
            return Err(CommandError::IoError(format!(
                "File not found: {}",
                input.file.display()
            )));
        }

        let file = File::open(&input.file).map_err(|e| {
            CommandError::IoError(format!(
                "Failed to open file {}: {}",
                input.file.display(),
                e
            ))
        })?;

        let reader = BufReader::new(file);
        let mut all_lines: Vec<String> = Vec::new();

        // Read all lines
        for line in reader.lines() {
            let line =
                line.map_err(|e| CommandError::IoError(format!("Failed to read file: {}", e)))?;
            all_lines.push(line);
        }

        let total_lines = all_lines.len();

        // Get the last N lines (or all if file is smaller)
        let start = if all_lines.len() > input.lines {
            all_lines.len() - input.lines
        } else {
            0
        };

        let tail_lines: Vec<String> = all_lines[start..].to_vec();

        // Process lines
        let mut result_lines: Vec<TailLine> = Vec::new();
        let mut line_number = start + 1; // 1-indexed

        for line in tail_lines {
            let is_error = Self::is_error_line(&line);

            // If filtering for errors, skip non-error lines
            if input.errors && !is_error {
                line_number += 1;
                continue;
            }

            result_lines.push(TailLine {
                line_number,
                line,
                is_error,
            });

            line_number += 1;
        }

        let lines_shown = result_lines.len();

        Ok(TailOutput {
            file: input.file.clone(),
            lines: result_lines,
            total_lines,
            lines_shown,
            filtering_errors: input.errors,
        })
    }

    /// Check if a line is an error line.
    fn is_error_line(line: &str) -> bool {
        let line_lower = line.to_lowercase();
        let line_trimmed = line.trim();

        // Check for common error patterns
        line_lower.contains("error")
            || line_lower.contains("exception")
            || line_lower.contains("fatal")
            || line_lower.contains("critical")
            || line_lower.contains("failed")
            || line_lower.contains("failure")
            || line_trimmed.starts_with("E/")
            || line_trimmed.starts_with("E:")
            || line_trimmed.starts_with("ERR")
            || line_trimmed.starts_with("[ERROR]")
            || line_trimmed.starts_with("[FATAL]")
            || line_trimmed.starts_with("[CRITICAL]")
            || line_trimmed.starts_with("ERROR:")
            || line_trimmed.starts_with("FATAL:")
            || line_trimmed.starts_with("CRITICAL:")
    }

    /// Stream new lines from a file (follow mode).
    fn stream_tail_lines(
        &self,
        input: &TailInput,
        last_line_count: usize,
        ctx: &CommandContext,
    ) -> CommandResult<()> {
        use std::io::{BufRead, BufReader};
        use std::time::Duration;

        // Open file for streaming
        let file = std::fs::File::open(&input.file).map_err(|e| {
            CommandError::IoError(format!(
                "Failed to open file {}: {}",
                input.file.display(),
                e
            ))
        })?;

        let mut reader = BufReader::new(file);
        let mut line_number = last_line_count + 1;

        // Skip to the position we've already read
        for _ in 0..last_line_count {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|e| CommandError::IoError(format!("Failed to read file: {}", e)))?;
        }

        // Continuously poll for new lines
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    // No new data, wait and retry
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Ok(_) => {
                    // New line available
                    let line = line.trim_end_matches('\n').trim_end_matches('\r');
                    let is_error = Self::is_error_line(line);

                    // If filtering for errors, skip non-error lines
                    if input.errors && !is_error {
                        line_number += 1;
                        continue;
                    }

                    // Format and print the line
                    let tail_line = TailLine {
                        line_number,
                        line: line.to_string(),
                        is_error,
                    };

                    let formatted = Self::format_streaming_line(&tail_line, ctx.format);
                    print!("{}", formatted);
                    std::io::Write::flush(&mut std::io::stdout()).map_err(|e| {
                        CommandError::IoError(format!("Failed to flush stdout: {}", e))
                    })?;

                    line_number += 1;
                }
                Err(e) => {
                    return Err(CommandError::IoError(format!("Failed to read file: {}", e)));
                }
            }
        }
    }

    /// Format a single line for streaming output.
    fn format_streaming_line(line: &TailLine, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => {
                serde_json::json!({
                    "line_number": line.line_number,
                    "line": line.line,
                    "is_error": line.is_error,
                })
                .to_string()
                    + "\n"
            }
            OutputFormat::Csv => {
                let line_escaped = Self::escape_csv_field(&line.line);
                format!("{},{},{}\n", line.line_number, line_escaped, line.is_error)
            }
            OutputFormat::Tsv => {
                let line_escaped = Self::escape_tsv_field(&line.line);
                format!(
                    "{}\t{}\t{}\n",
                    line.line_number, line_escaped, line.is_error
                )
            }
            OutputFormat::Agent => {
                if line.is_error {
                    format!("❌ {}:{}\n", line.line_number, line.line)
                } else {
                    format!("   {}:{}\n", line.line_number, line.line)
                }
            }
            OutputFormat::Compact => {
                if line.is_error {
                    format!("  ❌ {}:{}\n", line.line_number, line.line)
                } else {
                    format!("  {}:{}\n", line.line_number, line.line)
                }
            }
            OutputFormat::Raw => {
                format!("{}:{}\n", line.line_number, line.line)
            }
        }
    }

    /// Format tail output based on the specified format.
    fn format_output(output: &TailOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_json(output),
            OutputFormat::Csv => Self::format_csv(output),
            OutputFormat::Tsv => Self::format_tsv(output),
            OutputFormat::Agent => Self::format_agent(output),
            OutputFormat::Compact => Self::format_compact(output),
            OutputFormat::Raw => Self::format_raw(output),
        }
    }

    /// Format tail output as JSON.
    fn format_json(output: &TailOutput) -> String {
        let lines_json: Vec<serde_json::Value> = output
            .lines
            .iter()
            .map(|l| {
                serde_json::json!({
                    "line_number": l.line_number,
                    "line": l.line,
                    "is_error": l.is_error,
                })
            })
            .collect();

        serde_json::json!({
            "file": output.file.display().to_string(),
            "lines": lines_json,
            "total_lines": output.total_lines,
            "lines_shown": output.lines_shown,
            "filtering_errors": output.filtering_errors,
        })
        .to_string()
    }

    /// Format tail output as CSV.
    fn format_csv(output: &TailOutput) -> String {
        let mut result = String::new();
        result.push_str("line_number,line,is_error\n");

        for l in &output.lines {
            let line_escaped = Self::escape_csv_field(&l.line);
            result.push_str(&format!(
                "{},{},{}\n",
                l.line_number, line_escaped, l.is_error
            ));
        }

        result
    }

    /// Format tail output as TSV.
    fn format_tsv(output: &TailOutput) -> String {
        let mut result = String::new();
        result.push_str("line_number\tline\tis_error\n");

        for l in &output.lines {
            let line_escaped = Self::escape_tsv_field(&l.line);
            result.push_str(&format!(
                "{}\t{}\t{}\n",
                l.line_number, line_escaped, l.is_error
            ));
        }

        result
    }

    /// Format tail output as agent-optimized format.
    fn format_agent(output: &TailOutput) -> String {
        let mut result = String::new();

        result.push_str(&format!("File: {}\n", output.file.display()));

        if output.filtering_errors {
            result.push_str(&format!(
                "Error lines: {} of {} total\n\n",
                output.lines_shown, output.total_lines
            ));
        } else {
            result.push_str(&format!(
                "Lines: {} of {} total\n\n",
                output.lines_shown, output.total_lines
            ));
        }

        for l in &output.lines {
            if l.is_error {
                result.push_str(&format!("❌ {}:{}\n", l.line_number, l.line));
            } else {
                result.push_str(&format!("   {}:{}\n", l.line_number, l.line));
            }
        }

        result
    }

    /// Format tail output as compact.
    fn format_compact(output: &TailOutput) -> String {
        let mut result = String::new();

        if output.lines.is_empty() {
            if output.filtering_errors {
                result.push_str("No error lines found.\n");
            } else {
                result.push_str("File is empty.\n");
            }
            return result;
        }

        // Show header
        if output.filtering_errors {
            result.push_str(&format!(
                "Error lines from {} ({} of {} total):\n\n",
                output.file.display(),
                output.lines_shown,
                output.total_lines
            ));
        } else {
            result.push_str(&format!(
                "Last {} lines from {} (total: {}):\n\n",
                output.lines_shown,
                output.file.display(),
                output.total_lines
            ));
        }

        for l in &output.lines {
            if l.is_error {
                result.push_str(&format!("  ❌ {}:{}\n", l.line_number, l.line));
            } else {
                result.push_str(&format!("  {}:{}\n", l.line_number, l.line));
            }
        }

        result
    }

    /// Format tail output as raw.
    fn format_raw(output: &TailOutput) -> String {
        let mut result = String::new();

        for l in &output.lines {
            result.push_str(&format!("{}:{}\n", l.line_number, l.line));
        }

        result
    }

    /// Escape a field for CSV format.
    fn escape_csv_field(field: &str) -> String {
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

    /// Escape a field for TSV format.
    fn escape_tsv_field(field: &str) -> String {
        field
            .replace('\t', "\\t")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    }
}

impl CommandHandler for TailHandler {
    type Input = TailInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Read initial tail lines
        let output = self.read_tail_lines(input)?;

        // Format and print initial output
        let formatted = Self::format_output(&output, ctx.format);
        print!("{}", formatted);

        // If follow mode is enabled, stream new lines
        if input.follow {
            self.stream_tail_lines(input, output.total_lines, ctx)?;
        }

        Ok(())
    }
}

/// Input data for the `tail` command.
#[derive(Debug, Clone)]
pub struct TailInput {
    pub file: std::path::PathBuf,
    pub lines: usize,
    pub errors: bool,
    pub follow: bool,
}

/// Handler for the `clean` command.
pub struct CleanHandler;

impl CleanHandler {
    /// Read input from file or stdin.
    fn read_input(&self, file: &Option<std::path::PathBuf>) -> CommandResult<String> {
        use std::io::{self, Read};

        match file {
            Some(path) => {
                if !path.exists() {
                    return Err(CommandError::IoError(format!(
                        "File not found: {}",
                        path.display()
                    )));
                }
                std::fs::read_to_string(path).map_err(|e| {
                    CommandError::IoError(format!("Failed to read file {}: {}", path.display(), e))
                })
            }
            None => {
                let mut buffer = Vec::new();
                io::stdin()
                    .read_to_end(&mut buffer)
                    .map_err(|e| CommandError::IoError(format!("Failed to read stdin: {}", e)))?;
                Ok(String::from_utf8_lossy(&buffer).to_string())
            }
        }
    }

    /// Apply cleaning operations to the input.
    fn clean_text(&self, text: &str, options: &CleanInput) -> String {
        let mut result = text.to_string();

        // Strip ANSI escape codes FIRST (before sanitizing control chars)
        // because ANSI codes start with \x1b which is a control character
        if options.no_ansi {
            result = strip_ansi_codes(&result);
        }

        // Sanitize control characters (remove nulls, replace other control chars)
        result = sanitize_control_chars(&result);

        // Trim whitespace from lines
        if options.trim {
            result = result
                .lines()
                .map(|line| line.trim())
                .collect::<Vec<_>>()
                .join("\n");
        } else {
            // Always trim trailing whitespace from each line
            result = result
                .lines()
                .map(|line| line.trim_end())
                .collect::<Vec<_>>()
                .join("\n");
        }

        // Collapse repeated lines or blank lines
        if options.collapse_repeats {
            result = self.collapse_repeated_lines(&result);
        } else if options.collapse_blanks {
            result = self.collapse_blank_lines(&result);
        }

        // Remove leading/trailing blank lines
        result.trim().to_string()
    }

    /// Collapse consecutive blank lines into a single blank line.
    fn collapse_blank_lines(&self, text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let mut collapsed_lines = Vec::new();
        let mut prev_blank = false;

        for line in lines {
            let is_blank = line.trim().is_empty();
            if is_blank && prev_blank {
                continue; // Skip consecutive blank lines
            }
            collapsed_lines.push(line);
            prev_blank = is_blank;
        }

        collapsed_lines.join("\n")
    }

    /// Collapse consecutive repeated lines into a single occurrence.
    fn collapse_repeated_lines(&self, text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let mut collapsed_lines = Vec::new();
        let mut prev_line: Option<&str> = None;

        for line in lines {
            // Skip if this line is the same as the previous line
            if let Some(prev) = prev_line {
                if line == prev {
                    continue;
                }
            }
            collapsed_lines.push(line);
            prev_line = Some(line);
        }

        collapsed_lines.join("\n")
    }

    /// Format the cleaned output based on the output format.
    fn format_output(
        &self,
        original: &str,
        cleaned: &str,
        options: &CleanInput,
        format: OutputFormat,
    ) -> String {
        match format {
            OutputFormat::Json => serde_json::json!({
                "content": cleaned,
                "stats": {
                    "input_length": original.len(),
                    "output_length": cleaned.len(),
                    "reduction_percent": if original.is_empty() {
                        0.0
                    } else {
                        ((original.len() - cleaned.len()) as f64 / original.len() as f64) * 100.0
                    },
                },
                "options": {
                    "no_ansi": options.no_ansi,
                    "collapse_blanks": options.collapse_blanks,
                    "collapse_repeats": options.collapse_repeats,
                    "trim": options.trim,
                }
            })
            .to_string(),
            OutputFormat::Csv => {
                // Output as CSV with one row per line
                cleaned
                    .lines()
                    .map(|line| format!("\"{}\"", line.replace('"', "\"\"")))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            OutputFormat::Tsv => {
                // Output as TSV with one row per line
                cleaned.lines().collect::<Vec<_>>().join("\n")
            }
            OutputFormat::Agent => {
                let reduction = if original.is_empty() {
                    0
                } else {
                    ((original.len() - cleaned.len()) as f64 / original.len() as f64 * 100.0) as i32
                };
                format!("Content ({}% reduction):\n{}\n", reduction, cleaned)
            }
            OutputFormat::Compact => {
                let reduction = if original.is_empty() {
                    0
                } else {
                    ((original.len() - cleaned.len()) as f64 / original.len() as f64 * 100.0) as i32
                };
                if reduction > 0 {
                    format!("{} ({}% reduction)\n", cleaned, reduction)
                } else {
                    format!("{}\n", cleaned)
                }
            }
            OutputFormat::Raw => cleaned.to_string(),
        }
    }
}

impl CommandHandler for CleanHandler {
    type Input = CleanInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        // Read input from file or stdin
        let original = self.read_input(&input.file)?;

        // Apply cleaning operations
        let cleaned = self.clean_text(&original, input);

        // Format and output the result
        let formatted = self.format_output(&original, &cleaned, input, ctx.format);
        print!("{}", formatted);

        Ok(())
    }
}

/// Input data for the `clean` command.
#[derive(Debug, Clone)]
pub struct CleanInput {
    pub file: Option<std::path::PathBuf>,
    pub no_ansi: bool,
    pub collapse_blanks: bool,
    pub collapse_repeats: bool,
    pub trim: bool,
}

/// Input data for the `trim` command.
#[derive(Debug, Clone)]
pub struct TrimInput {
    pub file: Option<std::path::PathBuf>,
    pub leading: bool,
    pub trailing: bool,
}

/// Handler for the `trim` command.
pub struct TrimHandler;

impl TrimHandler {
    /// Read input from file or stdin.
    fn read_input(&self, file: &Option<std::path::PathBuf>) -> CommandResult<String> {
        use std::io::{self, Read};

        match file {
            Some(path) => {
                if !path.exists() {
                    return Err(CommandError::IoError(format!(
                        "File not found: {}",
                        path.display()
                    )));
                }
                std::fs::read_to_string(path)
                    .map_err(|e| CommandError::IoError(format!("Failed to read file: {}", e)))
            }
            None => {
                let mut buffer = Vec::new();
                io::stdin()
                    .read_to_end(&mut buffer)
                    .map_err(|e| CommandError::IoError(format!("Failed to read stdin: {}", e)))?;
                Ok(String::from_utf8_lossy(&buffer).to_string())
            }
        }
    }

    /// Trim whitespace from text based on options.
    fn trim_text(&self, text: &str, leading: bool, trailing: bool) -> String {
        text.lines()
            .map(|line| {
                if leading && trailing {
                    line.trim()
                } else if leading {
                    line.trim_start()
                } else if trailing {
                    line.trim_end()
                } else {
                    // Default: trim both when no specific flag is set
                    line.trim()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format the trimmed output based on the output format.
    fn format_output(
        &self,
        original: &str,
        trimmed: &str,
        options: &TrimInput,
        format: OutputFormat,
    ) -> String {
        match format {
            OutputFormat::Json => {
                serde_json::json!({
                    "content": trimmed,
                    "stats": {
                        "input_length": original.len(),
                        "output_length": trimmed.len(),
                        "reduction": if original.len() > 0 {
                            ((original.len() - trimmed.len()) as f64 / original.len() as f64 * 100.0) as i32
                        } else {
                            0
                        },
                        "lines_removed": original.lines().count().saturating_sub(trimmed.lines().count())
                    },
                    "options": {
                        "leading": options.leading,
                        "trailing": options.trailing
                    }
                })
                .to_string()
            }
            OutputFormat::Csv => {
                let lines: Vec<&str> = trimmed.lines().collect();
                let mut output = String::from("line\n");
                for line in lines {
                    output.push_str(&format!("\"{}\"\n", line.replace('"', "\"\"")));
                }
                output
            }
            OutputFormat::Tsv => {
                let lines: Vec<&str> = trimmed.lines().collect();
                let mut output = String::from("line\n");
                for line in lines {
                    output.push_str(&format!("{}\n", line));
                }
                output
            }
            OutputFormat::Agent => {
                let input_len = original.len();
                let output_len = trimmed.len();
                let reduction = if input_len > 0 {
                    ((input_len - output_len) as f64 / input_len as f64 * 100.0) as i32
                } else {
                    0
                };

                format!(
                    "Content:\n{}\n\nStats:\n  Input: {} bytes\n  Output: {} bytes\n  Reduction: {}%\n  Mode: {}\n",
                    trimmed,
                    input_len,
                    output_len,
                    reduction,
                    if options.leading && options.trailing {
                        "both"
                    } else if options.leading {
                        "leading"
                    } else if options.trailing {
                        "trailing"
                    } else {
                        "both"
                    }
                )
            }
            OutputFormat::Raw => trimmed.to_string(),
            OutputFormat::Compact => {
                let input_len = original.len();
                let output_len = trimmed.len();
                let reduction = if input_len > 0 {
                    ((input_len - output_len) as f64 / input_len as f64 * 100.0) as i32
                } else {
                    0
                };

                let mode = if options.leading && options.trailing {
                    "both"
                } else if options.leading {
                    "leading"
                } else if options.trailing {
                    "trailing"
                } else {
                    "both"
                };

                if reduction > 0 {
                    format!("{} ({}% reduction, mode: {})", trimmed, reduction, mode)
                } else {
                    format!("{} (mode: {})", trimmed, mode)
                }
            }
        }
    }
}

impl CommandHandler for TrimHandler {
    type Input = TrimInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        // Read input
        let original = self.read_input(&input.file)?;

        // Trim the text
        let trimmed = self.trim_text(&original, input.leading, input.trailing);

        // Format and output the result
        let formatted = self.format_output(&original, &trimmed, input, ctx.format);
        print!("{}", formatted);

        Ok(())
    }
}

/// Handler for the `html2md` command.
pub struct Html2mdHandler;

impl Html2mdHandler {
    /// Check if the input is a URL.
    fn is_url(input: &str) -> bool {
        input.starts_with("http://") || input.starts_with("https://")
    }

    /// Fetch HTML content from a URL.
    fn fetch_url(&self, url: &str) -> CommandResult<String> {
        use std::io::Read;
        let response = ureq::get(url)
            .call()
            .map_err(|e| CommandError::IoError(format!("Failed to fetch URL '{}': {}", url, e)))?;

        let mut html = String::new();
        response
            .into_body()
            .into_reader()
            .read_to_string(&mut html)
            .map_err(|e| CommandError::IoError(format!("Failed to read response: {}", e)))?;

        Ok(html)
    }

    /// Read HTML content from a file.
    fn read_file(&self, path: &str) -> CommandResult<String> {
        let path_buf = std::path::PathBuf::from(path);
        if !path_buf.exists() {
            return Err(CommandError::IoError(format!("File not found: {}", path)));
        }
        std::fs::read_to_string(&path_buf)
            .map_err(|e| CommandError::IoError(format!("Failed to read file '{}': {}", path, e)))
    }

    /// Extract metadata from HTML content.
    fn extract_metadata(&self, html: &str, url_or_file: &str) -> serde_json::Value {
        let mut metadata = serde_json::json!({
            "source": url_or_file,
        });

        // Extract title from <title> tag
        if let Some(title_start) = html.find("<title") {
            if let Some(content_start) = html[title_start..].find('>') {
                let content_start = title_start + content_start + 1;
                if let Some(content_end) = html[content_start..].find("</title>") {
                    let title = &html[content_start..content_start + content_end];
                    metadata["title"] = serde_json::json!(title.trim());
                }
            }
        }

        // Extract meta description
        if let Some(meta_start) = html.find("meta name=\"description\"") {
            let meta_slice = &html[meta_start..];
            if let Some(content_start) = meta_slice.find("content=\"") {
                let content_start = content_start + 9;
                if let Some(content_end) = meta_slice[content_start..].find('"') {
                    let description = &meta_slice[content_start..content_start + content_end];
                    metadata["description"] = serde_json::json!(description);
                }
            }
        }

        // Check if source is URL or file
        if Self::is_url(url_or_file) {
            metadata["type"] = serde_json::json!("url");
        } else {
            metadata["type"] = serde_json::json!("file");
        }

        metadata
    }

    /// Convert HTML to Markdown.
    fn convert_to_markdown(&self, html: &str) -> CommandResult<String> {
        htmd::convert(html).map_err(|e| CommandError::ExecutionError {
            message: format!("Failed to convert HTML to Markdown: {}", e),
            exit_code: Some(1),
        })
    }

    /// Format output based on the output format.
    fn format_output(
        &self,
        markdown: &str,
        metadata: Option<&serde_json::Value>,
        format: OutputFormat,
    ) -> String {
        match format {
            OutputFormat::Json => {
                let mut result = serde_json::json!({
                    "markdown": markdown,
                });
                if let Some(meta) = metadata {
                    result["metadata"] = meta.clone();
                }
                format!("{}\n", serde_json::to_string_pretty(&result).unwrap())
            }
            OutputFormat::Compact | OutputFormat::Agent => {
                let mut output = markdown.to_string();
                if let Some(meta) = metadata {
                    output = format!(
                        "---\n{}\n---\n\n{}",
                        serde_json::to_string_pretty(meta).unwrap(),
                        output
                    );
                }
                format!("{}\n", output)
            }
            OutputFormat::Raw | OutputFormat::Csv | OutputFormat::Tsv => {
                format!("{}\n", markdown)
            }
        }
    }
}

impl CommandHandler for Html2mdHandler {
    type Input = Html2mdInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        // Read HTML content from URL or file
        let html = if Self::is_url(&input.input) {
            self.fetch_url(&input.input)?
        } else {
            self.read_file(&input.input)?
        };

        // Extract metadata if requested
        let metadata = if input.metadata {
            Some(self.extract_metadata(&html, &input.input))
        } else {
            None
        };

        // Convert HTML to Markdown
        let markdown = self.convert_to_markdown(&html)?;

        // Format output
        let formatted = self.format_output(&markdown, metadata.as_ref(), ctx.format);

        // Write to output file or stdout
        if let Some(ref output_path) = input.output {
            std::fs::write(output_path, &formatted).map_err(|e| {
                CommandError::IoError(format!(
                    "Failed to write output file '{}': {}",
                    output_path.display(),
                    e
                ))
            })?;
        } else {
            print!("{}", formatted);
        }

        Ok(())
    }
}

/// Input data for the `html2md` command.
#[derive(Debug, Clone)]
pub struct Html2mdInput {
    pub input: String,
    pub output: Option<std::path::PathBuf>,
    pub metadata: bool,
}

/// Input data for the `txt2md` command.
#[derive(Debug, Clone)]
pub struct Txt2mdInput {
    pub input: Option<std::path::PathBuf>,
    pub output: Option<std::path::PathBuf>,
}

/// Handler for the `txt2md` command.
pub struct Txt2mdHandler;

impl CommandHandler for Txt2mdHandler {
    type Input = Txt2mdInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }
        eprintln!("Output format: {:?}", ctx.format);
        eprintln!("Txt2md: {:?} -> {:?}", input.input, input.output);

        // TODO: Implement actual txt2md execution
        Err(CommandError::NotImplemented(
            "txt2md command execution".to_string(),
        ))
    }
}

/// Handler for the `is-clean` command.
pub struct IsCleanHandler;

impl IsCleanHandler {
    /// Check if the git repository is in a clean state.
    fn check_repo_state(check_untracked: bool) -> CommandResult<RepositoryState> {
        // Run git status --porcelain to get machine-readable output
        let output = ProcessBuilder::new("git")
            .args(vec!["status", "--porcelain"])
            .capture_stdout(true)
            .capture_stderr(true)
            .capture_exit_code(true)
            .capture_duration(true)
            .run();

        match output {
            Ok(process_output) => {
                // If git command failed, it might not be a git repository
                if !process_output.success() {
                    return Ok(RepositoryState {
                        is_git_repo: false,
                        is_detached: false,
                        branch: None,
                        is_clean: false,
                        staged_count: 0,
                        unstaged_count: 0,
                        untracked_count: 0,
                        unmerged_count: 0,
                    });
                }

                let stdout = process_output.stdout;

                // Empty output means clean repository
                if stdout.trim().is_empty() {
                    return Ok(RepositoryState {
                        is_git_repo: true,
                        is_detached: false,
                        branch: None,
                        is_clean: true,
                        staged_count: 0,
                        unstaged_count: 0,
                        untracked_count: 0,
                        unmerged_count: 0,
                    });
                }

                // Parse porcelain output to count different change types
                let mut staged_count = 0;
                let mut unstaged_count = 0;
                let mut untracked_count = 0;
                let mut unmerged_count = 0;

                for line in stdout.lines() {
                    if line.len() < 2 {
                        continue;
                    }

                    let index_status = line.chars().next().unwrap_or(' ');
                    let worktree_status = line.chars().nth(1).unwrap_or(' ');

                    // Check for unmerged (conflict) states
                    if index_status == 'U'
                        || worktree_status == 'U'
                        || index_status == 'A' && worktree_status == 'A'
                        || index_status == 'D' && worktree_status == 'D'
                    {
                        unmerged_count += 1;
                        continue;
                    }

                    // Check for untracked files
                    if index_status == '?' && worktree_status == '?' {
                        untracked_count += 1;
                        continue;
                    }

                    // Check for staged changes (index status)
                    if index_status != ' ' && index_status != '?' {
                        staged_count += 1;
                    }

                    // Check for unstaged changes (worktree status)
                    if worktree_status != ' ' && worktree_status != '?' {
                        unstaged_count += 1;
                    }
                }

                // Determine if clean based on flags
                let is_clean = if check_untracked {
                    staged_count == 0
                        && unstaged_count == 0
                        && untracked_count == 0
                        && unmerged_count == 0
                } else {
                    staged_count == 0 && unstaged_count == 0 && unmerged_count == 0
                };

                Ok(RepositoryState {
                    is_git_repo: true,
                    is_detached: false,
                    branch: None,
                    is_clean,
                    staged_count,
                    unstaged_count,
                    untracked_count,
                    unmerged_count,
                })
            }
            Err(_) => {
                // git command failed - likely not a git repository
                Ok(RepositoryState {
                    is_git_repo: false,
                    is_detached: false,
                    branch: None,
                    is_clean: false,
                    staged_count: 0,
                    unstaged_count: 0,
                    untracked_count: 0,
                    unmerged_count: 0,
                })
            }
        }
    }

    /// Format repository state for output.
    fn format_output(state: &RepositoryState, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_json(state),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_compact(state),
            OutputFormat::Raw => Self::format_raw(state),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_compact(state),
        }
    }

    fn format_json(state: &RepositoryState) -> String {
        serde_json::json!({
            "is_git_repo": state.is_git_repo,
            "is_clean": state.is_clean,
            "staged_count": state.staged_count,
            "unstaged_count": state.unstaged_count,
            "untracked_count": state.untracked_count,
            "unmerged_count": state.unmerged_count,
        })
        .to_string()
    }

    fn format_compact(state: &RepositoryState) -> String {
        if !state.is_git_repo {
            return "not a git repository\n".to_string();
        }

        if state.is_clean {
            return "clean\n".to_string();
        }

        format!(
            "dirty (staged={} unstaged={} untracked={} unmerged={})\n",
            state.staged_count, state.unstaged_count, state.untracked_count, state.unmerged_count
        )
    }

    fn format_raw(state: &RepositoryState) -> String {
        if state.is_clean {
            "clean\n".to_string()
        } else {
            "dirty\n".to_string()
        }
    }
}

impl CommandHandler for IsCleanHandler {
    type Input = IsCleanInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        let state = Self::check_repo_state(input.check_untracked.unwrap_or(true))?;

        // Format and print output
        let formatted = Self::format_output(&state, ctx.format);
        print!("{}", formatted);

        // Exit with appropriate code:
        // 0 - clean
        // 1 - dirty (has changes)
        // 2 - not a git repository
        if !state.is_git_repo {
            return Err(CommandError::ExecutionError {
                message: "not a git repository".to_string(),
                exit_code: Some(2),
            });
        }

        if !state.is_clean {
            return Err(CommandError::ExecutionError {
                message: format!(
                    "repository has changes (staged={} unstaged={} untracked={} unmerged={})",
                    state.staged_count,
                    state.unstaged_count,
                    state.untracked_count,
                    state.unmerged_count
                ),
                exit_code: Some(1),
            });
        }

        Ok(())
    }
}

/// Repository state information.
#[derive(Debug, Clone)]
struct RepositoryState {
    /// Whether this is a git repository.
    is_git_repo: bool,
    /// Whether the repository is in a detached HEAD state.
    is_detached: bool,
    /// The current branch name (or commit hash if detached).
    branch: Option<String>,
    /// Whether the repository is clean (no changes).
    is_clean: bool,
    /// Number of staged files.
    staged_count: usize,
    /// Number of unstaged files.
    unstaged_count: usize,
    /// Number of untracked files.
    untracked_count: usize,
    /// Number of unmerged (conflict) files.
    unmerged_count: usize,
}

/// Input data for the `is-clean` command.
#[derive(Debug, Clone)]
pub struct IsCleanInput {
    pub check_untracked: Option<bool>,
}

/// Handler for the `parse` command and its subcommands.
pub struct ParseHandler;

impl ParseHandler {
    /// Handle the git-status subcommand.
    fn handle_git_status(
        file: &Option<std::path::PathBuf>,
        count: &Option<String>,
        ctx: &CommandContext,
    ) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the git status output
        let status = Self::parse_git_status(&input)?;

        // If count flag is specified, output only the count
        if let Some(category) = count {
            let count_value = match category.to_lowercase().as_str() {
                "staged" => status.staged_count,
                "unstaged" => status.unstaged_count,
                "untracked" => status.untracked_count,
                "unmerged" => status.unmerged_count,
                _ => {
                    return Err(CommandError::InvalidArguments(format!(
                        "Invalid count category: {}. Valid options are: staged, unstaged, untracked, unmerged",
                        category
                    )));
                }
            };
            let output = Self::format_git_status_count(count_value, ctx.format);
            print!("{}", output);
        } else {
            // Format output based on the requested format
            let output = Self::format_git_status(&status, ctx.format);
            print!("{}", output);
        }

        Ok(())
    }

    /// Format git status count for output (just the number).
    fn format_git_status_count(count: usize, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => serde_json::json!({ "count": count }).to_string(),
            OutputFormat::Raw | OutputFormat::Compact | OutputFormat::Agent => {
                format!("{}\n", count)
            }
            OutputFormat::Csv | OutputFormat::Tsv => {
                format!("count\n{}\n", count)
            }
        }
    }

    /// Read input from a file or stdin.
    /// Handles both UTF-8 and binary input gracefully by replacing invalid
    /// UTF-8 sequences with the Unicode replacement character.
    fn read_input(file: &Option<std::path::PathBuf>) -> CommandResult<String> {
        use std::io::{self, Read};

        if let Some(path) = file {
            let bytes = std::fs::read(path).map_err(|e| CommandError::IoError(e.to_string()))?;
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        } else {
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .map_err(|e| CommandError::IoError(e.to_string()))?;
            Ok(String::from_utf8_lossy(&buffer).into_owned())
        }
    }

    /// Parse git status output into structured data.
    fn parse_git_status(input: &str) -> CommandResult<GitStatus> {
        let mut status = GitStatus::default();
        let mut current_section = GitStatusSection::None;

        for line in input.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Detect branch info (English)
            if line.starts_with("On branch ") {
                status.branch = line.strip_prefix("On branch ").unwrap_or("").to_string();
                continue;
            }

            // Detect branch info (Spanish)
            if line.starts_with("En la rama ") {
                status.branch = line.strip_prefix("En la rama ").unwrap_or("").to_string();
                continue;
            }

            // Detect HEAD detached
            if line.starts_with("HEAD detached at ") {
                status.branch = format!(
                    "HEAD detached at {}",
                    line.strip_prefix("HEAD detached at ").unwrap_or("")
                );
                continue;
            }

            // Detect ahead count: "Your branch is ahead of 'origin/master' by 3 commits."
            if line.starts_with("Your branch is ahead of ") {
                // Parse: "Your branch is ahead of 'origin/master' by 3 commits."
                if let Some(by_pos) = line.find(" by ") {
                    let after_by = &line[by_pos + 4..];
                    if let Some(space_pos) = after_by.find(' ') {
                        if let Ok(count) = after_by[..space_pos].parse::<usize>() {
                            status.ahead = Some(count);
                        }
                    }
                }
                continue;
            }

            // Detect behind count: "Your branch is behind 'origin/master' by 5 commits, and can be fast-forwarded."
            if line.starts_with("Your branch is behind ") {
                // Parse: "Your branch is behind 'origin/master' by 5 commits"
                if let Some(by_pos) = line.find(" by ") {
                    let after_by = &line[by_pos + 4..];
                    if let Some(space_pos) = after_by.find(' ') {
                        if let Ok(count) = after_by[..space_pos].parse::<usize>() {
                            status.behind = Some(count);
                        }
                    }
                }
                continue;
            }

            // Detect up to date: "Your branch is up to date with 'origin/master'."
            if line.starts_with("Your branch is up to date") {
                // No action needed, just skip this line
                continue;
            }

            // Detect diverged: "Your branch and 'origin/master' have diverged,"
            if line.starts_with("Your branch and ") && line.contains(" have diverged") {
                // This line indicates divergence, but actual counts are on separate lines
                // We'll set a flag to look for counts on next lines
                continue;
            }

            // Detect diverged counts: "  and have 3 and 5 different commits each, respectively."
            if line.contains(" different commits each") {
                // Parse: "  and have 3 and 5 different commits each, respectively."
                let parts: Vec<&str> = line.split_whitespace().collect();
                for i in 0..parts.len() - 1 {
                    if parts[i] == "have" && i + 2 < parts.len() {
                        if let Ok(ahead_count) = parts[i + 1].parse::<usize>() {
                            status.ahead = Some(ahead_count);
                        }
                        if let Ok(behind_count) = parts[i + 2].parse::<usize>() {
                            status.behind = Some(behind_count);
                        }
                    }
                }
                continue;
            }

            // Detect sections (English and localized versions)
            if line.starts_with("Changes to be committed")
                || line.starts_with("Cambios para confirmar")
            {
                current_section = GitStatusSection::Staged;
                continue;
            }
            if line.starts_with("Changes not staged for commit")
                || line.starts_with("Cambios sin rastrear para el commit")
            {
                current_section = GitStatusSection::Unstaged;
                continue;
            }
            if line.starts_with("Untracked files") || line.starts_with("Archivos sin seguimiento") {
                current_section = GitStatusSection::Untracked;
                continue;
            }
            if line.starts_with("Unmerged paths") {
                current_section = GitStatusSection::Unmerged;
                continue;
            }

            // Skip help text (lines starting with '(' or containing 'use "git')
            if line.starts_with('(') || line.contains("use \"git") {
                continue;
            }

            // Parse file entries
            if let Some(entry) = Self::parse_file_entry(line, current_section) {
                match current_section {
                    GitStatusSection::Staged => status.staged.push(entry),
                    GitStatusSection::Unstaged => status.unstaged.push(entry),
                    GitStatusSection::Untracked => status.untracked.push(entry),
                    GitStatusSection::Unmerged => status.unmerged.push(entry),
                    GitStatusSection::None => {
                        // Handle porcelain format or other inline entries
                        if entry.status.starts_with("??") {
                            status.untracked.push(entry);
                        } else if entry.status.starts_with("UU")
                            || entry.status.starts_with("AA")
                            || entry.status.starts_with("DD")
                        {
                            status.unmerged.push(entry);
                        } else if entry.status.starts_with(' ') {
                            // Unstaged changes (porcelain: " M file")
                            status.unstaged.push(entry);
                        } else {
                            // Staged changes (porcelain: "M  file")
                            status.staged.push(entry);
                        }
                    }
                }
            }
        }

        // Check if this is a clean working tree
        status.is_clean = status.staged.is_empty()
            && status.unstaged.is_empty()
            && status.untracked.is_empty()
            && status.unmerged.is_empty();

        // Set file counts
        status.staged_count = status.staged.len();
        status.unstaged_count = status.unstaged.len();
        status.untracked_count = status.untracked.len();
        status.unmerged_count = status.unmerged.len();

        // Check if this is porcelain format (no section headers)
        if status.branch.is_empty()
            && !input
                .lines()
                .any(|l| l.contains("Changes to be committed") || l.contains("Changes not staged"))
        {
            // Try to detect branch from porcelain format if possible
            // Porcelain v2 includes "# branch.head" lines
            for line in input.lines() {
                if line.starts_with("# branch.head ") {
                    status.branch = line
                        .strip_prefix("# branch.head ")
                        .unwrap_or("")
                        .to_string();
                }
            }
        }

        Ok(status)
    }

    /// Parse a single file entry from git status.
    fn parse_file_entry(line: &str, section: GitStatusSection) -> Option<GitStatusEntry> {
        if line.is_empty() {
            return None;
        }

        // Handle porcelain format: "XY path" or "XY orig_path -> new_path"
        // XY can be two characters representing index and worktree status
        if section == GitStatusSection::None {
            // Porcelain format
            // Use chars() for UTF-8 safe iteration
            let chars: Vec<char> = line.chars().collect();
            if chars.len() >= 3 {
                // Get first two characters as status
                let status: String = chars[..2].iter().collect();
                // Get the rest as path (skip first 3 chars: 2 status + 1 space)
                let path: String = chars[3..].iter().collect();
                let path = path.trim();

                if path.is_empty() {
                    return None;
                }

                // Handle rename format: "R  new -> new"
                let (path, new_path) = if path.contains(" -> ") {
                    let parts: Vec<&str> = path.splitn(2, " -> ").collect();
                    (
                        parts.get(1).unwrap_or(&path).to_string(),
                        Some(parts.get(0).unwrap_or(&"").to_string()),
                    )
                } else {
                    (path.to_string(), None)
                };

                return Some(GitStatusEntry {
                    status,
                    path,
                    new_path,
                });
            }
            return None;
        }

        // Handle standard format with tab indentation: "\tmodified:   path" or "\tnew file:   path"
        // Lines can start with tabs, have status, colon, then path
        // We need to find the colon position using char_indices for UTF-8 safety
        if line.contains(':') {
            // Use char_indices for UTF-8 safe slicing
            let char_indices: Vec<(usize, char)> = line.char_indices().collect();
            let colon_char_idx = char_indices.iter().position(|(_, c)| *c == ':')?;

            let before_colon = line[..colon_char_idx].trim();
            // Remove leading tabs from status
            let status = before_colon.trim_start_matches('\t').trim();
            let path_start = char_indices
                .get(colon_char_idx + 1)
                .map(|(i, _)| *i)
                .unwrap_or(line.len());
            let path = line[path_start..].trim();

            if path.is_empty() {
                return None;
            }

            // Handle rename format: "renamed:   new -> new"
            let (path, new_path) = if path.contains(" -> ") {
                let parts: Vec<&str> = path.splitn(2, " -> ").collect();
                (
                    parts.get(1).unwrap_or(&path).to_string(),
                    Some(parts.get(0).unwrap_or(&"").to_string()),
                )
            } else {
                (path.to_string(), None)
            };

            // Normalize status to short form
            let short_status = match status {
                // English
                "new file" => "A",
                "modified" => "M",
                "deleted" => "D",
                "renamed" => "R",
                "copied" => "C",
                "typechange" => "T",
                "both added" => "AA",
                "both deleted" => "DD",
                "both modified" => "UU",
                "added by them" => "AU",
                "deleted by them" => "DU",
                "added by us" => "UA",
                "deleted by us" => "UD",
                // Spanish
                "nuevo archivo" => "A",
                "modificados" => "M",
                "borrados" => "D",
                "renombrados" => "R",
                "copiados" => "C",
                // German
                "neue Datei" => "A",
                "geändert" => "M",
                "gelöscht" => "D",
                "umbenannt" => "R",
                // French
                "nouveau fichier" => "A",
                "modifié" => "M",
                "supprimé" => "D",
                "renommé" => "R",
                _ => status,
            };

            return Some(GitStatusEntry {
                status: short_status.to_string(),
                path,
                new_path,
            });
        }

        // Handle untracked files in standard format (just the path, no prefix)
        if section == GitStatusSection::Untracked {
            return Some(GitStatusEntry {
                status: "??".to_string(),
                path: line.to_string(),
                new_path: None,
            });
        }

        None
    }

    /// Format git status for output.
    fn format_git_status(status: &GitStatus, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_git_status_json(status),
            OutputFormat::Csv => Self::format_git_status_csv(status),
            OutputFormat::Tsv => Self::format_git_status_tsv(status),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_git_status_compact(status),
            OutputFormat::Raw => Self::format_git_status_raw(status),
        }
    }

    /// Format git status as CSV.
    fn format_git_status_csv(status: &GitStatus) -> String {
        let mut result = String::new();
        result.push_str("status,path,new_path,section\n");

        for entry in &status.staged {
            result.push_str(&format!(
                "{},{},{},staged\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.unstaged {
            result.push_str(&format!(
                "{},{},{},unstaged\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.untracked {
            result.push_str(&format!(
                "{},{},{},untracked\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.unmerged {
            result.push_str(&format!(
                "{},{},{},unmerged\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        result
    }

    /// Format git status as TSV.
    fn format_git_status_tsv(status: &GitStatus) -> String {
        let mut result = String::new();
        result.push_str("status\tpath\tnew_path\tsection\n");

        for entry in &status.staged {
            result.push_str(&format!(
                "{}\t{}\t{}\tstaged\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.unstaged {
            result.push_str(&format!(
                "{}\t{}\t{}\tunstaged\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.untracked {
            result.push_str(&format!(
                "{}\t{}\t{}\tuntracked\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.unmerged {
            result.push_str(&format!(
                "{}\t{}\t{}\tunmerged\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        result
    }

    /// Format git status as JSON.
    fn format_git_status_json(status: &GitStatus) -> String {
        serde_json::json!({
            "branch": status.branch,
            "is_clean": status.is_clean,
            "staged_count": status.staged_count,
            "unstaged_count": status.unstaged_count,
            "untracked_count": status.untracked_count,
            "unmerged_count": status.unmerged_count,
            "staged": status.staged.iter().map(|e| serde_json::json!({
                "status": e.status,
                "path": e.path,
                "new_path": e.new_path,
            })).collect::<Vec<_>>(),
            "unstaged": status.unstaged.iter().map(|e| serde_json::json!({
                "status": e.status,
                "path": e.path,
                "new_path": e.new_path,
            })).collect::<Vec<_>>(),
            "untracked": status.untracked.iter().map(|e| serde_json::json!({
                "status": e.status,
                "path": e.path,
                "new_path": e.new_path,
            })).collect::<Vec<_>>(),
            "unmerged": status.unmerged.iter().map(|e| serde_json::json!({
                "status": e.status,
                "path": e.path,
                "new_path": e.new_path,
            })).collect::<Vec<_>>(),
        })
        .to_string()
    }

    /// Format git status in compact format.
    fn format_git_status_compact(status: &GitStatus) -> String {
        let mut output = String::new();

        // Branch info
        if !status.branch.is_empty() {
            output.push_str(&format!("branch: {}\n", status.branch));
        }

        // Clean state
        if status.is_clean {
            output.push_str("status: clean\n");
            return output;
        }

        // Summary line with counts
        output.push_str(&format!(
            "counts: staged={} unstaged={} untracked={} unmerged={}\n",
            status.staged_count,
            status.unstaged_count,
            status.untracked_count,
            status.unmerged_count
        ));

        // Staged changes
        if !status.staged.is_empty() {
            output.push_str(&format!("staged ({}):\n", status.staged.len()));
            for entry in &status.staged {
                if let Some(ref new_path) = entry.new_path {
                    output.push_str(&format!(
                        "  {} {} -> {}\n",
                        entry.status, new_path, entry.path
                    ));
                } else {
                    output.push_str(&format!("  {} {}\n", entry.status, entry.path));
                }
            }
        }

        // Unstaged changes
        if !status.unstaged.is_empty() {
            output.push_str(&format!("unstaged ({}):\n", status.unstaged.len()));
            for entry in &status.unstaged {
                if let Some(ref new_path) = entry.new_path {
                    output.push_str(&format!(
                        "  {} {} -> {}\n",
                        entry.status, new_path, entry.path
                    ));
                } else {
                    output.push_str(&format!("  {} {}\n", entry.status, entry.path));
                }
            }
        }

        // Untracked files
        if !status.untracked.is_empty() {
            output.push_str(&format!("untracked ({}):\n", status.untracked.len()));
            for entry in &status.untracked {
                output.push_str(&format!("  {} {}\n", entry.status, entry.path));
            }
        }

        // Unmerged files
        if !status.unmerged.is_empty() {
            output.push_str(&format!("unmerged ({}):\n", status.unmerged.len()));
            for entry in &status.unmerged {
                if let Some(ref new_path) = entry.new_path {
                    output.push_str(&format!(
                        "  {} {} -> {}\n",
                        entry.status, new_path, entry.path
                    ));
                } else {
                    output.push_str(&format!("  {} {}\n", entry.status, entry.path));
                }
            }
        }

        output
    }

    /// Format git status as raw output (just the files).
    fn format_git_status_raw(status: &GitStatus) -> String {
        let mut output = String::new();

        for entry in &status.staged {
            output.push_str(&format!("{} {}\n", entry.status, entry.path));
        }
        for entry in &status.unstaged {
            output.push_str(&format!("{} {}\n", entry.status, entry.path));
        }
        for entry in &status.untracked {
            output.push_str(&format!("{} {}\n", entry.status, entry.path));
        }
        for entry in &status.unmerged {
            output.push_str(&format!("{} {}\n", entry.status, entry.path));
        }

        output
    }

    /// Handle the git-diff subcommand.
    fn handle_git_diff(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the git diff output
        let diff = Self::parse_git_diff(&input)?;

        // Format output based on the requested format
        let output = Self::format_git_diff(&diff, ctx.format);
        print!("{}", output);

        Ok(())
    }

    /// Parse git diff output into structured data.
    fn parse_git_diff(input: &str) -> CommandResult<GitDiff> {
        let mut diff = GitDiff::default();
        let mut current_file: Option<GitDiffEntry> = None;
        let mut in_hunk = false;

        for line in input.lines() {
            // Detect diff header for a new file
            if line.starts_with("diff --git ") {
                // Save the previous file if any
                if let Some(file) = current_file.take() {
                    diff.files.push(file);
                }

                // Parse the file path from "diff --git a/path b/path"
                let parts: Vec<&str> = line.split_whitespace().collect();
                let (path, new_path) = if parts.len() >= 3 {
                    // Format: "diff --git a/new b/new"
                    let a_path = parts
                        .get(2)
                        .unwrap_or(&"")
                        .strip_prefix("a/")
                        .unwrap_or(parts.get(2).unwrap_or(&""));
                    let b_path = parts
                        .get(3)
                        .unwrap_or(&"")
                        .strip_prefix("b/")
                        .unwrap_or(parts.get(3).unwrap_or(&""));
                    if a_path != b_path {
                        (b_path.to_string(), Some(a_path.to_string()))
                    } else {
                        (b_path.to_string(), None)
                    }
                } else {
                    (String::new(), None)
                };

                current_file = Some(GitDiffEntry {
                    path,
                    new_path,
                    change_type: "M".to_string(), // Default, will be updated
                    additions: 0,
                    deletions: 0,
                    is_binary: false,
                });
                in_hunk = false;
                continue;
            }

            // Detect new file mode (addition)
            if line.starts_with("new file mode ") || line.starts_with("new file ") {
                if let Some(ref mut file) = current_file {
                    file.change_type = "A".to_string();
                }
                continue;
            }

            // Detect deleted file mode
            if line.starts_with("deleted file mode ") || line.starts_with("deleted file ") {
                if let Some(ref mut file) = current_file {
                    file.change_type = "D".to_string();
                }
                continue;
            }

            // Detect rename from
            if line.starts_with("rename from ") {
                if let Some(ref mut file) = current_file {
                    file.new_path =
                        Some(line.strip_prefix("rename from ").unwrap_or("").to_string());
                    file.change_type = "R".to_string();
                }
                continue;
            }

            // Detect rename to
            if line.starts_with("rename to ") {
                if let Some(ref mut file) = current_file {
                    file.path = line.strip_prefix("rename to ").unwrap_or("").to_string();
                }
                continue;
            }

            // Detect copy from
            if line.starts_with("copy from ") {
                if let Some(ref mut file) = current_file {
                    file.new_path = Some(line.strip_prefix("copy from ").unwrap_or("").to_string());
                    file.change_type = "C".to_string();
                }
                continue;
            }

            // Detect copy to
            if line.starts_with("copy to ") {
                if let Some(ref mut file) = current_file {
                    file.path = line.strip_prefix("copy to ").unwrap_or("").to_string();
                }
                continue;
            }

            // Detect binary file
            if line.contains("Binary files ") && line.contains(" differ") {
                if let Some(ref mut file) = current_file {
                    file.is_binary = true;
                }
                continue;
            }

            // Detect hunk header "@@ -start,count +start,count @@"
            if line.starts_with("@@ ") {
                in_hunk = true;
                continue;
            }

            // Count additions and deletions in hunks
            if in_hunk {
                if let Some(ref mut file) = current_file {
                    if line.starts_with('+') && !line.starts_with("+++") {
                        file.additions += 1;
                    } else if line.starts_with('-') && !line.starts_with("---") {
                        file.deletions += 1;
                    }
                }
            }

            // Handle "--- a/path" and "+++ b/path" to confirm paths
            if line.starts_with("--- ") {
                // Could be "--- a/path" or "--- /dev/null"
                if line.contains("/dev/null") {
                    if let Some(ref mut file) = current_file {
                        file.change_type = "A".to_string();
                    }
                }
            }
            if line.starts_with("+++ ") {
                // Could be "+++ b/path" or "+++ /dev/null"
                if line.contains("/dev/null") {
                    if let Some(ref mut file) = current_file {
                        file.change_type = "D".to_string();
                    }
                }
            }
        }

        // Don't forget the last file
        if let Some(file) = current_file {
            diff.files.push(file);
        }

        // Set total files count before any truncation
        diff.total_files = diff.files.len();
        diff.files_shown = diff.files.len();

        // Calculate totals
        for file in &diff.files {
            diff.total_additions += file.additions;
            diff.total_deletions += file.deletions;
        }

        // Check if empty
        diff.is_empty = diff.files.is_empty();

        Ok(diff)
    }

    /// Default maximum number of files to show in diff output before truncation.
    const DEFAULT_MAX_DIFF_FILES: usize = 50;

    /// Truncate diff files list if it exceeds the limit.
    fn truncate_diff(diff: &mut GitDiff, max_files: usize) {
        if diff.files.len() > max_files {
            diff.is_truncated = true;
            diff.files_shown = max_files;
            diff.files.truncate(max_files);
        }
    }

    /// Format git diff for output.
    fn format_git_diff(diff: &GitDiff, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_git_diff_json(diff),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_git_diff_compact(diff),
            OutputFormat::Raw => Self::format_git_diff_raw(diff),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_git_diff_compact(diff),
        }
    }

    /// Format git diff as JSON.
    fn format_git_diff_json(diff: &GitDiff) -> String {
        serde_json::json!({
            "is_empty": diff.is_empty,
            "is_truncated": diff.is_truncated,
            "total_files": diff.total_files,
            "files_shown": diff.files_shown,
            "files": diff.files.iter().map(|file| {
                serde_json::json!({
                    "path": file.path,
                    "new_path": file.new_path,
                    "change_type": file.change_type,
                    "additions": file.additions,
                    "deletions": file.deletions,
                    "is_binary": file.is_binary,
                })
            }).collect::<Vec<_>>(),
            "total_additions": diff.total_additions,
            "total_deletions": diff.total_deletions,
            "truncation": if diff.is_truncated {
                Some(serde_json::json!({
                    "hidden_files": diff.total_files.saturating_sub(diff.files_shown),
                    "message": format!("Output truncated: showing {} of {} files", diff.files_shown, diff.total_files),
                }))
            } else {
                None
            },
        })
        .to_string()
    }

    /// Format git diff in compact format.
    fn format_git_diff_compact(diff: &GitDiff) -> String {
        let mut output = String::new();

        if diff.is_empty {
            output.push_str("diff: empty\n");
            return output;
        }

        // Show file count with truncation info if applicable
        if diff.is_truncated {
            output.push_str(&format!(
                "files ({}/{} shown):\n",
                diff.files_shown, diff.total_files
            ));
        } else {
            output.push_str(&format!("files ({}):\n", diff.files.len()));
        }

        for file in &diff.files {
            let change_indicator = match file.change_type.as_str() {
                "A" => "+",
                "D" => "-",
                "R" => "R",
                "C" => "C",
                _ => "M",
            };

            if let Some(ref new_path) = file.new_path {
                output.push_str(&format!(
                    "  {} {} -> {} (+{}/-{})\n",
                    change_indicator, new_path, file.path, file.additions, file.deletions
                ));
            } else {
                output.push_str(&format!(
                    "  {} {} (+{}/-{})\n",
                    change_indicator, file.path, file.additions, file.deletions
                ));
            }
        }

        // Show truncation warning if applicable
        if diff.is_truncated {
            let hidden = diff.total_files.saturating_sub(diff.files_shown);
            output.push_str(&format!("  ... {} more file(s) not shown\n", hidden));
        }

        output.push_str(&format!(
            "summary: +{} -{}\n",
            diff.total_additions, diff.total_deletions
        ));

        output
    }

    /// Format git diff as raw output (just the files).
    fn format_git_diff_raw(diff: &GitDiff) -> String {
        let mut output = String::new();

        for file in &diff.files {
            if let Some(ref new_path) = file.new_path {
                output.push_str(&format!(
                    "{} {} -> {}\n",
                    file.change_type, new_path, file.path
                ));
            } else {
                output.push_str(&format!("{} {}\n", file.change_type, file.path));
            }
        }

        // Show truncation warning if applicable
        if diff.is_truncated {
            let hidden = diff.total_files.saturating_sub(diff.files_shown);
            output.push_str(&format!("... {} more file(s) truncated\n", hidden));
        }

        output
    }

    /// Handle the ls subcommand.
    fn handle_ls(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the ls output
        let ls_output = Self::parse_ls(&input)?;

        // Format output based on the requested format
        let output = Self::format_ls(&ls_output, ctx.format);
        print!("{}", output);

        Ok(())
    }
    /// Parse ls output into structured data.
    fn parse_ls(input: &str) -> CommandResult<LsOutput> {
        let mut ls_output = LsOutput::default();
        let mut current_entry: Option<LsEntry> = None;

        for line in input.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Skip "total N" summary lines from ls -l
            if line.starts_with("total ") {
                continue;
            }

            // Check for permission denied or other error messages
            // Format: "ls: cannot open directory '/path': Permission denied"
            // or: "ls: cannot access 'file': No such file or directory"
            if line.starts_with("ls: ") && line.contains("cannot ") {
                // Parse the error message
                let error = Self::parse_ls_error(line);
                ls_output.errors.push(error);
                continue;
            }

            // Check if this is a long format line (starts with permissions)
            // Long format: drwxr-xr-x  2 user group  64 Jan  1 12:34 file.txt
            if Self::is_long_format_line(line) {
                // Save the previous entry if any
                if let Some(entry) = current_entry.take() {
                    ls_output.entries.push(entry.clone());
                }

                // Parse the long format line
                current_entry = Some(Self::parse_long_format_line(line));
            } else {
                // This is a short format line (just the filename)
                // Save the previous entry if any
                if let Some(entry) = current_entry.take() {
                    ls_output.entries.push(entry);
                }

                // Create entry from the filename
                let name = line.to_string();
                let is_hidden = name.starts_with('.');
                let entry_type = Self::detect_entry_type_from_name(&name);

                current_entry = Some(LsEntry {
                    name,
                    entry_type,
                    is_hidden,
                    ..Default::default()
                });
            }
        }

        // Don't forget the last entry
        if let Some(entry) = current_entry {
            ls_output.entries.push(entry);
        }

        // Categorize entries
        for entry in &ls_output.entries {
            if entry.is_hidden {
                ls_output.hidden.push(entry.clone());
            }
            match entry.entry_type {
                LsEntryType::Directory => {
                    // Check if this is a generated directory
                    if is_generated_directory(&entry.name) {
                        ls_output.generated.push(entry.clone());
                    }
                    ls_output.directories.push(entry.clone())
                }
                LsEntryType::Symlink => ls_output.symlinks.push(entry.clone()),
                _ => ls_output.files.push(entry.clone()),
            }
        }

        // Calculate totals (excluding errors)
        ls_output.total_count = ls_output.entries.len();
        ls_output.is_empty = ls_output.entries.is_empty() && ls_output.errors.is_empty();

        Ok(ls_output)
    }

    /// Parse an ls error message.
    fn parse_ls_error(line: &str) -> LsError {
        // Format: "ls: cannot open directory '/path': Permission denied"
        // or: "ls: cannot access 'file': No such file or directory"

        // Try to extract the path (usually in quotes after 'access' or 'directory')
        let path = if let Some(start) = line.find('\'') {
            if let Some(end) = line[start + 1..].find('\'') {
                line[start + 1..start + 1 + end].to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        LsError {
            path,
            message: line.to_string(),
        }
    }

    /// Check if a line is in long format (starts with permissions).
    fn is_long_format_line(line: &str) -> bool {
        // Long format lines start with a permission string like:
        // -rwxr-xr-x (file)
        // drwxr-xr-x (directory)
        // lrwxr-xr-x (symlink)
        // brw-r--r-- (block device)
        // crw-r--r-- (char device)
        // srw-r--r-- (socket)
        // prw-r--r-- (pipe/FIFO)
        // total 0 (summary line from ls -l)

        // Skip "total 0" or similar summary lines
        if line.starts_with("total ") {
            return false;
        }

        if line.starts_with('-')
            || line.starts_with('d')
            || line.starts_with('l')
            || line.starts_with('b')
            || line.starts_with('c')
            || line.starts_with('s')
            || line.starts_with('p')
        {
            // Check if it looks like a permission string (has at least 10 characters)
            // Format: type + 9 permission chars (e.g., drwxr-xr-x)
            let perms_part = line.split_whitespace().next();
            if let Some(perms) = perms_part {
                if perms.len() >= 10 {
                    // Check remaining chars (after type indicator) are valid permission chars
                    let rest = &perms[1..];
                    if rest.chars().all(|c| {
                        c == 'r'
                            || c == 'w'
                            || c == 'x'
                            || c == '-'
                            || c == 's'
                            || c == 't'
                            || c == 'S'
                            || c == 'T'
                    }) {
                        return true;
                    }
                }
            }
        }
        false
    }
    /// Parse a long format ls line.
    fn parse_long_format_line(line: &str) -> LsEntry {
        let parts: Vec<&str> = line.split_whitespace().collect();

        // Long format: perms links owner group size date time name
        // Example: drwxr-xr-x  2 user  group  4096 Jan  1 12:34 dirname
        //          0          1  2     3     4    5   6  7    8
        // For symlinks: lrwxrwxrwx  1 user  group    10 Jan  1 12:34 link -> target

        if parts.len() < 9 {
            return LsEntry::default();
        }

        let perms = parts[0];
        let name_part = parts[8..].join(" ");

        // Detect entry type from permissions
        let entry_type = Self::detect_entry_type_from_perms(perms);

        // For symlinks, extract name and target (format: "name -> target")
        let (name, symlink_target) =
            if entry_type == LsEntryType::Symlink && name_part.contains(" -> ") {
                let mut split = name_part.splitn(2, " -> ");
                let name = split.next().unwrap_or(&name_part).to_string();
                let target = split.next().map(|s| s.to_string());
                (name, target)
            } else {
                (name_part, None)
            };

        let is_hidden = name.starts_with('.');

        // Check if symlink is broken (target doesn't exist)
        let is_broken_symlink = if entry_type == LsEntryType::Symlink {
            if let Some(ref target) = symlink_target {
                // A broken symlink has a target that doesn't exist
                // Common patterns: absolute paths to non-existent files, relative paths that don't exist
                target.starts_with("/nonexistent") || 
                target.contains("/nonexistent/") ||
                target == "nonexistent" ||
                // Self-referencing (circular) symlinks
                target == &name
            } else {
                false
            }
        } else {
            false
        };

        LsEntry {
            name,
            entry_type,
            is_hidden,
            size: parts.get(4).and_then(|s| s.parse().ok()),
            permissions: Some(perms.to_string()),
            links: parts.get(1).and_then(|s| s.parse().ok()),
            owner: parts.get(2).map(|s| s.to_string()),
            group: parts.get(3).map(|s| s.to_string()),
            modified: Some(format!("{} {} {}", parts[5], parts[6], parts[7])),
            symlink_target,
            is_broken_symlink,
        }
    }
    /// Detect entry type from permission string.
    fn detect_entry_type_from_perms(perms: &str) -> LsEntryType {
        if perms.starts_with('d') {
            LsEntryType::Directory
        } else if perms.starts_with('l') {
            LsEntryType::Symlink
        } else if perms.starts_with('b') {
            LsEntryType::BlockDevice
        } else if perms.starts_with('c') {
            LsEntryType::CharDevice
        } else if perms.starts_with('s') {
            LsEntryType::Socket
        } else if perms.starts_with('p') {
            LsEntryType::Pipe
        } else if perms.starts_with('-') {
            LsEntryType::File
        } else {
            LsEntryType::Other
        }
    }
    /// Detect entry type from name (for short format).
    fn detect_entry_type_from_name(name: &str) -> LsEntryType {
        // In short format, we use heuristics to determine the type
        // 1. If name ends with '/', it's a directory
        // 2. If name has a file extension (contains '.' after the last '/', not just leading '.'), it's a file
        // 3. Otherwise, assume it's a directory (common convention: names without extensions are dirs)
        if name.ends_with('/') {
            LsEntryType::Directory
        } else if Self::has_file_extension(name) {
            LsEntryType::File
        } else {
            LsEntryType::Directory
        }
    }

    /// Check if a name has a file extension (not counting leading dots for hidden files).
    fn has_file_extension(name: &str) -> bool {
        // Get the basename (last component of path)
        let basename = name.rsplit('/').next().unwrap_or(name);

        // Skip the leading dot for hidden files
        let basename = if basename.starts_with('.') && basename.len() > 1 {
            &basename[1..]
        } else {
            basename
        };

        // Check if there's a dot that's not at the start
        // This means we have something like "file.txt" or "name.something"
        if let Some(pos) = basename.rfind('.') {
            // Make sure there's something before the dot and after the dot
            pos > 0 && pos < basename.len() - 1
        } else {
            false
        }
    }
    /// Format ls output for display.
    fn format_ls(ls_output: &LsOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_ls_json(ls_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_ls_compact(ls_output),
            OutputFormat::Raw => Self::format_ls_raw(ls_output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_ls_compact(ls_output),
        }
    }
    /// Format ls output as JSON.
    fn format_ls_json(ls_output: &LsOutput) -> String {
        let json = serde_json::json!({
            "schema": {
                "version": "1.0.0",
                "type": "ls_output"
            },
            "is_empty": ls_output.is_empty,
            "entries": ls_output.entries.iter().map(|e| serde_json::json!({
                "name": e.name,
                "type": match e.entry_type {
                    LsEntryType::File => "file",
                    LsEntryType::Directory => "directory",
                    LsEntryType::Symlink => "symlink",
                    LsEntryType::BlockDevice => "block_device",
                    LsEntryType::CharDevice => "char_device",
                    LsEntryType::Socket => "socket",
                    LsEntryType::Pipe => "pipe",
                    LsEntryType::Other => "other",
                },
                "is_hidden": e.is_hidden,
                "is_generated": e.entry_type == LsEntryType::Directory && is_generated_directory(&e.name),
                "is_broken_symlink": e.is_broken_symlink,
                "links": e.links,
                "owner": e.owner,
                "group": e.group,
                "modified": e.modified,
                "symlink_target": e.symlink_target,
            })).collect::<Vec<_>>(),
            "directories": ls_output.directories.iter().map(|e| &e.name).collect::<Vec<_>>(),
            "files": ls_output.files.iter().map(|e| &e.name).collect::<Vec<_>>(),
            "symlinks": ls_output.symlinks.iter().map(|e| {
                if let Some(ref target) = e.symlink_target {
                    format!("{} -> {}", e.name, target)
                } else {
                    e.name.clone()
                }
            }).collect::<Vec<_>>(),
            "broken_symlinks": ls_output.symlinks.iter().filter(|e| e.is_broken_symlink).map(|e| &e.name).collect::<Vec<_>>(),
            "hidden": ls_output.hidden.iter().map(|e| &e.name).collect::<Vec<_>>(),
            "generated": ls_output.generated.iter().map(|e| &e.name).collect::<Vec<_>>(),
            "errors": ls_output.errors.iter().map(|e| serde_json::json!({
                "path": e.path,
                "message": e.message,
            })).collect::<Vec<_>>(),
            "counts": {
                "total": ls_output.total_count,
                "directories": ls_output.directories.len(),
                "files": ls_output.files.len(),
                "symlinks": ls_output.symlinks.len(),
                "hidden": ls_output.hidden.len(),
                "generated": ls_output.generated.len(),
                "errors": ls_output.errors.len(),
            }
        });
        Self::json_to_string(json)
    }

    /// Convert serde_json::Value to pretty-printed JSON string.
    fn json_to_string(value: serde_json::Value) -> String {
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
    }
    /// Format ls output in compact format.
    fn format_ls_compact(ls_output: &LsOutput) -> String {
        let mut output = String::new();

        // Show errors first (if any)
        if !ls_output.errors.is_empty() {
            for error in &ls_output.errors {
                output.push_str(&format!("error: {}\n", error.message));
            }
        }

        if ls_output.is_empty {
            output.push_str("ls: empty\n");
            return output;
        }

        output.push_str(&format!("total: {}\n", ls_output.total_count));

        if !ls_output.directories.is_empty() {
            output.push_str(&format!("directories ({}):\n", ls_output.directories.len()));
            for entry in &ls_output.directories {
                output.push_str(&format!("  {}\n", entry.name));
            }
        }

        if !ls_output.files.is_empty() {
            output.push_str(&format!("files ({}):\n", ls_output.files.len()));
            for entry in &ls_output.files {
                output.push_str(&format!("  {}\n", entry.name));
            }
        }

        if !ls_output.symlinks.is_empty() {
            output.push_str(&format!("symlinks ({}):\n", ls_output.symlinks.len()));
            for entry in &ls_output.symlinks {
                if let Some(ref target) = entry.symlink_target {
                    if entry.is_broken_symlink {
                        output.push_str(&format!("  {} -> {} [broken]\n", entry.name, target));
                    } else {
                        output.push_str(&format!("  {} -> {}\n", entry.name, target));
                    }
                } else {
                    output.push_str(&format!("  {}\n", entry.name));
                }
            }
        }

        if !ls_output.hidden.is_empty() {
            output.push_str(&format!("hidden ({}):\n", ls_output.hidden.len()));
            for entry in &ls_output.hidden {
                output.push_str(&format!("  {}\n", entry.name));
            }
        }

        if !ls_output.generated.is_empty() {
            output.push_str(&format!("generated ({}):\n", ls_output.generated.len()));
            for entry in &ls_output.generated {
                output.push_str(&format!("  {}\n", entry.name));
            }
        }

        output
    }
    /// Format ls output as raw (just filenames).
    fn format_ls_raw(ls_output: &LsOutput) -> String {
        let mut output = String::new();

        for entry in &ls_output.entries {
            output.push_str(&format!("{}\n", entry.name));
        }

        output
    }

    /// Handle the grep subcommand.
    fn handle_grep(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the grep output
        let mut grep_output = Self::parse_grep(&input)?;

        // Apply truncation for large result sets
        Self::truncate_grep(
            &mut grep_output,
            Self::DEFAULT_MAX_GREP_FILES,
            Self::DEFAULT_MAX_GREP_MATCHES_PER_FILE,
        );

        // Format output based on the requested format
        let output = Self::format_grep(&grep_output, ctx.format);
        print!("{}", output);

        Ok(())
    }

    /// Parse grep output into structured data.
    ///
    /// Supports multiple grep output formats:
    /// - Standard format: `filename:line_number:matched_line`
    /// - Without line numbers: `filename:matched_line`
    /// - With column: `filename:line_number:column:matched_line`
    /// - Recursive format (ripgrep): `filename:line_number:matched_line`
    ///
    /// Matches are grouped by file, preserving the order of first appearance.
    fn parse_grep(input: &str) -> CommandResult<GrepOutput> {
        use std::collections::HashMap;

        let mut grep_output = GrepOutput::default();
        // Use a HashMap to group matches by file path
        let mut matches_by_file: HashMap<String, Vec<GrepMatch>> = HashMap::new();
        // Track the order of file appearance
        let mut file_order: Vec<String> = Vec::new();

        for line in input.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Skip grep summary lines (e.g., from ripgrep)
            if line.starts_with("grep:") || line.contains("matched ") && line.ends_with(" files") {
                continue;
            }

            // Try to parse the grep line
            if let Some((path, grep_match)) = Self::parse_grep_line(line) {
                // Track file order on first appearance
                if !matches_by_file.contains_key(&path) {
                    file_order.push(path.clone());
                }
                // Add match to the file's group
                matches_by_file.entry(path).or_default().push(grep_match);
            }
        }

        // Convert HashMap to ordered Vec of GrepFile
        for path in file_order {
            if let Some(matches) = matches_by_file.remove(&path) {
                grep_output.files.push(GrepFile { path, matches });
            }
        }

        // Calculate totals
        grep_output.file_count = grep_output.files.len();
        for file in &grep_output.files {
            grep_output.match_count += file.matches.len();
        }

        // Set total counts before any truncation
        grep_output.total_files = grep_output.files.len();
        grep_output.total_matches = grep_output.match_count;
        grep_output.files_shown = grep_output.files.len();
        grep_output.matches_shown = grep_output.match_count;

        // Check if empty
        grep_output.is_empty = grep_output.files.is_empty();

        Ok(grep_output)
    }

    /// Default maximum number of files to show in grep output before truncation.
    const DEFAULT_MAX_GREP_FILES: usize = 50;

    /// Default maximum number of matches per file to show before truncation.
    const DEFAULT_MAX_GREP_MATCHES_PER_FILE: usize = 20;

    /// Truncate grep output if it exceeds the limits.
    ///
    /// This truncates both the number of files and the number of matches per file
    /// to prevent overwhelming output for large result sets.
    fn truncate_grep(grep_output: &mut GrepOutput, max_files: usize, max_matches_per_file: usize) {
        // First, truncate matches per file
        for file in &mut grep_output.files {
            if file.matches.len() > max_matches_per_file {
                file.matches.truncate(max_matches_per_file);
            }
        }

        // Then, truncate files if needed
        if grep_output.files.len() > max_files {
            grep_output.is_truncated = true;
            grep_output.files_shown = max_files;
            grep_output.files.truncate(max_files);
        } else if grep_output.total_matches
            > grep_output
                .files
                .iter()
                .map(|f| f.matches.len())
                .sum::<usize>()
        {
            // Some matches were truncated per-file but not files
            grep_output.is_truncated = true;
            grep_output.files_shown = grep_output.files.len();
        }

        // Calculate final matches shown
        grep_output.matches_shown = grep_output.files.iter().map(|f| f.matches.len()).sum();
    }

    /// Parse a single grep line.
    ///
    /// Formats supported:
    /// - `path:line_number:content` (standard with -n)
    /// - `path:line_number:column:content` (with --column)
    /// - `path:content` (without -n)
    /// - Binary file matches: `Binary file path matches`
    /// - Context lines: `path-line_number-content` (with -C/-B/-A flags)
    fn parse_grep_line(line: &str) -> Option<(String, GrepMatch)> {
        // Handle "Binary file path matches" format
        if line.starts_with("Binary file ") && line.ends_with(" matches") {
            let path = line
                .strip_prefix("Binary file ")
                .unwrap_or("")
                .strip_suffix(" matches")
                .unwrap_or("");
            if !path.is_empty() {
                return Some((
                    path.to_string(),
                    GrepMatch {
                        line_number: None,
                        column: None,
                        line: "[binary file]".to_string(),
                        is_context: false,
                        excerpt: None,
                    },
                ));
            }
        }

        // Determine if this is a context line or match line
        // Context lines use "-" as separator: "path-line-content"
        // Match lines use ":" as separator: "path:line:content"
        // Find the first separator (either : or -)
        let is_context_line = if let Some(dash_pos) = line.find('-') {
            // Check if dash comes before any colon (or no colon at all)
            match line.find(':') {
                Some(colon_pos) if colon_pos < dash_pos => false,
                _ => true,
            }
        } else {
            false
        };

        // Find the first separator to get the path
        let sep_pos = if is_context_line {
            line.find('-')?
        } else {
            line.find(':')?
        };

        let potential_path = &line[..sep_pos];

        // If the path is empty or the rest doesn't have content, skip
        if potential_path.is_empty() || line.len() <= sep_pos + 1 {
            return None;
        }

        let rest = &line[sep_pos + 1..];

        // Try to parse line number and optionally column
        // Format: line_number:content OR line_number:column:content OR just content
        // Context lines: line_number-content OR line_number-column-content
        let (line_number, column, content, is_context) =
            Self::parse_grep_line_content(rest, is_context_line);

        Some((
            potential_path.to_string(),
            GrepMatch {
                line_number,
                column,
                line: content.to_string(),
                is_context,
                excerpt: None,
            },
        ))
    }

    /// Parse the content part of a grep line (after the path: or path-).
    ///
    /// Context lines use "-" as separator (e.g., "10-content" for context)
    /// while match lines use ":" (e.g., "10:content" for matches).
    fn parse_grep_line_content(
        rest: &str,
        is_context_line: bool,
    ) -> (Option<usize>, Option<usize>, &str, bool) {
        if is_context_line {
            // Context line: use "-" as separator
            // Format: "10-content" or "10-5-content"
            if let Some(dash_pos) = rest.find('-') {
                let potential_line_num = &rest[..dash_pos];

                // Check if it's a valid line number before the dash
                if let Ok(line_number) = potential_line_num.parse::<usize>() {
                    let after_line = &rest[dash_pos + 1..];

                    // Try to parse column if present (context with column: "10-5-content")
                    if let Some(dash_pos2) = after_line.find('-') {
                        let potential_column = &after_line[..dash_pos2];
                        if let Ok(column) = potential_column.parse::<usize>() {
                            return (
                                Some(line_number),
                                Some(column),
                                &after_line[dash_pos2 + 1..],
                                true, // is_context
                            );
                        }
                    }

                    // No column, just line number with context
                    return (Some(line_number), None, after_line, true);
                }
            }
            // Couldn't parse as context line, return as content
            (None, None, rest, true)
        } else {
            // Match line: use ":" as separator
            // Try to find the first colon for line number
            if let Some(colon_pos) = rest.find(':') {
                let potential_line_num = &rest[..colon_pos];

                // Check if it's a valid line number
                if let Ok(line_number) = potential_line_num.parse::<usize>() {
                    let after_line = &rest[colon_pos + 1..];

                    // Try to parse column if present
                    if let Some(colon_pos2) = after_line.find(':') {
                        let potential_column = &after_line[..colon_pos2];
                        if let Ok(column) = potential_column.parse::<usize>() {
                            return (
                                Some(line_number),
                                Some(column),
                                &after_line[colon_pos2 + 1..],
                                false, // is_context
                            );
                        }
                    }

                    // No column, just line number
                    return (Some(line_number), None, after_line, false);
                }
            }

            // No line number, just content
            (None, None, rest, false)
        }
    }

    /// Format grep output for display.
    fn format_grep(grep_output: &GrepOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_grep_json(grep_output),
            OutputFormat::Csv => Self::format_grep_csv(grep_output),
            OutputFormat::Tsv => Self::format_grep_tsv(grep_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_grep_compact(grep_output),
            OutputFormat::Raw => Self::format_grep_raw(grep_output),
        }
    }

    /// Format grep output as JSON using the schema.
    fn format_grep_json(grep_output: &GrepOutput) -> String {
        use crate::schema::{
            GrepCounts, GrepFile as SchemaGrepFile, GrepMatch as SchemaGrepMatch, GrepOutputSchema,
        };

        // Count only non-context matches
        let match_count: usize = grep_output
            .files
            .iter()
            .map(|f| f.matches.iter().filter(|m| !m.is_context).count())
            .sum();

        let mut schema = GrepOutputSchema::new();
        schema.is_empty = grep_output.is_empty;
        schema.is_truncated = grep_output.is_truncated;

        // Convert internal GrepFile to schema GrepFile
        schema.files = grep_output
            .files
            .iter()
            .map(|f| SchemaGrepFile {
                path: f.path.clone(),
                matches: f
                    .matches
                    .iter()
                    .map(|m| SchemaGrepMatch {
                        line_number: m.line_number,
                        column: m.column,
                        line: m.line.clone(),
                        is_context: m.is_context,
                        excerpt: m.excerpt.clone(),
                    })
                    .collect(),
            })
            .collect();

        schema.counts = GrepCounts {
            files: grep_output.file_count,
            matches: match_count,
            total_files: grep_output.total_files,
            total_matches: grep_output.total_matches,
            files_shown: grep_output.files_shown,
            matches_shown: grep_output.matches_shown,
        };

        serde_json::to_string_pretty(&schema).unwrap_or_else(|e| {
            serde_json::json!({"error": format!("Failed to serialize: {}", e)}).to_string()
        })
    }

    /// Format grep output as CSV.
    fn format_grep_csv(grep_output: &GrepOutput) -> String {
        let mut result = String::new();
        result.push_str("path,line_number,column,is_context,line\n");

        for file in &grep_output.files {
            for m in &file.matches {
                let line_escaped = RunHandler::escape_csv_field(&m.line);
                result.push_str(&format!(
                    "{},{},{},{},{}\n",
                    file.path,
                    m.line_number.map(|n| n.to_string()).unwrap_or_default(),
                    m.column.map(|c| c.to_string()).unwrap_or_default(),
                    m.is_context,
                    line_escaped
                ));
            }
        }

        result
    }

    /// Format grep output as TSV.
    fn format_grep_tsv(grep_output: &GrepOutput) -> String {
        let mut result = String::new();
        result.push_str("path\tline_number\tcolumn\tis_context\tline\n");

        for file in &grep_output.files {
            for m in &file.matches {
                let line_escaped = RunHandler::escape_tsv_field(&m.line);
                result.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\n",
                    file.path,
                    m.line_number.map(|n| n.to_string()).unwrap_or_default(),
                    m.column.map(|c| c.to_string()).unwrap_or_default(),
                    m.is_context,
                    line_escaped
                ));
            }
        }

        result
    }

    /// Format grep output in compact format.
    ///
    /// Consecutive context lines are collapsed into a summary like "... (3 context lines)".
    fn format_grep_compact(grep_output: &GrepOutput) -> String {
        let mut output = String::new();

        if grep_output.is_empty {
            output.push_str("grep: no matches\n");
            return output;
        }

        // Count only non-context matches for the summary
        let match_count: usize = grep_output
            .files
            .iter()
            .map(|f| f.matches.iter().filter(|m| !m.is_context).count())
            .sum();

        // Show summary with truncation info if applicable
        if grep_output.is_truncated {
            output.push_str(&format!(
                "matches: {}/{} files, {}/{} results (truncated)\n",
                grep_output.files_shown,
                grep_output.total_files,
                grep_output.matches_shown,
                grep_output.total_matches
            ));
        } else {
            output.push_str(&format!(
                "matches: {} files, {} results\n",
                grep_output.file_count, match_count
            ));
        }

        for file in &grep_output.files {
            let non_context_count = file.matches.iter().filter(|m| !m.is_context).count();
            output.push_str(&format!("{} ({}):\n", file.path, non_context_count));

            // Track consecutive context lines for collapsing
            let mut context_start: Option<usize> = None;
            let mut context_count = 0;

            for m in &file.matches {
                if m.is_context {
                    // Start or continue a context block
                    if context_start.is_none() {
                        context_start = m.line_number;
                    }
                    context_count += 1;
                } else {
                    // Output any accumulated context lines first
                    if context_count > 0 {
                        if context_count == 1 {
                            // Single context line - show it
                            if let Some(ln) = context_start {
                                output.push_str(&format!("  {}: ...\n", ln));
                            }
                        } else {
                            // Multiple context lines - collapse
                            if let Some(start) = context_start {
                                output.push_str(&format!(
                                    "  {}-{}: ... ({} context lines)\n",
                                    start,
                                    start + context_count - 1,
                                    context_count
                                ));
                            }
                        }
                        context_start = None;
                        context_count = 0;
                    }

                    // Output the match line with excerpt if available
                    if let Some(ln) = m.line_number {
                        if let Some(col) = m.column {
                            let excerpt_str = m
                                .excerpt
                                .as_ref()
                                .map(|e| format!(" [{}]", e))
                                .unwrap_or_default();
                            output.push_str(&format!(
                                "  {}:{}: {}{}\n",
                                ln, col, m.line, excerpt_str
                            ));
                        } else {
                            let excerpt_str = m
                                .excerpt
                                .as_ref()
                                .map(|e| format!(" [{}]", e))
                                .unwrap_or_default();
                            output.push_str(&format!("  {}: {}{}\n", ln, m.line, excerpt_str));
                        }
                    } else {
                        let excerpt_str = m
                            .excerpt
                            .as_ref()
                            .map(|e| format!(" [{}]", e))
                            .unwrap_or_default();
                        output.push_str(&format!("  {}{}\n", m.line, excerpt_str));
                    }
                }
            }

            // Handle any trailing context lines
            if context_count > 0 {
                if context_count == 1 {
                    if let Some(ln) = context_start {
                        output.push_str(&format!("  {}: ...\n", ln));
                    }
                } else {
                    if let Some(start) = context_start {
                        output.push_str(&format!(
                            "  {}-{}: ... ({} context lines)\n",
                            start,
                            start + context_count - 1,
                            context_count
                        ));
                    }
                }
            }
        }

        // Show truncation warning if applicable
        if grep_output.is_truncated {
            let hidden_files = grep_output
                .total_files
                .saturating_sub(grep_output.files_shown);
            let hidden_matches = grep_output
                .total_matches
                .saturating_sub(grep_output.matches_shown);
            if hidden_files > 0 {
                output.push_str(&format!("  ... {} more file(s) not shown\n", hidden_files));
            }
            if hidden_matches > 0 && hidden_files == 0 {
                output.push_str(&format!(
                    "  ... {} more match(es) not shown\n",
                    hidden_matches
                ));
            }
        }

        // Add total files and match count at the end
        if grep_output.is_truncated {
            output.push_str(&format!(
                "total: {}/{} files, {}/{} matches\n",
                grep_output.files_shown,
                grep_output.total_files,
                grep_output.matches_shown,
                grep_output.total_matches
            ));
        } else {
            output.push_str(&format!(
                "total: {} files, {} matches\n",
                grep_output.file_count, match_count
            ));
        }

        output
    }

    /// Format grep output as raw (original format).
    fn format_grep_raw(grep_output: &GrepOutput) -> String {
        let mut output = String::new();

        for file in &grep_output.files {
            for m in &file.matches {
                // Use dash separator for context lines, colon for matches
                let sep = if m.is_context { "-" } else { ":" };
                if let Some(ln) = m.line_number {
                    if let Some(col) = m.column {
                        output.push_str(&format!(
                            "{}{}{}{}{}:{}\n",
                            file.path, sep, ln, sep, col, m.line
                        ));
                    } else {
                        output.push_str(&format!("{}{}{}{}{}\n", file.path, sep, ln, sep, m.line));
                    }
                } else {
                    output.push_str(&format!("{}:{}\n", file.path, m.line));
                }
            }
        }

        // Show truncation warning if applicable
        if grep_output.is_truncated {
            let hidden_files = grep_output
                .total_files
                .saturating_sub(grep_output.files_shown);
            let hidden_matches = grep_output
                .total_matches
                .saturating_sub(grep_output.matches_shown);
            if hidden_files > 0 {
                output.push_str(&format!("... {} more file(s) truncated\n", hidden_files));
            }
            if hidden_matches > 0 && hidden_files == 0 {
                output.push_str(&format!(
                    "... {} more match(es) truncated\n",
                    hidden_matches
                ));
            }
        }

        output
    }

    /// Handle the test subcommand.
    fn handle_test(
        runner: &Option<crate::TestRunner>,
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse based on the runner type (default to pytest)
        match runner {
            Some(crate::TestRunner::Pytest) | None => {
                let test_output = Self::parse_pytest(&input)?;
                let output = Self::format_pytest(&test_output, ctx.format);
                print!("{}", output);
            }
            Some(crate::TestRunner::Jest) => {
                let test_output = Self::parse_jest(&input)?;
                let output = Self::format_jest(&test_output, ctx.format);
                print!("{}", output);
            }
            Some(crate::TestRunner::Vitest) => {
                let test_output = Self::parse_vitest(&input)?;
                let output = Self::format_vitest(&test_output, ctx.format);
                print!("{}", output);
            }
            Some(crate::TestRunner::Npm) => {
                let test_output = Self::parse_npm_test(&input)?;
                let output = Self::format_npm_test(&test_output, ctx.format);
                print!("{}", output);
            }
            Some(crate::TestRunner::Pnpm) => {
                let test_output = Self::parse_pnpm_test(&input)?;
                let output = Self::format_pnpm_test(&test_output, ctx.format);
                print!("{}", output);
            }
            Some(crate::TestRunner::Bun) => {
                let test_output = Self::parse_bun_test(&input)?;
                let output = Self::format_bun_test(&test_output, ctx.format);
                print!("{}", output);
            }
        }

        Ok(())
    }

    /// Parse pytest output into structured data.
    fn parse_pytest(input: &str) -> CommandResult<PytestOutput> {
        let mut output = PytestOutput::default();
        let mut current_test: Option<TestResult> = None;
        let mut in_failure_section = false;
        let mut failure_buffer = String::new();
        let mut current_failed_test_name: Option<String> = None;

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Parse header info
            // "rootdir: /path/to/project"
            if trimmed.starts_with("rootdir:") {
                output.rootdir = Some(
                    trimmed
                        .strip_prefix("rootdir:")
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                );
                continue;
            }

            // "platform darwin -- Python 3.12.0, pytest-8.0.0, pluggy-1.4.0"
            if trimmed.starts_with("platform ") {
                output.platform = Some(trimmed.to_string());
                // Extract Python and pytest version
                if let Some(py_pos) = trimmed.find("Python ") {
                    let after_py = &trimmed[py_pos + 7..];
                    if let Some(comma_pos) = after_py.find(',') {
                        output.python_version = Some(after_py[..comma_pos].to_string());
                    }
                }
                if let Some(pytest_pos) = trimmed.find("pytest-") {
                    let after_pytest = &trimmed[pytest_pos + 7..];
                    if let Some(comma_pos) = after_pytest.find(',') {
                        output.pytest_version = Some(after_pytest[..comma_pos].to_string());
                    } else {
                        output.pytest_version = Some(after_pytest.to_string());
                    }
                }
                continue;
            }

            // Detect start of test session
            // "test session starts" or "collected N items"
            if trimmed.contains("test session starts") || trimmed.contains("collected") {
                continue;
            }

            // Detect test results with progress format
            // Format: "tests/test_file.py::test_name PASSED" or "tests/test_file.py::test_name FAILED"
            // Also handles the short format: "test_file.py .F.s" (dot=pass, F=fail, s=skip)
            if let Some(test_result) = Self::parse_pytest_test_line(trimmed) {
                // Save any pending test
                if let Some(test) = current_test.take() {
                    output.tests.push(test);
                }
                current_test = Some(test_result);
                continue;
            }

            // Detect summary line
            // "N passed, M failed, K skipped in X.XXs"
            // Also: "N passed in X.XXs"
            if Self::is_pytest_summary_line(trimmed) {
                let summary = Self::parse_pytest_summary(trimmed);
                output.summary = summary;
                continue;
            }

            // Detect failure section start
            // "=== FAILURES ===" or "=== short test summary info ==="
            if trimmed.starts_with("=== FAILURES") || trimmed.starts_with("FAILURES") {
                in_failure_section = true;
                continue;
            }
            if trimmed.starts_with("=== short test summary info ===") {
                in_failure_section = true;
                continue;
            }

            // Detect error section
            // "=== ERRORS ==="
            if trimmed.starts_with("=== ERRORS") || trimmed.starts_with("ERRORS") {
                in_failure_section = true;
                continue;
            }

            // Parse failure details
            if in_failure_section {
                // Check if this is a new failure header: "____ test_name ____"
                if trimmed.starts_with("____") && trimmed.ends_with("____") {
                    // Save any previous failure info
                    if let Some(name) = current_failed_test_name.take() {
                        // Find test by matching the name at the end (after ::)
                        // "____ test_name ____" matches "file.py::test_name"
                        if let Some(test) = output
                            .tests
                            .iter_mut()
                            .find(|t| t.name == name || t.name.ends_with(&format!("::{}", name)))
                        {
                            test.error_message = Some(failure_buffer.trim().to_string());
                        }
                    }
                    let name = trimmed.trim_matches('_').trim().to_string();
                    current_failed_test_name = Some(name);
                    failure_buffer = String::new();
                    continue;
                }

                // Check for ERROR instead of FAILURES
                // "ERROR at setup of test_name"
                if trimmed.starts_with("ERROR at") || trimmed.starts_with("ERROR:") {
                    in_failure_section = true;
                    if let Some(name) = current_failed_test_name.take() {
                        // Find test by matching the name at the end (after ::)
                        if let Some(test) = output
                            .tests
                            .iter_mut()
                            .find(|t| t.name == name || t.name.ends_with(&format!("::{}", name)))
                        {
                            test.error_message = Some(failure_buffer.trim().to_string());
                        }
                    }
                    // Extract test name from error line
                    let name = if trimmed.starts_with("ERROR at setup of ") {
                        trimmed
                            .strip_prefix("ERROR at setup of ")
                            .unwrap_or("")
                            .to_string()
                    } else if trimmed.starts_with("ERROR at teardown of ") {
                        trimmed
                            .strip_prefix("ERROR at teardown of ")
                            .unwrap_or("")
                            .to_string()
                    } else {
                        trimmed
                            .strip_prefix("ERROR:")
                            .unwrap_or("")
                            .trim()
                            .to_string()
                    };
                    current_failed_test_name = Some(name);
                    failure_buffer = String::new();
                    continue;
                }

                // Accumulate failure details
                if current_failed_test_name.is_some() {
                    failure_buffer.push_str(line);
                    failure_buffer.push('\n');
                }
            }
        }

        // Save any pending test
        if let Some(test) = current_test.take() {
            output.tests.push(test);
        }

        // Save last failure info
        if let Some(name) = current_failed_test_name.take() {
            // Find test by matching the name at the end (after ::)
            // "____ test_name ____" matches "file.py::test_name"
            if let Some(test) = output
                .tests
                .iter_mut()
                .find(|t| t.name == name || t.name.ends_with(&format!("::{}", name)))
            {
                test.error_message = Some(failure_buffer.trim().to_string());
            }
        }

        // Calculate totals if not already in summary
        if output.summary.total == 0 && !output.tests.is_empty() {
            output.summary.passed = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::Passed)
                .count();
            output.summary.failed = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::Failed)
                .count();
            output.summary.skipped = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::Skipped)
                .count();
            output.summary.xfailed = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::XFailed)
                .count();
            output.summary.xpassed = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::XPassed)
                .count();
            output.summary.errors = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::Error)
                .count();
            output.summary.total = output.tests.len();
        }

        // Determine success
        output.success =
            output.summary.failed == 0 && output.summary.errors == 0 && output.summary.total > 0;
        output.is_empty = output.tests.is_empty() && output.summary.total == 0;

        Ok(output)
    }

    /// Parse a single test result line from pytest output.
    fn parse_pytest_test_line(line: &str) -> Option<TestResult> {
        // Format: "tests/test_file.py::test_name PASSED"
        // or: "tests/test_file.py::test_name SKIPPED (reason)"
        // or: "tests/test_file.py::test_name FAILED"
        // or: "tests/test_file.py::test_name XFAIL (reason)"

        // Skip lines that are clearly not test results
        if line.starts_with("===")
            || line.starts_with("---")
            || line.starts_with("...")
            || line.is_empty()
        {
            return None;
        }

        // Look for PASSED, FAILED, SKIPPED, XFAIL, XPASS, ERROR
        let (status_str, remainder) = if line.ends_with(" PASSED") {
            ("PASSED", &line[..line.len() - 7])
        } else if line.ends_with(" FAILED") {
            ("FAILED", &line[..line.len() - 7])
        } else if line.ends_with(" SKIPPED") {
            ("SKIPPED", &line[..line.len() - 8])
        } else if line.ends_with(" XFAIL") {
            ("XFAIL", &line[..line.len() - 6])
        } else if line.ends_with(" XPASS") {
            ("XPASS", &line[..line.len() - 6])
        } else if line.ends_with(" ERROR") {
            ("ERROR", &line[..line.len() - 6])
        } else {
            // Check for inline format: "PASSED [50%]" or "FAILED [50%]"
            if let Some(pos) = line.find(" PASSED [") {
                ("PASSED", &line[..pos])
            } else if let Some(pos) = line.find(" FAILED [") {
                ("FAILED", &line[..pos])
            } else if let Some(pos) = line.find(" SKIPPED [") {
                ("SKIPPED", &line[..pos])
            } else if let Some(pos) = line.find(" XFAIL [") {
                ("XFAIL", &line[..pos])
            } else if let Some(pos) = line.find(" XPASS [") {
                ("XPASS", &line[..pos])
            } else if let Some(pos) = line.find(" ERROR [") {
                ("ERROR", &line[..pos])
            } else {
                return None;
            }
        };

        let status = match status_str {
            "PASSED" => TestStatus::Passed,
            "FAILED" => TestStatus::Failed,
            "SKIPPED" => TestStatus::Skipped,
            "XFAIL" => TestStatus::XFailed,
            "XPASS" => TestStatus::XPassed,
            "ERROR" => TestStatus::Error,
            _ => return None,
        };

        // Parse test name and file
        let test_name = remainder.trim().to_string();

        // Try to extract file and line from "file.py::test_name" format
        let (file, line) = if let Some(pos) = test_name.find("::") {
            let file = test_name[..pos].to_string();
            let rest = &test_name[pos + 2..];
            // Check for line number: "test_name[:lineno]"
            let line = if let Some(colon_pos) = rest.find(':') {
                rest[colon_pos + 1..].parse().ok()
            } else {
                None
            };
            (Some(file), line)
        } else {
            (None, None)
        };

        Some(TestResult {
            name: test_name,
            status,
            duration: None, // Duration is usually in the summary line
            file,
            line,
            error_message: None,
        })
    }

    /// Check if a line is a pytest summary line.
    fn is_pytest_summary_line(line: &str) -> bool {
        // Summary lines start with a number and contain "passed" or "failed"
        // Examples:
        // "2 passed in 0.01s"
        // "2 passed, 1 failed in 0.01s"
        // "2 passed, 1 failed, 3 skipped in 0.01s"
        // "1 failed, 2 passed in 0.01s"
        // "=== 2 passed in 0.01s ==="
        let lower = line.to_lowercase();
        let starts_with_equals = line.starts_with("===");
        let has_passed = lower.contains("passed");
        let has_failed = lower.contains("failed");
        let has_skipped = lower.contains("skipped");
        let has_error = lower.contains("error");
        let has_deselected = lower.contains("deselected");
        let has_xfailed = lower.contains("xfailed");
        let has_xpassed = lower.contains("xpassed");
        let has_warnings = lower.contains("warning");

        (starts_with_equals
            || line
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false))
            && (has_passed
                || has_failed
                || has_skipped
                || has_error
                || has_deselected
                || has_xfailed
                || has_xpassed
                || has_warnings)
    }

    /// Parse pytest summary line into TestSummary.
    fn parse_pytest_summary(line: &str) -> TestSummary {
        let mut summary = TestSummary::default();
        let lower = line.to_lowercase();

        // Remove wrapper like "=== ... ==="
        let cleaned = line.trim_matches('=').trim();

        // Parse counts
        // Pattern: "N passed", "N failed", "N skipped", etc.
        fn extract_count(text: &str, label: &str) -> usize {
            let pattern = format!(" {}", label);
            if let Some(pos) = text.find(&pattern) {
                // Look backwards for the number
                let before = &text[..pos];
                let words: Vec<&str> = before.split_whitespace().collect();
                if let Some(last) = words.last() {
                    return last.parse().unwrap_or(0);
                }
            }
            0
        }

        summary.passed = extract_count(&lower, "passed");
        summary.failed = extract_count(&lower, "failed");
        summary.skipped = extract_count(&lower, "skipped");
        summary.errors = extract_count(&lower, "error");
        summary.xfailed = extract_count(&lower, "xfailed");
        summary.xpassed = extract_count(&lower, "xpassed");

        // Calculate total
        summary.total = summary.passed
            + summary.failed
            + summary.skipped
            + summary.errors
            + summary.xfailed
            + summary.xpassed;

        // Parse duration
        // "in 0.01s" or "in 1.23 seconds"
        if let Some(pos) = lower.find(" in ") {
            let after_in = &cleaned[pos + 4..];
            // Extract number before 's' or 'seconds'
            let duration_str: String = after_in
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if let Ok(duration) = duration_str.parse::<f64>() {
                summary.duration = Some(duration);
            }
        }

        summary
    }

    /// Format pytest output based on the requested format.
    fn format_pytest(output: &PytestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_pytest_json(output),
            OutputFormat::Compact => Self::format_pytest_compact(output),
            OutputFormat::Raw => Self::format_pytest_raw(output),
            OutputFormat::Agent => Self::format_pytest_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_pytest_compact(output),
        }
    }

    /// Format pytest output as JSON.
    fn format_pytest_json(output: &PytestOutput) -> String {
        // Extract failing test identifiers
        let failed_tests: Vec<_> = output
            .tests
            .iter()
            .filter(|t| t.status == TestStatus::Failed || t.status == TestStatus::Error)
            .map(|t| t.name.clone())
            .collect();

        serde_json::json!({
            "success": output.success,
            "is_empty": output.is_empty,
            "summary": {
                "passed": output.summary.passed,
                "failed": output.summary.failed,
                "skipped": output.summary.skipped,
                "xfailed": output.summary.xfailed,
                "xpassed": output.summary.xpassed,
                "errors": output.summary.errors,
                "total": output.summary.total,
                "duration": output.summary.duration,
            },
            "failed_tests": failed_tests,
            "tests": output.tests.iter().map(|t| serde_json::json!({
                "name": t.name,
                "status": match t.status {
                    TestStatus::Passed => "passed",
                    TestStatus::Failed => "failed",
                    TestStatus::Skipped => "skipped",
                    TestStatus::XFailed => "xfailed",
                    TestStatus::XPassed => "xpassed",
                    TestStatus::Error => "error",
                },
                "duration": t.duration,
                "file": t.file,
                "line": t.line,
                "error_message": t.error_message,
            })).collect::<Vec<_>>(),
            "rootdir": output.rootdir,
            "platform": output.platform,
            "python_version": output.python_version,
            "pytest_version": output.pytest_version,
        })
        .to_string()
    }

    /// Format pytest output in compact format.
    fn format_pytest_compact(output: &PytestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("no tests found\n");
            return result;
        }

        // Compact success summary - minimal output when all tests pass
        if output.success {
            result.push_str(&format!("PASS: {} tests", output.summary.passed));
            if output.summary.skipped > 0 {
                result.push_str(&format!(", {} skipped", output.summary.skipped));
            }
            if output.summary.xfailed > 0 {
                result.push_str(&format!(", {} xfailed", output.summary.xfailed));
            }
            if output.summary.xpassed > 0 {
                result.push_str(&format!(", {} xpassed", output.summary.xpassed));
            }
            if let Some(duration) = output.summary.duration {
                result.push_str(&format!(" [{:.2}s]", duration));
            }
            result.push('\n');
            return result;
        }

        // Failure-focused summary - detailed output when tests fail
        result.push_str(&format!(
            "FAIL: {} passed, {} failed",
            output.summary.passed, output.summary.failed
        ));
        if output.summary.skipped > 0 {
            result.push_str(&format!(", {} skipped", output.summary.skipped));
        }
        if output.summary.xfailed > 0 {
            result.push_str(&format!(", {} xfailed", output.summary.xfailed));
        }
        if output.summary.xpassed > 0 {
            result.push_str(&format!(", {} xpassed", output.summary.xpassed));
        }
        if output.summary.errors > 0 {
            result.push_str(&format!(", {} errors", output.summary.errors));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!(" [{:.2}s]", duration));
        }
        result.push('\n');

        // List failed tests
        let failed_tests: Vec<_> = output
            .tests
            .iter()
            .filter(|t| t.status == TestStatus::Failed || t.status == TestStatus::Error)
            .collect();

        if !failed_tests.is_empty() {
            result.push_str(&format!("failed ({}):\n", failed_tests.len()));
            for test in failed_tests {
                result.push_str(&format!("  {}\n", test.name));
                if let Some(ref msg) = test.error_message {
                    // Show first line of error message
                    if let Some(first_line) = msg.lines().next() {
                        let truncated = if first_line.len() > 80 {
                            format!("{}...", &first_line[..77])
                        } else {
                            first_line.to_string()
                        };
                        result.push_str(&format!("    {}\n", truncated));
                    }
                }
            }
        }

        result
    }

    /// Format pytest output as raw (just test names with status).
    fn format_pytest_raw(output: &PytestOutput) -> String {
        let mut result = String::new();

        for test in &output.tests {
            let status = match test.status {
                TestStatus::Passed => "PASS",
                TestStatus::Failed => "FAIL",
                TestStatus::Skipped => "SKIP",
                TestStatus::XFailed => "XFAIL",
                TestStatus::XPassed => "XPASS",
                TestStatus::Error => "ERROR",
            };
            result.push_str(&format!("{} {}\n", status, test.name));
        }

        result
    }

    /// Format pytest output for AI agent consumption.
    fn format_pytest_agent(output: &PytestOutput) -> String {
        let mut result = String::new();

        result.push_str("# Test Results\n\n");

        if output.is_empty {
            result.push_str("Status: NO_TESTS\n");
            return result;
        }

        let status = if output.success { "SUCCESS" } else { "FAILURE" };
        result.push_str(&format!("Status: {}\n\n", status));

        // Summary
        result.push_str("## Summary\n");
        result.push_str(&format!("- Total: {}\n", output.summary.total));
        result.push_str(&format!("- Passed: {}\n", output.summary.passed));
        result.push_str(&format!("- Failed: {}\n", output.summary.failed));
        if output.summary.skipped > 0 {
            result.push_str(&format!("- Skipped: {}\n", output.summary.skipped));
        }
        if output.summary.xfailed > 0 {
            result.push_str(&format!("- XFailed: {}\n", output.summary.xfailed));
        }
        if output.summary.xpassed > 0 {
            result.push_str(&format!("- XPassed: {}\n", output.summary.xpassed));
        }
        if output.summary.errors > 0 {
            result.push_str(&format!("- Errors: {}\n", output.summary.errors));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!("- Duration: {:.2}s\n", duration));
        }
        result.push('\n');

        // Failed tests with details
        let failed_tests: Vec<_> = output
            .tests
            .iter()
            .filter(|t| t.status == TestStatus::Failed || t.status == TestStatus::Error)
            .collect();

        if !failed_tests.is_empty() {
            result.push_str("## Failed Tests\n\n");
            for test in failed_tests {
                result.push_str(&format!("### {}\n", test.name));
                if let Some(ref file) = test.file {
                    result.push_str(&format!("File: {}", file));
                    if let Some(line) = test.line {
                        result.push_str(&format!(":{}", line));
                    }
                    result.push('\n');
                }
                if let Some(ref msg) = test.error_message {
                    result.push_str(&format!("\n```\n{}\n```\n", msg));
                }
                result.push('\n');
            }
        }

        result
    }

    /// Parse Jest output into structured data.
    fn parse_jest(input: &str) -> CommandResult<JestOutput> {
        let mut output = JestOutput::default();
        let mut current_suite: Option<JestTestSuite> = None;
        let mut in_failure_details = false;
        let mut failure_buffer = String::new();
        let mut current_failed_test: Option<String> = None;

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines (but preserve them in failure details)
            if trimmed.is_empty() && !in_failure_details {
                continue;
            }

            // Detect test suite header: "PASS src/path/to/test.js" or "FAIL src/path/to/test.js"
            if trimmed.starts_with("PASS ") || trimmed.starts_with("FAIL ") {
                // Save any pending suite
                if let Some(suite) = current_suite.take() {
                    output.test_suites.push(suite);
                }

                let (passed, file) = if trimmed.starts_with("PASS ") {
                    (true, trimmed.strip_prefix("PASS ").unwrap_or("").trim())
                } else {
                    (false, trimmed.strip_prefix("FAIL ").unwrap_or("").trim())
                };

                current_suite = Some(JestTestSuite {
                    file: file.to_string(),
                    passed,
                    duration: None,
                    tests: Vec::new(),
                });
                in_failure_details = false;
                failure_buffer.clear();
                current_failed_test = None;
                continue;
            }

            // Detect individual test results
            // Format: "  ✓ test name (5 ms)" or "  ✕ test name" or "  ○ skipped"
            if let Some(test) = Self::parse_jest_test_line(trimmed) {
                if let Some(ref mut suite) = current_suite {
                    suite.tests.push(test);
                }
                continue;
            }

            // Detect test suite duration: "(5 ms)"
            if trimmed.starts_with('(') && trimmed.ends_with(')') && current_suite.is_some() {
                let duration_str = trimmed.trim_matches(|c| c == '(' || c == ')');
                let duration = Self::parse_jest_duration(duration_str);
                if let Some(ref mut suite) = current_suite {
                    suite.duration = duration;
                }
                continue;
            }

            // Detect failure details start
            // "  ● test name › should work"
            if trimmed.starts_with("● ") {
                in_failure_details = true;
                // Save any previous failure info
                if let Some(name) = current_failed_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        if let Some(test) = suite.tests.iter_mut().find(|t| t.name == name) {
                            test.error_message = Some(failure_buffer.trim().to_string());
                        }
                    }
                }
                let name = trimmed.strip_prefix("● ").unwrap_or("").trim().to_string();
                current_failed_test = Some(name);
                failure_buffer = String::new();
                continue;
            }

            // Accumulate failure details
            if in_failure_details && current_failed_test.is_some() {
                failure_buffer.push_str(line);
                failure_buffer.push('\n');
                continue;
            }

            // Detect summary line: "Test Suites: X passed, Y failed, Z total"
            if trimmed.starts_with("Test Suites:") {
                let summary = Self::parse_jest_summary(trimmed);
                output.summary = summary;
                continue;
            }

            // Additional summary lines: "Tests:", "Snapshots:", "Time:"
            if trimmed.starts_with("Tests:") {
                Self::parse_jest_tests_summary(trimmed, &mut output.summary);
            }
            if trimmed.starts_with("Snapshots:") {
                Self::parse_jest_snapshots_summary(trimmed, &mut output.summary);
            }
            if trimmed.starts_with("Time:") {
                Self::parse_jest_time_summary(trimmed, &mut output.summary);
            }
        }

        // Save any pending suite
        if let Some(suite) = current_suite.take() {
            output.test_suites.push(suite);
        }

        // Save last failure info (if any)
        // Note: Error messages are typically captured when we see the next test or suite
        // so we don't need to explicitly save the last one here

        // Calculate totals if not already in summary
        if output.summary.suites_total == 0 && !output.test_suites.is_empty() {
            output.summary.suites_passed = output.test_suites.iter().filter(|s| s.passed).count();
            output.summary.suites_failed = output.test_suites.iter().filter(|s| !s.passed).count();
            output.summary.suites_total = output.test_suites.len();

            for suite in &output.test_suites {
                for test in &suite.tests {
                    match test.status {
                        JestTestStatus::Passed => output.summary.tests_passed += 1,
                        JestTestStatus::Failed => output.summary.tests_failed += 1,
                        JestTestStatus::Skipped => output.summary.tests_skipped += 1,
                        JestTestStatus::Todo => output.summary.tests_todo += 1,
                    }
                    output.summary.tests_total += 1;
                }
            }
        }

        // Determine success
        output.success = output.summary.tests_failed == 0
            && output.summary.suites_failed == 0
            && output.summary.tests_total > 0;
        output.is_empty = output.test_suites.is_empty() && output.summary.tests_total == 0;

        Ok(output)
    }

    /// Parse a single Jest test result line.
    fn parse_jest_test_line(line: &str) -> Option<JestTest> {
        // Trim leading whitespace
        let line = line.trim_start();

        // Skip if doesn't start with proper prefix
        if !line.starts_with("✓") && !line.starts_with("✕") && !line.starts_with("○") {
            return None;
        }

        let (status, remainder) = if line.starts_with("✓") {
            (JestTestStatus::Passed, line.strip_prefix("✓").unwrap_or(""))
        } else if line.starts_with("✕") {
            (JestTestStatus::Failed, line.strip_prefix("✕").unwrap_or(""))
        } else if line.starts_with("○") {
            // Could be skipped or todo
            let rem = line.strip_prefix("○").unwrap_or("");
            if rem.contains("skipped") || rem.contains("skip") {
                (JestTestStatus::Skipped, rem)
            } else if rem.contains("todo") {
                (JestTestStatus::Todo, rem)
            } else {
                (JestTestStatus::Skipped, rem)
            }
        } else {
            return None;
        };

        // Parse test name and duration
        let trimmed = remainder.trim();

        // Extract duration if present: "test name (5 ms)"
        let (test_name, duration) = if let Some(paren_pos) = trimmed.rfind('(') {
            let name_part = trimmed[..paren_pos].trim();
            let duration_part = &trimmed[paren_pos..];
            let duration =
                Self::parse_jest_duration(duration_part.trim_matches(|c| c == '(' || c == ')'));
            (name_part.to_string(), duration)
        } else {
            (trimmed.to_string(), None)
        };

        // Parse ancestors (describe blocks) from test name
        // Format: "describe block > nested describe > test name"
        let (ancestors, final_name) = if test_name.contains('>') || test_name.contains("›") {
            let delimiter = if test_name.contains('>') { ">" } else { "›" };
            let parts: Vec<&str> = test_name.split(delimiter).map(|s| s.trim()).collect();
            if parts.len() > 1 {
                let ancestors: Vec<String> = parts[..parts.len() - 1]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                let name = parts.last().unwrap_or(&"").to_string();
                (ancestors, name)
            } else {
                (Vec::new(), test_name.clone())
            }
        } else {
            (Vec::new(), test_name.clone())
        };

        Some(JestTest {
            name: test_name,
            test_name: final_name,
            ancestors,
            status,
            duration,
            error_message: None,
        })
    }

    /// Parse Jest duration string (e.g., "5 ms", "1.23 s").
    fn parse_jest_duration(s: &str) -> Option<f64> {
        let s = s.trim();
        // Try to extract number and unit
        let num_str: String = s
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let num: f64 = num_str.parse().ok()?;

        // Convert to seconds based on unit
        if s.contains("ms") || s.ends_with("ms") {
            Some(num / 1000.0)
        } else if s.contains('s') && !s.contains("ms") {
            Some(num)
        } else {
            // Assume milliseconds if no unit
            Some(num / 1000.0)
        }
    }

    /// Parse Jest summary line for test suites.
    fn parse_jest_summary(line: &str) -> JestSummary {
        let mut summary = JestSummary::default();
        let line = line.strip_prefix("Test Suites:").unwrap_or("");

        // Parse pattern: "X passed, Y failed, Z total" or "X passed, Y total"
        fn extract_count(text: &str, label: &str) -> usize {
            let pattern = format!(" {}", label);
            if let Some(pos) = text.find(&pattern) {
                let before = &text[..pos];
                let words: Vec<&str> = before.split_whitespace().collect();
                if let Some(last) = words.last() {
                    return last.parse().unwrap_or(0);
                }
            }
            0
        }

        summary.suites_passed = extract_count(line, "passed");
        summary.suites_failed = extract_count(line, "failed");
        summary.suites_total = extract_count(line, "total");

        summary
    }

    /// Parse Jest summary line for tests.
    fn parse_jest_tests_summary(line: &str, summary: &mut JestSummary) {
        let line = line.strip_prefix("Tests:").unwrap_or("");

        fn extract_count(text: &str, label: &str) -> usize {
            let pattern = format!(" {}", label);
            if let Some(pos) = text.find(&pattern) {
                let before = &text[..pos];
                let words: Vec<&str> = before.split_whitespace().collect();
                if let Some(last) = words.last() {
                    return last.parse().unwrap_or(0);
                }
            }
            0
        }

        summary.tests_passed = extract_count(line, "passed");
        summary.tests_failed = extract_count(line, "failed");
        summary.tests_skipped = extract_count(line, "skipped");
        summary.tests_todo = extract_count(line, "todo");
        summary.tests_total = extract_count(line, "total");
    }

    /// Parse Jest summary line for snapshots.
    fn parse_jest_snapshots_summary(line: &str, summary: &mut JestSummary) {
        let line = line.strip_prefix("Snapshots:").unwrap_or("");
        // Try to extract a number from the line
        let num_str: String = line.chars().filter(|c| c.is_ascii_digit()).collect();
        if let Ok(num) = num_str.parse() {
            summary.snapshots = Some(num);
        }
    }

    /// Parse Jest summary line for time.
    fn parse_jest_time_summary(line: &str, summary: &mut JestSummary) {
        let line = line.strip_prefix("Time:").unwrap_or("").trim();
        summary.duration = Self::parse_jest_duration(line);
    }

    /// Format Jest output based on the requested format.
    fn format_jest(output: &JestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_jest_json(output),
            OutputFormat::Compact => Self::format_jest_compact(output),
            OutputFormat::Raw => Self::format_jest_raw(output),
            OutputFormat::Agent => Self::format_jest_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_jest_compact(output),
        }
    }

    /// Format Jest output as JSON.
    fn format_jest_json(output: &JestOutput) -> String {
        // Extract failing test identifiers (file::test_name format)
        let failed_tests: Vec<_> = output
            .test_suites
            .iter()
            .flat_map(|suite| {
                suite
                    .tests
                    .iter()
                    .filter(|t| t.status == JestTestStatus::Failed)
                    .map(|t| format!("{}::{}", suite.file, t.name))
            })
            .collect();

        serde_json::json!({
            "success": output.success,
            "is_empty": output.is_empty,
            "summary": {
                "suites": {
                    "passed": output.summary.suites_passed,
                    "failed": output.summary.suites_failed,
                    "total": output.summary.suites_total,
                },
                "tests": {
                    "passed": output.summary.tests_passed,
                    "failed": output.summary.tests_failed,
                    "skipped": output.summary.tests_skipped,
                    "todo": output.summary.tests_todo,
                    "total": output.summary.tests_total,
                },
                "snapshots": output.summary.snapshots,
                "duration": output.summary.duration,
            },
            "failed_tests": failed_tests,
            "test_suites": output.test_suites.iter().map(|suite| serde_json::json!({
                "file": suite.file,
                "passed": suite.passed,
                "duration": suite.duration,
                "tests": suite.tests.iter().map(|t| serde_json::json!({
                    "name": t.name,
                    "test_name": t.test_name,
                    "ancestors": t.ancestors,
                    "status": match t.status {
                        JestTestStatus::Passed => "passed",
                        JestTestStatus::Failed => "failed",
                        JestTestStatus::Skipped => "skipped",
                        JestTestStatus::Todo => "todo",
                    },
                    "duration": t.duration,
                    "error_message": t.error_message,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "jest_version": output.jest_version,
            "test_path_pattern": output.test_path_pattern,
        })
        .to_string()
    }

    /// Format Jest output in compact format.
    fn format_jest_compact(output: &JestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("no tests found\n");
            return result;
        }

        // Compact success summary - minimal output when all tests pass
        if output.success {
            result.push_str(&format!(
                "PASS: {} suites, {} tests",
                output.summary.suites_total, output.summary.tests_passed
            ));
            if output.summary.tests_skipped > 0 {
                result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
            }
            if output.summary.tests_todo > 0 {
                result.push_str(&format!(", {} todo", output.summary.tests_todo));
            }
            if let Some(duration) = output.summary.duration {
                result.push_str(&format!(" [{:.2}s]", duration));
            }
            result.push('\n');
            return result;
        }

        // Failure-focused summary - detailed output when tests fail
        result.push_str(&format!(
            "FAIL: {} suites ({} passed, {} failed), {} tests ({} passed, {} failed)",
            output.summary.suites_total,
            output.summary.suites_passed,
            output.summary.suites_failed,
            output.summary.tests_total,
            output.summary.tests_passed,
            output.summary.tests_failed
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!(", {} todo", output.summary.tests_todo));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!(" [{:.2}s]", duration));
        }
        result.push('\n');

        // List failed test suites
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str(&format!("failed suites ({}):\n", failed_suites.len()));
            for suite in failed_suites {
                result.push_str(&format!("  {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == JestTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("    ✕ {}\n", test.name));
                    if let Some(ref msg) = test.error_message {
                        if let Some(first_line) = msg.lines().next() {
                            let truncated = if first_line.len() > 80 {
                                format!("{}...", &first_line[..77])
                            } else {
                                first_line.to_string()
                            };
                            result.push_str(&format!("      {}\n", truncated));
                        }
                    }
                }
            }
        }

        result
    }

    /// Format Jest output as raw (just test names with status).
    fn format_jest_raw(output: &JestOutput) -> String {
        let mut result = String::new();

        for suite in &output.test_suites {
            let suite_status = if suite.passed { "PASS" } else { "FAIL" };
            result.push_str(&format!("{} {}\n", suite_status, suite.file));
            for test in &suite.tests {
                let status = match test.status {
                    JestTestStatus::Passed => "PASS",
                    JestTestStatus::Failed => "FAIL",
                    JestTestStatus::Skipped => "SKIP",
                    JestTestStatus::Todo => "TODO",
                };
                result.push_str(&format!("  {} {}\n", status, test.name));
            }
        }

        result
    }

    /// Format Jest output for AI agent consumption.
    fn format_jest_agent(output: &JestOutput) -> String {
        let mut result = String::new();

        result.push_str("# Test Results\n\n");

        if output.is_empty {
            result.push_str("Status: NO_TESTS\n");
            return result;
        }

        let status = if output.success { "SUCCESS" } else { "FAILURE" };
        result.push_str(&format!("Status: {}\n\n", status));

        // Summary
        result.push_str("## Summary\n");
        result.push_str(&format!(
            "- Suites: {} passed, {} failed, {} total\n",
            output.summary.suites_passed, output.summary.suites_failed, output.summary.suites_total
        ));
        result.push_str(&format!(
            "- Tests: {} passed, {} failed, {} total\n",
            output.summary.tests_passed, output.summary.tests_failed, output.summary.tests_total
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!("- Skipped: {}\n", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!("- Todo: {}\n", output.summary.tests_todo));
        }
        if let Some(snapshots) = output.summary.snapshots {
            result.push_str(&format!("- Snapshots: {}\n", snapshots));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!("- Duration: {:.2}s\n", duration));
        }
        result.push('\n');

        // Failed tests with details
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str("## Failed Test Suites\n\n");
            for suite in failed_suites {
                result.push_str(&format!("### {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == JestTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("- {}", test.name));
                    if let Some(duration) = test.duration {
                        result.push_str(&format!(" ({:.2}s)", duration));
                    }
                    result.push('\n');
                    if let Some(ref msg) = test.error_message {
                        result.push_str(&format!("\n```\n{}\n```\n", msg));
                    }
                }
                result.push('\n');
            }
        }

        result
    }

    // ============================================================
    // Vitest Parsing and Formatting
    // ============================================================

    /// Parse Vitest output into structured data.
    fn parse_vitest(input: &str) -> CommandResult<VitestOutput> {
        let mut output = VitestOutput::default();
        let mut current_suite: Option<VitestTestSuite> = None;
        let mut in_failure_details = false;
        let mut failure_buffer = String::new();
        let mut current_failed_test: Option<String> = None;
        let mut in_suite_tree = false;

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines (but preserve them in failure details)
            if trimmed.is_empty() && !in_failure_details {
                continue;
            }

            // Detect test suite header: "✓ test/example.test.ts (5 tests) 306ms"
            // or: "✓ test/example.test.ts (5 tests | 1 skipped) 306ms"
            // or: "✗ test/example.test.ts (5 tests | 1 failed) 306ms"
            if let Some(suite_info) = Self::parse_vitest_suite_header(trimmed) {
                // Save any pending suite
                if let Some(suite) = current_suite.take() {
                    output.test_suites.push(suite);
                }

                current_suite = Some(VitestTestSuite {
                    file: suite_info.file,
                    passed: suite_info.passed,
                    duration: suite_info.duration,
                    test_count: suite_info.test_count,
                    skipped_count: suite_info.skipped_count,
                    tests: Vec::new(),
                });
                in_failure_details = false;
                failure_buffer.clear();
                current_failed_test = None;
                in_suite_tree = true;
                continue;
            }

            // Detect test in tree format (indented test results)
            // "   ✓ test name 1ms" or "   ✕ test name"
            if in_suite_tree && line.starts_with("   ") {
                let test_line = line.trim_start();
                if let Some(test) = Self::parse_vitest_test_line(test_line) {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                    continue;
                }
            }

            // Detect failure details start
            // " ❯ test/file.test.ts:10:5"
            // "AssertionError: expected 5 to be 4"
            if trimmed.starts_with("❯ ") && trimmed.contains(".test.") {
                in_failure_details = true;
                // Save any previous failure info
                if let Some(name) = current_failed_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        if let Some(test) = suite.tests.iter_mut().find(|t| t.name == name) {
                            test.error_message = Some(failure_buffer.trim().to_string());
                        }
                    }
                }
                // Extract test name from file reference like "❯ test/file.test.ts:10:5 > test name"
                let remainder = trimmed.strip_prefix("❯ ").unwrap_or("");
                // The test name is often after the file location
                let name = if let Some(pos) = remainder.find('>') {
                    remainder[pos + 1..].trim().to_string()
                } else {
                    // Try to get just the file path context
                    remainder.to_string()
                };
                current_failed_test = Some(name);
                failure_buffer = String::new();
                continue;
            }

            // Detect assertion error line
            if trimmed.starts_with("AssertionError:") || trimmed.starts_with("Error:") {
                in_failure_details = true;
                failure_buffer.push_str(line);
                failure_buffer.push('\n');
                continue;
            }

            // Accumulate failure details
            if in_failure_details
                && (trimmed.starts_with("at ")
                    || trimmed.starts_with("expected")
                    || trimmed.contains("to be")
                    || failure_buffer.len() > 0)
            {
                failure_buffer.push_str(line);
                failure_buffer.push('\n');
                continue;
            }

            // Detect summary section
            // " Test Files  4 passed (4)"
            if trimmed.starts_with("Test Files") {
                let summary = Self::parse_vitest_test_files_summary(trimmed);
                output.summary.suites_passed = summary.suites_passed;
                output.summary.suites_failed = summary.suites_failed;
                output.summary.suites_total = summary.suites_total;
                in_suite_tree = false;
                continue;
            }

            // "      Tests  16 passed | 4 skipped (20)"
            if trimmed.starts_with("Tests") && !trimmed.starts_with("Tests:") {
                Self::parse_vitest_tests_summary(trimmed, &mut output.summary);
                continue;
            }

            // "   Start at  12:34:32"
            if trimmed.starts_with("Start at") {
                let time = trimmed.strip_prefix("Start at").unwrap_or("").trim();
                output.summary.start_at = Some(time.to_string());
                continue;
            }

            // "   Duration  1.26s"
            if trimmed.starts_with("Duration") {
                let duration_str = trimmed.strip_prefix("Duration").unwrap_or("").trim();
                output.summary.duration = Self::parse_vitest_duration(duration_str);
                continue;
            }
        }

        // Save any pending suite
        if let Some(suite) = current_suite.take() {
            output.test_suites.push(suite);
        }

        // Calculate totals if not already in summary
        if output.summary.suites_total == 0 && !output.test_suites.is_empty() {
            output.summary.suites_passed = output.test_suites.iter().filter(|s| s.passed).count();
            output.summary.suites_failed = output.test_suites.iter().filter(|s| !s.passed).count();
            output.summary.suites_total = output.test_suites.len();

            for suite in &output.test_suites {
                for test in &suite.tests {
                    match test.status {
                        VitestTestStatus::Passed => output.summary.tests_passed += 1,
                        VitestTestStatus::Failed => output.summary.tests_failed += 1,
                        VitestTestStatus::Skipped => output.summary.tests_skipped += 1,
                        VitestTestStatus::Todo => output.summary.tests_todo += 1,
                    }
                    output.summary.tests_total += 1;
                }
            }
        }

        // Determine success
        output.success = output.summary.tests_failed == 0
            && output.summary.suites_failed == 0
            && output.summary.tests_total > 0;
        output.is_empty = output.test_suites.is_empty() && output.summary.tests_total == 0;

        Ok(output)
    }

    /// Parse vitest suite header like "✓ test/example.test.ts (5 tests | 1 skipped) 306ms"
    fn parse_vitest_suite_header(line: &str) -> Option<VitestSuiteInfo> {
        let line = line.trim_start();

        let (passed, remainder) = if line.starts_with('✓') {
            (true, line.strip_prefix('✓')?.trim_start())
        } else if line.starts_with('✗') {
            (false, line.strip_prefix('✗')?.trim_start())
        } else if line.starts_with('×') {
            (false, line.strip_prefix('×')?.trim_start())
        } else if line.starts_with("FAIL") {
            (false, line.strip_prefix("FAIL")?.trim_start())
        } else if line.starts_with("PASS") {
            (true, line.strip_prefix("PASS")?.trim_start())
        } else {
            return None;
        };

        // Extract file path - everything before the parenthesis
        let paren_pos = remainder.find('(')?;
        let file = remainder[..paren_pos].trim().to_string();
        let rest = &remainder[paren_pos..];

        // Parse test count info: "(5 tests)" or "(5 tests | 1 skipped)" or "(5 tests | 1 failed)"
        let mut test_count = None;
        let mut skipped_count = None;

        if rest.starts_with('(') && rest.contains(')') {
            let end_paren = rest.find(')').unwrap_or(rest.len());
            let info = &rest[1..end_paren];

            // Extract test count
            if let Some(pos) = info.find(" test") {
                let num_str: String = info[..pos].chars().filter(|c| c.is_ascii_digit()).collect();
                if let Ok(num) = num_str.parse::<usize>() {
                    test_count = Some(num);
                }
            }

            // Extract skipped count
            if let Some(pos) = info.find("skipped") {
                let before = &info[..pos];
                if let Some(num_str) = before.rsplit('|').next() {
                    let num_str: String = num_str.chars().filter(|c| c.is_ascii_digit()).collect();
                    if let Ok(num) = num_str.parse::<usize>() {
                        skipped_count = Some(num);
                    }
                }
            }
        }

        // Extract duration - look for number followed by ms or s at the end
        let duration = if rest.contains("ms") || rest.contains('s') && !rest.contains("ms") {
            // Find duration at the end of the line
            let after_paren = rest.find(')').map(|p| &rest[p + 1..]).unwrap_or("");
            Self::parse_vitest_duration(after_paren.trim())
        } else {
            None
        };

        Some(VitestSuiteInfo {
            file,
            passed,
            duration,
            test_count,
            skipped_count,
        })
    }

    /// Parse a single Vitest test result line.
    fn parse_vitest_test_line(line: &str) -> Option<VitestTest> {
        // Trim leading whitespace
        let line = line.trim_start();

        // Skip if doesn't start with proper prefix
        // Vitest uses: ✓ (passed), ✕/× (failed), ↩ (skipped), etc.
        let (status, remainder) = if line.starts_with('✓') {
            (
                VitestTestStatus::Passed,
                line.strip_prefix('✓')?.trim_start(),
            )
        } else if line.starts_with('✕') {
            (
                VitestTestStatus::Failed,
                line.strip_prefix('✕')?.trim_start(),
            )
        } else if line.starts_with('×') {
            (
                VitestTestStatus::Failed,
                line.strip_prefix('×')?.trim_start(),
            )
        } else if line.starts_with('↩') {
            (
                VitestTestStatus::Skipped,
                line.strip_prefix('↩')?.trim_start(),
            )
        } else if line.starts_with("↓") {
            (
                VitestTestStatus::Skipped,
                line.strip_prefix("↓")?.trim_start(),
            )
        } else if line.contains("skipped") || line.contains("skip") {
            (VitestTestStatus::Skipped, line)
        } else if line.contains("todo") {
            (VitestTestStatus::Todo, line)
        } else {
            return None;
        };

        // Parse test name and duration
        let trimmed = remainder.trim();

        // Extract duration if present: "test name 1ms" or "test name 1.5s"
        let (test_name, duration) = if let Some(ms_pos) = trimmed.rfind("ms") {
            // Find the number before "ms"
            let before = &trimmed[..ms_pos];
            let num_start = before
                .rfind(|c: char| !c.is_ascii_digit() && c != '.')
                .map(|p| p + 1)
                .unwrap_or(0);
            let name_part = before[..num_start].trim();
            let duration_str = &before[num_start..];
            let duration = duration_str.parse::<f64>().ok().map(|d| d / 1000.0);
            (name_part.to_string(), duration)
        } else if let Some(s_pos) = trimmed.rfind('s') {
            // Check if it's a duration (not part of a word)
            let before = &trimmed[..s_pos];
            if before.ends_with(|c: char| c.is_ascii_digit()) {
                let num_start = before
                    .rfind(|c: char| !c.is_ascii_digit() && c != '.')
                    .map(|p| p + 1)
                    .unwrap_or(0);
                let name_part = before[..num_start].trim();
                let duration_str = &before[num_start..];
                let duration = duration_str.parse::<f64>().ok();
                (name_part.to_string(), duration)
            } else {
                (trimmed.to_string(), None)
            }
        } else {
            (trimmed.to_string(), None)
        };

        // Parse ancestors (describe blocks) from test name
        // Format: "describe block > nested describe > test name"
        let (ancestors, final_name) = if test_name.contains('>') || test_name.contains("›") {
            let delimiter = if test_name.contains('>') { ">" } else { "›" };
            let parts: Vec<&str> = test_name.split(delimiter).map(|s| s.trim()).collect();
            if parts.len() > 1 {
                let ancestors: Vec<String> = parts[..parts.len() - 1]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                let name = parts.last().unwrap_or(&"").to_string();
                (ancestors, name)
            } else {
                (Vec::new(), test_name.clone())
            }
        } else {
            (Vec::new(), test_name.clone())
        };

        Some(VitestTest {
            name: test_name,
            test_name: final_name,
            ancestors,
            status,
            duration,
            error_message: None,
        })
    }

    /// Parse Vitest duration string (e.g., "5ms", "1.26s").
    fn parse_vitest_duration(s: &str) -> Option<f64> {
        let s = s.trim();

        // Try to extract number and unit
        let num_str: String = s
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let num: f64 = num_str.parse().ok()?;

        // Convert to seconds based on unit
        if s.contains("ms") {
            Some(num / 1000.0)
        } else if s.contains('s') && !s.contains("ms") && !s.contains("start") {
            Some(num)
        } else if s.contains('m') && !s.contains("ms") {
            Some(num * 60.0)
        } else {
            // Assume milliseconds if no unit
            Some(num / 1000.0)
        }
    }

    /// Parse Vitest "Test Files" summary line.
    fn parse_vitest_test_files_summary(line: &str) -> VitestSummary {
        let mut summary = VitestSummary::default();
        let line = line.strip_prefix("Test Files").unwrap_or("").trim();

        // Parse pattern: "4 passed (4)" or "2 passed, 1 failed (3)"
        fn extract_count(text: &str, label: &str) -> usize {
            let pattern = format!(" {}", label);
            if let Some(pos) = text.find(&pattern) {
                let before = &text[..pos];
                let words: Vec<&str> = before.split_whitespace().collect();
                if let Some(last) = words.last() {
                    return last.parse().unwrap_or(0);
                }
            }
            0
        }

        summary.suites_passed = extract_count(line, "passed");
        summary.suites_failed = extract_count(line, "failed");

        // Total is in parentheses
        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                let total_str = &line[start + 1..end];
                summary.suites_total = total_str.parse().unwrap_or(0);
            }
        }

        summary
    }

    /// Parse Vitest "Tests" summary line.
    fn parse_vitest_tests_summary(line: &str, summary: &mut VitestSummary) {
        let line = line.strip_prefix("Tests").unwrap_or("").trim();

        fn extract_count(text: &str, label: &str) -> usize {
            let pattern = format!(" {}", label);
            if let Some(pos) = text.find(&pattern) {
                let before = &text[..pos];
                // Find the number before the label
                let words: Vec<&str> = before.split_whitespace().collect();
                if let Some(last) = words.last() {
                    return last.parse().unwrap_or(0);
                }
            }
            0
        }

        summary.tests_passed = extract_count(line, "passed");
        summary.tests_failed = extract_count(line, "failed");
        summary.tests_skipped = extract_count(line, "skipped");
        summary.tests_todo = extract_count(line, "todo");

        // Total is in parentheses at the end
        if let Some(start) = line.rfind('(') {
            if let Some(end) = line.rfind(')') {
                if end > start {
                    let total_str = &line[start + 1..end];
                    summary.tests_total = total_str.parse().unwrap_or(0);
                }
            }
        }
    }

    /// Format Vitest output based on the requested format.
    fn format_vitest(output: &VitestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_vitest_json(output),
            OutputFormat::Compact => Self::format_vitest_compact(output),
            OutputFormat::Raw => Self::format_vitest_raw(output),
            OutputFormat::Agent => Self::format_vitest_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_vitest_compact(output),
        }
    }

    /// Format Vitest output as JSON.
    fn format_vitest_json(output: &VitestOutput) -> String {
        // Extract failing test identifiers (file::test_name format)
        let failed_tests: Vec<_> = output
            .test_suites
            .iter()
            .flat_map(|suite| {
                suite
                    .tests
                    .iter()
                    .filter(|t| t.status == VitestTestStatus::Failed)
                    .map(|t| format!("{}::{}", suite.file, t.name))
            })
            .collect();

        serde_json::json!({
            "success": output.success,
            "is_empty": output.is_empty,
            "summary": {
                "suites": {
                    "passed": output.summary.suites_passed,
                    "failed": output.summary.suites_failed,
                    "total": output.summary.suites_total,
                },
                "tests": {
                    "passed": output.summary.tests_passed,
                    "failed": output.summary.tests_failed,
                    "skipped": output.summary.tests_skipped,
                    "todo": output.summary.tests_todo,
                    "total": output.summary.tests_total,
                },
                "duration": output.summary.duration,
                "start_at": output.summary.start_at,
            },
            "failed_tests": failed_tests,
            "test_suites": output.test_suites.iter().map(|suite| serde_json::json!({
                "file": suite.file,
                "passed": suite.passed,
                "duration": suite.duration,
                "test_count": suite.test_count,
                "skipped_count": suite.skipped_count,
                "tests": suite.tests.iter().map(|t| serde_json::json!({
                    "name": t.name,
                    "test_name": t.test_name,
                    "ancestors": t.ancestors,
                    "status": match t.status {
                        VitestTestStatus::Passed => "passed",
                        VitestTestStatus::Failed => "failed",
                        VitestTestStatus::Skipped => "skipped",
                        VitestTestStatus::Todo => "todo",
                    },
                    "duration": t.duration,
                    "error_message": t.error_message,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "vitest_version": output.vitest_version,
        })
        .to_string()
    }

    /// Format Vitest output in compact format.
    fn format_vitest_compact(output: &VitestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("no tests found\n");
            return result;
        }

        // Compact success summary - minimal output when all tests pass
        if output.success {
            result.push_str(&format!(
                "PASS: {} test files, {} tests",
                output.summary.suites_total, output.summary.tests_passed
            ));
            if output.summary.tests_skipped > 0 {
                result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
            }
            if output.summary.tests_todo > 0 {
                result.push_str(&format!(", {} todo", output.summary.tests_todo));
            }
            if let Some(duration) = output.summary.duration {
                result.push_str(&format!(" [{:.2}s]", duration));
            }
            result.push('\n');
            return result;
        }

        // Failure-focused summary - detailed output when tests fail
        result.push_str(&format!(
            "FAIL: {} test files ({} passed, {} failed), {} tests ({} passed, {} failed)",
            output.summary.suites_total,
            output.summary.suites_passed,
            output.summary.suites_failed,
            output.summary.tests_total,
            output.summary.tests_passed,
            output.summary.tests_failed
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!(", {} todo", output.summary.tests_todo));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!(" [{:.2}s]", duration));
        }
        result.push('\n');

        // List failed test suites
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str(&format!("failed suites ({}):\n", failed_suites.len()));
            for suite in failed_suites {
                result.push_str(&format!("  {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == VitestTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("    ✕ {}\n", test.name));
                    if let Some(ref msg) = test.error_message {
                        if let Some(first_line) = msg.lines().next() {
                            let truncated = if first_line.len() > 80 {
                                format!("{}...", &first_line[..77])
                            } else {
                                first_line.to_string()
                            };
                            result.push_str(&format!("      {}\n", truncated));
                        }
                    }
                }
            }
        }

        result
    }

    /// Format Vitest output as raw (just test names with status).
    fn format_vitest_raw(output: &VitestOutput) -> String {
        let mut result = String::new();

        for suite in &output.test_suites {
            let suite_status = if suite.passed { "PASS" } else { "FAIL" };
            result.push_str(&format!("{} {}\n", suite_status, suite.file));
            for test in &suite.tests {
                let status = match test.status {
                    VitestTestStatus::Passed => "PASS",
                    VitestTestStatus::Failed => "FAIL",
                    VitestTestStatus::Skipped => "SKIP",
                    VitestTestStatus::Todo => "TODO",
                };
                result.push_str(&format!("  {} {}\n", status, test.name));
            }
        }

        result
    }

    /// Format Vitest output for AI agent consumption.
    fn format_vitest_agent(output: &VitestOutput) -> String {
        let mut result = String::new();

        result.push_str("# Test Results\n\n");

        if output.is_empty {
            result.push_str("Status: NO_TESTS\n");
            return result;
        }

        let status = if output.success { "SUCCESS" } else { "FAILURE" };
        result.push_str(&format!("Status: {}\n\n", status));

        // Summary
        result.push_str("## Summary\n");
        result.push_str(&format!(
            "- Test Files: {} passed, {} failed, {} total\n",
            output.summary.suites_passed, output.summary.suites_failed, output.summary.suites_total
        ));
        result.push_str(&format!(
            "- Tests: {} passed, {} failed, {} total\n",
            output.summary.tests_passed, output.summary.tests_failed, output.summary.tests_total
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!("- Skipped: {}\n", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!("- Todo: {}\n", output.summary.tests_todo));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!("- Duration: {:.2}s\n", duration));
        }
        if let Some(ref start_at) = output.summary.start_at {
            result.push_str(&format!("- Start at: {}\n", start_at));
        }
        result.push('\n');

        // Failed tests with details
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str("## Failed Test Files\n\n");
            for suite in failed_suites {
                result.push_str(&format!("### {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == VitestTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("- {}", test.name));
                    if let Some(duration) = test.duration {
                        result.push_str(&format!(" ({:.2}s)", duration));
                    }
                    result.push('\n');
                    if let Some(ref msg) = test.error_message {
                        result.push_str(&format!("\n```\n{}\n```\n", msg));
                    }
                }
                result.push('\n');
            }
        }

        result
    }

    // ============================================================
    // NPM Test (Node.js built-in test runner) Parser
    // ============================================================

    /// Parse npm test output into structured data.
    ///
    /// The Node.js built-in test runner (node --test) with spec reporter outputs:
    /// ```text
    /// ▶ test/file.test.js
    ///   ✔ test name (5.123ms)
    ///   ✖ failing test
    ///     code: ...
    ///   ℹ skipped test # SKIP
    ///   ℹ todo test # TODO
    /// ▶ test/file.test.js (12.345ms)
    /// ```
    fn parse_npm_test(input: &str) -> CommandResult<NpmTestOutput> {
        let mut output = NpmTestOutput::default();
        let mut current_suite: Option<NpmTestSuite> = None;
        let mut current_test: Option<NpmTest> = None;
        let mut in_error_details = false;
        let mut error_buffer = String::new();
        let mut indent_stack: Vec<String> = Vec::new(); // Track nested test names

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Check for npm test output header (e.g., "> project@1.0.0 test")
            if trimmed.starts_with('>') && trimmed.contains("test") {
                continue;
            }

            // Check for summary lines at the end
            // "✔ tests 4 passed (4)" or "✖ tests 2 failed (4)"
            if trimmed.starts_with("✔ tests") || trimmed.starts_with("✖ tests") {
                Self::parse_npm_test_summary_tests(trimmed, &mut output.summary);
                continue;
            }

            // "✔ test files 2 passed (2)" or "✖ test files 1 failed (2)"
            if trimmed.starts_with("✔ test files") || trimmed.starts_with("✖ test files") {
                Self::parse_npm_test_summary_files(trimmed, &mut output.summary);
                continue;
            }

            // "ℹ tests 4 passed (4)" (alternative format)
            if trimmed.starts_with("ℹ tests") {
                Self::parse_npm_test_summary_tests_info(trimmed, &mut output.summary);
                continue;
            }

            // "ℹ test files 2 passed (2)" (alternative format)
            if trimmed.starts_with("ℹ test files") {
                Self::parse_npm_test_summary_files_info(trimmed, &mut output.summary);
                continue;
            }

            // "ℹ duration 123ms" or "ℹ duration 1.234s"
            if trimmed.starts_with("ℹ duration") {
                let duration_str = trimmed.strip_prefix("ℹ duration").unwrap_or("").trim();
                output.summary.duration = Self::parse_npm_duration(duration_str);
                continue;
            }

            // Check for test file start: "▶ path/to/test.js"
            if trimmed.starts_with('▶') && !trimmed.contains('(') {
                // Save any pending suite
                if let Some(suite) = current_suite.take() {
                    output.test_suites.push(suite);
                }

                let file = trimmed
                    .strip_prefix('▶')
                    .unwrap_or(trimmed)
                    .trim()
                    .to_string();
                current_suite = Some(NpmTestSuite {
                    file,
                    passed: true,
                    duration: None,
                    tests: Vec::new(),
                });
                indent_stack.clear();
                in_error_details = false;
                continue;
            }

            // Check for test file end with duration: "▶ path/to/test.js (123.456ms)"
            if trimmed.starts_with('▶') && trimmed.contains('(') {
                let duration = Self::extract_npm_suite_duration(trimmed);

                // First, save any pending test
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }

                if let Some(ref mut suite) = current_suite {
                    suite.duration = duration;
                }
                // Save the suite
                if let Some(suite) = current_suite.take() {
                    // Update suite passed status based on tests
                    let has_failures = suite
                        .tests
                        .iter()
                        .any(|t| t.status == NpmTestStatus::Failed);
                    let suite_to_save = NpmTestSuite {
                        passed: !has_failures,
                        ..suite
                    };
                    output.test_suites.push(suite_to_save);
                }
                indent_stack.clear();
                in_error_details = false;
                continue;
            }

            // Parse test results
            // Check if line is inside a test suite (indented or starts with test marker)
            let is_test_line = line.starts_with("  ")
                || line.starts_with("\t")
                || trimmed.starts_with("✔")
                || trimmed.starts_with("✖")
                || trimmed.starts_with("ℹ");

            if is_test_line && current_suite.is_some() {
                // Count indentation level (2 spaces per level)
                let indent = line.chars().take_while(|&c| c == ' ').count() / 2;

                // Adjust indent stack
                while indent_stack.len() > indent {
                    indent_stack.pop();
                }

                // Handle error details (indented more than test line, no marker)
                if in_error_details
                    && !trimmed.starts_with("✔")
                    && !trimmed.starts_with("✖")
                    && !trimmed.starts_with("ℹ")
                {
                    if let Some(ref mut test) = current_test {
                        if !error_buffer.is_empty() {
                            error_buffer.push('\n');
                        }
                        error_buffer.push_str(trimmed);
                        test.error_message = Some(error_buffer.clone());
                    }
                    continue;
                }

                // Save previous test if we're starting a new one at same or lower indent
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }

                // Parse test line
                if let Some(test) = Self::parse_npm_test_line(trimmed, &indent_stack) {
                    // Extract test_name before moving
                    let test_name = test.test_name.clone();
                    let is_failed = test.status == NpmTestStatus::Failed;

                    // Check for failed test to start collecting error details
                    if is_failed {
                        in_error_details = true;
                        error_buffer.clear();
                        current_test = Some(test);
                    } else {
                        in_error_details = false;
                        if let Some(ref mut suite) = current_suite {
                            suite.tests.push(test);
                        }
                    }

                    // Track nested test names
                    indent_stack.push(test_name);
                }
            }
        }

        // Save any pending test
        if let Some(test) = current_test {
            if let Some(ref mut suite) = current_suite {
                suite.tests.push(test);
            }
        }

        // Save any pending suite
        if let Some(suite) = current_suite.take() {
            let has_failures = suite
                .tests
                .iter()
                .any(|t| t.status == NpmTestStatus::Failed);
            let suite_to_save = NpmTestSuite {
                passed: !has_failures,
                ..suite
            };
            output.test_suites.push(suite_to_save);
        }

        // Set output properties
        output.is_empty = output.test_suites.is_empty();
        output.success = output.test_suites.iter().all(|s| s.passed);

        // Update summary counts from parsed tests
        Self::update_npm_summary_from_tests(&mut output);

        Ok(output)
    }

    /// Parse a single npm test result line.
    fn parse_npm_test_line(line: &str, ancestors: &[String]) -> Option<NpmTest> {
        let line = line.trim_start();

        // Parse passed test: "✔ test name (5.123ms)"
        if line.starts_with("✔") {
            let rest = line.strip_prefix("✔").unwrap_or(line).trim();
            let (name, duration) = Self::split_npm_test_name_and_duration(rest);
            return Some(NpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: NpmTestStatus::Passed,
                duration,
                error_message: None,
            });
        }

        // Parse failed test: "✖ test name"
        if line.starts_with("✖") {
            let rest = line.strip_prefix("✖").unwrap_or(line).trim();
            let name = rest.to_string();
            return Some(NpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: NpmTestStatus::Failed,
                duration: None,
                error_message: None,
            });
        }

        // Parse skipped test: "ℹ test name # SKIP"
        if line.starts_with("ℹ") && line.contains("# SKIP") {
            let rest = line.strip_prefix("ℹ").unwrap_or(line).trim();
            let name = rest
                .strip_suffix("# SKIP")
                .unwrap_or(rest)
                .trim()
                .to_string();
            return Some(NpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: NpmTestStatus::Skipped,
                duration: None,
                error_message: None,
            });
        }

        // Parse todo test: "ℹ test name # TODO"
        if line.starts_with("ℹ") && line.contains("# TODO") {
            let rest = line.strip_prefix("ℹ").unwrap_or(line).trim();
            let name = rest
                .strip_suffix("# TODO")
                .unwrap_or(rest)
                .trim()
                .to_string();
            return Some(NpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: NpmTestStatus::Todo,
                duration: None,
                error_message: None,
            });
        }

        None
    }

    /// Split test name and duration from a line like "test name (5.123ms)"
    fn split_npm_test_name_and_duration(line: &str) -> (String, Option<f64>) {
        // Look for duration at the end: "(5.123ms)" or "(1.234s)"
        if let Some(paren_pos) = line.rfind('(') {
            let name_part = line[..paren_pos].trim();
            let duration_part = &line[paren_pos..];
            if duration_part.ends_with("ms)") || duration_part.ends_with("s)") {
                let duration_str = &duration_part[1..duration_part.len() - 1]; // Remove parens
                let duration = Self::parse_npm_duration(duration_str);
                return (name_part.to_string(), duration);
            }
        }
        (line.to_string(), None)
    }

    /// Parse npm duration string (e.g., "5.123ms", "1.234s").
    fn parse_npm_duration(s: &str) -> Option<f64> {
        let s = s.trim();
        if s.ends_with("ms") {
            let num_str = &s[..s.len() - 2];
            num_str.parse::<f64>().ok().map(|n| n / 1000.0)
        } else if s.ends_with('s') {
            let num_str = &s[..s.len() - 1];
            num_str.parse::<f64>().ok()
        } else {
            None
        }
    }

    /// Extract duration from suite end line like "▶ path/to/test.js (123.456ms)"
    fn extract_npm_suite_duration(line: &str) -> Option<f64> {
        if let Some(paren_pos) = line.rfind('(') {
            let duration_part = &line[paren_pos..];
            if duration_part.ends_with("ms)") || duration_part.ends_with("s)") {
                let duration_str = &duration_part[1..duration_part.len() - 1];
                return Self::parse_npm_duration(duration_str);
            }
        }
        None
    }

    /// Parse npm test summary for tests: "✔ tests 4 passed (4)" or "✖ tests 2 failed (4)"
    fn parse_npm_test_summary_tests(line: &str, summary: &mut NpmTestSummary) {
        let line = line.trim_start_matches(|c| c == '✔' || c == '✖').trim();
        let line = line.strip_prefix("tests").unwrap_or("").trim();
        Self::parse_npm_counts(
            line,
            &mut summary.tests_passed,
            &mut summary.tests_failed,
            &mut summary.tests_skipped,
            &mut summary.tests_total,
        );
    }

    /// Parse npm test summary for test files: "✔ test files 2 passed (2)"
    fn parse_npm_test_summary_files(line: &str, summary: &mut NpmTestSummary) {
        let line = line.trim_start_matches(|c| c == '✔' || c == '✖').trim();
        let line = line.strip_prefix("test files").unwrap_or("").trim();
        Self::parse_npm_counts(
            line,
            &mut summary.suites_passed,
            &mut summary.suites_failed,
            &mut summary.suites_skipped,
            &mut summary.suites_total,
        );
    }

    /// Parse npm test summary for tests (info format): "ℹ tests 4 passed (4)"
    fn parse_npm_test_summary_tests_info(line: &str, summary: &mut NpmTestSummary) {
        let line = line.trim_start_matches('ℹ').trim();
        let line = line.strip_prefix("tests").unwrap_or("").trim();
        Self::parse_npm_counts_with_todo(
            line,
            &mut summary.tests_passed,
            &mut summary.tests_failed,
            &mut summary.tests_skipped,
            &mut summary.tests_todo,
            &mut summary.tests_total,
        );
    }

    /// Parse npm test summary for test files (info format): "ℹ test files 2 passed (2)"
    fn parse_npm_test_summary_files_info(line: &str, summary: &mut NpmTestSummary) {
        let line = line.trim_start_matches('ℹ').trim();
        let line = line.strip_prefix("test files").unwrap_or("").trim();
        Self::parse_npm_counts(
            line,
            &mut summary.suites_passed,
            &mut summary.suites_failed,
            &mut summary.suites_skipped,
            &mut summary.suites_total,
        );
    }

    /// Parse count pattern like "4 passed (4)" or "2 passed 1 failed (3)"
    fn parse_npm_counts(
        line: &str,
        passed: &mut usize,
        failed: &mut usize,
        skipped: &mut usize,
        total: &mut usize,
    ) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut i = 0;
        while i < parts.len() {
            if let Ok(count) = parts[i].parse::<usize>() {
                if i + 1 < parts.len() {
                    match parts[i + 1] {
                        "passed" => *passed = count,
                        "failed" => *failed = count,
                        "skipped" => *skipped = count,
                        _ => {}
                    }
                    i += 2;
                    continue;
                }
            }
            if parts[i].starts_with('(') && parts[i].ends_with(')') {
                let total_str = &parts[i][1..parts[i].len() - 1];
                if let Ok(t) = total_str.parse::<usize>() {
                    *total = t;
                }
            }
            i += 1;
        }
    }

    /// Parse npm test summary line with todo support.
    fn parse_npm_counts_with_todo(
        line: &str,
        passed: &mut usize,
        failed: &mut usize,
        skipped: &mut usize,
        todo: &mut usize,
        total: &mut usize,
    ) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut i = 0;
        while i < parts.len() {
            if let Ok(count) = parts[i].parse::<usize>() {
                if i + 1 < parts.len() {
                    match parts[i + 1] {
                        "passed" => *passed = count,
                        "failed" => *failed = count,
                        "skipped" => *skipped = count,
                        "todo" => *todo = count,
                        _ => {}
                    }
                    i += 2;
                    continue;
                }
            }
            if parts[i].starts_with('(') && parts[i].ends_with(')') {
                let total_str = &parts[i][1..parts[i].len() - 1];
                if let Ok(t) = total_str.parse::<usize>() {
                    *total = t;
                }
            }
            i += 1;
        }
    }

    /// Update summary counts from parsed tests.
    fn update_npm_summary_from_tests(output: &mut NpmTestOutput) {
        // Only update if summary wasn't already populated from output
        if output.summary.tests_total == 0 {
            for suite in &output.test_suites {
                output.summary.suites_total += 1;
                if suite.passed {
                    output.summary.suites_passed += 1;
                } else {
                    output.summary.suites_failed += 1;
                }

                for test in &suite.tests {
                    output.summary.tests_total += 1;
                    match test.status {
                        NpmTestStatus::Passed => output.summary.tests_passed += 1,
                        NpmTestStatus::Failed => output.summary.tests_failed += 1,
                        NpmTestStatus::Skipped => output.summary.tests_skipped += 1,
                        NpmTestStatus::Todo => output.summary.tests_todo += 1,
                    }
                }
            }
        }
    }

    /// Format npm test output based on the requested format.
    fn format_npm_test(output: &NpmTestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_npm_test_json(output),
            OutputFormat::Compact => Self::format_npm_test_compact(output),
            OutputFormat::Raw => Self::format_npm_test_raw(output),
            OutputFormat::Agent => Self::format_npm_test_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_npm_test_compact(output),
        }
    }

    /// Format npm test output as JSON.
    fn format_npm_test_json(output: &NpmTestOutput) -> String {
        // Extract failing test identifiers (file::test_name format)
        let failed_tests: Vec<_> = output
            .test_suites
            .iter()
            .flat_map(|suite| {
                suite
                    .tests
                    .iter()
                    .filter(|t| t.status == NpmTestStatus::Failed)
                    .map(|t| format!("{}::{}", suite.file, t.name))
            })
            .collect();

        serde_json::json!({
            "success": output.success,
            "is_empty": output.is_empty,
            "summary": {
                "suites_passed": output.summary.suites_passed,
                "suites_failed": output.summary.suites_failed,
                "suites_skipped": output.summary.suites_skipped,
                "suites_total": output.summary.suites_total,
                "tests_passed": output.summary.tests_passed,
                "tests_failed": output.summary.tests_failed,
                "tests_skipped": output.summary.tests_skipped,
                "tests_todo": output.summary.tests_todo,
                "tests_total": output.summary.tests_total,
                "duration": output.summary.duration,
            },
            "failed_tests": failed_tests,
            "test_suites": output.test_suites.iter().map(|suite| serde_json::json!({
                "file": suite.file,
                "passed": suite.passed,
                "duration": suite.duration,
                "tests": suite.tests.iter().map(|t| serde_json::json!({
                    "name": t.name,
                    "test_name": t.test_name,
                    "ancestors": t.ancestors,
                    "status": match t.status {
                        NpmTestStatus::Passed => "passed",
                        NpmTestStatus::Failed => "failed",
                        NpmTestStatus::Skipped => "skipped",
                        NpmTestStatus::Todo => "todo",
                    },
                    "duration": t.duration,
                    "error_message": t.error_message,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "node_version": output.node_version,
        })
        .to_string()
    }

    /// Format npm test output in compact format.
    fn format_npm_test_compact(output: &NpmTestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("npm test: no tests found\n");
            return result;
        }

        // Compact success summary - minimal output when all tests pass
        if output.success {
            result.push_str(&format!(
                "PASS: {} suites, {} tests",
                output.summary.suites_total, output.summary.tests_passed
            ));
            if output.summary.tests_skipped > 0 {
                result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
            }
            if let Some(duration) = output.summary.duration {
                result.push_str(&format!(", {:.2}s", duration));
            }
            result.push('\n');
            return result;
        }

        // Failure-focused summary - detailed output when tests fail
        // Group by passed/failed suites
        let passed_suites: Vec<_> = output.test_suites.iter().filter(|s| s.passed).collect();
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        // Show failed suites first
        for suite in &failed_suites {
            result.push_str(&format!(
                "FAIL: {} ({} tests)\n",
                suite.file,
                suite.tests.len()
            ));
            for test in &suite.tests {
                if test.status == NpmTestStatus::Failed {
                    result.push_str(&format!("  ✖ {}\n", test.test_name));
                }
            }
        }

        // Show passed suites summary
        if !passed_suites.is_empty() {
            result.push_str(&format!(
                "PASS: {} suites, {} tests\n",
                passed_suites.len(),
                passed_suites.iter().map(|s| s.tests.len()).sum::<usize>()
            ));
        }

        // Summary line
        result.push_str(&format!(
            "\n[FAIL] {} suites ({} passed, {} failed), {} tests ({} passed, {} failed)",
            output.summary.suites_total,
            output.summary.suites_passed,
            output.summary.suites_failed,
            output.summary.tests_total,
            output.summary.tests_passed,
            output.summary.tests_failed
        ));

        if output.summary.tests_skipped > 0 {
            result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
        }

        if let Some(duration) = output.summary.duration {
            result.push_str(&format!(", {:.2}s", duration));
        }

        result.push('\n');

        result
    }

    /// Format npm test output as raw (just test names with status).
    fn format_npm_test_raw(output: &NpmTestOutput) -> String {
        let mut result = String::new();

        for suite in &output.test_suites {
            let status = if suite.passed { "PASS" } else { "FAIL" };
            result.push_str(&format!("{} {}\n", status, suite.file));

            for test in &suite.tests {
                let test_status = match test.status {
                    NpmTestStatus::Passed => "PASS",
                    NpmTestStatus::Failed => "FAIL",
                    NpmTestStatus::Skipped => "SKIP",
                    NpmTestStatus::Todo => "TODO",
                };
                result.push_str(&format!("  {} {}\n", test_status, test.name));
            }
        }

        result
    }

    /// Format npm test output for AI agent consumption.
    fn format_npm_test_agent(output: &NpmTestOutput) -> String {
        let mut result = String::new();

        result.push_str("# Test Results\n\n");

        if output.is_empty {
            result.push_str("Status: NO_TESTS\n");
            return result;
        }

        let status = if output.success { "SUCCESS" } else { "FAILURE" };
        result.push_str(&format!("Status: {}\n\n", status));

        // Summary
        result.push_str("## Summary\n");
        result.push_str(&format!(
            "- Test Files: {} passed, {} failed, {} total\n",
            output.summary.suites_passed, output.summary.suites_failed, output.summary.suites_total
        ));
        result.push_str(&format!(
            "- Tests: {} passed, {} failed, {} total\n",
            output.summary.tests_passed, output.summary.tests_failed, output.summary.tests_total
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!("- Skipped: {}\n", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!("- Todo: {}\n", output.summary.tests_todo));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!("- Duration: {:.2}s\n", duration));
        }
        result.push('\n');

        // Failed tests with details
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str("## Failed Test Files\n\n");
            for suite in failed_suites {
                result.push_str(&format!("### {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == NpmTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("- {}", test.name));
                    if let Some(duration) = test.duration {
                        result.push_str(&format!(" ({:.2}s)", duration));
                    }
                    result.push('\n');
                    if let Some(ref msg) = test.error_message {
                        result.push_str(&format!("\n```\n{}\n```\n", msg));
                    }
                }
                result.push('\n');
            }
        }

        result
    }

    // ============================================================
    // PNPM Test Parser Implementation
    // ============================================================

    /// Parse pnpm test output into structured data.
    /// pnpm test output format is identical to npm test (Node.js built-in test runner).
    ///
    /// Expected format:
    /// ```text
    /// ▶ test/file.test.js
    ///   ✔ should work correctly (5.123ms)
    ///   ✖ should fail
    ///     AssertionError: values are not equal
    ///   ℹ skipped test # SKIP
    ///   ℹ todo test # TODO
    /// ▶ test/file.test.js (12.345ms)
    /// ```
    fn parse_pnpm_test(input: &str) -> CommandResult<PnpmTestOutput> {
        let mut output = PnpmTestOutput::default();
        let mut current_suite: Option<PnpmTestSuite> = None;
        let mut current_test: Option<PnpmTest> = None;
        let mut in_error_details = false;
        let mut error_buffer = String::new();
        let mut indent_stack: Vec<String> = Vec::new();

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Check for pnpm version line (e.g., "pnpm: 9.0.0")
            if trimmed.starts_with("pnpm:") || trimmed.starts_with("PNPM:") {
                output.pnpm_version = Some(
                    trimmed
                        .split(':')
                        .nth(1)
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default(),
                );
                continue;
            }

            // Check for summary lines at the end
            // "✔ tests 4 passed (4)" or "✖ tests 2 failed (4)"
            if trimmed.starts_with("✔ tests") || trimmed.starts_with("✖ tests") {
                Self::parse_pnpm_test_summary_tests(trimmed, &mut output.summary);
                continue;
            }

            // "✔ test files 2 passed (2)" or "✖ test files 1 failed (2)"
            if trimmed.starts_with("✔ test files") || trimmed.starts_with("✖ test files") {
                Self::parse_pnpm_test_summary_files(trimmed, &mut output.summary);
                continue;
            }

            // "ℹ tests 4 passed (4)" (alternative format)
            if trimmed.starts_with("ℹ tests") {
                Self::parse_pnpm_test_summary_tests_info(trimmed, &mut output.summary);
                continue;
            }

            // "ℹ test files 2 passed (2)" (alternative format)
            if trimmed.starts_with("ℹ test files") {
                Self::parse_pnpm_test_summary_files_info(trimmed, &mut output.summary);
                continue;
            }

            // "ℹ duration 123ms" or "ℹ duration 1.234s"
            if trimmed.starts_with("ℹ duration") {
                let duration_str = trimmed.strip_prefix("ℹ duration").unwrap_or("").trim();
                output.summary.duration = Self::parse_pnpm_duration(duration_str);
                continue;
            }

            // Check for test file start: "▶ path/to/test.js"
            if trimmed.starts_with('▶') && !trimmed.contains('(') {
                // Save any pending suite
                if let Some(suite) = current_suite.take() {
                    output.test_suites.push(suite);
                }

                let file = trimmed
                    .strip_prefix('▶')
                    .unwrap_or(trimmed)
                    .trim()
                    .to_string();
                current_suite = Some(PnpmTestSuite {
                    file,
                    passed: true,
                    duration: None,
                    tests: Vec::new(),
                });
                indent_stack.clear();
                in_error_details = false;
                continue;
            }

            // Check for test file end with duration: "▶ path/to/test.js (123.456ms)"
            if trimmed.starts_with('▶') && trimmed.contains('(') {
                let duration = Self::extract_pnpm_suite_duration(trimmed);

                // First, save any pending test
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }

                if let Some(ref mut suite) = current_suite {
                    suite.duration = duration;
                }
                // Save the suite
                if let Some(suite) = current_suite.take() {
                    // Update suite passed status based on tests
                    let has_failures = suite
                        .tests
                        .iter()
                        .any(|t| t.status == PnpmTestStatus::Failed);
                    let suite_to_save = PnpmTestSuite {
                        passed: !has_failures,
                        ..suite
                    };
                    output.test_suites.push(suite_to_save);
                }
                indent_stack.clear();
                in_error_details = false;
                continue;
            }

            // Parse test results
            // Check if line is inside a test suite (indented or starts with test marker)
            let is_test_line = line.starts_with("  ")
                || line.starts_with("\t")
                || trimmed.starts_with("✔")
                || trimmed.starts_with("✖")
                || trimmed.starts_with("ℹ");

            if is_test_line && current_suite.is_some() {
                // Count indentation level (2 spaces per level)
                let indent = line.chars().take_while(|&c| c == ' ').count() / 2;

                // Adjust indent stack
                while indent_stack.len() > indent {
                    indent_stack.pop();
                }

                // Handle error details (indented more than test line, no marker)
                if in_error_details
                    && !trimmed.starts_with("✔")
                    && !trimmed.starts_with("✖")
                    && !trimmed.starts_with("ℹ")
                {
                    if let Some(ref mut test) = current_test {
                        if !error_buffer.is_empty() {
                            error_buffer.push('\n');
                        }
                        error_buffer.push_str(trimmed);
                        test.error_message = Some(error_buffer.clone());
                    }
                    continue;
                }

                // Save previous test if we're starting a new one at same or lower indent
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }

                // Parse test line
                if let Some(test) = Self::parse_pnpm_test_line(trimmed, &indent_stack) {
                    // Extract test_name before moving
                    let test_name = test.test_name.clone();
                    let is_failed = test.status == PnpmTestStatus::Failed;

                    // Check for failed test to start collecting error details
                    if is_failed {
                        in_error_details = true;
                        error_buffer.clear();
                        current_test = Some(test);
                    } else {
                        in_error_details = false;
                        if let Some(ref mut suite) = current_suite {
                            suite.tests.push(test);
                        }
                    }

                    // Track nested test names
                    indent_stack.push(test_name);
                }
            }
        }

        // Save any pending test
        if let Some(test) = current_test {
            if let Some(ref mut suite) = current_suite {
                suite.tests.push(test);
            }
        }

        // Save any pending suite
        if let Some(suite) = current_suite.take() {
            let has_failures = suite
                .tests
                .iter()
                .any(|t| t.status == PnpmTestStatus::Failed);
            let suite_to_save = PnpmTestSuite {
                passed: !has_failures,
                ..suite
            };
            output.test_suites.push(suite_to_save);
        }

        // Set output properties
        output.is_empty = output.test_suites.is_empty();
        output.success = output.test_suites.iter().all(|s| s.passed);

        // Update summary counts from parsed tests
        Self::update_pnpm_summary_from_tests(&mut output);

        Ok(output)
    }

    /// Parse a single pnpm test result line.
    fn parse_pnpm_test_line(line: &str, ancestors: &[String]) -> Option<PnpmTest> {
        let line = line.trim_start();

        // Parse passed test: "✔ test name (5.123ms)"
        if line.starts_with("✔") {
            let rest = line.strip_prefix("✔").unwrap_or(line).trim();
            let (name, duration) = Self::split_pnpm_test_name_and_duration(rest);
            return Some(PnpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: PnpmTestStatus::Passed,
                duration,
                error_message: None,
            });
        }

        // Parse failed test: "✖ test name"
        if line.starts_with("✖") {
            let rest = line.strip_prefix("✖").unwrap_or(line).trim();
            let name = rest.to_string();
            return Some(PnpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: PnpmTestStatus::Failed,
                duration: None,
                error_message: None,
            });
        }

        // Parse skipped test: "ℹ test name # SKIP"
        if line.starts_with("ℹ") && line.contains("# SKIP") {
            let rest = line.strip_prefix("ℹ").unwrap_or(line).trim();
            let name = rest
                .strip_suffix("# SKIP")
                .unwrap_or(rest)
                .trim()
                .to_string();
            return Some(PnpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: PnpmTestStatus::Skipped,
                duration: None,
                error_message: None,
            });
        }

        // Parse todo test: "ℹ test name # TODO"
        if line.starts_with("ℹ") && line.contains("# TODO") {
            let rest = line.strip_prefix("ℹ").unwrap_or(line).trim();
            let name = rest
                .strip_suffix("# TODO")
                .unwrap_or(rest)
                .trim()
                .to_string();
            return Some(PnpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: PnpmTestStatus::Todo,
                duration: None,
                error_message: None,
            });
        }

        None
    }

    /// Parse duration string like "5.123ms" or "1.234s" into seconds.
    fn parse_pnpm_duration(s: &str) -> Option<f64> {
        let s = s.trim();
        if s.ends_with("ms") {
            s.strip_suffix("ms")
                .and_then(|n| n.parse::<f64>().ok())
                .map(|ms| ms / 1000.0)
        } else if s.ends_with("s") {
            s.strip_suffix("s").and_then(|n| n.parse::<f64>().ok())
        } else {
            None
        }
    }

    /// Split test name and duration from a string like "test name (5.123ms)".
    fn split_pnpm_test_name_and_duration(s: &str) -> (String, Option<f64>) {
        // Look for duration in parentheses at the end
        if let Some(start) = s.rfind('(') {
            if let Some(end) = s[start..].find(')') {
                let duration_str = &s[start + 1..start + end];
                let name = s[..start].trim().to_string();
                let duration = Self::parse_pnpm_duration(duration_str);
                return (name, duration);
            }
        }
        (s.to_string(), None)
    }

    /// Extract duration from suite end line like "▶ test.js (123.456ms)".
    fn extract_pnpm_suite_duration(line: &str) -> Option<f64> {
        if let Some(start) = line.rfind('(') {
            if let Some(end) = line[start..].find(')') {
                let duration_str = &line[start + 1..start + end];
                return Self::parse_pnpm_duration(duration_str);
            }
        }
        None
    }

    /// Parse pnpm test summary for tests: "✔ tests 4 passed (4)" or "✖ tests 2 failed (4)"
    fn parse_pnpm_test_summary_tests(line: &str, summary: &mut PnpmTestSummary) {
        let line = line.trim_start_matches(|c| c == '✔' || c == '✖').trim();
        let line = line.strip_prefix("tests").unwrap_or("").trim();
        Self::parse_pnpm_counts(
            line,
            &mut summary.tests_passed,
            &mut summary.tests_failed,
            &mut summary.tests_skipped,
            &mut summary.tests_total,
        );
    }

    /// Parse pnpm test summary for test files: "✔ test files 2 passed (2)"
    fn parse_pnpm_test_summary_files(line: &str, summary: &mut PnpmTestSummary) {
        let line = line.trim_start_matches(|c| c == '✔' || c == '✖').trim();
        let line = line.strip_prefix("test files").unwrap_or("").trim();
        Self::parse_pnpm_counts(
            line,
            &mut summary.suites_passed,
            &mut summary.suites_failed,
            &mut summary.suites_skipped,
            &mut summary.suites_total,
        );
    }

    /// Parse pnpm test summary for tests (info format): "ℹ tests 4 passed (4)"
    fn parse_pnpm_test_summary_tests_info(line: &str, summary: &mut PnpmTestSummary) {
        let line = line.trim_start_matches('ℹ').trim();
        let line = line.strip_prefix("tests").unwrap_or("").trim();
        Self::parse_pnpm_counts_with_todo(
            line,
            &mut summary.tests_passed,
            &mut summary.tests_failed,
            &mut summary.tests_skipped,
            &mut summary.tests_todo,
            &mut summary.tests_total,
        );
    }

    /// Parse pnpm test summary for test files (info format): "ℹ test files 2 passed (2)"
    fn parse_pnpm_test_summary_files_info(line: &str, summary: &mut PnpmTestSummary) {
        let line = line.trim_start_matches('ℹ').trim();
        let line = line.strip_prefix("test files").unwrap_or("").trim();
        Self::parse_pnpm_counts(
            line,
            &mut summary.suites_passed,
            &mut summary.suites_failed,
            &mut summary.suites_skipped,
            &mut summary.suites_total,
        );
    }

    /// Parse count pattern like "4 passed (4)" or "2 passed 1 failed (3)"
    fn parse_pnpm_counts(
        line: &str,
        passed: &mut usize,
        failed: &mut usize,
        skipped: &mut usize,
        total: &mut usize,
    ) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut i = 0;
        while i < parts.len() {
            if let Ok(count) = parts[i].parse::<usize>() {
                if i + 1 < parts.len() {
                    match parts[i + 1] {
                        "passed" => *passed = count,
                        "failed" => *failed = count,
                        "skipped" => *skipped = count,
                        _ => {}
                    }
                    i += 2;
                    continue;
                }
            }
            if parts[i].starts_with('(') && parts[i].ends_with(')') {
                let total_str = &parts[i][1..parts[i].len() - 1];
                if let Ok(t) = total_str.parse::<usize>() {
                    *total = t;
                }
            }
            i += 1;
        }
    }

    /// Parse pnpm test summary line with todo support.
    fn parse_pnpm_counts_with_todo(
        line: &str,
        passed: &mut usize,
        failed: &mut usize,
        skipped: &mut usize,
        todo: &mut usize,
        total: &mut usize,
    ) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut i = 0;
        while i < parts.len() {
            if let Ok(count) = parts[i].parse::<usize>() {
                if i + 1 < parts.len() {
                    match parts[i + 1] {
                        "passed" => *passed = count,
                        "failed" => *failed = count,
                        "skipped" => *skipped = count,
                        "todo" => *todo = count,
                        _ => {}
                    }
                    i += 2;
                    continue;
                }
            }
            if parts[i].starts_with('(') && parts[i].ends_with(')') {
                let total_str = &parts[i][1..parts[i].len() - 1];
                if let Ok(t) = total_str.parse::<usize>() {
                    *total = t;
                }
            }
            i += 1;
        }
    }

    /// Update summary counts from parsed tests.
    fn update_pnpm_summary_from_tests(output: &mut PnpmTestOutput) {
        // Only update if summary wasn't already populated from output
        if output.summary.tests_total == 0 {
            for suite in &output.test_suites {
                output.summary.suites_total += 1;
                if suite.passed {
                    output.summary.suites_passed += 1;
                } else {
                    output.summary.suites_failed += 1;
                }

                for test in &suite.tests {
                    output.summary.tests_total += 1;
                    match test.status {
                        PnpmTestStatus::Passed => output.summary.tests_passed += 1,
                        PnpmTestStatus::Failed => output.summary.tests_failed += 1,
                        PnpmTestStatus::Skipped => output.summary.tests_skipped += 1,
                        PnpmTestStatus::Todo => output.summary.tests_todo += 1,
                    }
                }
            }
        }
    }

    /// Format pnpm test output based on the requested format.
    fn format_pnpm_test(output: &PnpmTestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_pnpm_test_json(output),
            OutputFormat::Compact => Self::format_pnpm_test_compact(output),
            OutputFormat::Raw => Self::format_pnpm_test_raw(output),
            OutputFormat::Agent => Self::format_pnpm_test_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_pnpm_test_compact(output),
        }
    }

    /// Format pnpm test output as JSON.
    fn format_pnpm_test_json(output: &PnpmTestOutput) -> String {
        // Extract failing test identifiers (file::test_name format)
        let failed_tests: Vec<_> = output
            .test_suites
            .iter()
            .flat_map(|suite| {
                suite
                    .tests
                    .iter()
                    .filter(|t| t.status == PnpmTestStatus::Failed)
                    .map(|t| format!("{}::{}", suite.file, t.name))
            })
            .collect();

        serde_json::json!({
            "success": output.success,
            "is_empty": output.is_empty,
            "summary": {
                "suites_passed": output.summary.suites_passed,
                "suites_failed": output.summary.suites_failed,
                "suites_skipped": output.summary.suites_skipped,
                "suites_total": output.summary.suites_total,
                "tests_passed": output.summary.tests_passed,
                "tests_failed": output.summary.tests_failed,
                "tests_skipped": output.summary.tests_skipped,
                "tests_todo": output.summary.tests_todo,
                "tests_total": output.summary.tests_total,
                "duration": output.summary.duration,
            },
            "failed_tests": failed_tests,
            "test_suites": output.test_suites.iter().map(|suite| serde_json::json!({
                "file": suite.file,
                "passed": suite.passed,
                "duration": suite.duration,
                "tests": suite.tests.iter().map(|t| serde_json::json!({
                    "name": t.name,
                    "test_name": t.test_name,
                    "ancestors": t.ancestors,
                    "status": match t.status {
                        PnpmTestStatus::Passed => "passed",
                        PnpmTestStatus::Failed => "failed",
                        PnpmTestStatus::Skipped => "skipped",
                        PnpmTestStatus::Todo => "todo",
                    },
                    "duration": t.duration,
                    "error_message": t.error_message,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "pnpm_version": output.pnpm_version,
        })
        .to_string()
    }

    /// Format pnpm test output in compact format.
    fn format_pnpm_test_compact(output: &PnpmTestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("pnpm test: no tests found\n");
            return result;
        }

        // Compact success summary - minimal output when all tests pass
        if output.success {
            result.push_str(&format!(
                "PASS: {} suites, {} tests",
                output.summary.suites_total, output.summary.tests_passed
            ));
            if output.summary.tests_skipped > 0 {
                result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
            }
            if let Some(duration) = output.summary.duration {
                result.push_str(&format!(", {:.2}s", duration));
            }
            result.push('\n');
            return result;
        }

        // Failure-focused summary - detailed output when tests fail
        // Group by passed/failed suites
        let passed_suites: Vec<_> = output.test_suites.iter().filter(|s| s.passed).collect();
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        // Show failed suites first
        for suite in &failed_suites {
            result.push_str(&format!(
                "FAIL: {} ({} tests)\n",
                suite.file,
                suite.tests.len()
            ));
            for test in &suite.tests {
                if test.status == PnpmTestStatus::Failed {
                    result.push_str(&format!("  ✖ {}\n", test.test_name));
                }
            }
        }

        // Show passed suites summary
        if !passed_suites.is_empty() {
            result.push_str(&format!(
                "PASS: {} suites, {} tests\n",
                passed_suites.len(),
                passed_suites.iter().map(|s| s.tests.len()).sum::<usize>()
            ));
        }

        // Summary line
        result.push_str(&format!(
            "\n[FAIL] {} suites ({} passed, {} failed), {} tests ({} passed, {} failed)",
            output.summary.suites_total,
            output.summary.suites_passed,
            output.summary.suites_failed,
            output.summary.tests_total,
            output.summary.tests_passed,
            output.summary.tests_failed
        ));

        if output.summary.tests_skipped > 0 {
            result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
        }

        if let Some(duration) = output.summary.duration {
            result.push_str(&format!(", {:.2}s", duration));
        }

        result.push('\n');

        result
    }

    /// Format pnpm test output as raw (just test names with status).
    fn format_pnpm_test_raw(output: &PnpmTestOutput) -> String {
        let mut result = String::new();

        for suite in &output.test_suites {
            let status = if suite.passed { "PASS" } else { "FAIL" };
            result.push_str(&format!("{} {}\n", status, suite.file));

            for test in &suite.tests {
                let test_status = match test.status {
                    PnpmTestStatus::Passed => "PASS",
                    PnpmTestStatus::Failed => "FAIL",
                    PnpmTestStatus::Skipped => "SKIP",
                    PnpmTestStatus::Todo => "TODO",
                };
                result.push_str(&format!("  {} {}\n", test_status, test.name));
            }
        }

        result
    }

    /// Format pnpm test output for AI agent consumption.
    fn format_pnpm_test_agent(output: &PnpmTestOutput) -> String {
        let mut result = String::new();

        result.push_str("# Test Results\n\n");

        if output.is_empty {
            result.push_str("Status: NO_TESTS\n");
            return result;
        }

        let status = if output.success { "SUCCESS" } else { "FAILURE" };
        result.push_str(&format!("Status: {}\n\n", status));

        // Summary
        result.push_str("## Summary\n");
        result.push_str(&format!(
            "- Test Files: {} passed, {} failed, {} total\n",
            output.summary.suites_passed, output.summary.suites_failed, output.summary.suites_total
        ));
        result.push_str(&format!(
            "- Tests: {} passed, {} failed, {} total\n",
            output.summary.tests_passed, output.summary.tests_failed, output.summary.tests_total
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!("- Skipped: {}\n", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!("- Todo: {}\n", output.summary.tests_todo));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!("- Duration: {:.2}s\n", duration));
        }
        result.push('\n');

        // Failed tests with details
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str("## Failed Test Files\n\n");
            for suite in failed_suites {
                result.push_str(&format!("### {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == PnpmTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("- {}", test.name));
                    if let Some(duration) = test.duration {
                        result.push_str(&format!(" ({:.2}s)", duration));
                    }
                    result.push('\n');
                    if let Some(ref msg) = test.error_message {
                        result.push_str(&format!("\n```\n{}\n```\n", msg));
                    }
                }
                result.push('\n');
            }
        }

        result
    }

    // ============================================================
    // Bun Test Parsing and Formatting
    // ============================================================

    /// Parse Bun test output into structured data.
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
    fn parse_bun_test(input: &str) -> CommandResult<BunTestOutput> {
        let mut output = BunTestOutput::default();
        let mut current_suite: Option<BunTestSuite> = None;
        let mut current_test: Option<BunTest> = None;
        let mut in_error_details = false;
        let mut error_buffer = String::new();
        let mut indent_stack: Vec<String> = Vec::new();
        let mut in_suite = false;

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines, but first save any pending test
            if trimmed.is_empty() {
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }
                in_error_details = false;
                continue;
            }

            // Check for bun version line (e.g., "bun: 1.0.0" or "Bun v1.0.0")
            if trimmed.starts_with("bun:") || trimmed.starts_with("Bun v") {
                output.bun_version = Some(
                    trimmed
                        .split(|c| c == ':' || c == 'v')
                        .last()
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default(),
                );
                continue;
            }

            // Check for summary lines at the end
            // "X pass" or "Y fail" or "X expect() calls"
            if Self::is_bun_summary_line(trimmed) {
                // Save any pending test before processing summary
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }
                Self::parse_bun_summary_line(trimmed, &mut output.summary);
                continue;
            }

            // "Ran X tests in Yms" or "Ran X tests across Y files. [Zms]"
            if trimmed.starts_with("Ran ") && trimmed.contains(" tests") {
                Self::parse_bun_ran_line(trimmed, &mut output.summary);
                continue;
            }

            // Check for test file header: "test/file.test.ts:" (ends with colon)
            if trimmed.ends_with(':')
                && !trimmed.starts_with(|c| c == '✓' || c == '✗' || c == '×' || c == '(')
            {
                // Save any pending test
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }

                // Save any pending suite
                if let Some(suite) = current_suite.take() {
                    let has_failures = suite
                        .tests
                        .iter()
                        .any(|t| t.status == BunTestStatus::Failed);
                    let suite_to_save = BunTestSuite {
                        passed: !has_failures,
                        ..suite
                    };
                    output.test_suites.push(suite_to_save);
                }

                let file = trimmed.trim_end_matches(':').to_string();
                current_suite = Some(BunTestSuite {
                    file,
                    passed: true,
                    duration: None,
                    tests: Vec::new(),
                });
                indent_stack.clear();
                in_error_details = false;
                in_suite = true;
                continue;
            }

            // Parse test results if we're in a suite
            if in_suite && current_suite.is_some() {
                // Count indentation level (2 spaces per level)
                let indent = line.chars().take_while(|&c| c == ' ').count() / 2;

                // Adjust indent stack
                while indent_stack.len() > indent {
                    indent_stack.pop();
                }

                // Handle error details (indented more than test line, no marker)
                if in_error_details
                    && !trimmed.starts_with("✓")
                    && !trimmed.starts_with("✗")
                    && !trimmed.starts_with("×")
                    && !trimmed.starts_with("(pass)")
                    && !trimmed.starts_with("(fail)")
                    && !trimmed.starts_with("(skip)")
                    && !trimmed.starts_with("(todo)")
                {
                    if let Some(ref mut test) = current_test {
                        if !error_buffer.is_empty() {
                            error_buffer.push('\n');
                        }
                        error_buffer.push_str(trimmed);
                        test.error_message = Some(error_buffer.clone());
                    }
                    continue;
                }

                // Save previous test if we're starting a new one at same or lower indent
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }

                // Parse test line
                if let Some(test) = Self::parse_bun_test_line(trimmed, &indent_stack) {
                    let test_name = test.test_name.clone();
                    let is_failed = test.status == BunTestStatus::Failed;

                    // Check for failed test to start collecting error details
                    if is_failed {
                        in_error_details = true;
                        error_buffer.clear();
                        current_test = Some(test);
                    } else {
                        in_error_details = false;
                        if let Some(ref mut suite) = current_suite {
                            suite.tests.push(test);
                        }
                    }

                    // Track nested test names
                    indent_stack.push(test_name);
                }
            }
        }

        // Save any pending test
        if let Some(test) = current_test {
            if let Some(ref mut suite) = current_suite {
                suite.tests.push(test);
            }
        }

        // Save any pending suite
        if let Some(suite) = current_suite.take() {
            let has_failures = suite
                .tests
                .iter()
                .any(|t| t.status == BunTestStatus::Failed);
            let suite_to_save = BunTestSuite {
                passed: !has_failures,
                ..suite
            };
            output.test_suites.push(suite_to_save);
        }

        // Set output properties
        output.is_empty = output.test_suites.is_empty();
        output.success = output.test_suites.iter().all(|s| s.passed);

        // Update summary counts from parsed tests
        Self::update_bun_summary_from_tests(&mut output);

        Ok(output)
    }

    /// Parse a single Bun test result line.
    fn parse_bun_test_line(line: &str, ancestors: &[String]) -> Option<BunTest> {
        let line = line.trim_start();

        // Parse with color markers: "✓ test name [5.123ms]"
        if line.starts_with("✓") {
            let rest = line.strip_prefix("✓").unwrap_or(line).trim();
            let (name, duration) = Self::split_bun_test_name_and_duration(rest);
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Passed,
                duration,
                error_message: None,
            });
        }

        // Parse failed test with color markers: "✗ test name" or "× test name"
        if line.starts_with("✗") || line.starts_with("×") {
            let rest = line
                .strip_prefix("✗")
                .or_else(|| line.strip_prefix("×"))
                .unwrap_or(line)
                .trim();
            let name = rest.to_string();
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Failed,
                duration: None,
                error_message: None,
            });
        }

        // Parse non-TTY format: "(pass) test name [5.123ms]"
        if line.starts_with("(pass)") {
            let rest = line.strip_prefix("(pass)").unwrap_or(line).trim();
            let (name, duration) = Self::split_bun_test_name_and_duration(rest);
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Passed,
                duration,
                error_message: None,
            });
        }

        // Parse non-TTY format: "(fail) test name"
        if line.starts_with("(fail)") {
            let rest = line.strip_prefix("(fail)").unwrap_or(line).trim();
            let name = rest.to_string();
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Failed,
                duration: None,
                error_message: None,
            });
        }

        // Parse non-TTY format: "(skip) test name"
        if line.starts_with("(skip)") {
            let rest = line.strip_prefix("(skip)").unwrap_or(line).trim();
            let (name, _) = Self::split_bun_test_name_and_duration(rest);
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Skipped,
                duration: None,
                error_message: None,
            });
        }

        // Parse non-TTY format: "(todo) test name"
        if line.starts_with("(todo)") {
            let rest = line.strip_prefix("(todo)").unwrap_or(line).trim();
            let (name, _) = Self::split_bun_test_name_and_duration(rest);
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Todo,
                duration: None,
                error_message: None,
            });
        }

        None
    }

    /// Parse duration string like "5.123ms" or "1.234s" into seconds.
    fn parse_bun_duration(s: &str) -> Option<f64> {
        let s = s.trim();
        if s.ends_with("ms") {
            s.strip_suffix("ms")
                .and_then(|n| n.parse::<f64>().ok())
                .map(|ms| ms / 1000.0)
        } else if s.ends_with("s") {
            s.strip_suffix("s").and_then(|n| n.parse::<f64>().ok())
        } else {
            None
        }
    }

    /// Split test name and duration from a string like "test name [5.123ms]".
    fn split_bun_test_name_and_duration(s: &str) -> (String, Option<f64>) {
        // Look for duration in brackets at the end: "test name [5.123ms]"
        if let Some(start) = s.rfind('[') {
            if let Some(end) = s[start..].find(']') {
                let duration_str = &s[start + 1..start + end];
                let name = s[..start].trim().to_string();
                let duration = Self::parse_bun_duration(duration_str);
                return (name, duration);
            }
        }
        (s.to_string(), None)
    }

    /// Check if a line is a Bun summary line.
    fn is_bun_summary_line(line: &str) -> bool {
        let line = line.trim();
        // Match "X pass", "Y fail", "Z expect() calls", "W skipped"
        // These lines start with a number, not a test marker
        // Examples: " 4 pass", " 0 fail", " 4 expect() calls"
        // NOT: "✓ test pass" or "✗ should fail"

        // First check if line starts with a number (possibly with leading spaces)
        let starts_with_number = line
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false);

        if !starts_with_number {
            return false;
        }

        line.ends_with(" pass")
            || line.ends_with(" fail")
            || line.ends_with(" expect() calls")
            || line.ends_with(" skipped")
    }

    /// Parse a Bun summary line.
    fn parse_bun_summary_line(line: &str, summary: &mut BunTestSummary) {
        let line = line.trim();

        // Parse "X pass"
        if line.ends_with(" pass") {
            if let Some(count_str) = line.strip_suffix(" pass") {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    summary.tests_passed = count;
                }
            }
            return;
        }

        // Parse "Y fail"
        if line.ends_with(" fail") {
            if let Some(count_str) = line.strip_suffix(" fail") {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    summary.tests_failed = count;
                }
            }
            return;
        }

        // Parse "Z expect() calls"
        if line.ends_with(" expect() calls") {
            if let Some(count_str) = line.strip_suffix(" expect() calls") {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    summary.expect_calls = Some(count);
                }
            }
            return;
        }

        // Parse "X skipped"
        if line.ends_with(" skipped") {
            if let Some(count_str) = line.strip_suffix(" skipped") {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    summary.tests_skipped = count;
                }
            }
        }
    }

    /// Parse "Ran X tests in Yms" or "Ran X tests across Y files. [Zms]"
    fn parse_bun_ran_line(line: &str, summary: &mut BunTestSummary) {
        // Format: "Ran X tests in Yms" or "Ran X tests across Y files. [Zms]"
        let line = line.trim();

        // Extract total tests
        if let Some(start) = line.find("Ran ") {
            let after_ran = &line[start + 4..];
            if let Some(end) = after_ran.find(" tests") {
                if let Ok(count) = after_ran[..end].trim().parse::<usize>() {
                    summary.tests_total = count;
                }
            }
        }

        // Extract files count
        if let Some(start) = line.find("across ") {
            let after_across = &line[start + 7..];
            if let Some(end) = after_across.find(" files") {
                if let Ok(count) = after_across[..end].trim().parse::<usize>() {
                    summary.suites_total = count;
                }
            }
        }

        // Extract duration - format: "in 1.44ms" or "[1.44ms]"
        if let Some(start) = line.find("in ") {
            let after_in = &line[start + 3..];
            summary.duration = Self::parse_bun_duration(after_in);
        } else if let Some(start) = line.rfind('[') {
            if let Some(end) = line[start..].find(']') {
                let duration_str = &line[start + 1..start + end];
                summary.duration = Self::parse_bun_duration(duration_str);
            }
        }
    }

    /// Update summary counts from parsed tests.
    fn update_bun_summary_from_tests(output: &mut BunTestOutput) {
        // Always update suite counts since they may not be in the "Ran" line
        // (the "across X files" part is optional)
        if output.summary.suites_total == 0 {
            for suite in &output.test_suites {
                output.summary.suites_total += 1;
                if suite.passed {
                    output.summary.suites_passed += 1;
                } else {
                    output.summary.suites_failed += 1;
                }
            }
        }

        // Only update test counts if summary wasn't already populated from output
        if output.summary.tests_total == 0 {
            for suite in &output.test_suites {
                for test in &suite.tests {
                    output.summary.tests_total += 1;
                    match test.status {
                        BunTestStatus::Passed => output.summary.tests_passed += 1,
                        BunTestStatus::Failed => output.summary.tests_failed += 1,
                        BunTestStatus::Skipped => output.summary.tests_skipped += 1,
                        BunTestStatus::Todo => output.summary.tests_todo += 1,
                    }
                }
            }
        }
    }

    /// Format Bun test output based on the requested format.
    fn format_bun_test(output: &BunTestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_bun_test_json(output),
            OutputFormat::Compact => Self::format_bun_test_compact(output),
            OutputFormat::Raw => Self::format_bun_test_raw(output),
            OutputFormat::Agent => Self::format_bun_test_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_bun_test_compact(output),
        }
    }

    /// Format Bun test output as JSON.
    fn format_bun_test_json(output: &BunTestOutput) -> String {
        // Extract failing test identifiers (file::test_name format)
        let failed_tests: Vec<_> = output
            .test_suites
            .iter()
            .flat_map(|suite| {
                suite
                    .tests
                    .iter()
                    .filter(|t| t.status == BunTestStatus::Failed)
                    .map(|t| format!("{}::{}", suite.file, t.name))
            })
            .collect();

        serde_json::json!({
            "success": output.success,
            "is_empty": output.is_empty,
            "summary": {
                "suites_passed": output.summary.suites_passed,
                "suites_failed": output.summary.suites_failed,
                "suites_skipped": output.summary.suites_skipped,
                "suites_total": output.summary.suites_total,
                "tests_passed": output.summary.tests_passed,
                "tests_failed": output.summary.tests_failed,
                "tests_skipped": output.summary.tests_skipped,
                "tests_todo": output.summary.tests_todo,
                "tests_total": output.summary.tests_total,
                "expect_calls": output.summary.expect_calls,
                "duration": output.summary.duration,
            },
            "failed_tests": failed_tests,
            "test_suites": output.test_suites.iter().map(|suite| serde_json::json!({
                "file": suite.file,
                "passed": suite.passed,
                "duration": suite.duration,
                "tests": suite.tests.iter().map(|t| serde_json::json!({
                    "name": t.name,
                    "test_name": t.test_name,
                    "ancestors": t.ancestors,
                    "status": match t.status {
                        BunTestStatus::Passed => "passed",
                        BunTestStatus::Failed => "failed",
                        BunTestStatus::Skipped => "skipped",
                        BunTestStatus::Todo => "todo",
                    },
                    "duration": t.duration,
                    "error_message": t.error_message,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "bun_version": output.bun_version,
        })
        .to_string()
    }

    /// Format Bun test output in compact format.
    fn format_bun_test_compact(output: &BunTestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("bun test: no tests found\n");
            return result;
        }

        // Compact success summary - minimal output when all tests pass
        if output.success {
            result.push_str(&format!(
                "PASS: {} suites, {} tests",
                output.summary.suites_total, output.summary.tests_passed
            ));
            if output.summary.tests_skipped > 0 {
                result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
            }
            if let Some(duration) = output.summary.duration {
                result.push_str(&format!(", {:.2}s", duration));
            }
            result.push('\n');
            return result;
        }

        // Failure-focused summary - detailed output when tests fail
        // Group by passed/failed suites
        let passed_suites: Vec<_> = output.test_suites.iter().filter(|s| s.passed).collect();
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        // Show failed suites first
        for suite in &failed_suites {
            result.push_str(&format!(
                "FAIL: {} ({} tests)\n",
                suite.file,
                suite.tests.len()
            ));
            for test in &suite.tests {
                if test.status == BunTestStatus::Failed {
                    result.push_str(&format!("  ✖ {}\n", test.test_name));
                }
            }
        }

        // Show passed suites summary
        if !passed_suites.is_empty() {
            result.push_str(&format!(
                "PASS: {} suites, {} tests\n",
                passed_suites.len(),
                passed_suites.iter().map(|s| s.tests.len()).sum::<usize>()
            ));
        }

        // Summary line
        result.push_str(&format!(
            "\n[FAIL] {} suites ({} passed, {} failed), {} tests ({} passed, {} failed)",
            output.summary.suites_total,
            output.summary.suites_passed,
            output.summary.suites_failed,
            output.summary.tests_total,
            output.summary.tests_passed,
            output.summary.tests_failed
        ));

        if output.summary.tests_skipped > 0 {
            result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
        }

        if let Some(duration) = output.summary.duration {
            result.push_str(&format!(", {:.2}s", duration));
        }

        result.push('\n');

        result
    }

    /// Format Bun test output as raw (just test names with status).
    fn format_bun_test_raw(output: &BunTestOutput) -> String {
        let mut result = String::new();

        for suite in &output.test_suites {
            let status = if suite.passed { "PASS" } else { "FAIL" };
            result.push_str(&format!("{} {}\n", status, suite.file));

            for test in &suite.tests {
                let test_status = match test.status {
                    BunTestStatus::Passed => "PASS",
                    BunTestStatus::Failed => "FAIL",
                    BunTestStatus::Skipped => "SKIP",
                    BunTestStatus::Todo => "TODO",
                };
                result.push_str(&format!("  {} {}\n", test_status, test.name));
            }
        }

        result
    }

    /// Format Bun test output for AI agent consumption.
    fn format_bun_test_agent(output: &BunTestOutput) -> String {
        let mut result = String::new();

        result.push_str("# Test Results\n\n");

        if output.is_empty {
            result.push_str("Status: NO_TESTS\n");
            return result;
        }

        let status = if output.success { "SUCCESS" } else { "FAILURE" };
        result.push_str(&format!("Status: {}\n\n", status));

        // Summary
        result.push_str("## Summary\n");
        result.push_str(&format!(
            "- Test Files: {} passed, {} failed, {} total\n",
            output.summary.suites_passed, output.summary.suites_failed, output.summary.suites_total
        ));
        result.push_str(&format!(
            "- Tests: {} passed, {} failed, {} total\n",
            output.summary.tests_passed, output.summary.tests_failed, output.summary.tests_total
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!("- Skipped: {}\n", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!("- Todo: {}\n", output.summary.tests_todo));
        }
        if let Some(expect_calls) = output.summary.expect_calls {
            result.push_str(&format!("- Expect() calls: {}\n", expect_calls));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!("- Duration: {:.2}s\n", duration));
        }
        result.push('\n');

        // Failed tests with details
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str("## Failed Test Files\n\n");
            for suite in failed_suites {
                result.push_str(&format!("### {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == BunTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("- {}", test.name));
                    if let Some(duration) = test.duration {
                        result.push_str(&format!(" ({:.2}s)", duration));
                    }
                    result.push('\n');
                    if let Some(ref msg) = test.error_message {
                        result.push_str(&format!("\n```\n{}\n```\n", msg));
                    }
                }
                result.push('\n');
            }
        }

        result
    }

    /// Handle the logs subcommand.
    fn handle_logs(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the log output
        let logs_output = Self::parse_logs(&input);

        // Format output based on the requested format
        let output = Self::format_logs(&logs_output, ctx.format);
        print!("{}", output);

        Ok(())
    }

    /// Parse log output into structured data.
    ///
    /// Supports various log formats:
    /// - Common timestamp formats (ISO 8601, syslog, etc.)
    /// - Log levels: DEBUG, INFO, WARN/WARNING, ERROR, FATAL/CRITICAL
    /// - Various formats: `[LEVEL]`, `LEVEL:`, `|LEVEL|`, etc.
    fn parse_logs(input: &str) -> LogsOutput {
        let mut logs_output = LogsOutput::default();
        let mut line_tracker: std::collections::HashMap<String, (usize, usize, usize)> =
            std::collections::HashMap::new();

        for (idx, line) in input.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            // Skip empty lines but count them
            if trimmed.is_empty() {
                continue;
            }

            // Track repeated lines
            let entry = line_tracker
                .entry(trimmed.to_string())
                .or_insert((0, line_num, line_num));
            entry.0 += 1;
            entry.2 = line_num;

            // Parse the log line
            let log_entry = Self::parse_log_line(trimmed, line_num);
            logs_output.entries.push(log_entry.clone());
            logs_output.total_lines += 1;

            // Count by level
            match log_entry.level {
                LogLevel::Debug => logs_output.debug_count += 1,
                LogLevel::Info => logs_output.info_count += 1,
                LogLevel::Warning => logs_output.warning_count += 1,
                LogLevel::Error => logs_output.error_count += 1,
                LogLevel::Fatal => logs_output.fatal_count += 1,
                LogLevel::Unknown => logs_output.unknown_count += 1,
            }

            // Track recent critical lines (ERROR and FATAL)
            if log_entry.level == LogLevel::Error || log_entry.level == LogLevel::Fatal {
                logs_output.recent_critical.push(log_entry.clone());
                // Keep only the most recent MAX_RECENT_CRITICAL entries
                if logs_output.recent_critical.len() > MAX_RECENT_CRITICAL {
                    logs_output.recent_critical.remove(0);
                }
            }
        }

        // Build repeated lines list (only lines repeated more than once)
        for (line, (count, first_line, last_line)) in line_tracker {
            if count > 1 {
                logs_output.repeated_lines.push(RepeatedLine {
                    line,
                    count,
                    first_line,
                    last_line,
                });
            }
        }

        // Sort repeated lines by first occurrence
        logs_output.repeated_lines.sort_by_key(|r| r.first_line);

        logs_output.is_empty = logs_output.entries.is_empty();
        logs_output
    }

    /// Parse a single log line.
    fn parse_log_line(line: &str, line_number: usize) -> LogEntry {
        let mut entry = LogEntry {
            line: line.to_string(),
            level: LogLevel::Unknown,
            timestamp: None,
            source: None,
            message: line.to_string(),
            line_number,
        };

        // Try to extract timestamp
        entry.timestamp = Self::extract_timestamp(line);

        // Try to extract log level
        entry.level = Self::detect_log_level(line);

        // Extract message (remove timestamp and level prefix)
        entry.message = Self::extract_message(line, &entry.timestamp, &entry.level);

        entry
    }

    /// Extract timestamp from a log line.
    fn extract_timestamp(line: &str) -> Option<String> {
        // Common timestamp patterns:
        // - ISO 8601: 2024-01-15T10:30:00, 2024-01-15 10:30:00
        // - Syslog: Jan 15 10:30:00
        // - Common: 2024/01/15 10:30:00, 01/15/2024 10:30:00
        // - Time only: 10:30:00, 10:30:00.123

        let chars: Vec<char> = line.chars().collect();

        // ISO 8601 with T separator: 2024-01-15T10:30:00
        // Format: YYYY-MM-DDTHH:MM:SS
        if chars.len() >= 19 {
            let potential = &line[..19.min(line.len())];
            if Self::is_iso8601_timestamp(potential) {
                // Check for milliseconds and timezone
                let mut end = 19;
                if line.len() > 19 {
                    let rest = &line[19..];
                    // Check for milliseconds
                    if rest.starts_with('.') {
                        let ms_end = rest
                            .find(|c: char| !c.is_ascii_digit())
                            .unwrap_or(rest.len().min(4));
                        end += 1 + ms_end;
                    }
                    // Check for timezone (Z or +/-HH:MM)
                    if end < line.len() {
                        let tz_part = &line[end..];
                        if tz_part.starts_with('Z') {
                            end += 1;
                        } else if tz_part.starts_with('+') || tz_part.starts_with('-') {
                            // Timezone offset like +00:00 or +0000
                            let tz_len =
                                if tz_part.len() >= 6 && tz_part.chars().nth(3) == Some(':') {
                                    6
                                } else if tz_part.len() >= 5 {
                                    5
                                } else {
                                    0
                                };
                            end += tz_len;
                        }
                    }
                }
                return Some(line[..end].to_string());
            }
        }

        // ISO 8601 with space separator: 2024-01-15 10:30:00
        if chars.len() >= 19 {
            let potential = &line[..19.min(line.len())];
            if Self::is_iso8601_space_timestamp(potential) {
                let mut end = 19;
                if line.len() > 19 && line[19..].starts_with('.') {
                    let rest = &line[19..];
                    let ms_end = rest
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(rest.len().min(4));
                    end += 1 + ms_end;
                }
                return Some(line[..end].to_string());
            }
        }

        // Slash date format: 2024/01/15 10:30:00
        if chars.len() >= 19 {
            let potential = &line[..19.min(line.len())];
            if Self::is_slash_date_timestamp(potential) {
                return Some(potential.to_string());
            }
        }

        // Syslog format: Jan 15 10:30:00
        if chars.len() >= 15 {
            let potential = &line[..15.min(line.len())];
            if Self::is_syslog_timestamp(potential) {
                return Some(potential.to_string());
            }
        }

        // Time only at start: 10:30:00 or 10:30:00.123
        if chars.len() >= 8 {
            let potential = &line[..8.min(line.len())];
            if Self::is_time_only(potential) {
                let mut end = 8;
                if line.len() > 8 && line[8..].starts_with('.') {
                    let rest = &line[8..];
                    let ms_end = rest
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(rest.len().min(4));
                    end += 1 + ms_end;
                }
                return Some(line[..end].to_string());
            }
        }

        None
    }

    /// Check if string is an ISO 8601 timestamp with T separator.
    fn is_iso8601_timestamp(s: &str) -> bool {
        // Format: YYYY-MM-DDTHH:MM:SS
        if s.len() < 19 {
            return false;
        }
        let bytes = s.as_bytes();
        // Check structure: XXXX-XX-XXTXX:XX:XX
        bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[10] == b'T'
            && bytes[13] == b':'
            && bytes[16] == b':'
            && bytes[0..4].iter().all(|b| b.is_ascii_digit())
            && bytes[5..7].iter().all(|b| b.is_ascii_digit())
            && bytes[8..10].iter().all(|b| b.is_ascii_digit())
            && bytes[11..13].iter().all(|b| b.is_ascii_digit())
            && bytes[14..16].iter().all(|b| b.is_ascii_digit())
            && bytes[17..19].iter().all(|b| b.is_ascii_digit())
    }

    /// Check if string is an ISO 8601 timestamp with space separator.
    fn is_iso8601_space_timestamp(s: &str) -> bool {
        // Format: YYYY-MM-DD HH:MM:SS
        if s.len() < 19 {
            return false;
        }
        let bytes = s.as_bytes();
        bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[10] == b' '
            && bytes[13] == b':'
            && bytes[16] == b':'
            && bytes[0..4].iter().all(|b| b.is_ascii_digit())
            && bytes[5..7].iter().all(|b| b.is_ascii_digit())
            && bytes[8..10].iter().all(|b| b.is_ascii_digit())
            && bytes[11..13].iter().all(|b| b.is_ascii_digit())
            && bytes[14..16].iter().all(|b| b.is_ascii_digit())
            && bytes[17..19].iter().all(|b| b.is_ascii_digit())
    }

    /// Check if string is a slash date timestamp.
    fn is_slash_date_timestamp(s: &str) -> bool {
        // Format: YYYY/MM/DD HH:MM:SS
        if s.len() < 19 {
            return false;
        }
        let bytes = s.as_bytes();
        bytes[4] == b'/'
            && bytes[7] == b'/'
            && bytes[10] == b' '
            && bytes[13] == b':'
            && bytes[16] == b':'
            && bytes[0..4].iter().all(|b| b.is_ascii_digit())
            && bytes[5..7].iter().all(|b| b.is_ascii_digit())
            && bytes[8..10].iter().all(|b| b.is_ascii_digit())
            && bytes[11..13].iter().all(|b| b.is_ascii_digit())
            && bytes[14..16].iter().all(|b| b.is_ascii_digit())
            && bytes[17..19].iter().all(|b| b.is_ascii_digit())
    }

    /// Check if string is a syslog timestamp.
    fn is_syslog_timestamp(s: &str) -> bool {
        // Format: Mon DD HH:MM:SS (e.g., "Jan 15 10:30:00")
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 3 {
            return false;
        }
        months.contains(&parts[0])
            && parts[1].parse::<u8>().is_ok()
            && parts[2].len() == 8
            && parts[2].contains(':')
    }

    /// Check if string is time only (HH:MM:SS).
    fn is_time_only(s: &str) -> bool {
        // Format: HH:MM:SS
        if s.len() < 8 {
            return false;
        }
        let bytes = s.as_bytes();
        bytes[2] == b':'
            && bytes[5] == b':'
            && bytes[0..2].iter().all(|b| b.is_ascii_digit())
            && bytes[3..5].iter().all(|b| b.is_ascii_digit())
            && bytes[6..8].iter().all(|b| b.is_ascii_digit())
    }

    /// Detect log level from a log line.
    fn detect_log_level(line: &str) -> LogLevel {
        let line_upper = line.to_uppercase();

        // Check for various level indicators in order of severity (highest first)
        // Patterns: [FATAL], FATAL:, |FATAL|, FATAL - etc.

        // Fatal/Critical - includes panic, crash, abort
        if Self::contains_level_marker(&line_upper, "FATAL")
            || Self::contains_level_marker(&line_upper, "CRITICAL")
            || Self::contains_level_marker(&line_upper, "CRIT")
            || Self::contains_error_keyword(&line_upper, "PANIC")
            || Self::contains_error_keyword(&line_upper, "CRASH")
            || Self::contains_error_keyword(&line_upper, "ABORT")
            || Self::contains_error_keyword(&line_upper, "EMERG")
            || Self::contains_error_keyword(&line_upper, "ALERT")
        {
            return LogLevel::Fatal;
        }

        // Error - includes exceptions, failures, and common error patterns
        if Self::contains_level_marker(&line_upper, "ERROR")
            || Self::contains_level_marker(&line_upper, "ERR")
            || Self::contains_error_keyword(&line_upper, "EXCEPTION")
            || Self::contains_error_keyword(&line_upper, "FAILED")
            || Self::contains_error_keyword(&line_upper, "FAILURE")
            || Self::contains_error_keyword(&line_upper, "STACK TRACE")
            || Self::contains_error_keyword(&line_upper, "BACKTRACE")
            || Self::contains_error_keyword(&line_upper, "SEGFAULT")
            || Self::contains_error_keyword(&line_upper, "SEG FAULT")
            || Self::contains_error_keyword(&line_upper, "NULL POINTER")
            || Self::contains_error_keyword(&line_upper, "ACCESS DENIED")
            || Self::contains_error_keyword(&line_upper, "TIMEOUT ERROR")
            || Self::contains_error_keyword(&line_upper, "CONNECTION REFUSED")
            || Self::contains_error_keyword(&line_upper, "CONNECTION ERROR")
        {
            return LogLevel::Error;
        }

        // Warning - includes deprecation, caution notices
        if Self::contains_level_marker(&line_upper, "WARN")
            || Self::contains_level_marker(&line_upper, "WARNING")
            || Self::contains_warning_keyword(&line_upper, "DEPRECATED")
            || Self::contains_warning_keyword(&line_upper, "CAUTION")
            || Self::contains_warning_keyword(&line_upper, "ATTENTION")
            || Self::contains_warning_keyword(&line_upper, "BE AWARE")
            || Self::contains_warning_keyword(&line_upper, "PLEASE NOTE")
            || Self::contains_warning_keyword(&line_upper, "SLOW QUERY")
            || Self::contains_warning_keyword(&line_upper, "SLOW REQUEST")
        {
            return LogLevel::Warning;
        }

        // Info
        if Self::contains_level_marker(&line_upper, "INFO")
            || Self::contains_level_marker(&line_upper, "NOTICE")
        {
            return LogLevel::Info;
        }

        // Debug
        if Self::contains_level_marker(&line_upper, "DEBUG")
            || Self::contains_level_marker(&line_upper, "TRACE")
            || Self::contains_level_marker(&line_upper, "VERBOSE")
        {
            return LogLevel::Debug;
        }

        LogLevel::Unknown
    }

    /// Check if line contains an error-related keyword.
    /// This is more lenient than contains_level_marker and looks for keywords
    /// anywhere in the line that typically indicate an error condition.
    fn contains_error_keyword(line_upper: &str, keyword: &str) -> bool {
        // Check for the keyword with word boundaries
        if line_upper.contains(keyword) {
            // Avoid false positives by checking context
            // For example, "no errors" should not be detected as an error
            let keyword_lower = keyword.to_lowercase();
            let negation_patterns = [
                format!("no {}", keyword_lower),
                format!("without {}", keyword_lower),
                format!("not {}", keyword_lower),
                format!("0 {}", keyword_lower),
                format!("zero {}", keyword_lower),
            ];
            for neg in negation_patterns {
                if line_upper.contains(&neg.to_uppercase()) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    /// Check if line contains a warning-related keyword.
    fn contains_warning_keyword(line_upper: &str, keyword: &str) -> bool {
        line_upper.contains(keyword)
    }

    /// Check if line contains a level marker.
    fn contains_level_marker(line_upper: &str, level: &str) -> bool {
        // Check for patterns like [LEVEL], LEVEL:, |LEVEL|, <LEVEL>, (LEVEL)
        // These are precise patterns that indicate a log level
        let patterns = [
            format!("[{}]", level),
            format!("{}:", level),
            format!("|{}|", level),
            format!("<{}>", level),
            format!("({})", level),
            format!("{} -", level),
            format!("{}]", level), // Level followed by closing bracket
        ];

        for pattern in patterns {
            if line_upper.contains(&pattern) {
                return true;
            }
        }

        // Check if line starts with level followed by space or colon
        if line_upper.starts_with(level) {
            let after_level = &line_upper[level.len()..];
            if after_level.starts_with(':')
                || after_level.starts_with(' ')
                || after_level.is_empty()
            {
                return true;
            }
        }

        false
    }

    /// Extract message by removing timestamp and level prefix.
    fn extract_message(line: &str, timestamp: &Option<String>, level: &LogLevel) -> String {
        let mut message = line.to_string();

        // Remove timestamp prefix
        if let Some(ts) = timestamp {
            if message.starts_with(ts) {
                message = message[ts.len()..].to_string();
            }
        }

        // Trim leading whitespace after timestamp removal
        message = message.trim_start().to_string();

        // Remove common level prefixes
        let level_patterns: &[&str] = match level {
            LogLevel::Debug => &[
                "[DEBUG]", "DEBUG:", "|DEBUG|", "<DEBUG>", "(DEBUG)", "DEBUG -", "DEBUG ",
            ],
            LogLevel::Info => &[
                "[INFO]", "INFO:", "|INFO|", "<INFO>", "(INFO)", "INFO -", "INFO ",
            ],
            LogLevel::Warning => &[
                "[WARN]",
                "[WARNING]",
                "WARN:",
                "WARNING:",
                "|WARN|",
                "|WARNING|",
                "<WARN>",
                "<WARNING>",
                "(WARN)",
                "(WARNING)",
                "WARN -",
                "WARNING -",
                "WARN ",
                "WARNING ",
            ],
            LogLevel::Error => &[
                "[ERROR]", "ERROR:", "|ERROR|", "<ERROR>", "(ERROR)", "ERROR -", "ERROR ", "[ERR]",
                "ERR:", "ERR ",
            ],
            LogLevel::Fatal => &[
                "[FATAL]",
                "FATAL:",
                "|FATAL|",
                "<FATAL>",
                "(FATAL)",
                "FATAL -",
                "FATAL ",
                "[CRITICAL]",
                "CRITICAL:",
                "[CRIT]",
                "CRIT:",
            ],
            LogLevel::Unknown => &[],
        };

        for pattern in level_patterns {
            let pattern_upper = pattern.to_uppercase();
            let message_upper = message.to_uppercase();
            if message_upper.starts_with(&pattern_upper) {
                message = message[pattern.len()..].to_string();
                break;
            }
        }

        // Clean up leading whitespace and separators
        message = message.trim().to_string();
        if message.starts_with('-') || message.starts_with(':') || message.starts_with(']') {
            message = message[1..].trim().to_string();
        }

        message
    }

    /// Format logs output for display.
    fn format_logs(logs_output: &LogsOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_logs_json(logs_output),
            OutputFormat::Csv => Self::format_logs_csv(logs_output),
            OutputFormat::Tsv => Self::format_logs_tsv(logs_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_logs_compact(logs_output),
            OutputFormat::Raw => Self::format_logs_raw(logs_output),
        }
    }

    /// Format logs output as JSON.
    fn format_logs_json(logs_output: &LogsOutput) -> String {
        let total_critical = logs_output.error_count + logs_output.fatal_count;
        serde_json::json!({
            "counts": {
                "total_lines": logs_output.total_lines,
                "debug": logs_output.debug_count,
                "info": logs_output.info_count,
                "warning": logs_output.warning_count,
                "error": logs_output.error_count,
                "fatal": logs_output.fatal_count,
                "unknown": logs_output.unknown_count,
            },
            "repeated_lines": logs_output.repeated_lines.iter().map(|r| serde_json::json!({
                "line": r.line,
                "count": r.count,
                "first_line": r.first_line,
                "last_line": r.last_line,
            })).collect::<Vec<_>>(),
            "recent_critical": logs_output.recent_critical.iter().map(|e| serde_json::json!({
                "line_number": e.line_number,
                "level": match e.level {
                    LogLevel::Debug => "debug",
                    LogLevel::Info => "info",
                    LogLevel::Warning => "warning",
                    LogLevel::Error => "error",
                    LogLevel::Fatal => "fatal",
                    LogLevel::Unknown => "unknown",
                },
                "timestamp": e.timestamp,
                "message": e.message,
            })).collect::<Vec<_>>(),
            "recent_critical_count": logs_output.recent_critical.len(),
            "total_critical": total_critical,
            "entries": logs_output.entries.iter().map(|e| serde_json::json!({
                "line_number": e.line_number,
                "level": match e.level {
                    LogLevel::Debug => "debug",
                    LogLevel::Info => "info",
                    LogLevel::Warning => "warning",
                    LogLevel::Error => "error",
                    LogLevel::Fatal => "fatal",
                    LogLevel::Unknown => "unknown",
                },
                "timestamp": e.timestamp,
                "message": e.message,
            })).collect::<Vec<_>>(),
        })
        .to_string()
    }

    /// Format logs output as CSV.
    fn format_logs_csv(logs_output: &LogsOutput) -> String {
        let mut result = String::new();
        result.push_str("line_number,level,timestamp,message\n");

        for entry in &logs_output.entries {
            let level_str = match entry.level {
                LogLevel::Debug => "debug",
                LogLevel::Info => "info",
                LogLevel::Warning => "warning",
                LogLevel::Error => "error",
                LogLevel::Fatal => "fatal",
                LogLevel::Unknown => "unknown",
            };
            let timestamp = entry.timestamp.as_deref().unwrap_or("");
            let message_escaped = RunHandler::escape_csv_field(&entry.message);
            result.push_str(&format!(
                "{},{},{},{}\n",
                entry.line_number, level_str, timestamp, message_escaped
            ));
        }

        result
    }

    /// Format logs output as TSV.
    fn format_logs_tsv(logs_output: &LogsOutput) -> String {
        let mut result = String::new();
        result.push_str("line_number\tlevel\ttimestamp\tmessage\n");

        for entry in &logs_output.entries {
            let level_str = match entry.level {
                LogLevel::Debug => "debug",
                LogLevel::Info => "info",
                LogLevel::Warning => "warning",
                LogLevel::Error => "error",
                LogLevel::Fatal => "fatal",
                LogLevel::Unknown => "unknown",
            };
            let timestamp = entry.timestamp.as_deref().unwrap_or("");
            let message_escaped = RunHandler::escape_tsv_field(&entry.message);
            result.push_str(&format!(
                "{}\t{}\t{}\t{}\n",
                entry.line_number, level_str, timestamp, message_escaped
            ));
        }

        result
    }

    /// Format logs output in compact format.
    fn format_logs_compact(logs_output: &LogsOutput) -> String {
        let mut output = String::new();

        if logs_output.is_empty {
            output.push_str("logs: empty\n");
            return output;
        }

        // Summary header
        output.push_str(&format!("lines: {}\n", logs_output.total_lines));

        // Level summary (only show non-zero counts)
        let mut level_parts = Vec::new();
        if logs_output.fatal_count > 0 {
            level_parts.push(format!("fatal:{}", logs_output.fatal_count));
        }
        if logs_output.error_count > 0 {
            level_parts.push(format!("error:{}", logs_output.error_count));
        }
        if logs_output.warning_count > 0 {
            level_parts.push(format!("warn:{}", logs_output.warning_count));
        }
        if logs_output.info_count > 0 {
            level_parts.push(format!("info:{}", logs_output.info_count));
        }
        if logs_output.debug_count > 0 {
            level_parts.push(format!("debug:{}", logs_output.debug_count));
        }
        if logs_output.unknown_count > 0 {
            level_parts.push(format!("other:{}", logs_output.unknown_count));
        }

        if !level_parts.is_empty() {
            output.push_str(&format!("levels: {}\n", level_parts.join(", ")));
        }

        // Repeated lines summary
        if !logs_output.repeated_lines.is_empty() {
            output.push_str(&format!(
                "repeated: {} unique lines ({} occurrences)\n",
                logs_output.repeated_lines.len(),
                logs_output
                    .repeated_lines
                    .iter()
                    .map(|r| r.count)
                    .sum::<usize>()
            ));
        }

        output.push('\n');

        // Show repeated lines
        if !logs_output.repeated_lines.is_empty() {
            output.push_str("repeated lines:\n");
            for repeated in &logs_output.repeated_lines {
                if repeated.count > 1 {
                    let preview = if repeated.line.len() > 60 {
                        format!("{}...", &repeated.line[..57])
                    } else {
                        repeated.line.clone()
                    };
                    output.push_str(&format!(
                        "  [x{}] {} (lines {}-{})\n",
                        repeated.count, preview, repeated.first_line, repeated.last_line
                    ));
                }
            }
            output.push('\n');
        }

        // Show recent critical lines (ERROR and FATAL)
        if !logs_output.recent_critical.is_empty() {
            let total_critical = logs_output.error_count + logs_output.fatal_count;
            let shown = logs_output.recent_critical.len();
            if shown < total_critical {
                output.push_str(&format!(
                    "recent critical ({} of {}):\n",
                    shown, total_critical
                ));
            } else {
                output.push_str(&format!("recent critical ({}):\n", shown));
            }
            for entry in &logs_output.recent_critical {
                let level_indicator = match entry.level {
                    LogLevel::Error => "[E]",
                    LogLevel::Fatal => "[F]",
                    _ => "[!]",
                };
                let preview = if entry.message.len() > 80 {
                    format!("{}...", &entry.message[..77])
                } else {
                    entry.message.clone()
                };
                output.push_str(&format!(
                    "  {} {} {}\n",
                    level_indicator, entry.line_number, preview
                ));
            }
            output.push('\n');
        }

        // Show entries with detected levels (collapse consecutive duplicates)
        let has_levels = logs_output
            .entries
            .iter()
            .any(|e| e.level != LogLevel::Unknown);
        if has_levels {
            output.push_str("entries:\n");
            // Collapse consecutive entries with same level and message
            let mut i = 0;
            while i < logs_output.entries.len() {
                let entry = &logs_output.entries[i];
                let level_indicator = match entry.level {
                    LogLevel::Debug => "[D]",
                    LogLevel::Info => "[I]",
                    LogLevel::Warning => "[W]",
                    LogLevel::Error => "[E]",
                    LogLevel::Fatal => "[F]",
                    LogLevel::Unknown => "   ",
                };

                // Count consecutive entries with same level and message
                let mut count = 1;
                let mut last_line = entry.line_number;
                while i + count < logs_output.entries.len() {
                    let next = &logs_output.entries[i + count];
                    if next.level == entry.level && next.message == entry.message {
                        count += 1;
                        last_line = next.line_number;
                    } else {
                        break;
                    }
                }

                let preview = if entry.message.len() > 80 {
                    format!("{}...", &entry.message[..77])
                } else {
                    entry.message.clone()
                };

                if count > 1 {
                    output.push_str(&format!(
                        "{} {}-{} {} [x{}]\n",
                        level_indicator, entry.line_number, last_line, preview, count
                    ));
                } else {
                    output.push_str(&format!(
                        "{} {} {}\n",
                        level_indicator, entry.line_number, preview
                    ));
                }

                i += count;
            }
        } else {
            // No levels detected, just show raw lines with line numbers (collapse consecutive duplicates)
            output.push_str("lines:\n");
            let mut i = 0;
            while i < logs_output.entries.len() {
                let entry = &logs_output.entries[i];

                // Count consecutive entries with same line content
                let mut count = 1;
                let mut last_line = entry.line_number;
                while i + count < logs_output.entries.len() {
                    let next = &logs_output.entries[i + count];
                    if next.line == entry.line {
                        count += 1;
                        last_line = next.line_number;
                    } else {
                        break;
                    }
                }

                let preview = if entry.line.len() > 80 {
                    format!("{}...", &entry.line[..77])
                } else {
                    entry.line.clone()
                };

                if count > 1 {
                    output.push_str(&format!(
                        "  {}-{} {} [x{}]\n",
                        entry.line_number, last_line, preview, count
                    ));
                } else {
                    output.push_str(&format!("  {} {}\n", entry.line_number, preview));
                }

                i += count;
            }
        }

        output
    }

    /// Format logs output as raw (original format).
    fn format_logs_raw(logs_output: &LogsOutput) -> String {
        let mut output = String::new();

        for entry in &logs_output.entries {
            output.push_str(&entry.line);
            output.push('\n');
        }

        output
    }

    /// Handle the find subcommand.
    fn handle_find(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the find output
        let find_output = Self::parse_find(&input)?;

        // Format output based on the requested format
        let output = Self::format_find(&find_output, ctx.format);
        print!("{}", output);

        Ok(())
    }

    /// Parse find output into structured data.
    fn parse_find(input: &str) -> CommandResult<FindOutput> {
        let mut find_output = FindOutput::default();

        for line in input.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Check for permission denied or other error messages
            // Format: "find: '/path': Permission denied"
            // or: "find: cannot open directory '/path': Permission denied"
            // or: "find: 'path': No such file or directory"
            if line.starts_with("find: ") && line.contains(':') {
                let error = Self::parse_find_error(line);
                find_output.errors.push(error);
                continue;
            }

            // Each line is a file path
            let path = line.to_string();
            let is_directory = path.ends_with('/');
            let is_hidden = path
                .split('/')
                .last()
                .map(|s| s.starts_with('.'))
                .unwrap_or(false);

            let entry = FindEntry {
                path: path.clone(),
                is_directory,
                is_hidden,
                extension: Self::extract_extension(&path),
                depth: Self::calculate_path_depth(&path),
            };

            find_output.entries.push(entry.clone());
            find_output.total_count += 1;

            if is_directory {
                find_output.directories.push(path.clone());
            } else {
                find_output.files.push(path.clone());
            }

            if is_hidden {
                find_output.hidden.push(path);
            }

            // Track extensions
            if let Some(ext) = &entry.extension {
                *find_output.extensions.entry(ext.clone()).or_insert(0) += 1;
            }
        }

        // Check if empty (considering both entries and errors)
        find_output.is_empty = find_output.entries.is_empty();

        Ok(find_output)
    }

    /// Parse a find error message.
    fn parse_find_error(line: &str) -> FindError {
        // Format: "find: '/path': Permission denied"
        // or: "find: cannot open directory '/path': Permission denied"
        // or: "find: 'path': No such file or directory"

        // Try to extract the path (usually in quotes)
        let path = if let Some(start) = line.find('\'') {
            if let Some(end) = line[start + 1..].find('\'') {
                line[start + 1..start + 1 + end].to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        FindError {
            path,
            message: line.to_string(),
        }
    }

    /// Extract file extension from path.
    fn extract_extension(path: &str) -> Option<String> {
        let filename = path.split('/').last()?;
        // Skip hidden files starting with . and files with no extension
        if filename.starts_with('.') {
            return None;
        }
        let dot_pos = filename.rfind('.')?;
        if dot_pos == 0 {
            return None;
        }
        Some(filename[dot_pos + 1..].to_lowercase())
    }

    /// Calculate the depth of a path (number of path separators).
    fn calculate_path_depth(path: &str) -> usize {
        path.matches('/').count()
    }

    /// Format find output for display.
    fn format_find(find_output: &FindOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_find_json(find_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_find_compact(find_output),
            OutputFormat::Raw => Self::format_find_raw(find_output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_find_compact(find_output),
        }
    }

    /// Format find output as JSON.
    fn format_find_json(find_output: &FindOutput) -> String {
        serde_json::json!({
            "is_empty": find_output.is_empty,
            "total_count": find_output.total_count,
            "entries": find_output.entries.iter().map(|e| serde_json::json!({
                "path": e.path,
                "is_directory": e.is_directory,
                "is_hidden": e.is_hidden,
                "extension": e.extension,
                "depth": e.depth,
            })).collect::<Vec<_>>(),
            "directories": find_output.directories,
            "files": find_output.files,
            "hidden": find_output.hidden,
            "extensions": find_output.extensions,
            "errors": find_output.errors.iter().map(|e| serde_json::json!({
                "path": e.path,
                "message": e.message,
            })).collect::<Vec<_>>(),
        })
        .to_string()
    }

    /// Format find output in compact format.
    fn format_find_compact(find_output: &FindOutput) -> String {
        let mut output = String::new();

        // Show errors first (if any)
        if !find_output.errors.is_empty() {
            for error in &find_output.errors {
                output.push_str(&format!("error: {}\n", error.message));
            }
        }

        if find_output.is_empty && find_output.errors.is_empty() {
            output.push_str("find: empty\n");
            return output;
        }

        if !find_output.is_empty {
            output.push_str(&format!("total: {}\n", find_output.total_count));
        }

        if !find_output.directories.is_empty() {
            output.push_str(&format!(
                "directories ({}):\n",
                find_output.directories.len()
            ));
            for path in &find_output.directories {
                output.push_str(&format!("  {}\n", path));
            }
        }

        if !find_output.files.is_empty() {
            output.push_str(&format!("files ({}):\n", find_output.files.len()));
            for path in &find_output.files {
                output.push_str(&format!("  {}\n", path));
            }
        }

        if !find_output.hidden.is_empty() {
            output.push_str(&format!("hidden ({}):\n", find_output.hidden.len()));
            for path in &find_output.hidden {
                output.push_str(&format!("  {}\n", path));
            }
        }

        if !find_output.extensions.is_empty() {
            output.push_str(&format!("extensions ({}):\n", find_output.extensions.len()));
            let mut exts: Vec<_> = find_output.extensions.iter().collect();
            exts.sort_by(|a, b| b.1.cmp(a.1));
            for (ext, count) in exts {
                output.push_str(&format!("  {}: {}\n", ext, count));
            }
        }

        output
    }

    /// Format find output as raw (just paths).
    fn format_find_raw(find_output: &FindOutput) -> String {
        let mut output = String::new();

        for entry in &find_output.entries {
            output.push_str(&format!("{}\n", entry.path));
        }

        output
    }
}

impl CommandHandler for ParseHandler {
    type Input = ParseCommands;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        match input {
            ParseCommands::GitStatus { file, count } => Self::handle_git_status(file, count, ctx),
            ParseCommands::GitDiff { file } => Self::handle_git_diff(file, ctx),
            ParseCommands::Ls { file } => Self::handle_ls(file, ctx),
            ParseCommands::Grep { file } => Self::handle_grep(file, ctx),
            ParseCommands::Find { file } => Self::handle_find(file, ctx),
            ParseCommands::Test { runner, file } => Self::handle_test(runner, file, ctx),
            ParseCommands::Logs { file } => Self::handle_logs(file, ctx),
        }
    }
}

/// Router that dispatches commands to their handlers.
pub struct Router {
    run_handler: RunHandler,
    search_handler: SearchHandler,
    replace_handler: ReplaceHandler,
    tail_handler: TailHandler,
    clean_handler: CleanHandler,
    trim_handler: TrimHandler,
    html2md_handler: Html2mdHandler,
    txt2md_handler: Txt2mdHandler,
    is_clean_handler: IsCleanHandler,
    parse_handler: ParseHandler,
}

impl Router {
    /// Create a new router with all command handlers.
    pub fn new() -> Self {
        Self {
            run_handler: RunHandler,
            search_handler: SearchHandler,
            replace_handler: ReplaceHandler,
            tail_handler: TailHandler,
            clean_handler: CleanHandler,
            trim_handler: TrimHandler,
            html2md_handler: Html2mdHandler,
            txt2md_handler: Txt2mdHandler,
            is_clean_handler: IsCleanHandler,
            parse_handler: ParseHandler,
        }
    }

    /// Route a command to its handler and execute it.
    pub fn route(&self, command: &Commands, ctx: &CommandContext) -> CommandResult {
        match command {
            Commands::Run {
                command,
                args,
                capture_stdout,
                capture_stderr,
                capture_exit_code,
                capture_duration,
            } => {
                let input = RunInput::from((
                    command,
                    args,
                    capture_stdout.unwrap_or(true),
                    capture_stderr.unwrap_or(true),
                    capture_exit_code.unwrap_or(true),
                    capture_duration.unwrap_or(true),
                    None, // timeout not supported via CLI yet
                ));
                self.run_handler.execute(&input, ctx)
            }
            Commands::Search {
                path,
                query,
                extension,
                ignore_case,
                context,
                limit,
            } => {
                let input = SearchInput {
                    path: path.clone(),
                    query: query.clone(),
                    extension: extension.clone(),
                    ignore_case: *ignore_case,
                    context: *context,
                    limit: *limit,
                };
                self.search_handler.execute(&input, ctx)
            }
            Commands::Replace {
                path,
                search,
                replace,
                extension,
                dry_run,
                count,
            } => {
                let input = ReplaceInput {
                    path: path.clone(),
                    search: search.clone(),
                    replace: replace.clone(),
                    extension: extension.clone(),
                    dry_run: *dry_run,
                    count: *count,
                };
                self.replace_handler.execute(&input, ctx)
            }
            Commands::Tail {
                file,
                lines,
                errors,
                follow,
            } => {
                let input = TailInput {
                    file: file.clone(),
                    lines: *lines,
                    errors: *errors,
                    follow: *follow,
                };
                self.tail_handler.execute(&input, ctx)
            }
            Commands::Clean {
                file,
                no_ansi,
                collapse_blanks,
                collapse_repeats,
                trim,
            } => {
                let input = CleanInput {
                    file: file.clone(),
                    no_ansi: *no_ansi,
                    collapse_blanks: *collapse_blanks,
                    collapse_repeats: *collapse_repeats,
                    trim: *trim,
                };
                self.clean_handler.execute(&input, ctx)
            }
            Commands::Html2md {
                input,
                output,
                metadata,
            } => {
                let input = Html2mdInput {
                    input: input.clone(),
                    output: output.clone(),
                    metadata: *metadata,
                };
                self.html2md_handler.execute(&input, ctx)
            }
            Commands::Txt2md { input, output } => {
                let input = Txt2mdInput {
                    input: input.clone(),
                    output: output.clone(),
                };
                self.txt2md_handler.execute(&input, ctx)
            }
            Commands::Trim {
                file,
                leading,
                trailing,
            } => {
                let input = TrimInput {
                    file: file.clone(),
                    leading: *leading,
                    trailing: *trailing,
                };
                self.trim_handler.execute(&input, ctx)
            }
            Commands::IsClean { check_untracked } => {
                let input = IsCleanInput {
                    check_untracked: *check_untracked,
                };
                self.is_clean_handler.execute(&input, ctx)
            }
            Commands::Parse { parser } => self.parse_handler.execute(parser, ctx),
        }
    }

    /// Execute a command and print the result or error.
    pub fn execute_and_print(&self, command: &Commands, ctx: &CommandContext) {
        match self.route(command, ctx) {
            Ok(()) => {}
            Err(CommandError::NotImplemented(msg)) => {
                // Format not implemented message according to output format
                let formatted = Self::format_not_implemented(&msg, ctx.format);
                if ctx.format == OutputFormat::Json {
                    // For JSON, output to stderr (consistent with error handling)
                    eprintln!("{}", formatted);
                } else {
                    println!("{}", formatted);
                }
            }
            Err(e) => {
                // Format error according to output format
                let formatted = Self::format_command_error(&e, ctx.format);
                eprintln!("{}", formatted);
                // Propagate the exit code if available, otherwise default to 1
                let exit_code = e.exit_code().unwrap_or(1);
                std::process::exit(exit_code);
            }
        }
    }

    /// Process stdin input when no command is specified.
    ///
    /// This reads from stdin and applies basic text processing:
    /// - Strips ANSI codes
    /// - Trims whitespace
    /// - Collapses blank lines
    /// - Sanitizes control characters
    pub fn process_stdin(&self, input: &str, ctx: &CommandContext) -> CommandResult<String> {
        let mut result = input.to_string();

        // Strip ANSI escape codes FIRST (before sanitizing control chars)
        // because ANSI codes start with \x1b which is a control character
        result = strip_ansi_codes(&result);

        // Sanitize control characters (remove nulls, replace other control chars)
        result = sanitize_control_chars(&result);

        // Trim trailing whitespace from each line
        result = result
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n");

        // Collapse multiple blank lines into single blank lines
        let lines: Vec<&str> = result.lines().collect();
        let mut collapsed_lines = Vec::new();
        let mut prev_blank = false;

        for line in lines {
            let is_blank = line.trim().is_empty();
            if is_blank && prev_blank {
                continue; // Skip consecutive blank lines
            }
            collapsed_lines.push(line);
            prev_blank = is_blank;
        }

        result = collapsed_lines.join("\n");

        // Remove leading/trailing blank lines
        result = result.trim().to_string();

        // Format output based on the requested format
        let formatted = match ctx.format {
            OutputFormat::Raw => result.clone(),
            OutputFormat::Compact => result.clone(),
            OutputFormat::Json => serde_json::json!({
                "content": result,
                "stats": {
                    "input_length": input.len(),
                    "output_length": result.len(),
                }
            })
            .to_string(),
            OutputFormat::Agent => {
                format!("Content:\n{}\n", result)
            }
            OutputFormat::Csv => {
                // Output as CSV with one row per line
                result
                    .lines()
                    .map(|line| format!("\"{}\"", line.replace('"', "\"\"")))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            OutputFormat::Tsv => {
                // Output as TSV with one row per line
                result.lines().collect::<Vec<_>>().join("\n")
            }
        };

        Ok(formatted)
    }

    /// Format a not-implemented message based on the output format.
    fn format_not_implemented(msg: &str, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => serde_json::json!({
                "not_implemented": true,
                "message": format!("{} not yet implemented", msg),
            })
            .to_string(),
            OutputFormat::Raw
            | OutputFormat::Compact
            | OutputFormat::Agent
            | OutputFormat::Csv
            | OutputFormat::Tsv => format!("{} not yet implemented", msg),
        }
    }

    /// Format a CommandError based on the output format.
    fn format_command_error(error: &CommandError, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => serde_json::json!({
                "error": true,
                "message": error.to_string(),
                "exit_code": error.exit_code(),
            })
            .to_string(),
            OutputFormat::Raw
            | OutputFormat::Compact
            | OutputFormat::Agent
            | OutputFormat::Csv
            | OutputFormat::Tsv => format!("Error: {}", error),
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Malformed Input Handling Tests
    // ============================================================

    #[test]
    fn test_sanitize_control_chars_removes_nulls() {
        let input = "hello\x00world";
        let result = sanitize_control_chars(input);
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_sanitize_control_chars_replaces_control_chars() {
        let input = "hello\x01\x02\x03world";
        let result = sanitize_control_chars(input);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_sanitize_control_chars_preserves_newlines() {
        let input = "hello\nworld\r\ntest";
        let result = sanitize_control_chars(input);
        assert_eq!(result, "hello\nworld\r\ntest");
    }

    #[test]
    fn test_sanitize_control_chars_preserves_tabs() {
        let input = "hello\tworld";
        let result = sanitize_control_chars(input);
        assert_eq!(result, "hello\tworld");
    }

    #[test]
    fn test_sanitize_control_chars_normalizes_spaces() {
        let input = "hello\x01\x02world";
        let result = sanitize_control_chars(input);
        assert_eq!(result, "hello world"); // Multiple control chars become single space
    }

    #[test]
    fn test_sanitize_control_chars_preserves_unicode() {
        let input = "hello 世界 🌍";
        let result = sanitize_control_chars(input);
        assert_eq!(result, "hello 世界 🌍");
    }

    #[test]
    fn test_sanitize_control_chars_mixed() {
        let input = "line1\x00\nline2\x01\x02end";
        let result = sanitize_control_chars(input);
        assert_eq!(result, "line1\nline2 end");
    }

    #[test]
    fn test_sanitize_control_chars_empty() {
        let input = "";
        let result = sanitize_control_chars(input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_sanitize_control_chars_only_control() {
        let input = "\x00\x01\x02\x03";
        let result = sanitize_control_chars(input);
        assert_eq!(result, " ");
    }

    #[test]
    fn test_strip_ansi_codes_basic() {
        let input = "\x1b[31mRed text\x1b[0m";
        let result = strip_ansi_codes(input);
        assert_eq!(result, "Red text");
    }

    #[test]
    fn test_strip_ansi_codes_multiple() {
        let input = "\x1b[1;31mBnew Red\x1b[0m \x1b[32mGreen\x1b[0m";
        let result = strip_ansi_codes(input);
        assert_eq!(result, "Bnew Red Green");
    }

    #[test]
    fn test_process_stdin_with_null_bytes() {
        let router = Router::new();
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = "hello\x00world\nline2";
        let result = router.process_stdin(input, &ctx).unwrap();
        assert_eq!(result, "helloworld\nline2");
    }

    #[test]
    fn test_process_stdin_with_control_chars() {
        let router = Router::new();
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = "hello\x01world";
        let result = router.process_stdin(input, &ctx).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_process_stdin_with_ansi_and_control() {
        let router = Router::new();
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = "\x1b[31mhello\x1b[0m\x00world";
        let result = router.process_stdin(input, &ctx).unwrap();
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_process_stdin_json_format() {
        let router = Router::new();
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };
        let input = "hello\x00world";
        let result = router.process_stdin(input, &ctx).unwrap();
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["content"], "helloworld");
    }

    #[test]
    fn test_command_context_creation() {
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: true,
            enabled_formats: vec![OutputFormat::Json, OutputFormat::Csv],
        };

        assert_eq!(ctx.format, OutputFormat::Json);
        assert!(ctx.stats);
        assert!(ctx.has_conflicting_formats());
    }

    #[test]
    fn test_command_context_no_conflict() {
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![OutputFormat::Json],
        };

        assert!(!ctx.has_conflicting_formats());
    }

    #[test]
    fn test_command_error_display() {
        let err = CommandError::NotImplemented("test command".to_string());
        assert_eq!(format!("{}", err), "Not implemented: test command");

        let err = CommandError::ExecutionError {
            message: "failed".to_string(),
            exit_code: Some(1),
        };
        assert_eq!(format!("{}", err), "Execution error: failed");

        let err = CommandError::InvalidArguments("bad args".to_string());
        assert_eq!(format!("{}", err), "Invalid arguments: bad args");

        let err = CommandError::IoError("file not found".to_string());
        assert_eq!(format!("{}", err), "I/O error: file not found");
    }

    #[test]
    fn test_run_handler_success() {
        let handler = RunHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = RunInput {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            capture_stdout: true,
            capture_stderr: true,
            capture_exit_code: true,
            capture_duration: true,
            timeout: None,
        };

        let result = handler.execute(&input, &ctx);
        // echo should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_handler_command_not_found() {
        let handler = RunHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = RunInput {
            command: "nonexistent_command_xyz123".to_string(),
            args: vec![],
            capture_stdout: true,
            capture_stderr: true,
            capture_exit_code: true,
            capture_duration: true,
            timeout: None,
        };

        let result = handler.execute(&input, &ctx);
        // Should return an error for command not found
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(CommandError::ExecutionError {
                message: _,
                exit_code: _
            })
        ));
    }

    #[test]
    fn test_run_handler_non_zero_exit() {
        let handler = RunHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = RunInput {
            command: "false".to_string(),
            args: vec![],
            capture_stdout: true,
            capture_stderr: true,
            capture_exit_code: true,
            capture_duration: true,
            timeout: None,
        };

        let result = handler.execute(&input, &ctx);
        // false always exits with 1
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(CommandError::ExecutionError {
                message: _,
                exit_code: _
            })
        ));
    }

    #[test]
    fn test_run_handler_json_format() {
        let handler = RunHandler;
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![OutputFormat::Json],
        };
        let input = RunInput {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            capture_stdout: true,
            capture_stderr: true,
            capture_exit_code: true,
            capture_duration: true,
            timeout: None,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_handler_no_capture_stdout() {
        let handler = RunHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = RunInput {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            capture_stdout: false,
            capture_stderr: true,
            capture_exit_code: true,
            capture_duration: true,
            timeout: None,
        };

        // When stdout is not captured, the command should still succeed
        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_handler_no_capture_exit_code() {
        let handler = RunHandler;
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![OutputFormat::Json],
        };
        let input = RunInput {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), "exit 42".to_string()],
            capture_stdout: true,
            capture_stderr: true,
            capture_exit_code: false,
            capture_duration: true,
            timeout: None,
        };

        // When exit code is not captured, the error is NOT propagated
        // even though the command exited with a non-zero code
        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_handler() {
        let handler = SearchHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = SearchInput {
            path: std::path::PathBuf::from("src"),
            query: "SearchHandler".to_string(),
            extension: Some("rs".to_string()),
            ignore_case: false,
            context: None,
            limit: Some(10),
        };

        // The search handler should now execute successfully
        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_replace_handler() {
        let handler = ReplaceHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = ReplaceInput {
            path: std::path::PathBuf::from("."),
            search: "new_unique_string_xyz".to_string(),
            replace: "new".to_string(),
            extension: Some("rs".to_string()),
            dry_run: true,
            count: false,
        };

        // The replace handler should execute successfully (dry run, no actual changes)
        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_replace_handler_json_format() {
        let handler = ReplaceHandler;
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };
        let input = ReplaceInput {
            path: std::path::PathBuf::from("."),
            search: "nonexistent_pattern_abc123".to_string(),
            replace: "new".to_string(),
            extension: Some("rs".to_string()),
            dry_run: true,
            count: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_replace_truncate_line() {
        let short_line = "short line";
        assert_eq!(
            ReplaceHandler::truncate_line(short_line, 80),
            short_line.to_string()
        );

        let long_line = "a".repeat(100);
        let truncated = ReplaceHandler::truncate_line(&long_line, 80);
        assert!(truncated.len() <= 83); // 80 + "..."
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_replace_escape_csv_field() {
        assert_eq!(ReplaceHandler::escape_csv_field("simple"), "simple");
        assert_eq!(
            ReplaceHandler::escape_csv_field("with,comma"),
            "\"with,comma\""
        );
        assert_eq!(
            ReplaceHandler::escape_csv_field("with\"quote"),
            "\"with\"\"quote\""
        );
        assert_eq!(
            ReplaceHandler::escape_csv_field("with\nnewline"),
            "\"with\nnewline\""
        );
    }

    #[test]
    fn test_replace_escape_tsv_field() {
        assert_eq!(ReplaceHandler::escape_tsv_field("simple"), "simple");
        assert_eq!(ReplaceHandler::escape_tsv_field("with\ttab"), "with\\ttab");
        assert_eq!(
            ReplaceHandler::escape_tsv_field("with\nnewline"),
            "with\\nnewline"
        );
    }

    #[test]
    fn test_tail_handler_file_not_found() {
        let handler = TailHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = TailInput {
            file: std::path::PathBuf::from("/nonexistent/file.log"),
            lines: 20,
            errors: true,
            follow: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(matches!(result, Err(CommandError::IoError(_))));
    }

    #[test]
    fn test_tail_is_error_line() {
        assert!(TailHandler::is_error_line("ERROR: something went wrong"));
        assert!(TailHandler::is_error_line("error in processing"));
        assert!(TailHandler::is_error_line("FATAL: critical failure"));
        assert!(TailHandler::is_error_line("Exception: null pointer"));
        assert!(TailHandler::is_error_line("CRITICAL: system failure"));
        assert!(TailHandler::is_error_line("Failed to connect"));
        assert!(TailHandler::is_error_line("[ERROR] connection timeout"));
        assert!(TailHandler::is_error_line("ERR connection refused"));
        assert!(TailHandler::is_error_line(
            "E/AndroidRuntime: FATAL EXCEPTION"
        ));

        assert!(!TailHandler::is_error_line("INFO: process started"));
        assert!(!TailHandler::is_error_line("success: operation completed"));
        assert!(!TailHandler::is_error_line("warning: deprecated API"));
        assert!(!TailHandler::is_error_line("debug: processing request"));
    }

    #[test]
    fn test_clean_handler() {
        let handler = CleanHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };
        let input = CleanInput {
            file: None,
            no_ansi: true,
            collapse_blanks: true,
            collapse_repeats: false,
            trim: true,
        };

        // The handler should succeed now that it's implemented
        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_html2md_handler() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        // Create a temporary HTML file for testing
        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_html2md_input.html");
        let output_path = temp_dir.join("test_html2md_output.md");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
<h1>Hello World</h1>
<p>This is a <strong>test</strong> paragraph.</p>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: Some(output_path.clone()),
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        // Verify output file was created and contains markdown
        let output_content = std::fs::read_to_string(&output_path).unwrap();
        assert!(output_content.contains("Hello World"));
        assert!(output_content.contains("test"));

        // Cleanup
        let _ = std::fs::remove_file(&html_path);
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_html2md_handler_with_metadata() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };

        // Create a temporary HTML file for testing
        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_html2md_meta_input.html");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Test Title</title>
<meta name="description" content="Test description">
</head>
<body>
<h1>Heading</h1>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: None,
            metadata: true,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        // Cleanup
        let _ = std::fs::remove_file(&html_path);
    }

    #[test]
    fn test_html2md_is_url() {
        assert!(Html2mdHandler::is_url("http://example.com"));
        assert!(Html2mdHandler::is_url("https://example.com"));
        assert!(!Html2mdHandler::is_url("/path/to/file.html"));
        assert!(!Html2mdHandler::is_url("file.html"));
    }

    #[test]
    fn test_html2md_file_not_found() {
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let input = Html2mdInput {
            input: "/nonexistent/path/to/file.html".to_string(),
            output: None,
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(matches!(result, Err(CommandError::IoError(_))));
    }

    #[test]
    fn test_html2md_heading_conversion() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_heading_conversion.html");
        let output_path = temp_dir.join("test_heading_conversion_output.md");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Heading Test</title></head>
<body>
<h1>Heading 1</h1>
<h2>Heading 2</h2>
<h3>Heading 3</h3>
<h4>Heading 4</h4>
<h5>Heading 5</h5>
<h6>Heading 6</h6>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: Some(output_path.clone()),
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        let output_content = std::fs::read_to_string(&output_path).unwrap();
        assert!(output_content.contains("# Heading 1"));
        assert!(output_content.contains("## Heading 2"));
        assert!(output_content.contains("### Heading 3"));
        assert!(output_content.contains("#### Heading 4"));
        assert!(output_content.contains("##### Heading 5"));
        assert!(output_content.contains("###### Heading 6"));

        let _ = std::fs::remove_file(&html_path);
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_html2md_heading_with_inline_elements() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_heading_inline.html");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Heading Inline Test</title></head>
<body>
<h1>Heading with <em>emphasis</em></h1>
<h2>Heading with <strong>bold</strong></h2>
<h3>Heading with <code>code</code></h3>
<h4>Heading with <a href="https://example.com">link</a></h4>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: None,
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        let _ = std::fs::remove_file(&html_path);
    }

    #[test]
    fn test_html2md_link_conversion() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_link_conversion.html");
        let output_path = temp_dir.join("test_link_conversion_output.md");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Link Test</title></head>
<body>
<p>Visit <a href="https://example.com">Example</a> for more info.</p>
<p>Check <a href="https://rust-lang.org">Rust</a> language.</p>
<p><a href="/relative/path">Relative link</a> works too.</p>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: Some(output_path.clone()),
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        let output_content = std::fs::read_to_string(&output_path).unwrap();
        assert!(output_content.contains("[Example](https://example.com)"));
        assert!(output_content.contains("[Rust](https://rust-lang.org)"));
        assert!(output_content.contains("[Relative link](/relative/path)"));

        let _ = std::fs::remove_file(&html_path);
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_html2md_list_conversion() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_list_conversion.html");
        let output_path = temp_dir.join("test_list_conversion_output.md");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>List Test</title></head>
<body>
<ul>
  <li>Unordered item 1</li>
  <li>Unordered item 2</li>
  <li>Unordered item 3</li>
</ul>
<ol>
  <li>Ordered item 1</li>
  <li>Ordered item 2</li>
  <li>Ordered item 3</li>
</ol>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: Some(output_path.clone()),
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        let output_content = std::fs::read_to_string(&output_path).unwrap();
        // Check for unordered list items (with asterisks or dashes)
        assert!(output_content.contains("Unordered item 1"));
        assert!(output_content.contains("Unordered item 2"));
        assert!(output_content.contains("Unordered item 3"));
        // Check for ordered list items
        assert!(output_content.contains("Ordered item 1"));
        assert!(output_content.contains("Ordered item 2"));
        assert!(output_content.contains("Ordered item 3"));

        let _ = std::fs::remove_file(&html_path);
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_html2md_combined_elements() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_combined_elements.html");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Combined Test</title></head>
<body>
<h1>Main Heading</h1>
<p>Introduction paragraph with a <a href="https://example.com">link</a>.</p>
<h2>Features</h2>
<ul>
  <li>Feature 1 with <strong>bold</strong> text</li>
  <li>Feature 2 with <em>emphasis</em></li>
</ul>
<h2>Steps</h2>
<ol>
  <li>First step</li>
  <li>Second step with <code>code</code></li>
</ol>
<h3>Conclusion</h3>
<p>Final paragraph.</p>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: None,
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        let _ = std::fs::remove_file(&html_path);
    }

    #[test]
    fn test_txt2md_handler() {
        let handler = Txt2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = Txt2mdInput {
            input: Some(std::path::PathBuf::from("input.txt")),
            output: Some(std::path::PathBuf::from("output.md")),
        };

        let result = handler.execute(&input, &ctx);
        assert!(matches!(result, Err(CommandError::NotImplemented(_))));
    }

    #[test]
    fn test_parse_handler_git_status() {
        // Test with empty input (simulating empty stdin)
        // This should result in a clean status with empty branch
        let handler = ParseHandler;
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![OutputFormat::Json],
        };
        let input = ParseCommands::GitStatus {
            file: None,
            count: None,
        };

        // Note: This test reads from stdin which is empty, so it will succeed
        // with an empty/clean status
        let result = handler.execute(&input, &ctx);
        // The implementation is now complete, so it should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_handler_test() {
        let handler = ParseHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = ParseCommands::Test {
            runner: Some(crate::TestRunner::Pytest),
            file: None,
        };

        // Note: This test reads from stdin which is empty, so it will return
        // an empty test result but should succeed
        let result = handler.execute(&input, &ctx);
        // The implementation is now complete, so it should succeed
        assert!(result.is_ok());
    }

    // ============================================================
    // Pytest Parser Tests
    // ============================================================

    #[test]
    fn test_parse_pytest_empty() {
        let result = ParseHandler::parse_pytest("").unwrap();
        assert!(result.is_empty);
        assert!(result.tests.is_empty());
        assert_eq!(result.summary.total, 0);
    }

    #[test]
    fn test_parse_pytest_single_passed() {
        let input = r#"tests/test_main.py::test_add PASSED
1 passed in 0.01s"#;
        let result = ParseHandler::parse_pytest(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.tests.len(), 1);
        assert_eq!(result.summary.passed, 1);
        assert_eq!(result.summary.failed, 0);
        assert_eq!(result.summary.total, 1);
    }

    #[test]
    fn test_parse_pytest_single_failed() {
        let input = r#"tests/test_main.py::test_fail FAILED
____ test_fail ____
def test_fail():
    assert False
=== FAILURES ===
1 failed in 0.01s"#;
        let result = ParseHandler::parse_pytest(input).unwrap();

        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.tests.len(), 1);
        assert_eq!(result.summary.failed, 1);
        assert_eq!(result.summary.passed, 0);
    }

    #[test]
    fn test_parse_pytest_mixed_results() {
        let input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract PASSED
tests/test_main.py::test_multiply SKIPPED
tests/test_main.py::test_fail FAILED
2 passed, 1 failed, 1 skipped in 0.05s"#;
        let result = ParseHandler::parse_pytest(input).unwrap();

        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.tests.len(), 4);
        assert_eq!(result.summary.passed, 2);
        assert_eq!(result.summary.failed, 1);
        assert_eq!(result.summary.skipped, 1);
        assert_eq!(result.summary.total, 4);
    }

    #[test]
    fn test_parse_pytest_with_xfail() {
        let input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_expected_fail XFAIL
2 passed, 1 xfailed in 0.01s"#;
        let result = ParseHandler::parse_pytest(input).unwrap();

        assert!(result.success);
        assert_eq!(result.summary.xfailed, 1);
    }

    #[test]
    fn test_parse_pytest_summary_line() {
        let summary = ParseHandler::parse_pytest_summary("2 passed in 0.01s");
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 0);
        assert!(summary.duration.is_some());

        let summary = ParseHandler::parse_pytest_summary("2 passed, 1 failed in 0.05s");
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 1);

        let summary = ParseHandler::parse_pytest_summary("3 passed, 1 failed, 2 skipped in 1.23s");
        assert_eq!(summary.passed, 3);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.skipped, 2);
        assert_eq!(summary.duration, Some(1.23));
    }

    #[test]
    fn test_is_pytest_summary_line() {
        assert!(ParseHandler::is_pytest_summary_line("2 passed in 0.01s"));
        assert!(ParseHandler::is_pytest_summary_line(
            "2 passed, 1 failed in 0.05s"
        ));
        assert!(ParseHandler::is_pytest_summary_line(
            "=== 2 passed in 0.01s ==="
        ));
        assert!(ParseHandler::is_pytest_summary_line(
            "1 failed, 2 passed in 0.05s"
        ));
        assert!(!ParseHandler::is_pytest_summary_line(
            "test_file.py::test_name PASSED"
        ));
        assert!(!ParseHandler::is_pytest_summary_line("PASSED"));
    }

    #[test]
    fn test_parse_pytest_test_line() {
        let result =
            ParseHandler::parse_pytest_test_line("tests/test_main.py::test_add PASSED").unwrap();
        assert_eq!(result.name, "tests/test_main.py::test_add");
        assert_eq!(result.status, TestStatus::Passed);
        assert_eq!(result.file, Some("tests/test_main.py".to_string()));

        let result =
            ParseHandler::parse_pytest_test_line("tests/test_main.py::test_fail FAILED").unwrap();
        assert_eq!(result.status, TestStatus::Failed);

        let result =
            ParseHandler::parse_pytest_test_line("tests/test_main.py::test_skip SKIPPED").unwrap();
        assert_eq!(result.status, TestStatus::Skipped);
    }

    #[test]
    fn test_format_pytest_json() {
        let mut output = PytestOutput::default();
        output.tests.push(TestResult {
            name: "test_example".to_string(),
            status: TestStatus::Passed,
            duration: None,
            file: None,
            line: None,
            error_message: None,
        });
        output.summary.passed = 1;
        output.summary.total = 1;
        output.success = true;
        output.is_empty = false;

        let json = ParseHandler::format_pytest_json(&output);
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"passed\":1"));
        assert!(json.contains("\"total\":1"));
    }

    #[test]
    fn test_format_pytest_compact() {
        let mut output = PytestOutput::default();
        output.tests.push(TestResult {
            name: "test_example".to_string(),
            status: TestStatus::Passed,
            duration: None,
            file: None,
            line: None,
            error_message: None,
        });
        output.summary.passed = 1;
        output.summary.total = 1;
        output.success = true;
        output.is_empty = false;

        let compact = ParseHandler::format_pytest_compact(&output);
        assert!(compact.contains("PASS:"));
        // Compact success summary shows "X tests" not "X passed"
        assert!(compact.contains("1 tests"));
    }

    #[test]
    fn test_format_pytest_raw() {
        let mut output = PytestOutput::default();
        output.tests.push(TestResult {
            name: "test_example".to_string(),
            status: TestStatus::Passed,
            duration: None,
            file: None,
            line: None,
            error_message: None,
        });
        output.tests.push(TestResult {
            name: "test_fail".to_string(),
            status: TestStatus::Failed,
            duration: None,
            file: None,
            line: None,
            error_message: None,
        });

        let raw = ParseHandler::format_pytest_raw(&output);
        assert!(raw.contains("PASS test_example"));
        assert!(raw.contains("FAIL test_fail"));
    }

    #[test]
    fn test_format_pytest_agent() {
        let mut output = PytestOutput::default();
        output.tests.push(TestResult {
            name: "test_example".to_string(),
            status: TestStatus::Passed,
            duration: None,
            file: None,
            line: None,
            error_message: None,
        });
        output.summary.passed = 1;
        output.summary.total = 1;
        output.success = true;
        output.is_empty = false;

        let agent = ParseHandler::format_pytest_agent(&output);
        assert!(agent.contains("# Test Results"));
        assert!(agent.contains("Status: SUCCESS"));
        assert!(agent.contains("## Summary"));
    }

    #[test]
    fn test_parse_pytest_with_header_info() {
        let input = r#"============================= test session starts ==============================
platform darwin -- Python 3.12.0, pytest-8.0.0, pluggy-1.4.0
rootdir: /Users/user/project
collected 2 items

tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract PASSED

2 passed in 0.01s"#;
        let result = ParseHandler::parse_pytest(input).unwrap();

        assert!(result.success);
        assert_eq!(result.python_version, Some("3.12.0".to_string()));
        assert_eq!(result.pytest_version, Some("8.0.0".to_string()));
        assert_eq!(result.rootdir, Some("/Users/user/project".to_string()));
    }

    #[test]
    fn test_router_run_command_success() {
        let router = Router::new();
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let command = Commands::Run {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            capture_stdout: Some(true),
            capture_stderr: Some(true),
            capture_exit_code: Some(true),
            capture_duration: Some(true),
        };

        let result = router.route(&command, &ctx);
        // echo should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_router_run_command_failure() {
        let router = Router::new();
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let command = Commands::Run {
            command: "false".to_string(),
            args: vec![],
            capture_stdout: Some(true),
            capture_stderr: Some(true),
            capture_exit_code: Some(true),
            capture_duration: Some(true),
        };

        let result = router.route(&command, &ctx);
        // false exits with 1
        assert!(result.is_err());
    }

    #[test]
    fn test_router_default() {
        let router = Router::default();
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let command = Commands::Search {
            path: std::path::PathBuf::from("."),
            query: "test".to_string(),
            extension: None,
            ignore_case: false,
            context: None,
            limit: None,
        };

        let result = router.route(&command, &ctx);
        // Search is now implemented, so it should succeed
        assert!(result.is_ok());
    }

    // ============================================================
    // Jest Parser Tests
    // ============================================================

    #[test]
    fn test_parse_jest_empty() {
        let result = ParseHandler::parse_jest("").unwrap();
        assert!(result.is_empty);
        assert!(result.test_suites.is_empty());
        assert_eq!(result.summary.tests_total, 0);
    }

    #[test]
    fn test_parse_jest_single_suite_passed() {
        let input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)
  ✓ should subtract numbers (2 ms)

Test Suites: 1 passed, 1 total
Tests:       2 passed, 2 total"#;
        let result = ParseHandler::parse_jest(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites.len(), 1);
        assert_eq!(result.test_suites[0].file, "src/utils.test.js");
        assert!(result.test_suites[0].passed);
        assert_eq!(result.test_suites[0].tests.len(), 2);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.tests_total, 2);
    }

    #[test]
    fn test_parse_jest_single_suite_failed() {
        let input = r#"FAIL src/math.test.js
  ✕ should multiply numbers (3 ms)
  ✓ should divide numbers (1 ms)

Test Suites: 1 failed, 1 total
Tests:       1 passed, 1 failed, 2 total"#;
        let result = ParseHandler::parse_jest(input).unwrap();

        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.test_suites.len(), 1);
        assert_eq!(result.test_suites[0].file, "src/math.test.js");
        assert!(!result.test_suites[0].passed);
        assert_eq!(result.test_suites[0].tests.len(), 2);
        assert_eq!(result.summary.tests_passed, 1);
        assert_eq!(result.summary.tests_failed, 1);
        assert_eq!(result.summary.tests_total, 2);
    }

    #[test]
    fn test_parse_jest_multiple_suites() {
        let input = r#"PASS src/utils.test.js
  ✓ test 1 (5 ms)

FAIL src/api.test.js
  ✕ test 2 (10 ms)
  ✓ test 3 (3 ms)

Test Suites: 1 passed, 1 failed, 2 total
Tests:       2 passed, 1 failed, 3 total"#;
        let result = ParseHandler::parse_jest(input).unwrap();

        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.test_suites.len(), 2);
        assert_eq!(result.summary.suites_passed, 1);
        assert_eq!(result.summary.suites_failed, 1);
        assert_eq!(result.summary.suites_total, 2);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.tests_failed, 1);
        assert_eq!(result.summary.tests_total, 3);
    }

    #[test]
    fn test_parse_jest_test_with_skipped() {
        let input = r#"PASS src/test.js
  ✓ test 1 (5 ms)
  ○ skipped test 2
  ✓ test 3 (3 ms)

Test Suites: 1 passed, 1 total
Tests:       2 passed, 1 skipped, 3 total"#;
        let result = ParseHandler::parse_jest(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites[0].tests.len(), 3);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.tests_skipped, 1);
    }

    #[test]
    fn test_parse_jest_test_line() {
        let result =
            ParseHandler::parse_jest_test_line("  ✓ should work correctly (5 ms)").unwrap();
        assert_eq!(result.status, JestTestStatus::Passed);
        assert_eq!(result.test_name, "should work correctly");
        assert!(result.duration.is_some());

        let result = ParseHandler::parse_jest_test_line("  ✕ should fail").unwrap();
        assert_eq!(result.status, JestTestStatus::Failed);
        assert_eq!(result.test_name, "should fail");

        let result = ParseHandler::parse_jest_test_line("  ○ skipped test").unwrap();
        assert_eq!(result.status, JestTestStatus::Skipped);
    }

    #[test]
    fn test_parse_jest_duration() {
        assert_eq!(ParseHandler::parse_jest_duration("5 ms"), Some(0.005));
        assert_eq!(ParseHandler::parse_jest_duration("1.23 s"), Some(1.23));
        assert_eq!(ParseHandler::parse_jest_duration("1000ms"), Some(1.0));
        assert_eq!(ParseHandler::parse_jest_duration("invalid"), None);
    }

    #[test]
    fn test_parse_jest_summary() {
        let summary = ParseHandler::parse_jest_summary("Test Suites: 2 passed, 1 failed, 3 total");
        assert_eq!(summary.suites_passed, 2);
        assert_eq!(summary.suites_failed, 1);
        assert_eq!(summary.suites_total, 3);
    }

    #[test]
    fn test_parse_jest_tests_summary() {
        let mut summary = JestSummary::default();
        ParseHandler::parse_jest_tests_summary(
            "Tests:       5 passed, 2 failed, 1 skipped, 8 total",
            &mut summary,
        );
        assert_eq!(summary.tests_passed, 5);
        assert_eq!(summary.tests_failed, 2);
        assert_eq!(summary.tests_skipped, 1);
        assert_eq!(summary.tests_total, 8);
    }

    #[test]
    fn test_parse_jest_time_summary() {
        let mut summary = JestSummary::default();
        ParseHandler::parse_jest_time_summary("Time:        1.234 s", &mut summary);
        assert_eq!(summary.duration, Some(1.234));
    }

    #[test]
    fn test_format_jest_json() {
        let mut output = JestOutput::default();
        output.test_suites.push(JestTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.1),
            tests: vec![JestTest {
                name: "test example".to_string(),
                test_name: "test example".to_string(),
                ancestors: vec![],
                status: JestTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let json = ParseHandler::format_jest_json(&output);
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"passed\":1"));
        assert!(json.contains("\"test.js\""));
    }

    #[test]
    fn test_format_jest_compact() {
        let mut output = JestOutput::default();
        output.test_suites.push(JestTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.1),
            tests: vec![JestTest {
                name: "test example".to_string(),
                test_name: "test example".to_string(),
                ancestors: vec![],
                status: JestTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let compact = ParseHandler::format_jest_compact(&output);
        assert!(compact.contains("PASS:"));
        assert!(compact.contains("1 suites"));
        assert!(compact.contains("1 tests"));
    }

    #[test]
    fn test_format_jest_raw() {
        let mut output = JestOutput::default();
        output.test_suites.push(JestTestSuite {
            file: "test.js".to_string(),
            passed: false,
            duration: None,
            tests: vec![
                JestTest {
                    name: "passing test".to_string(),
                    test_name: "passing test".to_string(),
                    ancestors: vec![],
                    status: JestTestStatus::Passed,
                    duration: None,
                    error_message: None,
                },
                JestTest {
                    name: "failing test".to_string(),
                    test_name: "failing test".to_string(),
                    ancestors: vec![],
                    status: JestTestStatus::Failed,
                    duration: None,
                    error_message: None,
                },
            ],
        });
        output.is_empty = false;

        let raw = ParseHandler::format_jest_raw(&output);
        assert!(raw.contains("FAIL test.js"));
        assert!(raw.contains("PASS passing test"));
        assert!(raw.contains("FAIL failing test"));
    }

    #[test]
    fn test_format_jest_agent() {
        let mut output = JestOutput::default();
        output.test_suites.push(JestTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.1),
            tests: vec![JestTest {
                name: "test example".to_string(),
                test_name: "test example".to_string(),
                ancestors: vec![],
                status: JestTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let agent = ParseHandler::format_jest_agent(&output);
        assert!(agent.contains("# Test Results"));
        assert!(agent.contains("Status: SUCCESS"));
        assert!(agent.contains("## Summary"));
    }

    #[test]
    fn test_parse_jest_with_ancestors() {
        // Test with regular > separator
        let result =
            ParseHandler::parse_jest_test_line("✓ describe block > test name (5 ms)").unwrap();
        assert_eq!(result.test_name, "test name");
        assert_eq!(result.ancestors, vec!["describe block"]);

        // Test with fancy › separator (Unicode)
        let result =
            ParseHandler::parse_jest_test_line("✓ describe block › test name (5 ms)").unwrap();
        assert_eq!(result.test_name, "test name");
        assert_eq!(result.ancestors, vec!["describe block"]);
    }

    // ============================================================
    // Grep Parser Tests
    // ============================================================

    #[test]
    fn test_parse_grep_empty() {
        let result = ParseHandler::parse_grep("").unwrap();
        assert!(result.is_empty);
        assert_eq!(result.file_count, 0);
        assert_eq!(result.match_count, 0);
    }

    #[test]
    fn test_parse_grep_single_file_single_match() {
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert!(!result.is_empty);
        assert_eq!(result.file_count, 1);
        assert_eq!(result.match_count, 1);
        assert_eq!(result.files[0].path, "src/main.rs");
        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(result.files[0].matches[0].line, "fn main() {");
    }

    #[test]
    fn test_parse_grep_single_file_multiple_matches() {
        let input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:    println!";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.file_count, 1);
        assert_eq!(result.match_count, 2);
        assert_eq!(result.files[0].matches.len(), 2);
        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(result.files[0].matches[1].line_number, Some(45));
    }

    #[test]
    fn test_parse_grep_multiple_files() {
        let input = "src/main.rs:42:fn main() {\nsrc/lib.rs:10:pub fn helper()";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.file_count, 2);
        assert_eq!(result.match_count, 2);
        assert_eq!(result.files[0].path, "src/main.rs");
        assert_eq!(result.files[1].path, "src/lib.rs");
    }

    #[test]
    fn test_parse_grep_groups_interleaved_files() {
        // Test that matches from the same file are grouped together
        // even when they appear interleaved in the input
        let input = "src/main.rs:10:line one\nsrc/lib.rs:25:line two\nsrc/main.rs:30:line three";
        let result = ParseHandler::parse_grep(input).unwrap();

        // Should have 2 files, not 3
        assert_eq!(result.file_count, 2);
        assert_eq!(result.match_count, 3);

        // Files should preserve order of first appearance
        assert_eq!(result.files[0].path, "src/main.rs");
        assert_eq!(result.files[1].path, "src/lib.rs");

        // main.rs should have both its matches grouped together
        assert_eq!(result.files[0].matches.len(), 2);
        assert_eq!(result.files[0].matches[0].line_number, Some(10));
        assert_eq!(result.files[0].matches[0].line, "line one");
        assert_eq!(result.files[0].matches[1].line_number, Some(30));
        assert_eq!(result.files[0].matches[1].line, "line three");

        // lib.rs should have its single match
        assert_eq!(result.files[1].matches.len(), 1);
        assert_eq!(result.files[1].matches[0].line_number, Some(25));
        assert_eq!(result.files[1].matches[0].line, "line two");
    }

    #[test]
    fn test_parse_grep_with_column() {
        let input = "src/main.rs:42:10:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(result.files[0].matches[0].column, Some(10));
        assert_eq!(result.files[0].matches[0].line, "fn main() {");
    }

    #[test]
    fn test_parse_grep_without_line_number() {
        let input = "src/main.rs:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches[0].line_number, None);
        assert_eq!(result.files[0].matches[0].line, "fn main() {");
    }

    #[test]
    fn test_parse_grep_binary_file() {
        let input = "Binary file target/debug binary matches";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.file_count, 1);
        assert_eq!(result.files[0].path, "target/debug binary");
        assert_eq!(result.files[0].matches[0].line, "[binary file]");
    }

    #[test]
    fn test_parse_grep_format_compact() {
        let input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:    println!";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        assert!(output.contains("matches: 1 files, 2 results"));
        assert!(output.contains("src/main.rs (2):"));
        assert!(output.contains("42: fn main() {"));
        assert!(output.contains("45:     println!"));
    }

    #[test]
    fn test_parse_grep_format_json() {
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Json);

        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["schema"]["type"], "grep_output");
        assert_eq!(json["counts"]["files"], 1);
        assert_eq!(json["counts"]["matches"], 1);
        assert_eq!(json["files"][0]["path"], "src/main.rs");
        assert_eq!(json["files"][0]["matches"][0]["line_number"], 42);
        assert_eq!(json["files"][0]["matches"][0]["line"], "fn main() {");
    }

    #[test]
    fn test_parse_grep_format_csv() {
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Csv);

        assert!(output.starts_with("path,line_number,column,is_context,line\n"));
        assert!(output.contains("src/main.rs,42,,false,fn main() {"));
    }

    #[test]
    fn test_parse_grep_format_tsv() {
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Tsv);

        assert!(output.starts_with("path\tline_number\tcolumn\tis_context\tline\n"));
        assert!(output.contains("src/main.rs\t42\t\tfalse\tfn main() {"));
    }

    #[test]
    fn test_parse_grep_format_raw() {
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

        assert!(output.contains("src/main.rs:42:fn main() {"));
    }

    #[test]
    fn test_parse_grep_empty_compact() {
        let mut result = GrepOutput::default();
        result.is_empty = true;
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        assert!(output.contains("grep: no matches"));
    }

    #[test]
    fn test_parse_grep_line_with_colon_in_content() {
        // Content containing colons should be handled correctly
        let input = "src/main.rs:42:let x = \"http://example.com\";";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(
            result.files[0].matches[0].line,
            "let x = \"http://example.com\";"
        );
    }

    // ============================================================
    // Context Line Tests
    // ============================================================

    #[test]
    fn test_parse_grep_context_line() {
        // Context lines use "-" as separator (from grep -C/-B/-A)
        let input = "src/main.rs-42-context line";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(result.files[0].matches[0].line, "context line");
        assert!(result.files[0].matches[0].is_context);
    }

    #[test]
    fn test_parse_grep_context_line_with_column() {
        // Context line with column info
        let input = "src/main.rs-42-10-context line";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(result.files[0].matches[0].column, Some(10));
        assert_eq!(result.files[0].matches[0].line, "context line");
        assert!(result.files[0].matches[0].is_context);
    }

    #[test]
    fn test_parse_grep_mixed_match_and_context() {
        // Mix of match and context lines
        let input = "src/main.rs-41-context before\nsrc/main.rs:42:match line\nsrc/main.rs-43-context after";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches.len(), 3);

        // First line is context
        assert!(result.files[0].matches[0].is_context);
        assert_eq!(result.files[0].matches[0].line, "context before");

        // Second line is a match
        assert!(!result.files[0].matches[1].is_context);
        assert_eq!(result.files[0].matches[1].line, "match line");

        // Third line is context
        assert!(result.files[0].matches[2].is_context);
        assert_eq!(result.files[0].matches[2].line, "context after");
    }

    #[test]
    fn test_parse_grep_context_is_context_flag_false_for_matches() {
        let input = "src/main.rs:42:match line";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert!(!result.files[0].matches[0].is_context);
    }

    #[test]
    fn test_format_grep_compact_collapse_context_lines() {
        // Multiple consecutive context lines should be collapsed
        let input = "src/main.rs-10-context 1\nsrc/main.rs-11-context 2\nsrc/main.rs-12-context 3\nsrc/main.rs:13:match line";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        // Should collapse 3 context lines into a summary
        assert!(output.contains("10-12: ... (3 context lines)"));
        assert!(output.contains("13: match line"));
    }

    #[test]
    fn test_format_grep_compact_single_context_line() {
        // Single context line should show as "... (1 context lines)" format
        let input = "src/main.rs-10-context line\nsrc/main.rs:11:match line";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        assert!(output.contains("10: ..."));
        assert!(output.contains("11: match line"));
    }

    #[test]
    fn test_format_grep_compact_context_before_and_after() {
        // Context lines before and after match
        let input = "src/main.rs-10-before\nsrc/main.rs:11:match\nsrc/main.rs-12-after";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        assert!(output.contains("10: ..."));
        assert!(output.contains("11: match"));
        assert!(output.contains("12: ..."));
    }

    #[test]
    fn test_format_grep_compact_count_excludes_context() {
        // Match count should exclude context lines
        let input = "src/main.rs-10-context\nsrc/main.rs:11:match\nsrc/main.rs-12-context";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        // Should show 1 result (only the match), not 3
        assert!(output.contains("matches: 1 files, 1 results"));
    }

    #[test]
    fn test_format_grep_compact_trailing_context() {
        // Context lines at the end should be collapsed
        let input = "src/main.rs:10:match\nsrc/main.rs-11-context 1\nsrc/main.rs-12-context 2";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        assert!(output.contains("10: match"));
        assert!(output.contains("11-12: ... (2 context lines)"));
    }

    #[test]
    fn test_format_grep_json_includes_is_context() {
        let input = "src/main.rs-10-context\nsrc/main.rs:11:match";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Json);

        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["files"][0]["matches"][0]["is_context"], true);
        assert_eq!(json["files"][0]["matches"][1]["is_context"], false);
    }

    #[test]
    fn test_format_grep_raw_context_uses_dash() {
        // Raw format should preserve dash separator for context
        let input = "src/main.rs-10-context line";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

        assert!(output.contains("src/main.rs-10-context line"));
    }

    #[test]
    fn test_format_grep_raw_match_uses_colon() {
        // Raw format should use colon for matches
        let input = "src/main.rs:10:match line";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

        assert!(output.contains("src/main.rs:10:match line"));
    }

    // ============================================================
    // Grep Truncation Tests
    // ============================================================

    #[test]
    fn test_parse_grep_truncation_fields_not_truncated() {
        // Small result set should not be truncated
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.is_truncated, false);
        assert_eq!(result.total_files, 1);
        assert_eq!(result.total_matches, 1);
        assert_eq!(result.files_shown, 1);
        assert_eq!(result.matches_shown, 1);
    }

    #[test]
    fn test_truncate_grep_files() {
        // Create 60 files (exceeds DEFAULT_MAX_GREP_FILES = 50)
        let mut input = String::new();
        for i in 1..=60 {
            input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();

        // Before truncation
        assert_eq!(result.total_files, 60);
        assert_eq!(result.files.len(), 60);

        // Apply truncation
        ParseHandler::truncate_grep(&mut result, 50, 20);

        // After truncation
        assert_eq!(result.is_truncated, true);
        assert_eq!(result.files_shown, 50);
        assert_eq!(result.total_files, 60);
        assert_eq!(result.files.len(), 50);
    }

    #[test]
    fn test_truncate_grep_matches_per_file() {
        // Create 1 file with 25 matches (exceeds DEFAULT_MAX_GREP_MATCHES_PER_FILE = 20)
        let mut input = String::new();
        for i in 1..=25 {
            input.push_str(&format!("src/main.rs:{}:fn func{}() {{\n", i, i));
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();

        // Before truncation
        assert_eq!(result.total_matches, 25);
        assert_eq!(result.files[0].matches.len(), 25);

        // Apply truncation
        ParseHandler::truncate_grep(&mut result, 50, 20);

        // After truncation
        assert_eq!(result.is_truncated, true);
        assert_eq!(result.matches_shown, 20);
        assert_eq!(result.total_matches, 25);
        assert_eq!(result.files[0].matches.len(), 20);
    }

    #[test]
    fn test_truncate_grep_both_limits() {
        // Create 60 files, each with 25 matches
        let mut input = String::new();
        for i in 1..=60 {
            for j in 1..=25 {
                input.push_str(&format!("src/file{}.rs:{}:fn func{}() {{\n", i, j, j));
            }
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();

        // Before truncation: 60 files * 25 matches = 1500 total matches
        assert_eq!(result.total_files, 60);
        assert_eq!(result.total_matches, 1500);

        // Apply truncation
        ParseHandler::truncate_grep(&mut result, 50, 20);

        // After truncation: 50 files * 20 matches = 1000 matches shown
        assert_eq!(result.is_truncated, true);
        assert_eq!(result.files_shown, 50);
        assert_eq!(result.matches_shown, 1000);
        assert_eq!(result.files.len(), 50);
    }

    #[test]
    fn test_format_grep_json_truncation_info() {
        // Create 60 files to trigger truncation
        let mut input = String::new();
        for i in 1..=60 {
            input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();
        ParseHandler::truncate_grep(&mut result, 50, 20);

        let output = ParseHandler::format_grep(&result, OutputFormat::Json);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(json["is_truncated"], true);
        assert_eq!(json["counts"]["total_files"], 60);
        assert_eq!(json["counts"]["files_shown"], 50);
    }

    #[test]
    fn test_format_grep_compact_truncation_info() {
        // Create 60 files to trigger truncation
        let mut input = String::new();
        for i in 1..=60 {
            input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();
        ParseHandler::truncate_grep(&mut result, 50, 20);

        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        // Check for truncation indicators in compact output
        assert!(output.contains("truncated"));
        assert!(output.contains("50/60"));
        assert!(output.contains("10 more file"));
    }

    #[test]
    fn test_format_grep_raw_truncation_info() {
        // Create 60 files to trigger truncation
        let mut input = String::new();
        for i in 1..=60 {
            input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();
        ParseHandler::truncate_grep(&mut result, 50, 20);

        let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

        // Check for truncation indicator in raw output
        assert!(output.contains("10 more file"));
    }

    #[test]
    fn test_format_grep_json_no_truncation_when_within_limits() {
        // Small result set should not show truncation info
        let input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:println!()";
        let mut result = ParseHandler::parse_grep(input).unwrap();
        ParseHandler::truncate_grep(&mut result, 50, 20);

        let output = ParseHandler::format_grep(&result, OutputFormat::Json);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(json["is_truncated"], false);
        assert!(json["truncation"].is_null());
    }

    // ============================================================
    // NPM Test Parser Tests
    // ============================================================

    #[test]
    fn test_parse_npm_test_empty() {
        let result = ParseHandler::parse_npm_test("").unwrap();
        assert!(result.is_empty);
        assert!(result.test_suites.is_empty());
        assert_eq!(result.summary.tests_total, 0);
    }

    #[test]
    fn test_parse_npm_test_single_suite_passed() {
        let input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_npm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites.len(), 1);
        assert_eq!(result.test_suites[0].file, "test/utils.test.js");
        assert!(result.test_suites[0].passed);
        assert_eq!(result.test_suites[0].tests.len(), 2);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.suites_passed, 1);
    }

    #[test]
    fn test_parse_npm_test_single_suite_failed() {
        let input = r#"▶ test/math.test.js
  ✖ should multiply numbers
    AssertionError [ERR_ASSERTION]: values are not equal
  ✔ should divide numbers (1.234ms)
▶ test/math.test.js (5.678ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 10ms"#;
        let result = ParseHandler::parse_npm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.test_suites.len(), 1);
        assert!(!result.test_suites[0].passed);
        assert_eq!(result.test_suites[0].tests.len(), 2);
        assert_eq!(result.summary.tests_passed, 1);
        assert_eq!(result.summary.tests_failed, 1);
    }

    #[test]
    fn test_parse_npm_test_multiple_suites() {
        let input = r#"▶ test/utils.test.js
  ✔ test 1 (5.123ms)
▶ test/utils.test.js (7.234ms)

▶ test/math.test.js
  ✖ test 2
▶ test/math.test.js (3.456ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 passed 1 failed (2)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_npm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.test_suites.len(), 2);
        assert!(result.test_suites[0].passed);
        assert!(!result.test_suites[1].passed);
    }

    #[test]
    fn test_parse_npm_test_with_skipped() {
        let input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # SKIP
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 skipped (3)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_npm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites[0].tests.len(), 3);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.tests_skipped, 1);
    }

    #[test]
    fn test_parse_npm_test_with_todo() {
        let input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # TODO
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 todo (3)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_npm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites[0].tests.len(), 3);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.tests_todo, 1);
    }

    #[test]
    fn test_parse_npm_test_line() {
        let result =
            ParseHandler::parse_npm_test_line("✔ should work correctly (5.123ms)", &[]).unwrap();
        assert_eq!(result.status, NpmTestStatus::Passed);
        assert_eq!(result.test_name, "should work correctly");
        assert!(result.duration.is_some());

        let result = ParseHandler::parse_npm_test_line("✖ should fail", &[]).unwrap();
        assert_eq!(result.status, NpmTestStatus::Failed);
        assert_eq!(result.test_name, "should fail");

        let result = ParseHandler::parse_npm_test_line("ℹ skipped test # SKIP", &[]).unwrap();
        assert_eq!(result.status, NpmTestStatus::Skipped);
        assert_eq!(result.test_name, "skipped test");

        let result = ParseHandler::parse_npm_test_line("ℹ todo test # TODO", &[]).unwrap();
        assert_eq!(result.status, NpmTestStatus::Todo);
        assert_eq!(result.test_name, "todo test");
    }

    #[test]
    fn test_parse_npm_duration() {
        assert_eq!(ParseHandler::parse_npm_duration("5.123ms"), Some(0.005123));
        assert_eq!(ParseHandler::parse_npm_duration("1.234s"), Some(1.234));
        assert_eq!(ParseHandler::parse_npm_duration("1000ms"), Some(1.0));
        assert_eq!(ParseHandler::parse_npm_duration("invalid"), None);
    }

    #[test]
    fn test_split_npm_test_name_and_duration() {
        let (name, duration) =
            ParseHandler::split_npm_test_name_and_duration("test name (5.123ms)");
        assert_eq!(name, "test name");
        assert_eq!(duration, Some(0.005123));

        let (name, duration) = ParseHandler::split_npm_test_name_and_duration("test name (1.234s)");
        assert_eq!(name, "test name");
        assert_eq!(duration, Some(1.234));

        let (name, duration) =
            ParseHandler::split_npm_test_name_and_duration("test name without duration");
        assert_eq!(name, "test name without duration");
        assert!(duration.is_none());
    }

    #[test]
    fn test_format_npm_test_json() {
        let mut output = NpmTestOutput::default();
        output.test_suites.push(NpmTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.01),
            tests: vec![NpmTest {
                name: "test name".to_string(),
                test_name: "test name".to_string(),
                ancestors: vec![],
                status: NpmTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let json = ParseHandler::format_npm_test_json(&output);
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"tests_passed\":1"));
        assert!(json.contains("\"test.js\""));
    }

    #[test]
    fn test_format_npm_test_compact() {
        let mut output = NpmTestOutput::default();
        output.test_suites.push(NpmTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.01),
            tests: vec![NpmTest {
                name: "test name".to_string(),
                test_name: "test name".to_string(),
                ancestors: vec![],
                status: NpmTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let compact = ParseHandler::format_npm_test_compact(&output);
        assert!(compact.contains("PASS:"));
        assert!(compact.contains("1 suites"));
        assert!(compact.contains("1 tests"));
    }

    #[test]
    fn test_format_npm_test_raw() {
        let mut output = NpmTestOutput::default();
        output.test_suites.push(NpmTestSuite {
            file: "test.js".to_string(),
            passed: false,
            duration: Some(0.01),
            tests: vec![
                NpmTest {
                    name: "passing test".to_string(),
                    test_name: "passing test".to_string(),
                    ancestors: vec![],
                    status: NpmTestStatus::Passed,
                    duration: Some(0.005),
                    error_message: None,
                },
                NpmTest {
                    name: "failing test".to_string(),
                    test_name: "failing test".to_string(),
                    ancestors: vec![],
                    status: NpmTestStatus::Failed,
                    duration: None,
                    error_message: Some("Error message".to_string()),
                },
            ],
        });
        output.is_empty = false;

        let raw = ParseHandler::format_npm_test_raw(&output);
        assert!(raw.contains("FAIL test.js"));
        assert!(raw.contains("PASS passing test"));
        assert!(raw.contains("FAIL failing test"));
    }

    #[test]
    fn test_format_npm_test_agent() {
        let mut output = NpmTestOutput::default();
        output.test_suites.push(NpmTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.01),
            tests: vec![NpmTest {
                name: "test name".to_string(),
                test_name: "test name".to_string(),
                ancestors: vec![],
                status: NpmTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let agent = ParseHandler::format_npm_test_agent(&output);
        assert!(agent.contains("# Test Results"));
        assert!(agent.contains("Status: SUCCESS"));
        assert!(agent.contains("## Summary"));
    }

    #[test]
    fn test_parse_npm_test_with_ancestors() {
        // Test that nested tests track ancestor names
        let result = ParseHandler::parse_npm_test_line(
            "✔ nested test (5.123ms)",
            &["describe block".to_string()],
        )
        .unwrap();
        assert_eq!(result.test_name, "nested test");
        assert_eq!(result.ancestors, vec!["describe block"]);
        assert_eq!(result.name, "describe block > nested test");
    }

    // ============================================================
    // PNPM Test Parser Tests
    // ============================================================

    #[test]
    fn test_parse_pnpm_test_empty() {
        let result = ParseHandler::parse_pnpm_test("").unwrap();
        assert!(result.is_empty);
        assert!(result.test_suites.is_empty());
        assert_eq!(result.summary.tests_total, 0);
    }

    #[test]
    fn test_parse_pnpm_test_single_suite_passed() {
        let input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_pnpm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites.len(), 1);
        assert_eq!(result.test_suites[0].file, "test/utils.test.js");
        assert!(result.test_suites[0].passed);
        assert_eq!(result.test_suites[0].tests.len(), 2);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.suites_passed, 1);
    }

    #[test]
    fn test_parse_pnpm_test_single_suite_failed() {
        let input = r#"▶ test/math.test.js
  ✖ should multiply numbers
    AssertionError [ERR_ASSERTION]: values are not equal
  ✔ should divide numbers (1.234ms)
▶ test/math.test.js (5.678ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 10ms"#;
        let result = ParseHandler::parse_pnpm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.test_suites.len(), 1);
        assert!(!result.test_suites[0].passed);
        assert_eq!(result.test_suites[0].tests.len(), 2);
        assert_eq!(result.summary.tests_passed, 1);
        assert_eq!(result.summary.tests_failed, 1);
    }

    #[test]
    fn test_parse_pnpm_test_multiple_suites() {
        let input = r#"▶ test/utils.test.js
  ✔ test 1 (5.123ms)
▶ test/utils.test.js (7.234ms)

▶ test/math.test.js
  ✖ test 2
▶ test/math.test.js (3.456ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 passed 1 failed (2)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_pnpm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.test_suites.len(), 2);
        assert!(result.test_suites[0].passed);
        assert!(!result.test_suites[1].passed);
    }

    #[test]
    fn test_parse_pnpm_test_with_skipped() {
        let input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # SKIP
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 skipped (3)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_pnpm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites[0].tests.len(), 3);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.tests_skipped, 1);
    }

    #[test]
    fn test_parse_pnpm_test_with_todo() {
        let input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # TODO
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 todo (3)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_pnpm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites[0].tests.len(), 3);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.tests_todo, 1);
    }

    #[test]
    fn test_parse_pnpm_test_line() {
        let result =
            ParseHandler::parse_pnpm_test_line("✔ should work correctly (5.123ms)", &[]).unwrap();
        assert_eq!(result.status, PnpmTestStatus::Passed);
        assert_eq!(result.test_name, "should work correctly");
        assert!(result.duration.is_some());

        let result = ParseHandler::parse_pnpm_test_line("✖ should fail", &[]).unwrap();
        assert_eq!(result.status, PnpmTestStatus::Failed);
        assert_eq!(result.test_name, "should fail");

        let result = ParseHandler::parse_pnpm_test_line("ℹ skipped test # SKIP", &[]).unwrap();
        assert_eq!(result.status, PnpmTestStatus::Skipped);
        assert_eq!(result.test_name, "skipped test");

        let result = ParseHandler::parse_pnpm_test_line("ℹ todo test # TODO", &[]).unwrap();
        assert_eq!(result.status, PnpmTestStatus::Todo);
        assert_eq!(result.test_name, "todo test");
    }

    #[test]
    fn test_parse_pnpm_duration() {
        assert_eq!(ParseHandler::parse_pnpm_duration("5.123ms"), Some(0.005123));
        assert_eq!(ParseHandler::parse_pnpm_duration("1.234s"), Some(1.234));
        assert_eq!(ParseHandler::parse_pnpm_duration("1000ms"), Some(1.0));
        assert_eq!(ParseHandler::parse_pnpm_duration("invalid"), None);
    }

    #[test]
    fn test_split_pnpm_test_name_and_duration() {
        let (name, duration) =
            ParseHandler::split_pnpm_test_name_and_duration("test name (5.123ms)");
        assert_eq!(name, "test name");
        assert_eq!(duration, Some(0.005123));

        let (name, duration) =
            ParseHandler::split_pnpm_test_name_and_duration("test name (1.234s)");
        assert_eq!(name, "test name");
        assert_eq!(duration, Some(1.234));

        let (name, duration) =
            ParseHandler::split_pnpm_test_name_and_duration("test name without duration");
        assert_eq!(name, "test name without duration");
        assert!(duration.is_none());
    }

    #[test]
    fn test_format_pnpm_test_json() {
        let mut output = PnpmTestOutput::default();
        output.test_suites.push(PnpmTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.01),
            tests: vec![PnpmTest {
                name: "test name".to_string(),
                test_name: "test name".to_string(),
                ancestors: vec![],
                status: PnpmTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let json = ParseHandler::format_pnpm_test_json(&output);
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"tests_passed\":1"));
        assert!(json.contains("\"test.js\""));
    }

    #[test]
    fn test_format_pnpm_test_compact() {
        let mut output = PnpmTestOutput::default();
        output.test_suites.push(PnpmTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.01),
            tests: vec![PnpmTest {
                name: "test name".to_string(),
                test_name: "test name".to_string(),
                ancestors: vec![],
                status: PnpmTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let compact = ParseHandler::format_pnpm_test_compact(&output);
        assert!(compact.contains("PASS:"));
        assert!(compact.contains("1 suites"));
        assert!(compact.contains("1 tests"));
    }

    #[test]
    fn test_format_pnpm_test_raw() {
        let mut output = PnpmTestOutput::default();
        output.test_suites.push(PnpmTestSuite {
            file: "test.js".to_string(),
            passed: false,
            duration: None,
            tests: vec![
                PnpmTest {
                    name: "passing test".to_string(),
                    test_name: "passing test".to_string(),
                    ancestors: vec![],
                    status: PnpmTestStatus::Passed,
                    duration: None,
                    error_message: None,
                },
                PnpmTest {
                    name: "failing test".to_string(),
                    test_name: "failing test".to_string(),
                    ancestors: vec![],
                    status: PnpmTestStatus::Failed,
                    duration: None,
                    error_message: Some("Error message".to_string()),
                },
            ],
        });
        output.is_empty = false;

        let raw = ParseHandler::format_pnpm_test_raw(&output);
        assert!(raw.contains("FAIL test.js"));
        assert!(raw.contains("PASS passing test"));
        assert!(raw.contains("FAIL failing test"));
    }

    #[test]
    fn test_format_pnpm_test_agent() {
        let mut output = PnpmTestOutput::default();
        output.test_suites.push(PnpmTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.01),
            tests: vec![PnpmTest {
                name: "test name".to_string(),
                test_name: "test name".to_string(),
                ancestors: vec![],
                status: PnpmTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let agent = ParseHandler::format_pnpm_test_agent(&output);
        assert!(agent.contains("# Test Results"));
        assert!(agent.contains("Status: SUCCESS"));
        assert!(agent.contains("## Summary"));
    }

    #[test]
    fn test_parse_pnpm_test_with_ancestors() {
        // Test that nested tests track ancestor names
        let result = ParseHandler::parse_pnpm_test_line(
            "✔ nested test (5.123ms)",
            &["describe block".to_string()],
        )
        .unwrap();
        assert_eq!(result.test_name, "nested test");
        assert_eq!(result.ancestors, vec!["describe block"]);
        assert_eq!(result.name, "describe block > nested test");
    }

    // ============================================================
    // Bun Test Parser Tests
    // ============================================================

    #[test]
    fn test_parse_bun_test_empty() {
        let result = ParseHandler::parse_bun_test("").unwrap();
        assert!(result.is_empty);
        assert!(result.test_suites.is_empty());
    }

    #[test]
    fn test_parse_bun_test_single_suite_passed() {
        let input = r#"test/package-json-lint.test.ts:
✓ test/package.json [0.88ms]
✓ test/js/third_party/grpc-js/package.json [0.18ms]

 4 pass
 0 fail
 4 expect() calls
Ran 4 tests in 1.44ms"#;
        let result = ParseHandler::parse_bun_test(input).unwrap();
        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites.len(), 1);
        assert_eq!(result.test_suites[0].file, "test/package-json-lint.test.ts");
        assert!(result.test_suites[0].passed);
        assert_eq!(result.summary.tests_passed, 4);
        assert_eq!(result.summary.tests_failed, 0);
        assert_eq!(result.summary.expect_calls, Some(4));
        assert!(result.summary.duration.is_some());
    }

    #[test]
    fn test_parse_bun_test_single_suite_failed() {
        let input = r#"test/api.test.ts:
✓ should pass [0.88ms]
✗ should fail

 1 pass
 1 fail
 2 expect() calls
Ran 2 tests in 1.44ms"#;
        let result = ParseHandler::parse_bun_test(input).unwrap();
        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.test_suites.len(), 1);
        assert!(!result.test_suites[0].passed);
        assert_eq!(result.summary.tests_passed, 1);
        assert_eq!(result.summary.tests_failed, 1);
    }

    #[test]
    fn test_parse_bun_test_multiple_suites() {
        let input = r#"test/a.test.ts:
✓ test a [0.88ms]

test/b.test.ts:
✓ test b [0.18ms]

 2 pass
 0 fail
Ran 2 tests in 1.44ms"#;
        let result = ParseHandler::parse_bun_test(input).unwrap();
        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites.len(), 2);
        assert_eq!(result.summary.tests_passed, 2);
    }

    #[test]
    fn test_parse_bun_test_non_tty_format() {
        let input = r#"test/package-json-lint.test.ts:
(pass) test/package.json [0.48ms]
(pass) test/js/third_party/grpc-js/package.json [0.10ms]
(fail) test/failing.test.ts
(skip) test/skipped.test.ts

 2 pass
 1 fail
 1 skipped
Ran 4 tests across 1 files. [0.66ms]"#;
        let result = ParseHandler::parse_bun_test(input).unwrap();
        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.tests_failed, 1);
        assert_eq!(result.summary.tests_skipped, 1);
        assert_eq!(result.summary.suites_total, 1);
    }

    #[test]
    fn test_parse_bun_test_line() {
        // Test with checkmark
        let result =
            ParseHandler::parse_bun_test_line("✓ should work correctly [5.123ms]", &[]).unwrap();
        assert_eq!(result.status, BunTestStatus::Passed);
        assert_eq!(result.test_name, "should work correctly");
        assert_eq!(result.duration, Some(0.005123));

        // Test with x mark (failure)
        let result = ParseHandler::parse_bun_test_line("✗ should fail", &[]).unwrap();
        assert_eq!(result.status, BunTestStatus::Failed);
        assert_eq!(result.test_name, "should fail");

        // Test with × mark (failure alternative)
        let result = ParseHandler::parse_bun_test_line("× should also fail", &[]).unwrap();
        assert_eq!(result.status, BunTestStatus::Failed);

        // Test non-TTY pass format
        let result =
            ParseHandler::parse_bun_test_line("(pass) should work [5.123ms]", &[]).unwrap();
        assert_eq!(result.status, BunTestStatus::Passed);

        // Test non-TTY fail format
        let result = ParseHandler::parse_bun_test_line("(fail) should fail", &[]).unwrap();
        assert_eq!(result.status, BunTestStatus::Failed);

        // Test non-TTY skip format
        let result = ParseHandler::parse_bun_test_line("(skip) skipped test", &[]).unwrap();
        assert_eq!(result.status, BunTestStatus::Skipped);

        // Test non-TTY todo format
        let result = ParseHandler::parse_bun_test_line("(todo) todo test", &[]).unwrap();
        assert_eq!(result.status, BunTestStatus::Todo);
    }

    #[test]
    fn test_split_bun_test_name_and_duration() {
        let (name, duration) =
            ParseHandler::split_bun_test_name_and_duration("test name [5.123ms]");
        assert_eq!(name, "test name");
        assert_eq!(duration, Some(0.005123));

        let (name, duration) = ParseHandler::split_bun_test_name_and_duration("test name [1.234s]");
        assert_eq!(name, "test name");
        assert_eq!(duration, Some(1.234));

        let (name, duration) =
            ParseHandler::split_bun_test_name_and_duration("test name without duration");
        assert_eq!(name, "test name without duration");
        assert_eq!(duration, None);
    }

    #[test]
    fn test_parse_bun_duration() {
        assert_eq!(ParseHandler::parse_bun_duration("5.123ms"), Some(0.005123));
        assert_eq!(ParseHandler::parse_bun_duration("1.234s"), Some(1.234));
        assert_eq!(ParseHandler::parse_bun_duration("invalid"), None);
    }

    #[test]
    fn test_parse_bun_summary_line() {
        let mut summary = BunTestSummary::default();

        ParseHandler::parse_bun_summary_line("4 pass", &mut summary);
        assert_eq!(summary.tests_passed, 4);

        ParseHandler::parse_bun_summary_line("2 fail", &mut summary);
        assert_eq!(summary.tests_failed, 2);

        ParseHandler::parse_bun_summary_line("10 expect() calls", &mut summary);
        assert_eq!(summary.expect_calls, Some(10));

        ParseHandler::parse_bun_summary_line("3 skipped", &mut summary);
        assert_eq!(summary.tests_skipped, 3);
    }

    #[test]
    fn test_parse_bun_ran_line() {
        let mut summary = BunTestSummary::default();

        ParseHandler::parse_bun_ran_line("Ran 4 tests in 1.44ms", &mut summary);
        assert_eq!(summary.tests_total, 4);
        assert!(summary.duration.is_some());
        let duration = summary.duration.unwrap();
        assert!((duration - 0.00144).abs() < 1e-9);

        let mut summary2 = BunTestSummary::default();
        ParseHandler::parse_bun_ran_line("Ran 4 tests across 1 files. [0.66ms]", &mut summary2);
        assert_eq!(summary2.tests_total, 4);
        assert_eq!(summary2.suites_total, 1);
        assert!(summary2.duration.is_some());
        let duration2 = summary2.duration.unwrap();
        assert!((duration2 - 0.00066).abs() < 1e-9);
    }

    #[test]
    fn test_format_bun_test_json() {
        let mut output = BunTestOutput::default();
        output.test_suites.push(BunTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.01),
            tests: vec![BunTest {
                name: "should pass".to_string(),
                test_name: "should pass".to_string(),
                ancestors: vec![],
                status: BunTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.summary.expect_calls = Some(1);
        output.success = true;
        output.is_empty = false;

        let json = ParseHandler::format_bun_test_json(&output);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["summary"]["tests_passed"], 1);
        assert_eq!(parsed["summary"]["expect_calls"], 1);
    }

    #[test]
    fn test_format_bun_test_compact() {
        let mut output = BunTestOutput::default();
        output.test_suites.push(BunTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.01),
            tests: vec![BunTest {
                name: "should pass".to_string(),
                test_name: "should pass".to_string(),
                ancestors: vec![],
                status: BunTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.summary.duration = Some(0.01);
        output.success = true;
        output.is_empty = false;

        let compact = ParseHandler::format_bun_test_compact(&output);
        assert!(compact.contains("PASS"));
        assert!(compact.contains("1 suites"));
        assert!(compact.contains("1 tests"));
    }

    #[test]
    fn test_format_bun_test_raw() {
        let mut output = BunTestOutput::default();
        output.test_suites.push(BunTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: None,
            tests: vec![
                BunTest {
                    name: "passing test".to_string(),
                    test_name: "passing test".to_string(),
                    ancestors: vec![],
                    status: BunTestStatus::Passed,
                    duration: None,
                    error_message: None,
                },
                BunTest {
                    name: "failing test".to_string(),
                    test_name: "failing test".to_string(),
                    ancestors: vec![],
                    status: BunTestStatus::Failed,
                    duration: None,
                    error_message: None,
                },
            ],
        });

        let raw = ParseHandler::format_bun_test_raw(&output);
        assert!(raw.contains("PASS test.js"));
        assert!(raw.contains("PASS passing test"));
        assert!(raw.contains("FAIL failing test"));
    }

    #[test]
    fn test_format_bun_test_agent() {
        let mut output = BunTestOutput::default();
        output.test_suites.push(BunTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.01),
            tests: vec![BunTest {
                name: "should pass".to_string(),
                test_name: "should pass".to_string(),
                ancestors: vec![],
                status: BunTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.summary.expect_calls = Some(1);
        output.success = true;
        output.is_empty = false;

        let agent = ParseHandler::format_bun_test_agent(&output);
        assert!(agent.contains("# Test Results"));
        assert!(agent.contains("Status: SUCCESS"));
        assert!(agent.contains("## Summary"));
        assert!(agent.contains("Expect() calls: 1"));
    }

    #[test]
    fn test_parse_bun_test_with_ancestors() {
        // Test that nested tests track ancestor names
        let result = ParseHandler::parse_bun_test_line(
            "✓ nested test [5.123ms]",
            &["describe block".to_string()],
        )
        .unwrap();
        assert_eq!(result.test_name, "nested test");
        assert_eq!(result.ancestors, vec!["describe block"]);
        assert_eq!(result.name, "describe block > nested test");
    }

    // ============================================================
    // Logs Parser Tests
    // ============================================================

    #[test]
    fn test_parse_logs_empty() {
        let result = ParseHandler::parse_logs("");
        assert!(result.is_empty);
        assert_eq!(result.total_lines, 0);
    }

    #[test]
    fn test_parse_logs_single_line() {
        let input = "This is a log line";
        let result = ParseHandler::parse_logs(input);

        assert!(!result.is_empty);
        assert_eq!(result.total_lines, 1);
        assert_eq!(result.unknown_count, 1);
    }

    #[test]
    fn test_parse_logs_with_levels() {
        let input = r#"[INFO] Starting application
[ERROR] Something went wrong
[WARN] Warning message
[DEBUG] Debug info"#;
        let result = ParseHandler::parse_logs(input);

        assert_eq!(result.total_lines, 4);
        assert_eq!(result.info_count, 1);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.debug_count, 1);
    }

    #[test]
    fn test_parse_logs_with_fatal() {
        let input = "[FATAL] Critical error\n[CRITICAL] Also critical";
        let result = ParseHandler::parse_logs(input);

        assert_eq!(result.fatal_count, 2);
    }

    #[test]
    fn test_parse_logs_with_repeated_lines() {
        let input = "Repeated line\nDifferent line\nRepeated line\nRepeated line";
        let result = ParseHandler::parse_logs(input);

        assert_eq!(result.repeated_lines.len(), 1);
        assert_eq!(result.repeated_lines[0].line, "Repeated line");
        assert_eq!(result.repeated_lines[0].count, 3);
    }

    #[test]
    fn test_parse_logs_timestamp_iso8601() {
        let input = "2024-01-15T10:30:00 [INFO] Message";
        let result = ParseHandler::parse_logs(input);

        assert_eq!(
            result.entries[0].timestamp,
            Some("2024-01-15T10:30:00".to_string())
        );
        assert_eq!(result.entries[0].level, LogLevel::Info);
    }

    #[test]
    fn test_parse_logs_timestamp_iso8601_space() {
        let input = "2024-01-15 10:30:00 [ERROR] Error message";
        let result = ParseHandler::parse_logs(input);

        assert_eq!(
            result.entries[0].timestamp,
            Some("2024-01-15 10:30:00".to_string())
        );
        assert_eq!(result.entries[0].level, LogLevel::Error);
    }

    #[test]
    fn test_parse_logs_timestamp_syslog() {
        let input = "Jan 15 10:30:00 server daemon[123]: Message";
        let result = ParseHandler::parse_logs(input);

        assert_eq!(
            result.entries[0].timestamp,
            Some("Jan 15 10:30:00".to_string())
        );
    }

    #[test]
    fn test_parse_logs_timestamp_time_only() {
        let input = "10:30:00 INFO: Message";
        let result = ParseHandler::parse_logs(input);

        assert_eq!(result.entries[0].timestamp, Some("10:30:00".to_string()));
    }

    #[test]
    fn test_detect_log_level_brackets() {
        assert_eq!(
            ParseHandler::detect_log_level("[DEBUG] test"),
            LogLevel::Debug
        );
        assert_eq!(
            ParseHandler::detect_log_level("[INFO] test"),
            LogLevel::Info
        );
        assert_eq!(
            ParseHandler::detect_log_level("[WARN] test"),
            LogLevel::Warning
        );
        assert_eq!(
            ParseHandler::detect_log_level("[ERROR] test"),
            LogLevel::Error
        );
        assert_eq!(
            ParseHandler::detect_log_level("[FATAL] test"),
            LogLevel::Fatal
        );
    }

    #[test]
    fn test_detect_log_level_colon() {
        assert_eq!(
            ParseHandler::detect_log_level("DEBUG: test"),
            LogLevel::Debug
        );
        assert_eq!(ParseHandler::detect_log_level("INFO: test"), LogLevel::Info);
        assert_eq!(
            ParseHandler::detect_log_level("WARNING: test"),
            LogLevel::Warning
        );
        assert_eq!(
            ParseHandler::detect_log_level("ERROR: test"),
            LogLevel::Error
        );
    }

    #[test]
    fn test_detect_log_level_pipes() {
        assert_eq!(
            ParseHandler::detect_log_level("|DEBUG| test"),
            LogLevel::Debug
        );
        assert_eq!(
            ParseHandler::detect_log_level("|INFO| test"),
            LogLevel::Info
        );
        assert_eq!(
            ParseHandler::detect_log_level("|ERROR| test"),
            LogLevel::Error
        );
    }

    #[test]
    fn test_detect_log_level_case_insensitive() {
        assert_eq!(
            ParseHandler::detect_log_level("[info] test"),
            LogLevel::Info
        );
        assert_eq!(
            ParseHandler::detect_log_level("[Error] test"),
            LogLevel::Error
        );
        assert_eq!(
            ParseHandler::detect_log_level("WARN: test"),
            LogLevel::Warning
        );
    }

    #[test]
    fn test_detect_log_level_exception() {
        assert_eq!(
            ParseHandler::detect_log_level("Exception: test"),
            LogLevel::Error
        );
        assert_eq!(
            ParseHandler::detect_log_level("java.lang.Exception: test"),
            LogLevel::Error
        );
    }

    #[test]
    fn test_extract_message_removes_timestamp() {
        let timestamp = Some("2024-01-15T10:30:00".to_string());
        let message = ParseHandler::extract_message(
            "2024-01-15T10:30:00 [INFO] Hello world",
            &timestamp,
            &LogLevel::Info,
        );
        assert!(message.contains("Hello world"));
        assert!(!message.contains("2024-01-15T10:30:00"));
    }

    #[test]
    fn test_extract_message_removes_level() {
        let message = ParseHandler::extract_message("[INFO] Hello world", &None, &LogLevel::Info);
        assert!(message.contains("Hello world"));
        assert!(!message.contains("[INFO]"));
    }

    #[test]
    fn test_format_logs_json() {
        let input = "[INFO] Test message\n[ERROR] Error message";
        let result = ParseHandler::parse_logs(input);
        let output = ParseHandler::format_logs(&result, OutputFormat::Json);

        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["counts"]["total_lines"], 2);
        assert_eq!(json["counts"]["info"], 1);
        assert_eq!(json["counts"]["error"], 1);
    }

    #[test]
    fn test_format_logs_csv() {
        let input = "[INFO] Test message";
        let result = ParseHandler::parse_logs(input);
        let output = ParseHandler::format_logs(&result, OutputFormat::Csv);

        assert!(output.starts_with("line_number,level,timestamp,message\n"));
        assert!(output.contains("1,info,,Test message"));
    }

    #[test]
    fn test_format_logs_tsv() {
        let input = "[INFO] Test message";
        let result = ParseHandler::parse_logs(input);
        let output = ParseHandler::format_logs(&result, OutputFormat::Tsv);

        assert!(output.starts_with("line_number\tlevel\ttimestamp\tmessage\n"));
        assert!(output.contains("1\tinfo\t\tTest message"));
    }

    #[test]
    fn test_format_logs_compact() {
        let input = "[INFO] Info message\n[ERROR] Error message\n[WARN] Warning";
        let result = ParseHandler::parse_logs(input);
        let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

        assert!(output.contains("lines: 3"));
        assert!(output.contains("levels:"));
        assert!(output.contains("error:1"));
        assert!(output.contains("warn:1"));
        assert!(output.contains("info:1"));
    }

    #[test]
    fn test_format_logs_compact_empty() {
        let mut result = LogsOutput::default();
        result.is_empty = true;
        let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

        assert!(output.contains("logs: empty"));
    }

    #[test]
    fn test_format_logs_raw() {
        let input = "[INFO] Test message\n[ERROR] Error";
        let result = ParseHandler::parse_logs(input);
        let output = ParseHandler::format_logs(&result, OutputFormat::Raw);

        assert!(output.contains("[INFO] Test message"));
        assert!(output.contains("[ERROR] Error"));
    }

    #[test]
    fn test_format_logs_compact_with_repeated() {
        let input = "Same line\nSame line\nSame line";
        let result = ParseHandler::parse_logs(input);
        let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

        assert!(output.contains("repeated:"));
        assert!(output.contains("[x3]"));
    }

    #[test]
    fn test_format_logs_compact_collapses_consecutive_entries_no_levels() {
        // Test that consecutive identical lines (no log levels) are collapsed in output
        let input = "Same line\nSame line\nSame line\nDifferent\nDifferent\nUnique";
        let result = ParseHandler::parse_logs(input);
        let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

        // Should show line ranges for collapsed entries
        assert!(output.contains("1-3 Same line [x3]"));
        assert!(output.contains("4-5 Different [x2]"));
        assert!(output.contains("6 Unique"));
    }

    #[test]
    fn test_format_logs_compact_collapses_consecutive_entries_with_levels() {
        // Test that consecutive identical entries with log levels are collapsed
        let input = "[INFO] Starting\n[INFO] Starting\n[INFO] Starting\n[ERROR] Failed\n[ERROR] Failed\n[WARN] Warning";
        let result = ParseHandler::parse_logs(input);
        let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

        // Should show collapsed entries with line ranges
        assert!(output.contains("[I] 1-3 Starting [x3]"));
        assert!(output.contains("[E] 4-5 Failed [x2]"));
        assert!(output.contains("[W] 6 Warning"));
    }

    #[test]
    fn test_format_logs_compact_no_collapse_non_consecutive() {
        // Test that non-consecutive identical lines are NOT collapsed in entries section
        let input = "Line A\nLine B\nLine A";
        let result = ParseHandler::parse_logs(input);
        let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

        // The entries section should show lines separately (not collapsed since not consecutive)
        // The format is "  N content" (2-space indent)
        // Should contain all three entries individually
        assert!(output.contains("  1 Line A"));
        assert!(output.contains("  2 Line B"));
        assert!(output.contains("  3 Line A"));
        // Should not have collapsed format like "1-3 Line A [x2]" in lines section
        // (non-consecutive entries don't collapse)
    }

    // ============================================================
    // Enhanced Error/Warning Level Detection Tests
    // ============================================================

    #[test]
    fn test_detect_log_level_failed() {
        assert_eq!(
            ParseHandler::detect_log_level("Test failed"),
            LogLevel::Error
        );
        assert_eq!(
            ParseHandler::detect_log_level("FAILED: assertion failed"),
            LogLevel::Error
        );
        assert_eq!(
            ParseHandler::detect_log_level("Build failure"),
            LogLevel::Error
        );
    }

    #[test]
    fn test_detect_log_level_stack_trace() {
        assert_eq!(
            ParseHandler::detect_log_level("STACK TRACE:"),
            LogLevel::Error
        );
        assert_eq!(
            ParseHandler::detect_log_level("Backtrace:"),
            LogLevel::Error
        );
        assert_eq!(
            ParseHandler::detect_log_level(
                "Exception at com.example.MyClass.myMethod(MyClass.java:42)"
            ),
            LogLevel::Error
        );
    }

    #[test]
    fn test_detect_log_level_panic_crash() {
        assert_eq!(
            ParseHandler::detect_log_level("PANIC: unrecoverable error"),
            LogLevel::Fatal
        );
        assert_eq!(
            ParseHandler::detect_log_level("Application crashed"),
            LogLevel::Fatal
        );
        assert_eq!(
            ParseHandler::detect_log_level("Aborting due to critical error"),
            LogLevel::Fatal
        );
    }

    #[test]
    fn test_detect_log_level_deprecated() {
        assert_eq!(
            ParseHandler::detect_log_level("DEPRECATED: use newFunction instead"),
            LogLevel::Warning
        );
        assert_eq!(
            ParseHandler::detect_log_level("This method is deprecated"),
            LogLevel::Warning
        );
    }

    #[test]
    fn test_detect_log_level_caution() {
        assert_eq!(
            ParseHandler::detect_log_level("CAUTION: data may be lost"),
            LogLevel::Warning
        );
        assert_eq!(
            ParseHandler::detect_log_level("ATTENTION: read carefully"),
            LogLevel::Warning
        );
    }

    #[test]
    fn test_detect_log_level_connection_errors() {
        assert_eq!(
            ParseHandler::detect_log_level("Connection refused"),
            LogLevel::Error
        );
        assert_eq!(
            ParseHandler::detect_log_level("CONNECTION ERROR: timeout"),
            LogLevel::Error
        );
        assert_eq!(
            ParseHandler::detect_log_level("ACCESS DENIED for user"),
            LogLevel::Error
        );
    }

    #[test]
    fn test_detect_log_level_timeout() {
        assert_eq!(
            ParseHandler::detect_log_level("TIMEOUT ERROR"),
            LogLevel::Error
        );
        assert_eq!(
            ParseHandler::detect_log_level("Request timed out"),
            LogLevel::Unknown
        );
    }

    #[test]
    fn test_detect_log_level_segfault() {
        assert_eq!(ParseHandler::detect_log_level("SEG FAULT"), LogLevel::Error);
        assert_eq!(
            ParseHandler::detect_log_level("SEGFAULT at 0x0"),
            LogLevel::Error
        );
        assert_eq!(
            ParseHandler::detect_log_level("NULL POINTER exception"),
            LogLevel::Error
        );
    }

    #[test]
    fn test_detect_log_level_notice() {
        assert_eq!(
            ParseHandler::detect_log_level("NOTICE: system maintenance"),
            LogLevel::Info
        );
        assert_eq!(
            ParseHandler::detect_log_level("[NOTICE] Server starting"),
            LogLevel::Info
        );
    }

    #[test]
    fn test_detect_log_level_slow_queries() {
        assert_eq!(
            ParseHandler::detect_log_level("SLOW QUERY detected"),
            LogLevel::Warning
        );
        assert_eq!(
            ParseHandler::detect_log_level("SLOW REQUEST: 5.2s"),
            LogLevel::Warning
        );
    }

    #[test]
    fn test_detect_log_level_negation_patterns() {
        // These should NOT be detected as errors due to negation patterns
        assert_eq!(
            ParseHandler::detect_log_level("No errors found"),
            LogLevel::Unknown
        );
        assert_eq!(
            ParseHandler::detect_log_level("Completed with 0 errors"),
            LogLevel::Unknown
        );
        assert_eq!(
            ParseHandler::detect_log_level("Zero failures detected"),
            LogLevel::Unknown
        );
    }

    #[test]
    fn test_detect_log_level_case_variations() {
        // Test mixed case detection
        assert_eq!(
            ParseHandler::detect_log_level("ERROR: something bad"),
            LogLevel::Error
        );
        assert_eq!(
            ParseHandler::detect_log_level("Error: something bad"),
            LogLevel::Error
        );
        assert_eq!(
            ParseHandler::detect_log_level("error: something bad"),
            LogLevel::Error
        );
        assert_eq!(ParseHandler::detect_log_level("FAILED"), LogLevel::Error);
        assert_eq!(ParseHandler::detect_log_level("failed"), LogLevel::Error);
        assert_eq!(ParseHandler::detect_log_level("Failed"), LogLevel::Error);
    }

    #[test]
    fn test_parse_logs_counts_levels_correctly() {
        let input = r#"[INFO] Starting
[ERROR] Connection failed
[WARN] Deprecated API
[FATAL] System panic
[DEBUG] Trace info
Unknown line"#;
        let result = ParseHandler::parse_logs(input);

        assert_eq!(result.total_lines, 6);
        assert_eq!(result.info_count, 1);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.fatal_count, 1);
        assert_eq!(result.debug_count, 1);
        assert_eq!(result.unknown_count, 1);
    }

    #[test]
    fn test_parse_logs_multiple_errors() {
        let input = r#"Test case 1 FAILED
Test case 2 FAILED
Exception in thread main
Connection refused"#;
        let result = ParseHandler::parse_logs(input);

        assert_eq!(result.error_count, 4);
    }

    #[test]
    fn test_parse_logs_mixed_levels() {
        let input = r#"Starting application...
PANIC: unrecoverable error
Build failure
Deprecated method used
All systems operational"#;
        let result = ParseHandler::parse_logs(input);

        assert_eq!(result.fatal_count, 1);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.unknown_count, 2);
    }

    // ============================================================
    // Recent Critical Lines Tests
    // ============================================================

    #[test]
    fn test_parse_logs_tracks_recent_critical() {
        let input = r#"[INFO] Starting
[ERROR] First error
[WARN] Warning
[FATAL] Fatal error
[ERROR] Second error
[INFO] Done"#;
        let result = ParseHandler::parse_logs(input);

        // Should track 3 critical lines (ERROR and FATAL)
        assert_eq!(result.recent_critical.len(), 3);
        assert_eq!(result.recent_critical[0].message, "First error");
        assert_eq!(result.recent_critical[1].message, "Fatal error");
        assert_eq!(result.recent_critical[2].message, "Second error");
    }

    #[test]
    fn test_parse_logs_recent_critical_only_errors_and_fatals() {
        let input = r#"[INFO] Info message
[WARN] Warning message
[DEBUG] Debug message
[ERROR] Error message
[FATAL] Fatal message"#;
        let result = ParseHandler::parse_logs(input);

        // Only ERROR and FATAL should be in recent_critical
        assert_eq!(result.recent_critical.len(), 2);
        assert_eq!(result.recent_critical[0].level, LogLevel::Error);
        assert_eq!(result.recent_critical[1].level, LogLevel::Fatal);
    }

    #[test]
    fn test_parse_logs_recent_critical_limit() {
        // Create input with more than MAX_RECENT_CRITICAL (10) errors
        let mut input = String::new();
        for i in 1..=15 {
            input.push_str(&format!("[ERROR] Error message {}\n", i));
        }
        let result = ParseHandler::parse_logs(&input);

        // Should be limited to MAX_RECENT_CRITICAL (10)
        assert_eq!(result.recent_critical.len(), 10);
        // Should keep the most recent (last 10)
        assert_eq!(result.recent_critical[0].message, "Error message 6");
        assert_eq!(result.recent_critical[9].message, "Error message 15");
    }

    #[test]
    fn test_parse_logs_recent_critical_order() {
        let input = r#"[ERROR] Error at line 1
[INFO] Info
[FATAL] Fatal at line 3
[ERROR] Error at line 4
[FATAL] Fatal at line 5"#;
        let result = ParseHandler::parse_logs(input);

        assert_eq!(result.recent_critical.len(), 4);
        // Should be in order of appearance
        assert_eq!(result.recent_critical[0].line_number, 1);
        assert_eq!(result.recent_critical[1].line_number, 3);
        assert_eq!(result.recent_critical[2].line_number, 4);
        assert_eq!(result.recent_critical[3].line_number, 5);
    }

    #[test]
    fn test_parse_logs_no_critical_lines() {
        let input = r#"[INFO] Starting
[DEBUG] Debug info
[WARN] Warning"#;
        let result = ParseHandler::parse_logs(input);

        assert!(result.recent_critical.is_empty());
    }

    #[test]
    fn test_format_logs_json_includes_recent_critical() {
        let input = "[INFO] Info\n[ERROR] Error 1\n[ERROR] Error 2\n[FATAL] Fatal";
        let result = ParseHandler::parse_logs(input);
        let output = ParseHandler::format_logs(&result, OutputFormat::Json);

        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        // Should have recent_critical array
        assert!(json["recent_critical"].is_array());
        let recent = json["recent_critical"].as_array().unwrap();
        assert_eq!(recent.len(), 3);

        // Should have counts
        assert_eq!(json["recent_critical_count"], 3);
        assert_eq!(json["total_critical"], 3);
    }

    #[test]
    fn test_format_logs_compact_shows_recent_critical() {
        let input = "[INFO] Starting\n[ERROR] Something failed\n[FATAL] System crash\n[INFO] Done";
        let result = ParseHandler::parse_logs(input);
        let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

        // Should show recent critical section
        assert!(output.contains("recent critical"));
        assert!(output.contains("[E]"));
        assert!(output.contains("[F]"));
        assert!(output.contains("Something failed"));
        assert!(output.contains("System crash"));
    }

    #[test]
    fn test_format_logs_compact_recent_critical_count() {
        // Create more than MAX_RECENT_CRITICAL errors
        let mut input = String::new();
        for i in 1..=15 {
            input.push_str(&format!("[ERROR] Error {}\n", i));
        }
        let result = ParseHandler::parse_logs(&input);
        let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

        // Should show count as "10 of 15"
        assert!(output.contains("10 of 15"));
    }
}
