//! Test runner schema types.

use serde::{Deserialize, Serialize};

use super::SchemaVersion;

// ============================================================
// Test Output Schema (Unified)
// ============================================================

/// Schema for test runner output (unified across all runners).
///
/// Supports: pytest, jest, vitest, npm test, pnpm test, bun test.
///
/// # Example JSON
///
/// ```json
/// {
///   "schema": { "version": "1.0.0", "type": "test_output" },
///   "runner": "pytest",
///   "is_empty": false,
///   "success": true,
///   "test_suites": [
///     {
///       "file": "tests/test_main.py",
///       "passed": true,
///       "duration_ms": 150,
///       "tests": [
///         { "name": "test_example", "status": "passed", "duration_ms": 10 }
///       ]
///     }
///   ],
///   "summary": {
///     "total": 10,
///     "passed": 8,
///     "failed": 2,
///     "skipped": 0,
///     "duration_ms": 1500
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestOutputSchema {
    /// Schema version information.
    pub schema: SchemaVersion,
    /// Test runner type.
    pub runner: TestRunnerType,
    /// Whether the output is empty.
    pub is_empty: bool,
    /// Whether all tests passed.
    pub success: bool,
    /// List of test suites (files).
    #[serde(default)]
    pub test_suites: Vec<TestSuite>,
    /// Summary statistics.
    pub summary: TestSummary,
    /// Runner version (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runner_version: Option<String>,
    /// Platform info (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    /// Working directory (rootdir for pytest, cwd for others).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
}

impl TestOutputSchema {
    /// Create a new test output schema.
    pub fn new(runner: TestRunnerType) -> Self {
        Self {
            schema: SchemaVersion::new("test_output"),
            runner,
            is_empty: true,
            success: true,
            test_suites: Vec::new(),
            summary: TestSummary::default(),
            runner_version: None,
            platform: None,
            working_directory: None,
        }
    }
}

/// Supported test runner types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestRunnerType {
    /// Python pytest.
    Pytest,
    /// JavaScript Jest.
    Jest,
    /// JavaScript Vitest.
    Vitest,
    /// npm test (Node.js built-in test runner).
    Npm,
    /// pnpm test.
    Pnpm,
    /// bun test.
    Bun,
}

impl std::fmt::Display for TestRunnerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestRunnerType::Pytest => write!(f, "pytest"),
            TestRunnerType::Jest => write!(f, "jest"),
            TestRunnerType::Vitest => write!(f, "vitest"),
            TestRunnerType::Npm => write!(f, "npm"),
            TestRunnerType::Pnpm => write!(f, "pnpm"),
            TestRunnerType::Bun => write!(f, "bun"),
        }
    }
}

/// A test suite (typically a file).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestSuite {
    /// Test file path.
    pub file: String,
    /// Whether the suite passed.
    pub passed: bool,
    /// Execution time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Number of tests in suite.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_count: Option<usize>,
    /// List of test results in this suite.
    #[serde(default)]
    pub tests: Vec<TestResult>,
}

impl TestSuite {
    /// Create a new test suite.
    pub fn new(file: &str) -> Self {
        Self {
            file: file.to_string(),
            passed: true,
            duration_ms: None,
            test_count: None,
            tests: Vec::new(),
        }
    }
}

/// A single test result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestResult {
    /// Full test name (module::test_name or file::test_name).
    pub name: String,
    /// Test name only (last part).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_name: Option<String>,
    /// Ancestor names (describe blocks).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ancestors: Vec<String>,
    /// Status of the test.
    pub status: TestStatus,
    /// Duration in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Error message (for failed tests).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// File path (if different from suite).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Line number (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

impl TestResult {
    /// Create a new test result.
    pub fn new(name: &str, status: TestStatus) -> Self {
        Self {
            name: name.to_string(),
            test_name: None,
            ancestors: Vec::new(),
            status,
            duration_ms: None,
            error_message: None,
            file: None,
            line: None,
        }
    }
}

/// Status of a test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
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
    /// Test was todo.
    Todo,
}

/// Test summary statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestSummary {
    /// Total number of tests.
    pub total: usize,
    /// Number of passed tests.
    pub passed: usize,
    /// Number of failed tests.
    pub failed: usize,
    /// Number of skipped tests.
    #[serde(default)]
    pub skipped: usize,
    /// Number of xfailed tests.
    #[serde(default)]
    pub xfailed: usize,
    /// Number of xpassed tests.
    #[serde(default)]
    pub xpassed: usize,
    /// Number of error tests.
    #[serde(default)]
    pub errors: usize,
    /// Number of todo tests.
    #[serde(default)]
    pub todo: usize,
    /// Number of test suites passed.
    #[serde(default)]
    pub suites_passed: usize,
    /// Number of test suites failed.
    #[serde(default)]
    pub suites_failed: usize,
    /// Number of test suites total.
    #[serde(default)]
    pub suites_total: usize,
    /// Total duration in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}
