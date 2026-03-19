#![allow(dead_code)]
use super::load_fixture;

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
