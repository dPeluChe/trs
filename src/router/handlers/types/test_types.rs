//! Test runner data structures for command handlers.

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
