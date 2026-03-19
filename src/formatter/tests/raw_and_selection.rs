use super::*;

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
