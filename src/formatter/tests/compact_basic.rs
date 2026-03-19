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
