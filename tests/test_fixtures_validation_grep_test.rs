//! Validation tests for grep, pytest, and jest fixture loader functions.
//!
//! These tests verify that grep, pytest, and jest fixture files exist,
//! load correctly, and contain expected content markers.

mod fixtures;
use fixtures::*;

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
    assert!(content.contains("unicode_\u{00F1}ame.rs"));
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
    assert!(content.contains("\u{2713}"));
    assert!(content.contains("1 passed"));
}

#[test]
fn test_load_jest_single_suite_failed() {
    let content = jest_single_suite_failed();
    assert!(content.contains("FAIL"));
    assert!(content.contains("\u{2715}"));
    assert!(content.contains("expect(received).toBe(expected)"));
    assert!(content.contains("1 failed"));
}

#[test]
fn test_load_jest_mixed() {
    let content = jest_mixed();
    assert!(content.contains("PASS"));
    assert!(content.contains("FAIL"));
    assert!(content.contains("\u{2713}"));
    assert!(content.contains("\u{2715}"));
    assert!(content.contains("\u{25CB} skipped"));
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
    assert!(content.contains("\u{25CB} skipped"));
    assert!(content.contains("2 skipped"));
}

#[test]
fn test_load_jest_with_todo() {
    let content = jest_with_todo();
    assert!(content.contains("\u{25CB} todo"));
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
