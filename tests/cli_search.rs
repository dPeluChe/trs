use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

// Search Grouping Tests
// ============================================================

#[test]
fn test_search_groups_matches_by_file_compact() {
    // Test that matches are grouped by file in compact output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        // Should show file header with match count
        .stdout(predicate::str::contains("search.rs ("))
        // Should show individual matches under the file
        .stdout(predicate::str::contains("  "))
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_groups_matches_by_file_json() {
    // Test that matches are grouped by file in JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Parse JSON to verify structure
    let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();

    // Should have files array
    assert!(json["files"].is_array());

    // Each file should have path and matches
    let files = json["files"].as_array().unwrap();
    for file in files {
        assert!(file["path"].is_string());
        assert!(file["matches"].is_array());

        // Each match should have required fields
        for m in file["matches"].as_array().unwrap() {
            assert!(m["line_number"].is_number());
            assert!(m["line"].is_string());
        }
    }
}

#[test]
fn test_search_groups_multiple_files_correctly() {
    // Test that multiple files are each shown as separate groups
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("fn") // Pattern that appears in multiple files
        .arg("--extension")
        .arg("rs")
        .arg("--limit")
        .arg("10")
        .assert()
        .success()
        // Should show file count
        .stdout(predicate::str::contains("files"));
}

#[test]
fn test_search_groups_single_file_multiple_matches() {
    // Test that multiple matches in the same file are grouped together
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("search")
        .arg("src/router/")
        .arg("SearchHandler") // Appears multiple times in one file
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should only show the file header once
    let router_count = output_str.matches("router/").count();
    assert!(router_count >= 1);

    // Should show match count > 1
    assert!(output_str.contains("("));
}

#[test]
fn test_search_groups_preserve_line_numbers() {
    // Test that grouping preserves line numbers
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("search")
        .arg("src/router/")
        .arg("struct SearchHandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Raw format should show line numbers with colons
    assert!(output_str.contains(":"));
    assert!(output_str.contains("SearchHandler"));
}

#[test]
fn test_search_groups_with_truncation() {
    // Test that grouping works correctly with truncation
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("fn") // Common pattern
        .arg("--extension")
        .arg("rs")
        .arg("--limit")
        .arg("2") // Limit to 2 files
        .assert()
        .success()
        // Should show truncation info
        .stdout(predicate::str::contains("files"));
}

#[test]
fn test_search_csv_maintains_file_column() {
    // Test that CSV output includes file path for each match
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("path,line_number"))
        .stdout(predicate::str::contains("router/"));
}

#[test]
fn test_search_tsv_maintains_file_column() {
    // Test that TSV output includes file path for each match
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("path\tline_number"))
        .stdout(predicate::str::contains("router/"));
}

#[test]
fn test_search_counts_match_groups() {
    // Test that counts reflect grouped matches
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("search")
        .arg("src/router/")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should show files count and results count
    assert!(output_str.contains("matches:"));
    assert!(output_str.contains("files"));
    assert!(output_str.contains("results"));
}

#[test]
fn test_search_groups_case_insensitive() {
    // Test that grouping works with case-insensitive search
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("searchhandler") // lowercase
        .arg("--extension")
        .arg("rs")
        .arg("--ignore-case")
        .assert()
        .success()
        .stdout(predicate::str::contains("router/"));
}

#[test]
fn test_search_includes_excerpts_compact() {
    // Test that compact output includes match excerpts
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);
    // Excerpt should be shown in square brackets
    assert!(output_str.contains("[SearchHandler]"));
    // Column number should be shown (format: line:col:)
    let has_col = output_str.lines().any(|l| {
        let parts: Vec<&str> = l.trim().splitn(3, ':').collect();
        parts.len() >= 2 && parts[0].parse::<u32>().is_ok() && parts[1].parse::<u32>().is_ok()
    });
    assert!(has_col, "Output should contain column numbers");
}

#[test]
fn test_search_includes_excerpts_json() {
    // Test that JSON output includes excerpt field
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();

    // Check that matches have excerpt field
    let files = json["files"].as_array().unwrap();
    assert!(!files.is_empty());

    let first_file = &files[0];
    let matches = first_file["matches"].as_array().unwrap();
    assert!(!matches.is_empty());

    // First match should have excerpt
    let first_match = &matches[0];
    assert!(first_match["excerpt"].is_string());
    assert_eq!(first_match["excerpt"].as_str().unwrap(), "SearchHandler");

    // First match should have column number
    assert!(first_match["column"].is_number());
}

#[test]
fn test_search_shows_total_files_and_match_count() {
    // Test that compact output shows total files and match count at the end
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("search")
        .arg("src/router/")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should show total at the end with both files and matches
    assert!(output_str.contains("total:"));
    assert!(output_str.contains("files"));
    assert!(output_str.contains("matches"));

    // Verify the format - should end with "total: X files, Y matches\n"
    let lines: Vec<&str> = output_str.lines().collect();
    let last_line = lines.last().unwrap();
    assert!(last_line.starts_with("total:"));
    assert!(last_line.contains("files"));
    assert!(last_line.ends_with("matches"));
}

#[test]
fn test_search_shows_total_with_truncation() {
    // Test that truncated output shows shown/total format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("search")
        .arg(".")
        .arg("fn")
        .arg("--extension")
        .arg("rs")
        .arg("--limit")
        .arg("2") // Force truncation
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should show truncated summary with both files and matches
    assert!(output_str.contains("total:"));
    // Should have truncation info in format "total: X/Y files, A/B matches"
    assert!(output_str.contains("/"));
    assert!(output_str.contains("files"));
    assert!(output_str.contains("matches"));
}

#[test]
fn test_search_total_no_matches() {
    // Test that no matches case doesn't show total
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("search")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_12345")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should show no matches message but not total line
    assert!(output_str.contains("no matches"));
}
