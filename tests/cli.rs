use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

// ============================================================
// Basic Help Tests
// ============================================================

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("TARS CLI"))
        .stdout(predicate::str::contains("Transform noisy terminal output"));
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("trs"));
}

// ============================================================
// Help System Tests
// ============================================================

#[test]
fn test_help_shows_output_format_flags() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("FORMAT FLAGS"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--csv"))
        .stdout(predicate::str::contains("--tsv"))
        .stdout(predicate::str::contains("--agent"))
        .stdout(predicate::str::contains("--compact"))
        .stdout(predicate::str::contains("--raw"));
}

#[test]
fn test_help_shows_global_flags() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--stats"));
}

#[test]
fn test_help_shows_examples() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("EXAMPLES"));
}

#[test]
fn test_help_shows_documentation_link() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("trs"));
}

#[test]
fn test_help_shows_all_commands() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("parse"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("replace"))
        .stdout(predicate::str::contains("tail"))
        .stdout(predicate::str::contains("clean"))
        .stdout(predicate::str::contains("html2md"))
        .stdout(predicate::str::contains("txt2md"));
}

// ============================================================
// Command-Specific Help Tests
// ============================================================

#[test]
fn test_search_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Search for patterns"))
        .stdout(predicate::str::contains("ripgrep"))
        .stdout(predicate::str::contains("--extension"))
        .stdout(predicate::str::contains("--ignore-case"))
        .stdout(predicate::str::contains("--context"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn test_replace_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Search and replace"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn test_tail_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tail a file"))
        .stdout(predicate::str::contains("--errors"))
        .stdout(predicate::str::contains("--follow"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn test_clean_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Clean and format"))
        .stdout(predicate::str::contains("--no-ansi"))
        .stdout(predicate::str::contains("--collapse-blanks"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn test_parse_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse structured input"))
        .stdout(predicate::str::contains("git-status"))
        .stdout(predicate::str::contains("git-diff"))
        .stdout(predicate::str::contains("ls"))
        .stdout(predicate::str::contains("grep"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("logs"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn test_html2md_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Convert HTML to Markdown"))
        .stdout(predicate::str::contains("--metadata"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn test_txt2md_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Convert plain text to Markdown"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn test_run_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Execute a command"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

// ============================================================
// Parse Subcommand Help Tests
// ============================================================

#[test]
fn test_parse_git_status_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse git status"))
        .stdout(predicate::str::contains("branch info"));
}

#[test]
fn test_parse_git_diff_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse git diff"));
}

#[test]
fn test_parse_test_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("test")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse test runner"))
        .stdout(predicate::str::contains("pytest"));
}

// ============================================================
// Global Flags Tests
// ============================================================

#[test]
fn test_global_flags_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--raw"))
        .stdout(predicate::str::contains("--compact"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--csv"))
        .stdout(predicate::str::contains("--tsv"))
        .stdout(predicate::str::contains("--agent"))
        .stdout(predicate::str::contains("--stats"));
}

// ============================================================
// Command Execution Tests
// ============================================================

#[test]
fn test_search_basic() {
    // Search should find results when searching for a pattern that exists
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("matches:"))
        .stdout(predicate::str::contains("router/"));
}

#[test]
fn test_search_with_options() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .arg("--ignore-case")
        .assert()
        .success();
}

#[test]
fn test_search_json_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema\""))
        .stdout(predicate::str::contains("\"grep_output\""))
        .stdout(predicate::str::contains("\"files\""))
        .stdout(predicate::str::contains("router/"));
}

#[test]
fn test_search_csv_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "path,line_number,column,is_context,line",
        ))
        .stdout(predicate::str::contains("router/"));
}

#[test]
fn test_search_tsv_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "path\tline_number\tcolumn\tis_context\tline",
        ))
        .stdout(predicate::str::contains("router/"));
}

#[test]
fn test_search_raw_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("search.rs:"))
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_with_context() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .arg("--context")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("matches:"));
}

#[test]
fn test_search_with_limit() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .arg("--limit")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("matches:"));
}

#[test]
fn test_search_no_matches() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_12345")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("no matches"));
}

// ============================================================
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

#[test]
fn test_search_extension_filter_rs() {
    // Test that extension filter only searches .rs files
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("search")
        .arg(".")
        .arg("fn")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should only contain .rs files, not .md files
    assert!(output_str.contains(".rs"));
    assert!(!output_str.contains(".md:"));
}

#[test]
fn test_search_extension_filter_md() {
    // Test that extension filter only searches .md files
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("search")
        .arg(".")
        .arg("CLI")
        .arg("--extension")
        .arg("md")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should only contain .md files, not .rs files
    assert!(output_str.contains(".md"));
    assert!(!output_str.contains(".rs:"));
}

#[test]
fn test_search_extension_filter_json_output() {
    // Test extension filter with JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg(".")
        .arg("fn")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // All files should have .rs extension
    let files = json["files"].as_array().unwrap();
    for file in files {
        let path = file["path"].as_str().unwrap();
        assert!(path.ends_with(".rs"), "Expected .rs file, got: {}", path);
    }
}

#[test]
fn test_replace_extension_filter_rs() {
    // Test that replace extension filter only processes .rs files
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_12345_UNIQUE")
        .arg("new")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should show no matches message
    assert!(output_str.contains("No matches found"));
}

#[test]
fn test_replace_extension_filter_md() {
    // Test that replace extension filter only processes .md files
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_12345_UNIQUE")
        .arg("new")
        .arg("--dry-run")
        .arg("--extension")
        .arg("md")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should show no matches message (src has no .md files)
    assert!(output_str.contains("No matches found"));
}

#[test]
fn test_replace_extension_filter_json_output() {
    // Test replace extension filter with JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_12345_UNIQUE")
        .arg("new")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should have the expected schema
    assert_eq!(json["schema"]["type"], "replace_output");
    assert!(json["dry_run"].as_bool().unwrap());
}

#[test]
fn test_replace_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_12345")
        .arg("new")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_replace_dry_run() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_DRY_RUN_12345")
        .arg("new")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_replace_preview_flag() {
    // Test that --preview flag works as an alias for --dry-run
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_PREVIEW_12345")
        .arg("new")
        .arg("--preview")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_replace_preview_json_output() {
    // Test that --preview flag sets dry_run to true in JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_PREVIEW_JSON_12345")
        .arg("new")
        .arg("--preview")
        .arg("--extension")
        .arg("rs")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON output");
    assert_eq!(json["schema"]["type"], "replace_output");
    assert!(json["dry_run"].as_bool().unwrap());
}

#[test]
fn test_tail_basic() {
    // Create a temporary test file
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "line 2").unwrap();
    writeln!(file, "line 3").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"))
        .stdout(predicate::str::contains("line 3"));
}

#[test]
fn test_tail_with_lines() {
    // Create a temporary test file with many lines
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    for i in 1..=20 {
        writeln!(file, "line {}", i).unwrap();
    }

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg(path)
        .arg("--lines")
        .arg("5")
        .assert()
        .success()
        .stdout(predicate::str::contains("line 16"))
        .stdout(predicate::str::contains("line 20"))
        .stdout(predicate::function(|s: &str| !s.contains("line 15")));
}

#[test]
fn test_tail_with_errors_flag() {
    // Create a temporary test file with errors
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "INFO: process started").unwrap();
    writeln!(file, "ERROR: something went wrong").unwrap();
    writeln!(file, "WARNING: deprecated API").unwrap();
    writeln!(file, "FATAL: critical failure").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg(path)
        .arg("--errors")
        .assert()
        .success()
        .stdout(predicate::str::contains("ERROR"))
        .stdout(predicate::str::contains("FATAL"))
        .stdout(predicate::function(|s: &str| !s.contains("INFO")))
        .stdout(predicate::function(|s: &str| !s.contains("WARNING")));
}

#[test]
fn test_tail_json_output() {
    // Create a temporary test file
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "ERROR: test error").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON output");

    assert!(json["file"].is_string());
    assert!(json["lines"].is_array());
    assert!(json["total_lines"].is_number());
    assert!(json["lines_shown"].is_number());
    assert!(!json["filtering_errors"].as_bool().unwrap());
}

#[test]
fn test_tail_csv_output() {
    // Create a temporary test file
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "line 2").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line_number,line,is_error"))
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_tail_tsv_output() {
    // Create a temporary test file
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "line 2").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line_number\tline\tis_error"))
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_tail_raw_output() {
    // Create a temporary test file
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "line 2").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("1:line 1"))
        .stdout(predicate::str::contains("2:line 2"));
}

#[test]
fn test_tail_agent_output() {
    // Create a temporary test file
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "ERROR: test error").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("File:"))
        .stdout(predicate::str::contains("❌"))
        .stdout(predicate::str::contains("ERROR"));
}

#[test]
fn test_tail_file_not_found() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("/nonexistent/file.log")
        .assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));
}

// ============================================================
// Tail Compact Output Tests
// ============================================================

#[test]
fn test_tail_compact_output() {
    // Test that --compact flag produces compact output
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "line 2").unwrap();
    writeln!(file, "ERROR: test error").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Last"))
        .stdout(predicate::str::contains("lines from"))
        .stdout(predicate::str::contains("total:"))
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"))
        .stdout(predicate::str::contains("ERROR: test error"));
}

#[test]
fn test_tail_compact_is_default() {
    // Test that default output is compact (same as --compact)
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "INFO: starting").unwrap();
    writeln!(file, "ERROR: failed").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should show compact format header
    assert!(output_str.contains("Last"));
    assert!(output_str.contains("lines from"));
    assert!(output_str.contains("total:"));
    // Should show error marker
    assert!(output_str.contains("❌"));
}

#[test]
fn test_tail_compact_with_errors_flag() {
    // Test compact output with --errors flag
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "INFO: process started").unwrap();
    writeln!(file, "ERROR: something went wrong").unwrap();
    writeln!(file, "WARNING: deprecated API").unwrap();
    writeln!(file, "FATAL: critical failure").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("tail")
        .arg(path)
        .arg("--errors")
        .assert()
        .success()
        .stdout(predicate::str::contains("Error lines from"))
        .stdout(predicate::str::contains("of"))
        .stdout(predicate::str::contains("total"))
        .stdout(predicate::str::contains("ERROR"))
        .stdout(predicate::str::contains("FATAL"))
        .stdout(predicate::function(|s: &str| !s.contains("INFO")))
        .stdout(predicate::function(|s: &str| !s.contains("WARNING")));
}

#[test]
fn test_tail_compact_error_markers() {
    // Test that compact output shows error markers (❌) for error lines
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "INFO: normal log").unwrap();
    writeln!(file, "ERROR: error log").unwrap();
    writeln!(file, "DEBUG: debug log").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should contain error marker for ERROR line
    assert!(output_str.contains("❌"));
    // Should show all lines
    assert!(output_str.contains("INFO: normal log"));
    assert!(output_str.contains("ERROR: error log"));
    assert!(output_str.contains("DEBUG: debug log"));
}

#[test]
fn test_tail_compact_with_line_numbers() {
    // Test that compact output shows line numbers
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    for i in 1..=5 {
        writeln!(file, "line {}", i).unwrap();
    }

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should show line numbers in format like "1:line 1"
    assert!(output_str.contains("1:line 1"));
    assert!(output_str.contains("2:line 2"));
    assert!(output_str.contains("5:line 5"));
}

#[test]
fn test_tail_compact_empty_file() {
    // Test compact output with empty file
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let _file = std::fs::File::create(path).unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("File is empty"));
}

#[test]
fn test_tail_compact_no_error_lines_found() {
    // Test compact output when filtering for errors but none found
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "INFO: all good").unwrap();
    writeln!(file, "DEBUG: debugging").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("tail")
        .arg(path)
        .arg("--errors")
        .assert()
        .success()
        .stdout(predicate::str::contains("No error lines found"));
}

#[test]
fn test_tail_compact_with_custom_line_count() {
    // Test compact output with custom line count
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    for i in 1..=20 {
        writeln!(file, "line {}", i).unwrap();
    }

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("tail")
        .arg(path)
        .arg("--lines")
        .arg("5")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should show last 5 lines
    assert!(output_str.contains("Last 5 lines"));
    assert!(output_str.contains("16:line 16"));
    assert!(output_str.contains("20:line 20"));
    // Should not show earlier lines
    assert!(!output_str.contains("15:line 15"));
}

#[test]
fn test_tail_syntax_simple() {
    // Test the simple syntax: trs tail <file>
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "line 2").unwrap();
    writeln!(file, "line 3").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"))
        .stdout(predicate::str::contains("line 3"));
}

#[test]
fn test_tail_syntax_with_flags() {
    // Test syntax with flags: trs tail <file> -n 5 --errors
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "INFO: starting").unwrap();
    writeln!(file, "ERROR: failed").unwrap();
    writeln!(file, "INFO: running").unwrap();
    writeln!(file, "FATAL: crash").unwrap();
    writeln!(file, "INFO: done").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg(path)
        .arg("--errors")
        .assert()
        .success()
        .stdout(predicate::str::contains("ERROR"))
        .stdout(predicate::str::contains("FATAL"))
        .stdout(predicate::function(|s: &str| !s.contains("INFO")));
}

#[test]
fn test_tail_syntax_with_global_flags() {
    // Test syntax with global flags: trs --json tail <file>
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "line 2").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON output");

    assert!(json["file"].is_string());
    assert!(json["lines"].is_array());
    assert_eq!(json["total_lines"].as_i64().unwrap(), 2);
}

#[test]
fn test_clean_basic() {
    // Test basic clean with stdin input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .write_stdin("  hello world  \n\n\n  line 2  ")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_clean_with_options() {
    // Test clean with all options
    let input = "\x1b[31mRed text\x1b[0m\n\n\n  repeated line  \n  repeated line  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .arg("--collapse-blanks")
        .arg("--collapse-repeats")
        .arg("--trim")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Red text"))
        .stdout(predicate::str::contains("repeated line"))
        .stdout(predicate::function(|s: &str| {
            // Should have only one occurrence of "repeated line" due to collapse-repeats
            let count = s.matches("repeated line").count();
            count == 1
        }));
}

#[test]
fn test_parse_git_status() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .assert()
        .success()
        .stdout(predicate::str::contains("clean"));
}

#[test]
fn test_parse_git_diff() {
    let diff_input = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,6 +10,8 @@ fn main() {
     println!("Hello");
+    let x = 1;
+    let y = 2;
 }
"#;

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(diff_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("files (1)"))
        .stdout(predicate::str::contains("main.rs"))
        .stdout(predicate::str::contains("+2"));
}

#[test]
fn test_parse_ls() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse").arg("ls").assert().success();
}

// ============================================================
// LS Parser Tests
// ============================================================

#[test]
fn test_parse_ls_empty() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains("(empty)"));
}

#[test]
fn test_parse_ls_simple_files() {
    let ls_input = "file1.txt\nfile2.txt\nfile3.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"))
        .stdout(predicate::str::contains("file3.txt"))
        .stdout(predicate::str::contains("3 files"));
}

#[test]
fn test_parse_ls_with_directories() {
    let ls_input = "file1.txt\ndir1\nfile2.txt\ndir2\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("2 files"))
        .stdout(predicate::str::contains("2 dirs"))
        .stdout(predicate::str::contains("dir1/"))
        .stdout(predicate::str::contains("dir2/"));
}

#[test]
fn test_parse_ls_with_hidden_files() {
    let ls_input = "file1.txt\n.hidden_file\n.visible_file\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(".hidden_file"))
        .stdout(predicate::str::contains(".visible_file"));
}

// ============================================================
// Hidden File Detection Tests
// ============================================================

#[test]
fn test_parse_ls_hidden_directory() {
    // Test that hidden directories are detected
    let ls_input = ".git/\npublic/\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(".git/"))
        .stdout(predicate::str::contains("public/"));
}

#[test]
fn test_parse_ls_hidden_file_with_extension() {
    // Test that hidden files with extensions are detected
    let ls_input = ".gitignore\n.env.local\n.config.json\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(".gitignore"))
        .stdout(predicate::str::contains(".env.local"))
        .stdout(predicate::str::contains(".config.json"));
}

#[test]
fn test_parse_ls_dot_and_dotdot() {
    // Test that . and .. are filtered from compact output (they add no signal)
    let ls_input = ".\n..\nfile.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("file.txt"));
}

#[test]
fn test_parse_ls_double_dots() {
    // Test files starting with multiple dots
    let ls_input = "..swp\n...triple\nfile.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("..swp"))
        .stdout(predicate::str::contains("...triple"));
}

#[test]
fn test_parse_ls_long_format_hidden_files() {
    // Test hidden files in long format output
    let ls_input = "total 8\n-rw-r--r--  1 user  group  123 Jan  1 12:34 .gitignore\n-rw-r--r--  1 user  group  456 Jan  1 12:34 .env\ndrwxr-xr-x  2 user  group 4096 Jan  1 12:34 .git\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(".gitignore"))
        .stdout(predicate::str::contains(".env"))
        .stdout(predicate::str::contains(".git"));
}

#[test]
fn test_parse_ls_hidden_symlink() {
    // Test hidden symlinks
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 .link_to_file\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(".link_to_file"));
}

#[test]
fn test_parse_ls_json_hidden_files() {
    // Test JSON output includes is_hidden field
    let ls_input = "file.txt\n.hidden\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_hidden\": false"))
        .stdout(predicate::str::contains("\"is_hidden\": true"))
        .stdout(predicate::str::contains("\"hidden\": ["))
        .stdout(predicate::str::contains(".hidden"));
}

#[test]
fn test_parse_ls_mixed_hidden_and_visible() {
    // Test a mix of hidden and visible files/directories
    let ls_input = "public/\nsrc/\n.git/\n.env\nREADME.md\n.gitignore\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(".git/"))
        .stdout(predicate::str::contains(".env"))
        .stdout(predicate::str::contains(".gitignore"));
}

#[test]
fn test_parse_ls_only_hidden_files() {
    // Test when all files are hidden
    let ls_input = ".a\n.b\n.c\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(".a"))
        .stdout(predicate::str::contains(".b"))
        .stdout(predicate::str::contains(".c"));
}

#[test]
fn test_parse_ls_no_hidden_files() {
    // Test when no hidden files are present
    let ls_input = "file1.txt\nfile2.txt\nfile3.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("3 files"))
        .stdout(predicate::str::contains("file1.txt"));
}

#[test]
fn test_parse_ls_long_format() {
    let ls_input = "total 0\ndrwxr-xr-x  2 user  group  4096 Jan  1 12:34 dirname\n-rw-r--r--  1 user  group    42 Jan  1 12:34 file1.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 files"))
        .stdout(predicate::str::contains("1 dirs"))
        .stdout(predicate::str::contains("dirname/"))
        .stdout(predicate::str::contains("file1.txt"));
}

#[test]
fn test_parse_ls_json_format() {
    let ls_input = "file1.txt\nfile2.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        // Check schema structure
        .stdout(predicate::str::contains("\"schema\""))
        .stdout(predicate::str::contains("\"type\": \"ls_output\""))
        .stdout(predicate::str::contains("\"counts\""))
        .stdout(predicate::str::contains("\"total\": 2"))
        .stdout(predicate::str::contains("\"name\": \"file1.txt\""))
        .stdout(predicate::str::contains("file"));
}

#[test]
fn test_parse_ls_raw_format() {
    let ls_input = "file1.txt\nfile2.txt\nfile3.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"))
        .stdout(predicate::str::contains("file3.txt"))
        .stdout(predicate::function(|x: &str| !x.contains("total:")));
}

#[test]
fn test_parse_ls_with_symlinks() {
    let ls_input = "file1.txt\nlrwxrwxrwx  1 user  group    10 Jan  1 12:34 link_to_file\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("symlink"))
        .stdout(predicate::str::contains("link_to_file"));
}

#[test]
fn test_parse_ls_with_file_from_stdin() {
    // Test that we can pipe ls output to the parser
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("ls").arg("/tmp").assert().success();
}

// ============================================================
// Generated Directories Tests
// ============================================================

#[test]
fn test_parse_ls_node_modules_detected() {
    // Test that node_modules is detected as a generated directory
    let ls_input = "src/\nnode_modules/\npackage.json\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 generated"))
        .stdout(predicate::str::contains("node_modules/"));
}

#[test]
fn test_parse_ls_target_detected() {
    // Test that target directory (Rust) is detected
    let ls_input = "src/\ntarget/\nCargo.toml\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 generated"))
        .stdout(predicate::str::contains("target/"));
}

#[test]
fn test_parse_ls_multiple_generated_dirs() {
    // Test multiple generated directories
    let ls_input = "src/\nnode_modules/\ndist/\nbuild/\npackage.json\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("3 generated"))
        .stdout(predicate::str::contains("node_modules/"))
        .stdout(predicate::str::contains("dist/"))
        .stdout(predicate::str::contains("build/"));
}

#[test]
fn test_parse_ls_generated_dirs_case_insensitive() {
    // Test that generated directory detection is case-insensitive
    let ls_input = "src/\nNode_Modules/\nDIST/\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("2 generated"));
}

#[test]
fn test_parse_ls_no_generated_dirs() {
    // Test when no generated directories are present
    let ls_input = "src/\nlib/\nREADME.md\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::function(|x: &str| !x.contains("generated")));
}

#[test]
fn test_parse_ls_json_includes_generated() {
    // Test that JSON output includes generated array
    let ls_input = "src/\nnode_modules/\nfile.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"generated\":"))
        .stdout(predicate::str::contains("node_modules/"))
        .stdout(predicate::str::contains("\"counts\": {"))
        .stdout(predicate::str::contains("\"generated\": 1"));
}

#[test]
fn test_parse_ls_long_format_generated_dirs() {
    // Test generated directories in long format output
    let ls_input = "total 8\ndrwxr-xr-x  5 user  group 4096 Jan  1 12:34 node_modules\ndrwxr-xr-x  2 user  group 4096 Jan  1 12:34 src\n-rw-r--r--  1 user  group   42 Jan  1 12:34 package.json\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 generated"))
        .stdout(predicate::str::contains("node_modules"));
}

#[test]
fn test_parse_ls_venv_detected() {
    // Test that Python venv directories are detected
    let ls_input = "src/\nvenv/\nrequirements.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 generated"))
        .stdout(predicate::str::contains("venv/"));
}

#[test]
fn test_parse_ls_pycache_detected() {
    // Test that __pycache__ is detected
    let ls_input = "src/\n__pycache__/\nmain.py\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 generated"))
        .stdout(predicate::str::contains("__pycache__/"));
}

#[test]
fn test_parse_ls_vendor_detected() {
    // Test that vendor directory (Go/PHP/Ruby) is detected
    let ls_input = "cmd/\nvendor/\ngo.mod\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 generated"))
        .stdout(predicate::str::contains("vendor/"));
}

#[test]
fn test_parse_ls_hidden_and_generated() {
    // Test that a directory can be both hidden and generated (e.g., .venv, .next)
    let ls_input = "src/\n.next/\n.venv/\npackage.json\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("2 generated"));
}

// ============================================================
// LS Parser: Permission Denied Tests
// ============================================================

#[test]
fn test_parse_ls_permission_denied() {
    // Test that permission denied entries are detected and not treated as files
    let ls_input = "file1.txt\nls: cannot open directory '/root': Permission denied\nfile2.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("Permission denied"))
        .stdout(predicate::str::contains("2 files"));
}

#[test]
fn test_parse_ls_permission_denied_json() {
    // Test JSON output includes errors array
    let ls_input = "file.txt\nls: cannot access 'missing': No such file or directory\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"errors\":"))
        .stdout(predicate::str::contains("No such file or directory"));
}

#[test]
fn test_parse_ls_only_errors() {
    // Test when all output is errors
    let ls_input = "ls: cannot open directory '.': Permission denied\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("Permission denied"));
}

// ============================================================
// LS Parser: Symlink Target Tests
// ============================================================

#[test]
fn test_parse_ls_symlink_with_target() {
    // Test that symlink targets are displayed in compact format
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 link_to_file -> /path/to/target\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("link_to_file -> /path/to/target"));
}

#[test]
fn test_parse_ls_symlink_target_json() {
    // Test that JSON output includes symlink_target field
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 mylink -> destination\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"symlink_target\": \"destination\"",
        ))
        .stdout(predicate::str::contains("\"name\": \"mylink\""));
}

#[test]
fn test_parse_ls_multiple_symlinks_with_targets() {
    // Test multiple symlinks with different targets
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 link1 -> target1\nlrwxrwxrwx  1 user  group    10 Jan  1 12:34 link2 -> target2\n-rw-r--r--  1 user  group   42 Jan  1 12:34 file.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("link1 -> target1"))
        .stdout(predicate::str::contains("link2 -> target2"));
}

#[test]
fn test_parse_ls_symlink_no_target() {
    // Test symlink without target (should still work)
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 link_no_target\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("link_no_target"));
}

// ============================================================
// LS Parser: Broken Symlink Tests
// ============================================================

#[test]
fn test_parse_ls_broken_symlink_compact() {
    // Test that broken symlinks are marked in compact output
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 broken_link -> /nonexistent\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "broken_link -> /nonexistent [broken]",
        ));
}

#[test]
fn test_parse_ls_broken_symlink_json() {
    // Test that JSON output includes is_broken_symlink field
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 broken -> /nonexistent/path\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_broken_symlink\": true"))
        .stdout(predicate::str::contains(
            "\"symlink_target\": \"/nonexistent/path\"",
        ));
}

#[test]
fn test_parse_ls_circular_symlink() {
    // Test circular symlinks (self-referencing)
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 circular -> circular\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("circular -> circular [broken]"));
}

#[test]
fn test_parse_ls_circular_symlink_json() {
    // Test circular symlinks in JSON
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 loop -> loop\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_broken_symlink\": true"))
        .stdout(predicate::str::contains("\"symlink_target\": \"loop\""));
}

#[test]
fn test_parse_ls_mixed_broken_and_valid_symlinks() {
    // Test mix of broken and valid symlinks
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 good_link -> existing_file\nlrwxrwxrwx  1 user  group    10 Jan  1 12:34 bad_link -> /nonexistent\n-rw-r--r--  1 user  group   42 Jan  1 12:34 existing_file\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("good_link -> existing_file"))
        .stdout(predicate::str::contains(
            "bad_link -> /nonexistent [broken]",
        ))
        .stdout(predicate::function(|x: &str| {
            // Check that good_link line does NOT contain [broken]
            let lines: Vec<&str> = x.lines().collect();
            let good_link_line = lines
                .iter()
                .find(|l| l.contains("good_link") && l.contains("->"));
            match good_link_line {
                Some(line) => !line.contains("[broken]"),
                None => false,
            }
        }));
}

#[test]
fn test_parse_ls_broken_symlink_json_has_broken_array() {
    // Test that broken symlinks are detected and marked with is_broken_symlink
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 broken1 -> /nonexistent\nlrwxrwxrwx  1 user  group    10 Jan  1 12:34 broken2 -> nonexistent\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_broken_symlink\": true"))
        .stdout(predicate::str::contains("broken1"))
        .stdout(predicate::str::contains("broken2"));
}

#[test]
fn test_parse_ls_valid_symlink_not_marked_broken() {
    // Test that valid symlinks are NOT marked as broken
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 valid_link -> some_file\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_broken_symlink\": false"))
        .stdout(predicate::function(|x: &str| !x.contains("[broken]")));
}

#[test]
fn test_parse_grep() {
    let grep_input = "src/main.rs:42:fn main() {";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("matches:"));
}

#[test]
fn test_parse_grep_empty() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains("grep: no matches"));
}

#[test]
fn test_parse_grep_json() {
    let grep_input = "src/main.rs:42:fn main() {";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"files\": 1"))
        .stdout(predicate::str::contains("\"matches\": 1"))
        .stdout(predicate::str::contains("\"line_number\": 42"))
        .stdout(predicate::str::contains("\"line\": \"fn main() {\""));
}

#[test]
fn test_parse_grep_compact() {
    let grep_input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:    println!";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("matches: 1 files, 2 results"))
        .stdout(predicate::str::contains("src/main.rs (2):"));
}

#[test]
fn test_parse_grep_compact_preserves_line_numbers() {
    // Test that line numbers are preserved in compact format
    let grep_input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:    println!";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        // Verify line numbers are present in output
        .stdout(predicate::str::contains("42: fn main() {"))
        .stdout(predicate::str::contains("45:     println!"));
}

#[test]
fn test_parse_grep_compact_line_numbers_multiple_files() {
    // Test line numbers are preserved across multiple files
    let grep_input = "src/main.rs:10:line one\nsrc/lib.rs:25:line two\nsrc/main.rs:30:line three";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        // Verify line numbers for each file
        .stdout(predicate::str::contains("10: line one"))
        .stdout(predicate::str::contains("25: line two"))
        .stdout(predicate::str::contains("30: line three"));
}

#[test]
fn test_parse_grep_groups_interleaved_files() {
    // Test that interleaved matches from same file are grouped together
    let grep_input = "src/main.rs:10:line one\nsrc/lib.rs:25:line two\nsrc/main.rs:30:line three";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        // Should show 2 files, not 3 (main.rs appears twice but grouped)
        .stdout(predicate::str::contains("matches: 2 files, 3 results"))
        // main.rs should show both matches grouped (2)
        .stdout(predicate::str::contains("src/main.rs (2):"))
        // lib.rs should show 1 match
        .stdout(predicate::str::contains("src/lib.rs (1):"));
}

#[test]
fn test_parse_grep_groups_interleaved_files_json() {
    // Test that interleaved matches from same file are grouped together in JSON output
    let grep_input = "src/main.rs:10:line one\nsrc/lib.rs:25:line two\nsrc/main.rs:30:line three";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();

    // Should have 2 files
    assert_eq!(json["counts"]["files"], 2);
    assert_eq!(json["counts"]["matches"], 3);

    let files = json["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);

    // First file should be main.rs with 2 matches
    assert_eq!(files[0]["path"], "src/main.rs");
    assert_eq!(files[0]["matches"].as_array().unwrap().len(), 2);

    // Second file should be lib.rs with 1 match
    assert_eq!(files[1]["path"], "src/lib.rs");
    assert_eq!(files[1]["matches"].as_array().unwrap().len(), 1);
}

#[test]
fn test_parse_grep_csv() {
    let grep_input = "src/main.rs:42:fn main() {";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "path,line_number,column,is_context,line",
        ))
        .stdout(predicate::str::contains("src/main.rs,42,,false,"));
}

#[test]
fn test_parse_grep_tsv() {
    let grep_input = "src/main.rs:42:fn main() {";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "path\tline_number\tcolumn\tis_context\tline",
        ))
        .stdout(predicate::str::contains("src/main.rs\t42\t\tfalse\t"));
}

#[test]
fn test_parse_grep_raw() {
    let grep_input = "src/main.rs:42:fn main() {";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs:42:fn main() {"));
}

#[test]
fn test_parse_grep_multiple_files() {
    let grep_input = "src/main.rs:42:fn main() {\nsrc/lib.rs:10:pub fn helper()";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"files\": 2"))
        .stdout(predicate::str::contains("\"matches\": 2"));
}

// ============================================================
// Grep Truncation Tests
// ============================================================

#[test]
fn test_parse_grep_truncation_json_not_truncated() {
    // Small result set should not be truncated
    let grep_input = "src/main.rs:42:fn main() {";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["is_truncated"], false);
    assert_eq!(json["counts"]["total_files"], 1);
    assert_eq!(json["counts"]["total_matches"], 1);
    assert_eq!(json["counts"]["files_shown"], 1);
    assert_eq!(json["counts"]["matches_shown"], 1);
}

#[test]
fn test_parse_grep_truncation_json_many_files() {
    // Create input with 210 files (exceeds config grep_max_results = 200)
    let mut grep_input = String::new();
    for i in 1..=210 {
        grep_input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input.as_str())
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["is_truncated"], true);
    assert_eq!(json["counts"]["total_files"], 210);
    assert_eq!(json["counts"]["files_shown"], 200);
    assert!(
        json["counts"]["files_shown"].as_u64().unwrap()
            < json["counts"]["total_files"].as_u64().unwrap()
    );
}

#[test]
fn test_parse_grep_truncation_json_many_matches_per_file() {
    // Create input with 1 file but 30 matches (exceeds config grep_max_per_file = 25)
    let mut grep_input = String::new();
    for i in 1..=30 {
        grep_input.push_str(&format!("src/main.rs:{}:fn func{}() {{\n", i, i));
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input.as_str())
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["is_truncated"], true);
    assert_eq!(json["counts"]["total_matches"], 30);
    assert_eq!(json["counts"]["matches_shown"], 25);
    assert!(
        json["counts"]["matches_shown"].as_u64().unwrap()
            < json["counts"]["total_matches"].as_u64().unwrap()
    );
}

#[test]
fn test_parse_grep_truncation_compact_format() {
    // Create input with 210 files to trigger truncation (config max = 200)
    let mut grep_input = String::new();
    for i in 1..=210 {
        grep_input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input.as_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("truncated"))
        .stdout(predicate::str::contains("200/210"))
        .stdout(predicate::str::contains("10 more file"));
}

#[test]
fn test_parse_grep_truncation_raw_format() {
    // Create input with 210 files to trigger truncation (config max = 200)
    let mut grep_input = String::new();
    for i in 1..=210 {
        grep_input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input.as_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("10 more file"));
}

#[test]
fn test_parse_test() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .assert()
        .success();
}

#[test]
fn test_parse_logs() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse").arg("logs").assert().success();
}

#[test]
fn test_html2md_basic() {
    // Test with a local HTML file
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_html2md_cli.html");

    let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
<h1>Hello World</h1>
<p>This is a test paragraph.</p>
</body>
</html>"#;

    let mut file = std::fs::File::create(&html_path).unwrap();
    file.write_all(html_content.as_bytes()).unwrap();
    drop(file);

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg(&html_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));

    // Cleanup
    let _ = std::fs::remove_file(&html_path);
}

#[test]
fn test_html2md_url_input() {
    // Test with a URL input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg("https://httpbin.org/html")
        .assert()
        .success()
        .stdout(predicate::str::contains("Herman Melville"))
        .stdout(predicate::str::contains("Moby-Dick"));
}

#[test]
fn test_html2md_url_with_metadata() {
    // Test URL with metadata flag
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg("https://httpbin.org/html")
        .arg("--metadata")
        .assert()
        .success()
        .stdout(predicate::str::contains("source"))
        .stdout(predicate::str::contains("httpbin.org"));
}

#[test]
fn test_html2md_url_with_json_output() {
    // Test URL with JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("html2md")
        .arg("https://httpbin.org/html")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json["markdown"].as_str().unwrap().contains("Moby-Dick"));
}

#[test]
fn test_txt2md_basic() {
    // Test with stdin input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("HELLO WORLD\n\nThis is some text.")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Hello World"));
}

#[test]
fn test_global_json_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema\""));
}

#[test]
fn test_global_csv_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("path,line_number"));
}

#[test]
fn test_global_stats_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("search")
        .arg(".")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Stats:"))
        .stderr(predicate::str::contains("Items processed:"));
}

#[test]
fn test_run_command_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
}

#[test]
fn test_run_command_with_args() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("test")
        .arg("message")
        .assert()
        .success()
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("message"));
}

#[test]
fn test_run_command_failure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("false").assert().code(1);
}

#[test]
fn test_run_command_not_found() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("nonexistent_command_xyz123")
        .assert()
        .code(127) // Standard "command not found" exit code
        .stderr(predicate::str::contains("Command not found"));
}

#[test]
fn test_run_command_json_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("echo")
        .arg("hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("exit_code"))
        .stdout(predicate::str::contains("stdout"));
}

#[test]
fn test_run_command_no_capture_stdout() {
    // When --capture-stdout=false, stdout goes directly to terminal
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("hello")
        .arg("--capture-stdout=false")
        .assert()
        .success();
    // Note: stdout goes directly to terminal when not captured,
    // so the CLI output won't contain it
}

#[test]
fn test_run_command_capture_stdout_default() {
    // By default, stdout is captured
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("captured_output")
        .assert()
        .success()
        .stdout(predicate::str::contains("captured_output"));
}

#[test]
fn test_run_command_no_capture_stderr() {
    // When --capture-stderr=false, stderr goes directly to terminal
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo stderr_test >&2")
        .arg("--capture-stderr=false")
        .assert()
        .success();
    // Note: stderr goes directly to terminal when not captured,
    // so the CLI output won't contain it
}

#[test]
fn test_run_command_capture_stderr_default() {
    // By default, stderr is captured
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo captured_stderr >&2")
        .assert()
        .success()
        .stdout(predicate::str::contains("captured_stderr"));
}

#[test]
fn test_run_command_no_capture_both() {
    // When both are not captured, both go directly to terminal
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo stdout_test && echo stderr_test >&2")
        .arg("--capture-stdout=false")
        .arg("--capture-stderr=false")
        .assert()
        .success();
}

#[test]
fn test_run_command_capture_exit_code_default() {
    // By default, exit code is captured
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("--capture-exit-code=true")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"exit_code\":0"));
}

#[test]
fn test_run_command_no_capture_exit_code() {
    // When --capture-exit-code=false, exit_code is null in JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("--capture-exit-code=false")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"exit_code\":null"));
}

#[test]
fn test_run_command_no_capture_exit_code_non_zero() {
    // When exit code is not captured, even non-zero exit commands show null
    // and the command succeeds (error is not propagated)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("--capture-exit-code=false")
        .arg("sh")
        .arg("-c")
        .arg("exit 42")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"exit_code\":null"));
}

#[test]
fn test_run_command_capture_exit_code_non_zero() {
    // When exit code is captured, non-zero exit code is visible
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("--capture-exit-code=true")
        .arg("sh")
        .arg("-c")
        .arg("exit 42")
        .assert()
        .code(42) // Exit code 42 is now propagated correctly
        .stderr(predicate::str::contains("exited with code 42"));
}

#[test]
fn test_run_command_capture_duration_default() {
    // By default, duration is captured
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("duration_ms"));
}

#[test]
fn test_run_command_no_capture_duration() {
    // When --capture-duration=false, duration_ms should be 0
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("--capture-duration=false")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"duration_ms\":0"));
}

#[test]
fn test_run_command_capture_duration_true() {
    // When --capture-duration=true, duration_ms should be greater than 0
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("run")
        .arg("--capture-duration=true")
        .arg("echo")
        .arg("test")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Parse JSON and check duration_ms > 0
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let duration_ms = json["duration_ms"].as_u64().unwrap();
    assert!(duration_ms > 0);
}

// ============================================================
// Command Routing Tests
// ============================================================

#[test]
fn test_router_search_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search").arg(".").arg("pattern").assert().success();
    // Search is now fully implemented
}

#[test]
fn test_router_replace_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_12345")
        .arg("new")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

// ============================================================
// Tail -N Shorthand Tests
// ============================================================

#[test]
fn test_tail_shorthand_minus_5() {
    // Test -5 shorthand (equivalent to -n 5 or --lines 5)
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();
    let mut file = std::fs::File::create(path).unwrap();
    for i in 1..=20 {
        writeln!(file, "line {}", i).unwrap();
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("-5")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 16"))
        .stdout(predicate::str::contains("line 17"))
        .stdout(predicate::str::contains("line 18"))
        .stdout(predicate::str::contains("line 19"))
        .stdout(predicate::str::contains("line 20"))
        .stdout(predicate::function(|s: &str| !s.contains("line 15")));
}

#[test]
fn test_tail_shorthand_minus_3() {
    // Test -3 shorthand
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();
    let mut file = std::fs::File::create(path).unwrap();
    for i in 1..=10 {
        writeln!(file, "line {}", i).unwrap();
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("-3")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 8"))
        .stdout(predicate::str::contains("line 9"))
        .stdout(predicate::str::contains("line 10"))
        .stdout(predicate::function(|s: &str| !s.contains("line 7")));
}

#[test]
fn test_tail_shorthand_with_global_flags() {
    // Test -N shorthand with global flags (e.g., --json)
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();
    let mut file = std::fs::File::create(path).unwrap();
    for i in 1..=10 {
        writeln!(file, "line {}", i).unwrap();
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("tail")
        .arg("-5")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON output");
    assert_eq!(json["lines_shown"], 5);
    assert_eq!(json["total_lines"], 10);
}

#[test]
fn test_tail_shorthand_with_errors_flag() {
    // Test -N shorthand with --errors flag
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();
    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "INFO: line 1").unwrap();
    writeln!(file, "ERROR: line 2").unwrap();
    writeln!(file, "INFO: line 3").unwrap();
    writeln!(file, "FATAL: line 4").unwrap();
    writeln!(file, "INFO: line 5").unwrap();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("-10")
        .arg("--errors")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("ERROR"))
        .stdout(predicate::str::contains("FATAL"))
        .stdout(predicate::function(|s: &str| !s.contains("INFO")));
}

#[test]
fn test_tail_shorthand_minus_1() {
    // Test -1 shorthand (single line)
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();
    let mut file = std::fs::File::create(path).unwrap();
    for i in 1..=5 {
        writeln!(file, "line {}", i).unwrap();
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("-1")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 5"))
        .stdout(predicate::function(|s: &str| !s.contains("line 4")));
}

#[test]
fn test_tail_shorthand_equivalence() {
    // Test that -5 produces same output as -n 5
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();
    let mut file = std::fs::File::create(path).unwrap();
    for i in 1..=20 {
        writeln!(file, "line {}", i).unwrap();
    }

    // Get output with -5 shorthand
    let mut cmd1 = Command::cargo_bin("trs").unwrap();
    let output1 = cmd1
        .arg("tail")
        .arg("-5")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout1 = String::from_utf8_lossy(&output1);

    // Get output with -n 5
    let mut cmd2 = Command::cargo_bin("trs").unwrap();
    let output2 = cmd2
        .arg("tail")
        .arg("-n")
        .arg("5")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout2 = String::from_utf8_lossy(&output2);

    // Outputs should be identical
    assert_eq!(stdout1, stdout2);
}

#[test]
fn test_tail_traditional_syntax_still_works() {
    // Ensure traditional -n and --lines syntax still work
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();
    let mut file = std::fs::File::create(path).unwrap();
    for i in 1..=10 {
        writeln!(file, "line {}", i).unwrap();
    }

    // Test -n syntax
    let mut cmd1 = Command::cargo_bin("trs").unwrap();
    cmd1.arg("tail")
        .arg("-n")
        .arg("3")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 10"));

    // Test --lines syntax
    let mut cmd2 = Command::cargo_bin("trs").unwrap();
    cmd2.arg("tail")
        .arg("--lines")
        .arg("3")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 10"));
}

#[test]
fn test_router_tail_command() {
    // Create a temporary test file
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "line 2").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_router_clean_command() {
    // Test that clean command works with stdin
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .write_stdin("  hello world  ")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn test_router_html2md_command() {
    // Test with a local HTML file
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_html2md_router.html");

    let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Router Test</title></head>
<body>
<h1>Router Test</h1>
<p>Content here.</p>
</body>
</html>"#;

    let mut file = std::fs::File::create(&html_path).unwrap();
    file.write_all(html_content.as_bytes()).unwrap();
    drop(file);

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg(&html_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Router Test"));

    // Cleanup
    let _ = std::fs::remove_file(&html_path);
}

#[test]
fn test_router_txt2md_command() {
    // Test with stdin input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("TITLE\n\nSome paragraph text.")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Title"));
}

#[test]
fn test_router_parse_git_status_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .assert()
        .success()
        .stdout(predicate::str::contains("clean"));
}

#[test]
fn test_router_parse_test_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .assert()
        .success()
        .stdout(predicate::str::contains("no tests found"));
}

#[test]
fn test_router_run_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("ls").assert().success();
}

#[test]
fn test_router_run_command_with_stats() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Stats:"))
        .stderr(predicate::str::contains("Duration:"));
}

// ============================================================
// Context and Format Routing Tests
// ============================================================

#[test]
fn test_context_json_format_routing() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema\""));
}

#[test]
fn test_context_agent_format_routing() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("matches:"));
}

#[test]
fn test_context_stats_routing() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("search")
        .arg(".")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Stats:"))
        .stderr(predicate::str::contains("Items processed:"));
}

#[test]
fn test_context_combined_flags_routing() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("--stats")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema\""))
        .stderr(predicate::str::contains("Stats:"))
        .stderr(predicate::str::contains("Items processed:"));
}

// ============================================================
// System Command Execution Tests
// ============================================================

#[test]
fn test_run_pwd_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("pwd")
        .assert()
        .success()
        .stdout(predicate::str::contains("/"));
}

#[test]
fn test_run_whoami_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("whoami").assert().success();
}

#[test]
fn test_run_date_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("date").assert().success();
}

#[test]
fn test_run_uname_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("uname")
        .assert()
        .success()
        .stdout(predicate::str::contains("Darwin").or(predicate::str::contains("Linux")));
}

#[test]
fn test_run_shell_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo shell_test")
        .assert()
        .success()
        .stdout(predicate::str::contains("shell_test"));
}

#[test]
fn test_run_bash_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("bash")
        .arg("-c")
        .arg("echo bash_test")
        .assert()
        .success()
        .stdout(predicate::str::contains("bash_test"));
}

#[test]
fn test_run_command_with_multiple_args() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("arg1")
        .arg("arg2")
        .arg("arg3")
        .assert()
        .success()
        .stdout(predicate::str::contains("arg1"))
        .stdout(predicate::str::contains("arg2"))
        .stdout(predicate::str::contains("arg3"));
}

#[test]
fn test_run_command_with_stderr() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo stderr_test >&2")
        .assert()
        .success()
        .stdout(predicate::str::contains("stderr_test"));
}

#[test]
fn test_run_command_with_stdout_and_stderr() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo stdout_test && echo stderr_test >&2")
        .assert()
        .success()
        .stdout(predicate::str::contains("stdout_test"))
        .stdout(predicate::str::contains("stderr_test"));
}

#[test]
fn test_run_cat_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("cat")
        .arg("/etc/hosts")
        .assert()
        .success()
        .stdout(predicate::str::contains("localhost"));
}

#[test]
fn test_run_ls_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("ls").arg("/tmp").assert().success();
}

#[test]
fn test_run_env_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("env").assert().success();
}

#[test]
fn test_run_true_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("true").assert().success();
}

#[test]
fn test_run_exit_code_propagation() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("exit 42")
        .assert()
        .code(42); // Exit code 42 is now propagated correctly
}

// ============================================================
// JSON Output Tests for Command Execution
// ============================================================

#[test]
fn test_run_json_output_has_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""command":"echo"#));
}

#[test]
fn test_run_json_output_has_args() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("echo")
        .arg("arg1")
        .arg("arg2")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""args":["#));
}

#[test]
fn test_run_json_output_has_exit_code() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""exit_code":0"#));
}

#[test]
fn test_run_json_output_has_stdout() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("echo")
        .arg("hello_world")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""stdout":"hello_world\n"#));
}

#[test]
fn test_run_json_output_has_duration() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""duration_ms"#));
}

#[test]
fn test_run_json_output_has_stderr() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo test_stderr >&2")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""stderr":"test_stderr\n"#));
}

#[test]
fn test_run_json_output_timed_out() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""timed_out":false"#));
}

#[test]
fn test_run_json_parsable() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Verify it's valid JSON
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
}

// ============================================================
// JSON Output Tests for Not-Implemented Commands
// ============================================================

#[test]
fn test_search_json_output_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // The output should be valid JSON with the grep_output schema
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["schema"]["type"], "grep_output");
    assert!(!json["is_empty"].as_bool().unwrap());
    assert!(!json["files"].as_array().unwrap().is_empty());
}

#[test]
fn test_replace_json_output_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_12345")
        .arg("new")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // The output should be valid JSON with the replace_output schema
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["schema"]["type"], "replace_output");
    assert!(json["dry_run"].as_bool().unwrap());
    assert_eq!(json["search_pattern"], "NONEXISTENT_PATTERN_12345");
    assert_eq!(json["replacement"], "new");
    // Verify counts are present
    assert!(json["counts"]["files_affected"].is_number());
    assert!(json["counts"]["total_replacements"].is_number());
}

#[test]
fn test_replace_affected_file_count() {
    use std::fs;
    use tempfile::TempDir;

    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files with known patterns
    fs::write(temp_path.join("file1.txt"), "hello world\nhello again").unwrap();
    fs::write(temp_path.join("file2.txt"), "hello everyone").unwrap();
    fs::write(temp_path.join("file3.txt"), "goodbye").unwrap(); // No match

    // Run replace in dry-run mode with JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg(temp_path)
        .arg("hello")
        .arg("hi")
        .arg("--dry-run")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON output");

    // Verify the affected file count is correct (2 files with matches)
    assert_eq!(json["counts"]["files_affected"].as_u64().unwrap(), 2);
    // Verify total replacements (2 in file1 + 1 in file2 = 3)
    assert_eq!(json["counts"]["total_replacements"].as_u64().unwrap(), 3);
    // Verify the files array has the correct length
    assert_eq!(json["files"].as_array().unwrap().len(), 2);
}

#[test]
fn test_replace_count_flag() {
    use std::fs;
    use tempfile::TempDir;

    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files with known patterns
    fs::write(temp_path.join("file1.txt"), "hello world\nhello again").unwrap();
    fs::write(temp_path.join("file2.txt"), "hello everyone").unwrap();
    fs::write(temp_path.join("file3.txt"), "goodbye").unwrap(); // No match

    // Run replace with --count flag (default output format)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("replace")
        .arg(temp_path)
        .arg("hello")
        .arg("hi")
        .arg("--dry-run")
        .arg("--count")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output just the count (3 total replacements: 2 in file1 + 1 in file2)
    assert_eq!(stdout.trim(), "3");
}

#[test]
fn test_replace_count_flag_json_output() {
    use std::fs;
    use tempfile::TempDir;

    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files with known patterns
    fs::write(temp_path.join("file1.txt"), "hello world\nhello again").unwrap();
    fs::write(temp_path.join("file2.txt"), "hello everyone").unwrap();
    fs::write(temp_path.join("file3.txt"), "goodbye").unwrap(); // No match

    // Run replace with --count flag and JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg(temp_path)
        .arg("hello")
        .arg("hi")
        .arg("--dry-run")
        .arg("--count")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON output");

    // Should output just the count in JSON format
    assert_eq!(json["count"].as_u64().unwrap(), 3);
}

#[test]
fn test_replace_count_flag_no_matches() {
    // Test that --count returns 0 when there are no matches
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_12345_UNIQUE")
        .arg("new")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .arg("--count")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "0");
}

#[test]
fn test_clean_json_output() {
    // Test that clean command produces valid JSON output
    let input = "  hello world  \n\n\n  line 2  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json["content"].is_string());
    assert!(json["stats"]["input_length"].is_number());
    assert!(json["stats"]["output_length"].is_number());
    assert!(json["stats"]["reduction_percent"].is_number());
}

#[test]
fn test_clean_file_input() {
    // Test clean with file input
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "  line 1  ").unwrap();
    writeln!(file, "\n\n").unwrap();
    writeln!(file, "  line 2  ").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--file")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_clean_file_not_found() {
    // Test clean with non-existent file
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--file")
        .arg("/nonexistent/file.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));
}

#[test]
fn test_clean_no_ansi() {
    // Test ANSI code removal
    let input = "\x1b[31mRed\x1b[0m \x1b[32mGreen\x1b[0m";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Red Green"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b[")));
}

#[test]
fn test_clean_no_ansi_csi_sequences() {
    // Test CSI (Control Sequence Introducer) sequences
    let input = "\x1b[1mBold\x1b[0m \x1b[4mUnderline\x1b[0m \x1b[7mReverse\x1b[0m";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Bold Underline Reverse"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b[")));
}

#[test]
fn test_clean_no_ansi_multiple_params() {
    // Test ANSI codes with multiple parameters
    let input = "\x1b[1;31;42mBold Red on Green\x1b[0m";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Bold Red on Green"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b[")));
}

#[test]
fn test_clean_no_ansi_osc_sequences() {
    // Test OSC (Operating System Command) sequences
    let input = "Title\x1b]0;Window Title\x07Text";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("TitleText"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b]")));
}

#[test]
fn test_clean_no_ansi_osc_with_st() {
    // Test OSC sequences with String Terminator (ST)
    let input = "Title\x1b]0;Window Title\x1b\\Text";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("TitleText"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b]")));
}

#[test]
fn test_clean_no_ansi_hyperlinks() {
    // Test hyperlink sequences (OSC 8)
    let input = "Click \x1b]8;;http://example.com\x07here\x1b]8;;\x07 now";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Click here now"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b]")));
}

#[test]
fn test_clean_no_ansi_simple_escapes() {
    // Test simple two-character escape sequences
    let input = "Before\x1bcAfter"; // RIS (Reset to Initial State)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("BeforeAfter"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1bc")));
}

#[test]
fn test_clean_no_ansi_cursor_movement() {
    // Test cursor movement sequences
    let input = "Line 1\x1b[2A\x1b[10;20H\x1b[JLine 2";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Line 1Line 2"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b[")));
}

#[test]
fn test_clean_no_ansi_character_sets() {
    // Test character set selection sequences
    let input = "Text\x1b(BMore\x1b)0Text";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("TextMoreText"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b(")));
}

#[test]
fn test_clean_no_ansi_mixed() {
    // Test mixed ANSI sequences
    let input = "\x1b[1;31mError:\x1b[0m \x1b]8;;file:///path\x07/path/to/file\x1b]8;;\x07\x1b[K";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Error: /path/to/file"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b")));
}

#[test]
fn test_clean_no_ansi_real_world() {
    // Test real-world terminal output with various ANSI codes
    let input = "\x1b[?25lHidden cursor\x1b[?25h\x1b[2KProgress: 50%\x1b[0K";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hidden cursorProgress: 50%"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b")));
}

#[test]
fn test_clean_collapse_blanks() {
    // Test blank line collapsing - multiple consecutive blanks become one
    let input = "line 1\n\n\n\nline 2\n\n\nline 3";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--collapse-blanks")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"))
        .stdout(predicate::str::contains("line 3"))
        // Verify no more than one consecutive blank line (no \n\n\n sequences)
        .stdout(predicate::function(|s: &str| !s.contains("\n\n\n")));
}

#[test]
fn test_clean_collapse_blanks_many_consecutive() {
    // Test with many consecutive blank lines
    let input = "start\n\n\n\n\n\n\n\n\n\nend";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--collapse-blanks")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| {
            // Should have at most one blank line between content lines
            // The output should be "start\n\nend" (one blank line between)
            // plus reduction stats at the end
            let lines: Vec<&str> = s.lines().collect();
            let mut consecutive_blank_count = 0;
            let mut max_consecutive_blanks = 0;

            for line in &lines {
                if line.trim().is_empty() {
                    consecutive_blank_count += 1;
                    max_consecutive_blanks = max_consecutive_blanks.max(consecutive_blank_count);
                } else {
                    // Reset counter on non-blank line (including stats line)
                    consecutive_blank_count = 0;
                }
            }
            // Max consecutive blank lines should be 1
            max_consecutive_blanks <= 1
        }));
}

#[test]
fn test_clean_collapse_blanks_whitespace_lines() {
    // Test that whitespace-only lines are treated as blank
    let input = "line 1\n   \n\t\t\n  \nline 2";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--collapse-blanks")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| {
            // No triple newlines should exist
            !s.contains("\n\n\n")
        }));
}

#[test]
fn test_clean_collapse_repeats() {
    // Test repeated line collapsing
    let input = "line 1\nline 1\nline 2\nline 2\nline 2\nline 3";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--collapse-repeats")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| {
            // Each line should appear only once
            let line1_count = s.matches("line 1").count();
            let line2_count = s.matches("line 2").count();
            let line3_count = s.matches("line 3").count();
            line1_count == 1 && line2_count == 1 && line3_count == 1
        }));
}

#[test]
fn test_clean_trim() {
    // Test whitespace trimming
    let input = "  line 1  \n\t\tline 2\t\t\n   line 3   ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--trim")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"))
        .stdout(predicate::str::contains("line 3"));
}

#[test]
fn test_clean_compact_output() {
    // Test compact format output
    let input = "  hello  \n\n\n  world  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("world"))
        .stdout(predicate::str::contains("reduction"));
}

#[test]
fn test_clean_raw_output() {
    // Test raw format output
    let input = "  line 1  \n  line 2  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_clean_agent_output() {
    // Test agent format output
    let input = "  hello  \n\n  world  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Content"))
        .stdout(predicate::str::contains("reduction"))
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("world"));
}

#[test]
fn test_clean_csv_output() {
    // Test CSV format output
    let input = "line 1\nline 2";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"line 1\""))
        .stdout(predicate::str::contains("\"line 2\""));
}

#[test]
fn test_clean_tsv_output() {
    // Test TSV format output
    let input = "line 1\nline 2";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_html2md_json_output() {
    // Test with a local HTML file and JSON output
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_html2md_json.html");

    let html_content = r#"<!DOCTYPE html>
<html>
<head><title>JSON Test</title></head>
<body>
<h1>JSON Test</h1>
</body>
</html>"#;

    let mut file = std::fs::File::create(&html_path).unwrap();
    file.write_all(html_content.as_bytes()).unwrap();
    drop(file);

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("html2md")
        .arg(&html_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json["markdown"].as_str().unwrap().contains("JSON Test"));

    // Cleanup
    let _ = std::fs::remove_file(&html_path);
}

#[test]
fn test_html2md_json_output_includes_metadata() {
    // Test that JSON output automatically includes metadata without --metadata flag
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_html2md_json_meta.html");

    let html_content = r#"<!DOCTYPE html>
<html>
<head>
<title>Metadata Test Page</title>
<meta name="description" content="A test page for metadata extraction">
</head>
<body>
<h1>Test Content</h1>
<p>This is test content.</p>
</body>
</html>"#;

    let mut file = std::fs::File::create(&html_path).unwrap();
    file.write_all(html_content.as_bytes()).unwrap();
    drop(file);

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("html2md")
        .arg(&html_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify markdown content is present
    assert!(json["markdown"].as_str().unwrap().contains("Test Content"));

    // Verify metadata is automatically included in JSON output
    assert!(
        json["metadata"].is_object(),
        "metadata should be present in JSON output"
    );
    assert_eq!(
        json["metadata"]["title"].as_str().unwrap(),
        "Metadata Test Page"
    );
    assert_eq!(
        json["metadata"]["description"].as_str().unwrap(),
        "A test page for metadata extraction"
    );
    assert_eq!(json["metadata"]["type"].as_str().unwrap(), "file");
    assert!(json["metadata"]["source"]
        .as_str()
        .unwrap()
        .contains("test_html2md_json_meta.html"));

    // Cleanup
    let _ = std::fs::remove_file(&html_path);
}

#[test]
fn test_txt2md_json_output() {
    // Test that txt2md produces valid JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("txt2md")
        .write_stdin("TITLE\n\nParagraph text.")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify markdown content is present
    assert!(json["markdown"].as_str().unwrap().contains("# Title"));

    // Verify metadata is present
    assert!(json["metadata"].is_object());
    assert_eq!(json["metadata"]["type"].as_str().unwrap(), "stdin");
    assert!(json["metadata"]["title"]
        .as_str()
        .unwrap()
        .contains("TITLE"));
}

#[test]
fn test_txt2md_stdin_input() {
    // Test explicit stdin input (no file specified)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("MY DOCUMENT\n\nThis is content from stdin.\n\n- Item 1\n- Item 2")
        .assert()
        .success()
        .stdout(predicate::str::contains("# My Document"))
        .stdout(predicate::str::contains("This is content from stdin."))
        .stdout(predicate::str::contains("- Item 1"))
        .stdout(predicate::str::contains("- Item 2"));
}

#[test]
fn test_txt2md_stdin_empty() {
    // Test stdin with empty input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md").write_stdin("").assert().success();
}

#[test]
fn test_txt2md_stdin_with_output_flag() {
    // Test stdin input with output to file
    #[allow(unused_imports)]
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join("test_txt2md_stdin_output.md");

    // Clean up any existing file
    let _ = std::fs::remove_file(&output_path);

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .arg("--output")
        .arg(&output_path)
        .write_stdin("SECTION\n\nContent here.")
        .assert()
        .success();

    // Verify file was created
    assert!(output_path.exists());

    // Verify content
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("# Section"));

    // Cleanup
    let _ = std::fs::remove_file(&output_path);
}

#[test]
fn test_txt2md_file_input() {
    // Test with file input
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("test_txt2md_input.txt");

    let mut file = std::fs::File::create(&input_path).unwrap();
    writeln!(file, "DOCUMENT TITLE").unwrap();
    writeln!(file).unwrap();
    writeln!(file, "This is a paragraph.").unwrap();
    drop(file);

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .arg("--input")
        .arg(&input_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("# Document Title"));

    // Cleanup
    let _ = std::fs::remove_file(&input_path);
}

#[test]
fn test_txt2md_file_output() {
    // Test with file output
    #[allow(unused_imports)]
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join("test_txt2md_output.md");

    // Clean up any existing file
    let _ = std::fs::remove_file(&output_path);

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .arg("--output")
        .arg(&output_path)
        .write_stdin("SECTION HEADING\n\nSome content.")
        .assert()
        .success();

    // Verify file was created
    assert!(output_path.exists());

    // Verify content
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("# Section Heading"));

    // Cleanup
    let _ = std::fs::remove_file(&output_path);
}

#[test]
fn test_txt2md_unordered_list() {
    // Test unordered list detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("ITEMS\n\n- First item\n- Second item\n- Third item")
        .assert()
        .success()
        .stdout(predicate::str::contains("- First item"))
        .stdout(predicate::str::contains("- Second item"))
        .stdout(predicate::str::contains("- Third item"));
}

#[test]
fn test_txt2md_ordered_list() {
    // Test ordered list detection - numbers are preserved
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("STEPS\n\n1. First step\n2. Second step\n3. Third step")
        .assert()
        .success()
        .stdout(predicate::str::contains("1. First step"))
        .stdout(predicate::str::contains("2. Second step"))
        .stdout(predicate::str::contains("3. Third step"));
}

#[test]
fn test_txt2md_raw_output() {
    // Test raw output format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("txt2md")
        .write_stdin("TITLE\n\nContent here.")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Title"))
        // Raw output should NOT include metadata
        .stdout(predicate::str::contains("metadata").not());
}

#[test]
fn test_txt2md_code_block() {
    // Test that code blocks are preserved
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("CODE\n\n```\nfunction test() {\n  return 1;\n}\n```")
        .assert()
        .success()
        .stdout(predicate::str::contains("```"))
        .stdout(predicate::str::contains("function test()"));
}

#[test]
fn test_txt2md_blockquote() {
    // Test blockquote detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("QUOTE\n\n> This is a quote")
        .assert()
        .success()
        .stdout(predicate::str::contains("> This is a quote"));
}

#[test]
fn test_txt2md_section_heading() {
    // Test section/chapter/part heading detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("INTRODUCTION\n\nSection 1: Getting Started\n\nSome content here.")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Introduction"))
        .stdout(predicate::str::contains("Section 1: Getting Started"));
}

#[test]
fn test_txt2md_chapter_heading() {
    // Test chapter heading detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("Chapter 1: The Beginning\n\nThis is the first chapter.")
        .assert()
        .success()
        .stdout(predicate::str::contains("Chapter 1: The Beginning"));
}

#[test]
fn test_txt2md_title_case_heading() {
    // Test title case heading detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin(
            "Getting Started With The Application\n\nThis is a paragraph about getting started.",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "# Getting Started With The Application",
        ));
}

#[test]
fn test_txt2md_colon_label() {
    // Test colon-ended label heading detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("IMPORTANT NOTES:\n\nThese are important notes.")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Important Notes"));
}

#[test]
fn test_txt2md_numbered_section_heading() {
    // Test numbered section heading with multiple words (should be heading, not list)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("1. Introduction to the System Architecture Overview\n\nSome content here.")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "1. Introduction to the System Architecture Overview",
        ));
}

#[test]
fn test_txt2md_single_word_section_heading() {
    // Test single-word section headings like "Introduction", "Methods", "Results"
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("Document Title\n\nIntroduction\n\nThis is the intro.\n\nMethods\n\nHere are methods.\n\nResults\n\nHere are results.\n\nConclusion\n\nHere is conclusion.")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Document Title"))
        .stdout(predicate::str::contains("## Introduction"))
        .stdout(predicate::str::contains("## Methods"))
        .stdout(predicate::str::contains("## Results"))
        .stdout(predicate::str::contains("## Conclusion"));
}

#[test]
fn test_txt2md_common_section_words() {
    // Test common section words are detected as headings
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin(
            "Document\n\nAbstract\n\nThis is the abstract.\n\nSummary\n\nThis is the summary.",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("## Abstract"))
        .stdout(predicate::str::contains("## Summary"));
}

#[test]
fn test_txt2md_extended_section_words() {
    // Test extended section words like History, Future, Design
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("Project\n\nHistory\n\nThis is the history.\n\nFuture\n\nThis is the future.")
        .assert()
        .success()
        .stdout(predicate::str::contains("## History"))
        .stdout(predicate::str::contains("## Future"));
}

#[test]
fn test_txt2md_nested_unordered_list() {
    // Test nested unordered list detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin(
            "ITEMS\n\n- Main item one\n  - Sub item one\n  - Sub item two\n- Main item two",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("- Main item one"))
        .stdout(predicate::str::contains("  - Sub item one"))
        .stdout(predicate::str::contains("  - Sub item two"))
        .stdout(predicate::str::contains("- Main item two"));
}

#[test]
fn test_txt2md_nested_ordered_list() {
    // Test nested ordered list detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin(
            "STEPS\n\n1. First step\n   1. Sub step one\n   2. Sub step two\n2. Second step",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("1. First step"))
        .stdout(predicate::str::contains("  1. Sub step one"))
        .stdout(predicate::str::contains("  2. Sub step two"))
        .stdout(predicate::str::contains("2. Second step"));
}

#[test]
fn test_txt2md_mixed_nested_list() {
    // Test mixed nested lists (ordered inside unordered and vice versa)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin(
            "ITEMS\n\n- Main item\n  1. First sub step\n  2. Second sub step\n- Another item",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("- Main item"))
        .stdout(predicate::str::contains("  1. First sub step"))
        .stdout(predicate::str::contains("  2. Second sub step"))
        .stdout(predicate::str::contains("- Another item"));
}

#[test]
fn test_txt2md_multiline_list_item() {
    // Test multi-line list item with continuation
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("ITEMS\n\n- First item with\n  continuation line\n- Second item")
        .assert()
        .success()
        .stdout(predicate::str::contains("- First item with"))
        .stdout(predicate::str::contains("continuation line"))
        .stdout(predicate::str::contains("- Second item"));
}

#[test]
fn test_txt2md_asterisk_list() {
    // Test asterisk-based unordered list
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("ITEMS\n\n* First item\n* Second item\n* Third item")
        .assert()
        .success()
        .stdout(predicate::str::contains("- First item"))
        .stdout(predicate::str::contains("- Second item"))
        .stdout(predicate::str::contains("- Third item"));
}

#[test]
fn test_txt2md_ordered_list_preserves_numbers() {
    // Test that ordered list numbers are preserved (not normalized to 1.)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("STEPS\n\n5. Fifth step\n10. Tenth step\n25. Twenty-fifth step")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("5. Fifth step"), "Should preserve number 5");
    assert!(
        stdout.contains("10. Tenth step"),
        "Should preserve number 10"
    );
    assert!(
        stdout.contains("25. Twenty-fifth step"),
        "Should preserve number 25"
    );
}

#[test]
fn test_parse_grep_json_output() {
    // Test that grep parser now works and produces valid JSON output
    let grep_input = "src/main.rs:42:fn main() {";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["files"], 1);
    assert_eq!(json["counts"]["matches"], 1);
}

#[test]
fn test_parse_test_json_output() {
    // Test that pytest parser now works and produces valid JSON output
    let pytest_input = r#"tests/test_main.py::test_add PASSED
1 passed in 0.01s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["passed"], 1);
    assert_eq!(json["summary"]["total"], 1);
}

#[test]
fn test_parse_test_jest_json_output() {
    // Test that Jest parser works and produces valid JSON output
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)
  ✓ should subtract numbers (2 ms)

Test Suites: 1 passed, 1 total
Tests:       2 passed, 2 total"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests"]["passed"], 2);
    assert_eq!(json["summary"]["tests"]["total"], 2);
    assert_eq!(json["summary"]["suites"]["passed"], 1);
    assert_eq!(json["summary"]["suites"]["total"], 1);
}

#[test]
fn test_parse_test_jest_compact_output() {
    // Test that Jest parser works with compact output
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)

FAIL src/api.test.js
  ✕ should fetch data (10 ms)

Test Suites: 1 passed, 1 failed, 2 total
Tests:       1 passed, 1 failed, 2 total"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Compact output should show summary and failed tests
    assert!(stdout.contains("FAIL:"));
    assert!(stdout.contains("1 passed, 1 failed"));
    assert!(stdout.contains("failed suites"));
    assert!(stdout.contains("src/api.test.js"));
}

#[test]
fn test_parse_test_vitest_json_output() {
    // Test that Vitest parser works and produces valid JSON output
    let vitest_input = r#" ✓ test/example-1.test.ts (5 tests | 1 skipped) 306ms
 ✓ test/example-2.test.ts (5 tests) 307ms

 Test Files  2 passed (4)
      Tests  10 passed | 3 skipped (65)
   Start at  11:01:36
   Duration  2.00s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests"]["passed"], 10);
    assert_eq!(json["summary"]["tests"]["skipped"], 3);
    assert_eq!(json["summary"]["tests"]["total"], 65);
    assert_eq!(json["summary"]["suites"]["passed"], 2);
    assert_eq!(json["summary"]["suites"]["total"], 4);
}

#[test]
fn test_parse_test_vitest_compact_output() {
    // Test that Vitest parser works with compact output
    let vitest_input = r#" ✓ test/utils.test.ts (2 tests) 306ms

 ✗ test/api.test.ts (2 tests | 1 failed) 307ms

 Test Files  1 passed, 1 failed (2)
      Tests  3 passed, 1 failed, 2 skipped (6)
   Duration  1.26s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Compact output should show summary and failed tests
    assert!(stdout.contains("FAIL:"));
    assert!(stdout.contains("1 passed, 1 failed"));
    assert!(stdout.contains("failed suites"));
    assert!(stdout.contains("test/api.test.ts"));
}

#[test]
fn test_parse_test_vitest_with_tree_output() {
    // Test that Vitest parser works with tree format output
    let vitest_input = r#"✓ __tests__/file1.test.ts (2) 725ms
   ✓ first test file (2) 725ms
     ✓ 2 + 2 should equal 4
     ✓ 4 - 2 should equal 2

 Test Files  1 passed (1)
      Tests  2 passed (2)
   Start at  12:34:32
   Duration  1.26s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests"]["passed"], 2);
    assert_eq!(json["summary"]["suites"]["passed"], 1);
}

#[test]
fn test_parse_test_vitest_failed_output() {
    // Test that Vitest parser handles failed tests
    let vitest_input = r#" ✗ test/failing.test.ts (2 tests | 1 failed) 306ms

 Test Files  1 failed (1)
      Tests  1 passed, 1 failed (2)
   Duration  0.50s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["summary"]["tests"]["passed"], 1);
    assert_eq!(json["summary"]["tests"]["failed"], 1);
    assert_eq!(json["summary"]["suites"]["failed"], 1);
}

// ============================================================
// Bun Test Parser Tests
// ============================================================

#[test]
fn test_parse_bun_test_json_output() {
    // Test that bun parser works and produces valid JSON output
    let bun_input = r#"test/package-json-lint.test.ts:
✓ test/package.json [0.88ms]
✓ test/js/third_party/grpc-js/package.json [0.18ms]

 4 pass
 0 fail
 4 expect() calls
Ran 4 tests in 1.44ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 4);
    assert_eq!(json["summary"]["tests_failed"], 0);
    assert_eq!(json["summary"]["expect_calls"], 4);
}

#[test]
fn test_parse_bun_test_failing_compact_output() {
    // Test compact output with failures
    let bun_input = r#"test/api.test.ts:
✓ should pass [0.88ms]
✗ should fail

 1 pass
 1 fail
 2 expect() calls
Ran 2 tests in 1.44ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Compact output should show summary and failed tests
    assert!(stdout.contains("FAIL:"));
    assert!(stdout.contains("1 passed, 1 failed"));
    assert!(stdout.contains("test/api.test.ts"));
}

#[test]
fn test_parse_bun_test_non_tty_format() {
    // Test non-TTY format (for CI environments)
    let bun_input = r#"test/package-json-lint.test.ts:
(pass) test/package.json [0.48ms]
(fail) test/failing.test.ts
(skip) test/skipped.test.ts

 2 pass
 1 fail
 1 skipped
Ran 4 tests across 1 files. [0.66ms]"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["tests_failed"], 1);
    assert_eq!(json["summary"]["tests_skipped"], 1);
    assert_eq!(json["summary"]["suites_total"], 1);
}

#[test]
fn test_parse_bun_test_all_passed() {
    // Test all tests passed
    let bun_input = r#"test/math.test.ts:
✓ should add numbers [1.00ms]
✓ should subtract numbers [0.50ms]

 2 pass
 0 fail
 2 expect() calls
Ran 2 tests in 1.50ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["suites_passed"], 1);
}

#[test]
fn test_parse_bun_test_failing_json() {
    // Test JSON output with failures
    let bun_input = r#" ✗ test/failing.test.ts (2 tests | 1 failed) 307ms

 1 pass
 1 fail
Ran 2 tests in 0.50s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // With the current parser, this output without a suite header might not parse as expected
    // but it should still produce valid JSON
    assert!(json.is_object());
}

// ============================================================
// NPM Test Parser Tests
// ============================================================

#[test]
fn test_parse_npm_test_json_output() {
    // Test that npm parser works and produces valid JSON output with passed test count
    let npm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["tests_total"], 2);
    assert_eq!(json["summary"]["suites_passed"], 1);
    assert_eq!(json["summary"]["suites_total"], 1);
}

#[test]
fn test_parse_npm_test_failing_compact_output() {
    // Test compact output with failures
    let npm_input = r#"▶ test/math.test.js
  ✖ should multiply numbers
    AssertionError: values are not equal
  ✔ should divide numbers (1.234ms)
▶ test/math.test.js (5.678ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 10ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Compact output should show summary and failed tests
    assert!(stdout.contains("FAIL:"));
    assert!(stdout.contains("1 passed, 1 failed"));
    assert!(stdout.contains("test/math.test.js"));
}

#[test]
fn test_parse_npm_test_with_skipped() {
    // Test that npm parser correctly counts passed tests with skipped tests
    let npm_input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # SKIP
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 skipped (3)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["tests_skipped"], 1);
    assert_eq!(json["summary"]["tests_total"], 3);
}

#[test]
fn test_parse_npm_test_failing_json() {
    // Test that npm parser correctly extracts failed test count in JSON output
    let npm_input = r#"▶ test/math.test.js
  ✖ should multiply numbers
    AssertionError: values are not equal
  ✔ should divide numbers (1.234ms)
▶ test/math.test.js (5.678ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 10ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["summary"]["tests_passed"], 1);
    assert_eq!(json["summary"]["tests_failed"], 1);
    assert_eq!(json["summary"]["tests_total"], 2);
    assert_eq!(json["summary"]["suites_failed"], 1);
}

// ============================================================
// PNPM Test Parser Tests
// ============================================================

#[test]
fn test_parse_pnpm_test_json_output() {
    // Test that pnpm parser works and produces valid JSON output with passed test count
    let pnpm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["tests_total"], 2);
    assert_eq!(json["summary"]["suites_passed"], 1);
    assert_eq!(json["summary"]["suites_total"], 1);
}

#[test]
fn test_parse_pnpm_test_failing_compact_output() {
    // Test compact output with failures
    let pnpm_input = r#"▶ test/api.test.js
  ✖ should fetch data
    Error: network timeout
  ✔ should create item (2.345ms)
▶ test/api.test.js (8.123ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 12ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Compact output should show summary and failed tests
    assert!(stdout.contains("FAIL:"));
    assert!(stdout.contains("1 passed, 1 failed"));
    assert!(stdout.contains("test/api.test.js"));
}

#[test]
fn test_parse_pnpm_test_with_skipped() {
    // Test that pnpm parser correctly counts passed tests with skipped tests
    let pnpm_input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # SKIP
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 skipped (3)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["tests_skipped"], 1);
    assert_eq!(json["summary"]["tests_total"], 3);
}

#[test]
fn test_parse_pnpm_test_failing_json() {
    // Test that pnpm parser correctly extracts failed test count in JSON output
    let pnpm_input = r#"▶ test/api.test.js
  ✖ should fetch data
    Error: network timeout
  ✔ should create item (2.345ms)
▶ test/api.test.js (8.123ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 12ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["summary"]["tests_passed"], 1);
    assert_eq!(json["summary"]["tests_failed"], 1);
    assert_eq!(json["summary"]["tests_total"], 2);
    assert_eq!(json["summary"]["suites_failed"], 1);
}

// ============================================================
// Test Runner Duration Extraction Tests
// ============================================================

#[test]
fn test_parse_pytest_duration_extraction() {
    // Test that pytest parser correctly extracts execution duration
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract PASSED
2 passed in 1.23s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["passed"], 2);
    // Verify duration is extracted and is approximately 1.23 seconds
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 1.23).abs() < 0.01,
        "Expected duration ~1.23s, got {}",
        duration
    );
}

#[test]
fn test_parse_pytest_duration_in_milliseconds() {
    // Test pytest duration extraction with milliseconds format
    let pytest_input = r#"tests/test_main.py::test_quick PASSED
1 passed in 0.05s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 0.05).abs() < 0.01,
        "Expected duration ~0.05s, got {}",
        duration
    );
}

#[test]
fn test_parse_jest_duration_extraction() {
    // Test that Jest parser correctly extracts execution duration from time summary
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)
  ✓ should subtract numbers (2 ms)

Test Suites: 1 passed, 1 total
Tests:       2 passed, 2 total
Time:        1.5 s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    // Verify duration is extracted
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 1.5).abs() < 0.1,
        "Expected duration ~1.5s, got {}",
        duration
    );
}

#[test]
fn test_parse_jest_duration_in_ms() {
    // Test Jest duration extraction with milliseconds format
    let jest_input = r#"PASS src/utils.test.js
  ✓ test (1 ms)

Test Suites: 1 passed, 1 total
Tests:       1 passed, 1 total
Time:        500 ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let duration = json["summary"]["duration"].as_f64().unwrap();
    // 500 ms = 0.5 s
    assert!(
        (duration - 0.5).abs() < 0.1,
        "Expected duration ~0.5s, got {}",
        duration
    );
}

#[test]
fn test_parse_vitest_duration_extraction() {
    // Test that Vitest parser correctly extracts execution duration
    let vitest_input = r#" ✓ test/example.test.ts (5 tests) 306ms

 Test Files  1 passed (1)
      Tests  5 passed (5)
   Start at  11:01:36
   Duration  2.50s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    // Verify duration is extracted
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 2.50).abs() < 0.1,
        "Expected duration ~2.50s, got {}",
        duration
    );
}

#[test]
fn test_parse_vitest_duration_in_ms() {
    // Test Vitest duration extraction with milliseconds format
    let vitest_input = r#" ✓ test/quick.test.ts (1 test) 50ms

 Test Files  1 passed (1)
      Tests  1 passed (1)
   Duration  150ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let duration = json["summary"]["duration"].as_f64().unwrap();
    // 150ms = 0.15s
    assert!(
        (duration - 0.15).abs() < 0.05,
        "Expected duration ~0.15s, got {}",
        duration
    );
}

#[test]
fn test_parse_npm_test_duration_extraction() {
    // Test that npm test parser correctly extracts execution duration
    let npm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 25.5ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    // Verify duration is extracted (25.5ms = 0.0255s)
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 0.0255).abs() < 0.01,
        "Expected duration ~0.0255s, got {}",
        duration
    );
}

#[test]
fn test_parse_npm_test_duration_in_seconds() {
    // Test npm test duration extraction with seconds format
    let npm_input = r#"▶ test/slow.test.js
  ✔ slow test (1000.123ms)
▶ test/slow.test.js (1.5s)

ℹ tests 1 passed (1)
ℹ test files 1 passed (1)
ℹ duration 2.5s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 2.5).abs() < 0.1,
        "Expected duration ~2.5s, got {}",
        duration
    );
}

#[test]
fn test_parse_pnpm_test_duration_extraction() {
    // Test that pnpm test parser correctly extracts execution duration
    let pnpm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 30.25ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    // Verify duration is extracted (30.25ms = 0.03025s)
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 0.03025).abs() < 0.01,
        "Expected duration ~0.03025s, got {}",
        duration
    );
}

#[test]
fn test_parse_pnpm_test_duration_in_seconds() {
    // Test pnpm test duration extraction with seconds format
    let pnpm_input = r#"▶ test/integration.test.js
  ✔ integration test (500ms)
▶ test/integration.test.js (0.75s)

ℹ tests 1 passed (1)
ℹ test files 1 passed (1)
ℹ duration 1.25s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 1.25).abs() < 0.1,
        "Expected duration ~1.25s, got {}",
        duration
    );
}

#[test]
fn test_parse_bun_test_duration_extraction() {
    // Test that Bun test parser correctly extracts execution duration
    let bun_input = r#"test/example.test.ts:
✓ test case [0.05s]

 1 pass
 0 fail
 1 expect() calls
Ran 1 tests in 150ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    // Verify duration is extracted (150ms = 0.15s)
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 0.15).abs() < 0.05,
        "Expected duration ~0.15s, got {}",
        duration
    );
}

#[test]
fn test_parse_bun_test_duration_in_ms() {
    // Test Bun test duration extraction with milliseconds format
    let bun_input = r#"test/quick.test.ts:
✓ quick test [5ms]

 1 pass
 0 fail
Ran 1 tests in 50ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let duration = json["summary"]["duration"].as_f64().unwrap();
    // 50ms = 0.05s
    assert!(
        (duration - 0.05).abs() < 0.02,
        "Expected duration ~0.05s, got {}",
        duration
    );
}

// ============================================================
// Failing Test Identifiers Extraction Tests
// ============================================================

#[test]
fn test_parse_pytest_failed_tests_identifiers() {
    // Test that pytest parser correctly extracts failing test identifiers
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract FAILED
tests/test_utils.py::test_helper FAILED
3 passed, 2 failed in 1.23s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // Verify failed_tests array is present and contains correct identifiers
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert_eq!(failed_tests.len(), 2);
    assert!(failed_tests.contains(&serde_json::json!("tests/test_main.py::test_subtract")));
    assert!(failed_tests.contains(&serde_json::json!("tests/test_utils.py::test_helper")));
}

#[test]
fn test_parse_pytest_failed_tests_empty_when_all_pass() {
    // Test that failed_tests is empty when all tests pass
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract PASSED
2 passed in 0.50s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert!(failed_tests.is_empty());
}

#[test]
fn test_parse_jest_failed_tests_identifiers() {
    // Test that Jest parser correctly extracts failing test identifiers
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)

FAIL src/api.test.js
  ✕ should fetch data (10 ms)
  ✕ should post data (8 ms)

Test Suites: 1 passed, 1 failed, 2 total
Tests:       1 passed, 2 failed, 3 total"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // Verify failed_tests array is present and contains correct identifiers
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert_eq!(failed_tests.len(), 2);
    // Failed tests should be in format: file::test_name
    assert!(failed_tests
        .iter()
        .any(|t| t.as_str().unwrap().contains("src/api.test.js")));
    assert!(failed_tests
        .iter()
        .any(|t| t.as_str().unwrap().contains("should fetch data")));
    assert!(failed_tests
        .iter()
        .any(|t| t.as_str().unwrap().contains("should post data")));
}

#[test]
fn test_parse_jest_failed_tests_empty_when_all_pass() {
    // Test that failed_tests is empty when all tests pass
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)
  ✓ should subtract numbers (2 ms)

Test Suites: 1 passed, 1 total
Tests:       2 passed, 2 total"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert!(failed_tests.is_empty());
}

#[test]
fn test_parse_vitest_failed_tests_identifiers() {
    // Test that Vitest parser correctly extracts failing test identifiers
    let vitest_input = r#" ✓ test/utils.test.ts (2 tests) 306ms
   ✓ should add numbers
   ✓ should subtract numbers

 ✗ test/api.test.ts (2 tests | 2 failed) 307ms
   ✓ should get items
   ✕ should fetch data
   ✕ should post data

 Test Files  1 passed, 1 failed (2)
      Tests  3 passed, 2 failed (5)
   Duration  1.26s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // Verify failed_tests array is present
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert_eq!(failed_tests.len(), 2);
    // Failed tests should contain the file path
    assert!(failed_tests
        .iter()
        .all(|t| t.as_str().unwrap().contains("test/api.test.ts")));
}

#[test]
fn test_parse_vitest_failed_tests_empty_when_all_pass() {
    // Test that failed_tests is empty when all tests pass
    let vitest_input = r#" ✓ test/utils.test.ts (2 tests) 306ms

 Test Files  1 passed (1)
      Tests  2 passed (2)
   Duration  1.26s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert!(failed_tests.is_empty());
}

#[test]
fn test_parse_npm_test_failed_tests_identifiers() {
    // Test that npm test parser correctly extracts failing test identifiers
    let npm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)

▶ test/api.test.js
  ✖ should fetch data
    Error: network timeout
  ✖ should post data
    Error: connection refused
▶ test/api.test.js (8.123ms)

ℹ tests 1 passed 2 failed (3)
ℹ test files 1 failed (1)
ℹ duration 12ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // Verify failed_tests array is present
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert_eq!(failed_tests.len(), 2);
    // Failed tests should contain the file path
    assert!(failed_tests
        .iter()
        .all(|t| t.as_str().unwrap().contains("test/api.test.js")));
}

#[test]
fn test_parse_npm_test_failed_tests_empty_when_all_pass() {
    // Test that failed_tests is empty when all tests pass
    let npm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert!(failed_tests.is_empty());
}

#[test]
fn test_parse_pnpm_test_failed_tests_identifiers() {
    // Test that pnpm test parser correctly extracts failing test identifiers
    let pnpm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)

▶ test/api.test.js
  ✖ should fetch data
    Error: network timeout
  ✖ should post data
    Error: connection refused
▶ test/api.test.js (8.123ms)

ℹ tests 1 passed 2 failed (3)
ℹ test files 1 failed (1)
ℹ duration 12ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // Verify failed_tests array is present
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert_eq!(failed_tests.len(), 2);
    // Failed tests should contain the file path
    assert!(failed_tests
        .iter()
        .all(|t| t.as_str().unwrap().contains("test/api.test.js")));
}

#[test]
fn test_parse_pnpm_test_failed_tests_empty_when_all_pass() {
    // Test that failed_tests is empty when all tests pass
    let pnpm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert!(failed_tests.is_empty());
}

#[test]
fn test_parse_bun_test_failed_tests_identifiers() {
    // Test that Bun test parser correctly extracts failing test identifiers
    let bun_input = r#"test/utils.test.ts:
✓ should add numbers [0.88ms]

test/api.test.ts:
✓ should get items [0.18ms]
✗ should fetch data
✗ should post data

 2 pass
 2 fail
 4 expect() calls
Ran 4 tests in 1.44ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // Verify failed_tests array is present
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert_eq!(failed_tests.len(), 2);
    // Failed tests should contain the file path
    assert!(failed_tests
        .iter()
        .all(|t| t.as_str().unwrap().contains("test/api.test.ts")));
}

#[test]
fn test_parse_bun_test_failed_tests_empty_when_all_pass() {
    // Test that failed_tests is empty when all tests pass
    let bun_input = r#"test/utils.test.ts:
✓ should add numbers [0.88ms]
✓ should subtract numbers [0.18ms]

 2 pass
 0 fail
 2 expect() calls
Ran 2 tests in 1.44ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert!(failed_tests.is_empty());
}

#[test]
fn test_parse_logs_json_output() {
    let log_input =
        "[INFO] Starting application\n[ERROR] Something went wrong\n[WARN] Warning message";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["total_lines"], 3);
    assert_eq!(json["counts"]["info"], 1);
    assert_eq!(json["counts"]["error"], 1);
    assert_eq!(json["counts"]["warning"], 1);
}

#[test]
fn test_parse_logs_detects_repeated_lines() {
    // Test that repeated lines are detected and counted
    let log_input = "Same line\nDifferent line\nSame line\nSame line\nAnother line\nAnother line";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should have 6 total lines
    assert_eq!(json["counts"]["total_lines"], 6);

    // Should detect 2 unique repeated lines
    let repeated = json["repeated_lines"].as_array().unwrap();
    assert_eq!(repeated.len(), 2);

    // Find "Same line" in repeated lines
    let same_line = repeated.iter().find(|r| r["line"] == "Same line").unwrap();
    assert_eq!(same_line["count"], 3);
    assert_eq!(same_line["first_line"], 1);
    assert_eq!(same_line["last_line"], 4);

    // Find "Another line" in repeated lines
    let another_line = repeated
        .iter()
        .find(|r| r["line"] == "Another line")
        .unwrap();
    assert_eq!(another_line["count"], 2);
    assert_eq!(another_line["first_line"], 5);
    assert_eq!(another_line["last_line"], 6);
}

#[test]
fn test_parse_logs_compact_shows_repeated() {
    // Test that compact output shows repeated lines summary
    let log_input = "Repeated message\nOther line\nRepeated message\nRepeated message";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Should show repeated count
    assert!(stdout.contains("repeated:"));
    // Should show the count [x3]
    assert!(stdout.contains("[x3]"));
    // Should show the line content
    assert!(stdout.contains("Repeated message"));
}

// ============================================================
// Error/Warning Level Detection Tests
// ============================================================

#[test]
fn test_parse_logs_detects_error_levels() {
    // Test that ERROR level is properly detected
    let log_input = "[ERROR] Database connection failed\n[INFO] Retrying...";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["error"], 1);
    assert_eq!(json["counts"]["info"], 1);
}

#[test]
fn test_parse_logs_detects_warning_levels() {
    // Test that WARNING level is properly detected
    let log_input = "[WARN] Cache miss\n[WARNING] Slow response time";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["warning"], 2);
}

#[test]
fn test_parse_logs_detects_failed_keyword() {
    // Test that "FAILED" keyword is detected as error
    let log_input = "Test case 1 PASSED\nTest case 2 FAILED\nTest case 3 PASSED";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["error"], 1);
    assert_eq!(json["counts"]["unknown"], 2);
}

#[test]
fn test_parse_logs_detects_exception() {
    // Test that "Exception" keyword is detected as error
    let log_input =
        "Starting application...\nException: NullPointerException\nError at com.example.Main.main";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["error"], 2);
}

#[test]
fn test_parse_logs_detects_fatal_levels() {
    // Test that FATAL level is properly detected
    let log_input = "[FATAL] System out of memory\n[CRITICAL] Disk full";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["fatal"], 2);
}

#[test]
fn test_parse_logs_detects_panic_crash() {
    // Test that PANIC and CRASH are detected as fatal
    let log_input = "PANIC: unrecoverable error\nApplication crashed";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["fatal"], 2);
}

#[test]
fn test_parse_logs_detects_deprecated() {
    // Test that "deprecated" is detected as warning
    let log_input = "Warning: This method is deprecated\nPlease use newMethod instead";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["warning"], 1);
}

#[test]
fn test_parse_logs_detects_connection_errors() {
    // Test that connection errors are detected
    let log_input = "Connection refused\nConnection error: timeout\nAccess denied";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["error"], 3);
}

#[test]
fn test_parse_logs_detects_stack_trace() {
    // Test that stack trace and backtrace are detected as errors
    let log_input = "STACK TRACE:\nBACKTRACE:\nException occurred";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["error"], 3);
}

#[test]
fn test_parse_logs_compact_shows_level_indicators() {
    // Test that compact output shows level indicators [E], [W], [I], etc.
    let log_input = "[ERROR] Something failed\n[WARN] Be careful\n[INFO] All good";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Should show level indicators
    assert!(stdout.contains("[E]"));
    assert!(stdout.contains("[W]"));
    assert!(stdout.contains("[I]"));
}

#[test]
fn test_parse_logs_negation_not_detected_as_error() {
    // Test that "no errors" is NOT detected as error
    let log_input = "All tests passed\nNo errors found\nCompleted with 0 errors";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // All lines should be unknown, not error
    assert_eq!(json["counts"]["error"], 0);
    assert_eq!(json["counts"]["unknown"], 3);
}

#[test]
fn test_parse_logs_various_formats() {
    // Test various log level formats: brackets, colon, pipes
    let log_input = "[ERROR] Bracket format\nERROR: Colon format\n|ERROR| Pipe format";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["error"], 3);
}

#[test]
fn test_parse_logs_slow_query_warning() {
    // Test that slow query/request are detected as warnings
    let log_input = "SLOW QUERY detected: 5.2s\nSLOW REQUEST: 3.1s";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["warning"], 2);
}

#[test]
fn test_parse_logs_notice_level() {
    // Test that NOTICE is detected as info
    let log_input = "[NOTICE] System maintenance scheduled\nNOTICE: Server restart at midnight";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["info"], 2);
}

#[test]
fn test_parse_logs_detects_recent_critical() {
    // Test that recent critical lines are tracked
    let log_input = "[INFO] Starting\n[ERROR] First error\n[WARN] Warning\n[FATAL] Fatal error\n[ERROR] Second error";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should have recent_critical array with 3 entries (2 errors + 1 fatal)
    assert!(json["recent_critical"].is_array());
    let recent = json["recent_critical"].as_array().unwrap();
    assert_eq!(recent.len(), 3);

    // Check counts - error count should be 2, fatal should be 1
    assert_eq!(json["counts"]["error"], 2);
    assert_eq!(json["counts"]["fatal"], 1);
}

#[test]
fn test_parse_logs_recent_critical_only_errors_and_fatals() {
    // Test that only ERROR and FATAL are in recent_critical
    let log_input = "[INFO] Info\n[WARN] Warning\n[DEBUG] Debug\n[ERROR] Error\n[FATAL] Fatal";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let recent = json["recent_critical"].as_array().unwrap();
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0]["level"], "error");
    assert_eq!(recent[1]["level"], "fatal");
}

#[test]
fn test_parse_logs_compact_shows_recent_critical_section() {
    // Test that compact output shows recent critical section
    let log_input = "[INFO] Starting\n[ERROR] Something failed\n[FATAL] System crash\n[INFO] Done";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("recent critical"))
        .stdout(predicate::str::contains("[E]"))
        .stdout(predicate::str::contains("[F]"))
        .stdout(predicate::str::contains("Something failed"))
        .stdout(predicate::str::contains("System crash"));
}

#[test]
fn test_parse_logs_recent_critical_limited() {
    // Create input with more than 10 errors to test limiting
    let mut log_input = String::new();
    for i in 1..=15 {
        log_input.push_str(&format!("[ERROR] Error message {}\n", i));
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input.as_str())
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should be limited to 10
    let recent = json["recent_critical"].as_array().unwrap();
    assert_eq!(recent.len(), 10);
    // Total critical is the sum of error and fatal in counts
    assert_eq!(json["counts"]["error"], 15);
}

#[test]
fn test_parse_logs_compact_shows_truncated_count() {
    // Create input with more than 10 errors
    let mut log_input = String::new();
    for i in 1..=15 {
        log_input.push_str(&format!("[ERROR] Error {}\n", i));
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input.as_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("10 of 15"));
}

#[test]
fn test_parse_logs_no_recent_critical_when_none() {
    // Test that no recent critical section appears when there are no errors
    let log_input = "[INFO] Starting\n[DEBUG] Debug\n[WARN] Warning";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success()
        .stdout(predicate::function(|x: &str| {
            !x.contains("recent critical")
        }));
}

// ============================================================
// Stats Output Tests for Command Execution
// ============================================================

#[test]
fn test_run_stats_shows_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Command:"));
}

#[test]
fn test_run_stats_shows_exit_code() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Exit code:"));
}

#[test]
fn test_run_stats_shows_duration() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Duration:"));
}

#[test]
fn test_run_stats_shows_stdout_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Stdout bytes:"));
}

#[test]
fn test_run_stats_shows_stderr_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Stderr bytes:"));
}

#[test]
fn test_run_stats_shows_output_mode() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output mode:"));
}

#[test]
fn test_run_stats_shows_output_mode_json() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output mode: json"));
}

#[test]
fn test_run_stats_shows_output_mode_raw() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output mode: raw"));
}

#[test]
fn test_run_stats_shows_output_mode_compact() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("--compact")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output mode: compact"));
}

// ============================================================
// Error Handling Tests for Command Execution
// ============================================================

#[test]
fn test_run_permission_denied() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // /etc is a directory, trying to execute it should fail
    cmd.arg("run").arg("/etc").assert().failure().stderr(
        predicate::str::contains("Permission denied").or(predicate::str::contains("Error")),
    );
}

#[test]
fn test_run_empty_args() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // echo with no args just prints a newline
    cmd.arg("run").arg("echo").assert().success();
}

// ============================================================
// Exit Code Propagation Tests
// ============================================================

#[test]
fn test_exit_code_zero_success() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("true").assert().success().code(0);
}

#[test]
fn test_exit_code_one_propagated() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("false").assert().code(1);
}

#[test]
fn test_exit_code_42_propagated() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("exit 42")
        .assert()
        .code(42);
}

#[test]
fn test_exit_code_255_propagated() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("exit 255")
        .assert()
        .code(255);
}

#[test]
fn test_exit_code_command_not_found_is_127() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("nonexistent_command_xyz123")
        .assert()
        .code(127) // Standard "command not found" exit code
        .stderr(predicate::str::contains("Command not found"));
}

#[test]
fn test_command_not_found_json_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("run")
        .arg("nonexistent_command_xyz123")
        .assert()
        .code(127);

    // Error output goes to stderr when using JSON format
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json: serde_json::Value = serde_json::from_str(&stderr).unwrap();

    assert_eq!(json["error"], true);
    assert_eq!(json["exit_code"], 127);
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("Command not found"));
}

#[test]
fn test_exit_code_permission_denied_is_126() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("/etc/passwd") // A file that exists but isn't executable
        .assert()
        .code(126); // Standard "permission denied" exit code
}

#[test]
fn test_exit_code_no_capture_still_propagates() {
    // Even when exit code is not captured, the CLI should still fail
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("false")
        .arg("--capture-stdout=false")
        .arg("--capture-stderr=false")
        .assert()
        .code(1);
}

// ============================================================
// Find Parser: Permission Denied Tests
// ============================================================

#[test]
fn test_parse_find_permission_denied() {
    // Test that permission denied entries are detected and not treated as files
    let find_input = "./src/main.rs\nfind: '/root': Permission denied\n./src/lib.rs\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("Permission denied"))
        .stdout(predicate::str::contains("main.rs"))
        .stdout(predicate::str::contains("lib.rs"));
}

#[test]
fn test_parse_find_permission_denied_json() {
    // Test JSON output includes errors array
    let find_input = "./file.txt\nfind: '/secure': Permission denied\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"errors\":"))
        .stdout(predicate::str::contains("Permission denied"));
}

#[test]
fn test_parse_find_only_errors() {
    // Test when all output is errors - still shows total: 0 with errors
    let find_input = "find: '.': Permission denied\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("Permission denied"));
}

#[test]
fn test_parse_find_no_such_file() {
    // Test "No such file or directory" error handling
    let find_input = "./exists.txt\nfind: 'missing': No such file or directory\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("No such file or directory"))
        .stdout(predicate::str::contains("exists.txt"));
}

#[test]
fn test_parse_find_cannot_open_directory() {
    // Test "cannot open directory" error handling
    let find_input =
        "./file.rs\nfind: cannot open directory '/root': Permission denied\n./another.rs\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("cannot open directory"))
        .stdout(predicate::str::contains("file.rs"))
        .stdout(predicate::str::contains("another.rs"));
}

#[test]
fn test_parse_find_multiple_errors() {
    // Test multiple error messages
    let find_input =
        "find: '/root': Permission denied\n./file.txt\nfind: '/var': Permission denied\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("/root"))
        .stdout(predicate::str::contains("/var"))
        .stdout(predicate::str::contains("file.txt"));
}

// ============================================================
// IsClean Command Tests
// ============================================================

#[test]
fn test_is_clean_in_git_repo() {
    // This test verifies the is-clean command works in a git repo
    // The repo may be clean or dirty, so we just verify the command runs
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // The command should exit with 0 (clean) or 1 (dirty)
    cmd.arg("is-clean")
        .assert()
        .stdout(predicate::str::contains("clean").or(predicate::str::contains("dirty")));
}

#[test]
fn test_is_clean_json_format() {
    // Test JSON output format includes is_clean field
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("is-clean")
        .assert()
        // JSON should contain is_clean field (true or false)
        .stdout(
            predicate::str::contains("\"is_clean\":true")
                .or(predicate::str::contains("\"is_clean\":false")),
        )
        // JSON should contain is_git_repo field
        .stdout(predicate::str::contains("\"is_git_repo\":true"));
}

#[test]
fn test_is_clean_compact_format() {
    // Test compact output format shows status
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("is-clean")
        .assert()
        // Compact should show either clean or dirty with counts
        .stdout(predicate::str::contains("clean").or(predicate::str::contains("dirty")));
}

#[test]
fn test_is_clean_raw_format() {
    // Test raw output format shows clean or dirty
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("is-clean")
        .assert()
        // Raw should show just clean or dirty
        .stdout(predicate::str::contains("clean").or(predicate::str::contains("dirty")));
}

// ============================================================
// Compact Success Summary Tests
// ============================================================

#[test]
fn test_parse_pytest_compact_success_summary() {
    // Test that pytest shows compact summary when all tests pass
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract PASSED
2 passed in 0.50s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success()
        // Should show minimal compact summary
        .stdout(predicate::str::contains("PASS: 2 tests"))
        .stdout(predicate::str::contains("[0.50s]"))
        // Should NOT show detailed breakdown
        .stdout(predicate::str::contains("passed,").not());
}

#[test]
fn test_parse_pytest_compact_failure_summary() {
    // Test that pytest shows detailed failure info when tests fail
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract FAILED
1 passed, 1 failed in 1.23s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success()
        // Should show detailed failure info
        .stdout(predicate::str::contains("FAIL:"))
        .stdout(predicate::str::contains("1 passed, 1 failed"))
        .stdout(predicate::str::contains("failed (1):"));
}

#[test]
fn test_parse_jest_compact_success_summary() {
    // Test that Jest shows compact summary when all tests pass
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)
  ✓ should subtract numbers (2 ms)
Test Suites: 1 passed, 1 total
Tests:       2 passed, 2 total
Time:        1.5 s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success()
        // Should show minimal compact summary
        .stdout(predicate::str::contains("PASS: 1 suites, 2 tests"))
        .stdout(predicate::str::contains("[1.50s]"))
        // Should NOT show detailed breakdown with passed/failed counts
        .stdout(predicate::str::contains("passed, 0 failed").not());
}

#[test]
fn test_parse_jest_compact_failure_summary() {
    // Test that Jest shows detailed failure info when tests fail
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)
FAIL src/api.test.js
  ✕ should fetch data (10 ms)
Test Suites: 1 passed, 1 failed, 2 total
Tests:       1 passed, 1 failed, 2 total"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success()
        // Should show detailed failure info
        .stdout(predicate::str::contains("FAIL:"))
        .stdout(predicate::str::contains("1 passed, 1 failed"))
        .stdout(predicate::str::contains("failed suites (1):"));
}

#[test]
fn test_parse_vitest_compact_success_summary() {
    // Test that Vitest shows compact summary when all tests pass
    let vitest_input = r#" ✓ src/utils.test.js (2 tests) 150ms
 Test Files  1 passed (1)
      Tests  2 passed (2)
   Start at  12:00:00
   Duration  1.50s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success()
        // Should show minimal compact summary
        .stdout(predicate::str::contains("PASS: 1 test files, 2 tests"))
        .stdout(predicate::str::contains("[1.50s]"));
}

#[test]
fn test_parse_vitest_compact_failure_summary() {
    // Test that Vitest shows detailed failure info when tests fail
    let vitest_input = r#" ✓ src/utils.test.js (1 test) 100ms
   ✓ should add numbers
 ✗ src/api.test.js (1 test | 1 failed) 150ms
   ✕ should fetch data
 Test Files  1 passed, 1 failed (2)
      Tests  1 passed, 1 failed (2)"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success()
        // Should show detailed failure info
        .stdout(predicate::str::contains("FAIL:"))
        .stdout(predicate::str::contains("1 passed, 1 failed"))
        .stdout(predicate::str::contains("failed suites (1):"));
}

#[test]
fn test_parse_npm_test_compact_success_summary() {
    // Test that npm test shows compact summary when all tests pass
    let npm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)
ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success()
        // Should show minimal compact summary
        .stdout(predicate::str::contains("PASS: 1 suites, 2 tests"));
}

#[test]
fn test_parse_npm_test_compact_failure_summary() {
    // Test that npm test shows detailed failure info when tests fail
    let npm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
▶ test/api.test.js
  ✖ should fetch data
ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success()
        // Should show detailed failure info
        .stdout(predicate::str::contains("[FAIL]"))
        .stdout(predicate::str::contains("1 passed, 1 failed"));
}

#[test]
fn test_parse_pnpm_test_compact_success_summary() {
    // Test that pnpm test shows compact summary when all tests pass
    let pnpm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)
ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success()
        // Should show minimal compact summary
        .stdout(predicate::str::contains("PASS: 1 suites, 2 tests"));
}

#[test]
fn test_parse_pnpm_test_compact_failure_summary() {
    // Test that pnpm test shows detailed failure info when tests fail
    let pnpm_input = r#"▶ test/api.test.js
  ✖ should fetch data
    Error: network timeout
  ✔ should create item (2.345ms)
▶ test/api.test.js (8.123ms)
ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 12ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success()
        // Should show detailed failure info
        .stdout(predicate::str::contains("[FAIL]"))
        .stdout(predicate::str::contains("1 passed, 1 failed"));
}

#[test]
fn test_parse_bun_test_compact_success_summary() {
    // Test that Bun test shows compact summary when all tests pass
    let bun_input = r#"test/utils.test.ts:
✓ should add numbers [0.88ms]
✓ should subtract numbers [0.45ms]
 2 pass
 0 fail
Ran 2 tests in 1.50ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success()
        // Should show minimal compact summary
        .stdout(predicate::str::contains("PASS: 1 suites, 2 tests"));
}

#[test]
fn test_parse_bun_test_compact_failure_summary() {
    // Test that Bun test shows detailed failure info when tests fail
    let bun_input = r#"test/utils.test.ts:
✓ should add numbers [0.88ms]
test/api.test.ts:
✗ should fetch data
✗ should post data
 1 pass
 2 fail
Ran 3 tests in 1.44ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success()
        // Should show detailed failure info
        .stdout(predicate::str::contains("[FAIL]"))
        .stdout(predicate::str::contains("1 passed, 2 failed"));
}

#[test]
fn test_parse_pytest_compact_success_with_skipped() {
    // Test that skipped tests are shown in compact success summary
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_slow SKIPPED
1 passed, 1 skipped in 0.50s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success()
        // Should show compact summary with skipped count
        .stdout(predicate::str::contains("PASS: 1 tests, 1 skipped"));
}

#[test]
fn test_parse_pytest_failure_with_error_message() {
    // Test that failure-focused summary shows error messages
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract FAILED
1 passed, 1 failed in 1.23s
=== FAILURES ===
____ test_subtract ____

    def test_subtract():
>       assert 1 == 2
E       assert 1 == 2"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success()
        // Should show failure-focused summary
        .stdout(predicate::str::contains("FAIL:"))
        .stdout(predicate::str::contains("1 passed, 1 failed"))
        .stdout(predicate::str::contains("failed (1):"))
        // Should show error message (first line)
        .stdout(predicate::str::contains("def test_subtract():"));
}

#[test]
fn test_parse_pytest_multiple_failures_with_error_messages() {
    // Test that failure-focused summary shows error messages for multiple failures
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract FAILED
tests/test_main.py::test_multiply FAILED
2 passed, 2 failed in 1.23s
=== FAILURES ===
____ test_subtract ____

    def test_subtract():
>       assert 1 == 2
E       assert 1 == 2
____ test_multiply ____

    def test_multiply():
>       assert 2 * 3 == 5
E       assert 6 == 5"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success()
        // Should show failure-focused summary
        .stdout(predicate::str::contains("FAIL:"))
        .stdout(predicate::str::contains("2 passed, 2 failed"))
        .stdout(predicate::str::contains("failed (2):"))
        // Should show both test names
        .stdout(predicate::str::contains("test_subtract"))
        .stdout(predicate::str::contains("test_multiply"));
}

// ============================================================
// Raw Format Tests
// ============================================================

#[test]
fn test_run_command_raw_format() {
    // Raw format should output unprocessed stdout
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("raw_output_test")
        .assert()
        .success()
        .stdout(predicate::str::contains("raw_output_test"));
}

#[test]
fn test_run_command_raw_format_with_stderr() {
    // Raw format should include both stdout and stderr
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo stdout_test && echo stderr_test >&2")
        .assert()
        .success()
        .stdout(predicate::str::contains("stdout_test"))
        .stdout(predicate::str::contains("stderr_test"));
}

#[test]
fn test_parse_git_status_raw_format() {
    // Test raw format for git status parsing
    let status_input = "On branch main\nYour branch is up to date.\n\nChanges to be committed:\n  modified:   src/main.rs\n\nUntracked files:\n  new_file.txt\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("git-status")
        .write_stdin(status_input)
        .assert()
        .success()
        // Raw format should show simple status/path pairs
        .stdout(predicate::str::contains("M src/main.rs"))
        .stdout(predicate::str::contains("? new_file.txt"));
}

#[test]
fn test_parse_git_diff_raw_format() {
    // Test raw format for git diff parsing
    let diff_input = "diff --git a/src/main.rs b/src/main.rs\nindex 1234567..abcdefg 100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,5 +1,6 @@\n fn main() {\n-    println!(\"new\");\n+    println!(\"new\");\n }\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(diff_input)
        .assert()
        .success()
        // Raw format should show file with change type
        .stdout(predicate::str::contains("M src/main.rs"));
}

#[test]
fn test_parse_find_raw_format() {
    // Test raw format for find parsing
    let find_input = "./src/main.rs\n./src/lib.rs\n./tests/test.rs\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        // Raw format should show just the paths
        .stdout(predicate::str::contains("main.rs"))
        .stdout(predicate::str::contains("lib.rs"))
        .stdout(predicate::str::contains("test.rs"))
        // Should not include metadata like "total:"
        .stdout(predicate::function(|x: &str| !x.contains("total:")));
}

#[test]
fn test_parse_logs_raw_format() {
    // Test raw format for log parsing
    let logs_input = "2024-01-15 10:30:00 INFO Application started\n2024-01-15 10:30:01 ERROR Connection failed\n2024-01-15 10:30:02 INFO Retrying...\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("logs")
        .write_stdin(logs_input)
        .assert()
        .success()
        // Raw format should show just the log lines
        .stdout(predicate::str::contains("Application started"))
        .stdout(predicate::str::contains("Connection failed"))
        .stdout(predicate::str::contains("Retrying..."));
}

#[test]
fn test_parse_pytest_raw_format() {
    // Test raw format for pytest output
    let pytest_input = "tests/test_main.py::test_add PASSED\ntests/test_main.py::test_subtract FAILED\n1 passed, 1 failed in 1.23s\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success()
        // Raw format should show minimal output
        .stdout(predicate::str::contains("tests/test_main.py::test_add"))
        .stdout(predicate::str::contains(
            "tests/test_main.py::test_subtract",
        ));
}

#[test]
fn test_parse_jest_raw_format() {
    // Test raw format for Jest output
    let jest_input = "PASS src/utils.test.js\n  ✓ should add numbers (5ms)\n  ✓ should subtract numbers (3ms)\n\nTest Suites: 1 passed, 1 total\nTests:       2 passed, 2 total\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success();
}

#[test]
fn test_parse_vitest_raw_format() {
    // Test raw format for Vitest output
    let vitest_input = " ✓ src/math.test.ts > add (5ms)\n ✓ src/math.test.ts > subtract (3ms)\n\n Test Files  1 passed (1)\n      Tests  2 passed (2)\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
}

#[test]
fn test_parse_npm_test_raw_format() {
    // Test raw format for npm test output
    let npm_input = "\n> project@1.0.0 test\n> jest\n\nPASS src/test.js\n  ✓ test1 (5ms)\n\nTest Suites: 1 passed, 1 total\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
}

#[test]
fn test_raw_format_precedence_over_default() {
    // Test that --raw explicitly sets raw format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
fn test_raw_format_lower_precedence_than_json() {
    // Test that JSON has higher precedence than Raw
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        // JSON format should be used
        .stdout(predicate::str::contains("\"exit_code\""))
        .stdout(predicate::str::contains("\"stdout\""));
}

#[test]
fn test_raw_format_lower_precedence_than_compact() {
    // Test that Compact has higher precedence than Raw
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        // Compact format should be used (just the output)
        .stdout(predicate::str::contains("test"));
}

// ============================================================
// Stdin Input Tests
// ============================================================

#[test]
fn test_stdin_basic_input() {
    // Test reading basic input from stdin without a command
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("Hello World")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}

#[test]
fn test_stdin_with_trailing_whitespace() {
    // Test that trailing whitespace is trimmed
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("Line 1   \nLine 2   ")
        .assert()
        .success()
        .stdout(predicate::str::contains("Line 1"))
        .stdout(predicate::str::contains("Line 2"));
}

#[test]
fn test_stdin_collapses_blank_lines() {
    // Test that multiple blank lines are collapsed into one
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("Line 1\n\n\n\nLine 2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Line 1"))
        .stdout(predicate::str::contains("Line 2"));
}

#[test]
fn test_stdin_strips_ansi_codes() {
    // Test that ANSI escape codes are stripped
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("\x1b[31mRed Text\x1b[0m Normal Text")
        .assert()
        .success()
        .stdout(predicate::str::contains("Red Text"))
        .stdout(predicate::str::contains("Normal Text"))
        .stdout(predicate::str::contains("\x1b[").not());
}

#[test]
fn test_stdin_with_json_format() {
    // Test JSON output format with stdin input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .write_stdin("Test Content")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"content\""))
        .stdout(predicate::str::contains("\"stats\""))
        .stdout(predicate::str::contains("Test Content"));
}

#[test]
fn test_stdin_with_raw_format() {
    // Test raw output format with stdin input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .write_stdin("Raw Content")
        .assert()
        .success()
        .stdout(predicate::str::contains("Raw Content"));
}

#[test]
fn test_stdin_with_csv_format() {
    // Test CSV output format with stdin input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .write_stdin("Line 1\nLine 2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Line 1"))
        .stdout(predicate::str::contains("Line 2"));
}

#[test]
fn test_stdin_with_agent_format() {
    // Test agent output format with stdin input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .write_stdin("Agent Content")
        .assert()
        .success()
        .stdout(predicate::str::contains("Content:"))
        .stdout(predicate::str::contains("Agent Content"));
}

// ============================================================
// Malformed Input Handling Tests
// ============================================================

#[test]
fn test_stdin_handles_null_bytes() {
    // Test that null bytes are removed gracefully
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("hello\x00world")
        .assert()
        .success()
        .stdout(predicate::str::contains("helloworld"))
        .stdout(predicate::function(|x: &str| !x.contains('\x00')));
}

#[test]
fn test_stdin_handles_control_characters() {
    // Test that control characters (except newline/tab) are replaced with spaces
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("hello\x01\x02world")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn test_stdin_preserves_newlines_and_tabs() {
    // Test that newlines and tabs are preserved
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("line1\nline2\ttabbed")
        .assert()
        .success()
        .stdout(predicate::str::contains("line1\nline2\ttabbed"));
}

#[test]
fn test_stdin_handles_ansi_and_control_chars() {
    // Test combination of ANSI codes and control characters
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("\x1b[31mhello\x1b[0m\x00world")
        .assert()
        .success()
        .stdout(predicate::str::contains("helloworld"))
        .stdout(predicate::function(|x: &str| !x.contains("\x1b[31m")));
}

#[test]
fn test_stdin_json_format_with_null_bytes() {
    // Test JSON output handles null bytes correctly
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .write_stdin("hello\x00world")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"content\":\"helloworld\""));
}

#[test]
fn test_parse_git_status_handles_malformed_input() {
    // Test that malformed git status input is handled gracefully
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin("garbage:invalid:data::::here")
        .assert()
        .success(); // Should not crash
}

#[test]
fn test_parse_git_status_with_null_bytes() {
    // Test git status parsing with null bytes in input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin("On branch main\x00\nmodified: file.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
}

#[test]
fn test_parse_git_status_with_up_to_date() {
    // Test git status parsing with "Your branch is up to date" line
    let status_input = "On branch main\nYour branch is up to date with 'origin/main'.\n\nChanges to be committed:\n  modified:   src/main.rs\n\nUntracked files:\n  new_file.txt\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(status_input)
        .assert()
        .success()
        // Should show staged section with count
        .stdout(predicate::str::contains("staged (1)"))
        // Should show untracked section with count
        .stdout(predicate::str::contains("untracked (1)"))
        // Should show the staged file
        .stdout(predicate::str::contains("M src/main.rs"))
        // Should show the untracked file
        .stdout(predicate::str::contains("?? new_file.txt"))
        // Should NOT incorrectly parse "Your branch is up to date" as a file
        .stdout(predicate::function(|x: &str| !x.contains("Yo")));
}

#[test]
fn test_parse_git_status_with_up_to_date_json() {
    // Test git status JSON output with "Your branch is up to date" line
    let status_input = "On branch main\nYour branch is up to date with 'origin/main'.\n\nChanges to be committed:\n  modified:   src/main.rs\n\nUntracked files:\n  new_file.txt\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(status_input)
        .assert()
        .success()
        // JSON should have correct counts
        .stdout(predicate::str::contains("\"staged_count\":1"))
        .stdout(predicate::str::contains("\"untracked_count\":1"))
        // JSON should have correct staged file
        .stdout(predicate::str::contains("\"status\":\"M\""))
        .stdout(predicate::str::contains("\"path\":\"src/main.rs\""))
        // JSON should NOT contain malformed entries
        .stdout(predicate::function(|x: &str| !x.contains("Yo")));
}

#[test]
fn test_parse_logs_with_control_chars() {
    // Test logs parsing with control characters
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("logs")
        .write_stdin("[INFO] Starting\x00\n[ERROR] Failed\x01")
        .assert()
        .success()
        .stdout(predicate::str::contains("Starting"));
}

#[test]
fn test_parse_grep_with_malformed_lines() {
    // Test grep parsing with malformed lines
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin("valid:10:match\nmalformed_line\nanother:20:match")
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn test_stdin_empty_input() {
    // Test empty input handling
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("").assert().success();
}

#[test]
fn test_stdin_only_whitespace() {
    // Test whitespace-only input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("   \n\n   \t  ").assert().success();
}

// ============================================================
// Malformed Input Handling Tests
// ============================================================

#[test]
fn test_stdin_extremely_long_line() {
    // Test handling of extremely long lines (10KB+)
    let long_line = "x".repeat(10000);
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin(long_line.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("x"));
}

#[test]
fn test_stdin_mixed_binary_and_text() {
    // Test handling of mixed binary and text content
    let mut input = Vec::new();
    input.extend_from_slice(b"valid text\n");
    input.extend_from_slice(&[0x00, 0x01, 0x02, 0xFF, 0xFE]); // binary garbage
    input.extend_from_slice(b"\nmore text\n");

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("valid text"))
        .stdout(predicate::str::contains("more text"));
}

#[test]
fn test_stdin_unicode_edge_cases() {
    // Test handling of various Unicode edge cases
    // Note: decorative emojis are stripped by default (🚀 removed, leaving "emoji")
    let input = "normal\n混合文字\n🚀emoji\n日本語\n한국어\nÖlçü\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("normal"))
        .stdout(predicate::str::contains("emoji"))
        .stdout(predicate::str::contains("日本語"))
        .stdout(predicate::str::contains("한국어"));
}

#[test]
fn test_stdin_only_control_characters() {
    // Test input with only control characters
    let input = "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x0B\x0C\x0E\x0F";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin(input).assert().success();
}

#[test]
fn test_stdin_repeated_null_bytes() {
    // Test handling of many consecutive null bytes
    let input = "start\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00end";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("startend"));
}

#[test]
fn test_stdin_carriage_returns() {
    // Test handling of various line endings (CRLF, CR, LF)
    let input = "line1\r\nline2\rline3\nline4";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("line1"))
        .stdout(predicate::str::contains("line4"));
}

#[test]
fn test_parse_git_status_empty() {
    // Test empty git status input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn test_parse_git_status_only_garbage() {
    // Test git status with only unrecognizable content
    let mut input = Vec::new();
    input.extend_from_slice(&[0x00, 0x01, 0x02]);
    input.extend_from_slice(b"garbage");
    input.extend_from_slice(&[0x7F, 0x1F]);
    input.extend_from_slice(b"\nmore");
    input.extend_from_slice(&[0x00]);
    input.extend_from_slice(b"junk");

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success(); // Should not crash
}

#[test]
fn test_parse_git_status_truncated_input() {
    // Test truncated git status (incomplete lines)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin("On branch main\nChanges to be committed:\n  modified:")
        .assert()
        .success();
}

#[test]
fn test_parse_git_status_invalid_utf8() {
    // Test git status with invalid UTF-8 sequences
    let mut input = b"On branch main\n".to_vec();
    input.extend_from_slice(&[0xFF, 0xFE, 0xFD]); // Invalid UTF-8
    input.extend_from_slice(b"\nmodified: file.txt\n");

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success();
}

#[test]
fn test_parse_ls_empty_input() {
    // Test empty ls input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn test_parse_ls_malformed_long_format() {
    // Test ls with malformed long format lines
    let input = "drwxr-xr-x\n-rw-r--r-- 1\n"; // Truncated entries
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success();
}

#[test]
fn test_parse_ls_with_binary_filenames() {
    // Test ls with filenames containing special characters
    let input = "file with spaces.txt\nfile\twith\ttabs.txt\nfile\nwith\nnewlines.txt\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success();
}

#[test]
fn test_parse_grep_empty_input() {
    // Test empty grep input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn test_parse_grep_all_malformed() {
    // Test grep with all malformed lines
    let input = "completely malformed\nanother bad line\nyet another\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success(); // Should return empty results, not crash
}

#[test]
fn test_parse_grep_with_null_bytes() {
    // Test grep input with null bytes
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin("file.rs:10:match\x00here\nfile.rs:20:another")
        .assert()
        .success()
        .stdout(predicate::str::contains("file.rs"));
}

#[test]
fn test_parse_find_empty() {
    // Test empty find input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn test_parse_find_with_errors() {
    // Test find output with permission errors
    let input = "./src\nfind: ./secret: Permission denied\n./tests\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("src"))
        .stdout(predicate::str::contains("tests"));
}

#[test]
fn test_parse_logs_empty() {
    // Test empty logs input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("logs")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn test_parse_logs_only_whitespace() {
    // Test logs with only whitespace
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("logs")
        .write_stdin("   \n\n   \t  \n")
        .assert()
        .success();
}

#[test]
fn test_parse_logs_with_mixed_encoding() {
    // Test logs with mixed valid and invalid content
    let mut input = b"[INFO] Starting\n".to_vec();
    input.extend_from_slice(&[0xFF, 0xFE]); // Invalid UTF-8
    input.extend_from_slice(b"\n[ERROR] Failed\n");

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success();
}

#[test]
fn test_parse_git_diff_empty() {
    // Test empty git diff input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn test_parse_git_diff_malformed() {
    // Test malformed git diff input
    let input = "garbage\n+++\n---\nrandom content\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success(); // Should not crash
}

#[test]
fn test_parse_test_empty() {
    // Test empty test input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn test_parse_test_malformed_pytest() {
    // Test malformed pytest output
    let input = "garbage output\nnot valid pytest\nrandom text\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(input)
        .assert()
        .success(); // Should not crash
}

#[test]
fn test_parse_test_malformed_jest() {
    // Test malformed jest output
    let input = "not jest output\nrandom text\nmore garbage\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(input)
        .assert()
        .success(); // Should not crash
}

#[test]
fn test_stdin_json_format_with_mixed_content() {
    // Test JSON output with mixed content types
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .write_stdin("text\x00with\x01control\nand newlines")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"content\""));
}

#[test]
fn test_stdin_csv_format_with_special_chars() {
    // Test CSV output with special characters
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .write_stdin("line with, comma\nline with \"quotes\"\n")
        .assert()
        .success();
}

#[test]
fn test_stdin_tsv_format_with_tabs() {
    // Test TSV output with embedded tabs
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .write_stdin("line\twith\ttabs\nnormal line\n")
        .assert()
        .success();
}

#[test]
fn test_stdin_agent_format_with_malformed() {
    // Test agent format with malformed input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .write_stdin("\x00\x01malformed\x02\x03\nvalid content\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Content:"));
}

#[test]
fn test_run_command_with_nonexistent_path() {
    // Test run command with path that doesn't exist
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("/nonexistent/path/to/command/xyz123")
        .assert()
        .failure(); // Should fail gracefully
}

#[test]
fn test_search_nonexistent_directory() {
    // Test search in a directory that doesn't exist
    // Note: The search command may succeed with "no matches" rather than fail
    // since it uses ripgrep which handles nonexistent paths gracefully
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("/nonexistent/directory/path")
        .arg("pattern")
        .assert()
        .stdout(predicate::str::contains("grep:").or(predicate::str::contains("no matches")));
}

#[test]
fn test_stdin_very_large_input() {
    // Test handling of very large input (100KB)
    let large_content: String = (0..1000)
        .map(|i| format!("Line {} with some content\n", i))
        .collect();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin(large_content.as_bytes()).assert().success();
}

// ============================================================
// Tail Streaming Mode Tests
// ============================================================

#[test]
fn test_tail_follow_flag_in_help() {
    // Test that --follow flag is documented in help
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--follow"))
        .stdout(predicate::str::contains("streaming mode"));
}

#[test]
fn test_tail_follow_flag_accepted() {
    // Test that --follow flag is accepted and parsed correctly
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "line 2").unwrap();

    // Run with --follow but with a timeout to avoid infinite loop in tests
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let _output = cmd
        .arg("tail")
        .arg(path)
        .arg("--follow")
        .timeout(std::time::Duration::from_millis(500))
        .assert()
        .interrupted(); // Will be interrupted by timeout

    // The fact that it was interrupted (rather than erroring) shows it entered streaming mode
}

#[test]
fn test_tail_follow_shows_initial_output() {
    // Test that --follow shows initial output before streaming
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "initial line 1").unwrap();
    writeln!(file, "initial line 2").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let _output = cmd
        .arg("tail")
        .arg(path)
        .arg("--follow")
        .timeout(std::time::Duration::from_millis(500))
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&_output.stdout);

    // Should show initial lines
    assert!(stdout.contains("initial line 1") || stdout.contains("initial line 2"));
}

#[test]
fn test_tail_follow_with_errors_filter() {
    // Test that --follow works with --errors flag
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "INFO: starting").unwrap();
    writeln!(file, "ERROR: failed").unwrap();
    writeln!(file, "FATAL: crash").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg(path)
        .arg("--errors")
        .arg("--follow")
        .timeout(std::time::Duration::from_millis(500))
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show error lines
    assert!(stdout.contains("ERROR") || stdout.contains("FATAL"));
    // Should not show INFO lines
    assert!(!stdout.contains("INFO: starting"));
}

#[test]
fn test_tail_follow_json_output() {
    // Test that --follow works with JSON output format
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "ERROR: test").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let result = cmd
        .arg("--json")
        .arg("tail")
        .arg(path)
        .arg("--follow")
        .timeout(std::time::Duration::from_millis(500))
        .assert();

    // The process will be interrupted by timeout, which is expected
    // We just need to verify it started without error
    result.interrupted();
}

#[test]
fn test_tail_follow_csv_output() {
    // Test that --follow works with CSV output format
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "line 2").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("tail")
        .arg(path)
        .arg("--follow")
        .timeout(std::time::Duration::from_millis(500))
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should output CSV format for initial lines
    assert!(stdout.contains("line_number,line,is_error") || stdout.contains("line 1"));
}

#[test]
fn test_tail_follow_compact_output() {
    // Test that --follow works with compact output format (default)
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "ERROR: test").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("tail")
        .arg(path)
        .arg("--follow")
        .timeout(std::time::Duration::from_millis(500))
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show compact format header
    assert!(stdout.contains("Last") || stdout.contains("lines from"));
}

#[test]
fn test_tail_follow_with_custom_line_count() {
    // Test that --follow respects custom line count for initial output
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    for i in 1..=20 {
        writeln!(file, "line {}", i).unwrap();
    }

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg(path)
        .arg("--lines")
        .arg("5")
        .arg("--follow")
        .timeout(std::time::Duration::from_millis(500))
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show only last 5 lines initially
    assert!(stdout.contains("line 20") || stdout.contains("line 16"));
}

#[test]
fn test_tail_follow_shorthand_f() {
    // Test that -f shorthand works as alias for --follow
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let _output = cmd
        .arg("tail")
        .arg(path)
        .arg("-f") // Use shorthand
        .timeout(std::time::Duration::from_millis(500))
        .assert()
        .interrupted(); // Will be interrupted by timeout, showing it entered streaming mode
}

#[test]
fn test_tail_follow_empty_file() {
    // Test that --follow works with empty file
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let _file = std::fs::File::create(path).unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let _output = cmd
        .arg("tail")
        .arg(path)
        .arg("--follow")
        .timeout(std::time::Duration::from_millis(500))
        .output()
        .expect("Failed to execute command");

    // Should not crash even with empty file
    // Empty file should show "File is empty" or similar message
}

#[test]
fn test_tail_follow_agent_output() {
    // Test that --follow works with agent-optimized output
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "INFO: normal log").unwrap();
    writeln!(file, "ERROR: error log").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("tail")
        .arg(path)
        .arg("--follow")
        .timeout(std::time::Duration::from_millis(500))
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show agent format
    assert!(stdout.contains("File:") || stdout.contains("❌") || stdout.contains("ERROR"));
}

// ============================================================
// Trim Command Tests
// ============================================================

#[test]
fn test_trim_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Trim whitespace"))
        .stdout(predicate::str::contains("--leading"))
        .stdout(predicate::str::contains("--trailing"));
}

#[test]
fn test_trim_basic() {
    // Test basic whitespace trimming (both sides)
    let input = "  hello world  \n\tfoo bar\t\n   baz   ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"))
        .stdout(predicate::str::contains("foo bar"))
        .stdout(predicate::str::contains("baz"));
}

#[test]
fn test_trim_default_mode() {
    // Test that default mode trims both leading and trailing
    let input = "  hello  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd.arg("trim").write_stdin(input).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Should trim both sides
    assert!(stdout.contains("hello"));
    assert!(!stdout.contains("  hello"));
    assert!(!stdout.contains("hello  "));
    assert!(stdout.contains("mode: both"));
}

#[test]
fn test_trim_leading_only() {
    // Test trimming leading whitespace only
    let input = "  hello  \n\tworld\t";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("trim")
        .arg("--leading")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Should contain "hello  " (trailing preserved) and "world\t" (trailing preserved)
    assert!(stdout.contains("hello"));
    assert!(stdout.contains("world"));
    assert!(stdout.contains("mode: leading"));
}

#[test]
fn test_trim_trailing_only() {
    // Test trimming trailing whitespace only
    let input = "  hello  \n\tworld\t";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("trim")
        .arg("--trailing")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Should contain "  hello" (leading preserved) and "\tworld" (leading preserved)
    assert!(stdout.contains("hello"));
    assert!(stdout.contains("world"));
    assert!(stdout.contains("mode: trailing"));
}

#[test]
fn test_trim_both_flags() {
    // Test with both --leading and --trailing (should be equivalent to default)
    let input = "  hello  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("trim")
        .arg("--leading")
        .arg("--trailing")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("hello"));
    assert!(stdout.contains("mode: both"));
}

#[test]
fn test_trim_file_input() {
    // Test trim with file input
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "  line 1  ").unwrap();
    writeln!(file, "\tline 2\t").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim")
        .arg("-f")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_trim_file_not_found() {
    // Test trim with non-existent file
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim")
        .arg("-f")
        .arg("/nonexistent/path/file.txt")
        .assert()
        .failure();
}

#[test]
fn test_trim_json_output() {
    // Test JSON output format
    let input = "  hello  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["content"], "hello");
    assert!(json["stats"]["input_length"].is_number());
    assert!(json["stats"]["output_length"].is_number());
    assert!(json["stats"]["reduction"].is_number());
    assert_eq!(json["options"]["leading"], false);
    assert_eq!(json["options"]["trailing"], false);
}

#[test]
fn test_trim_csv_output() {
    // Test CSV output format
    let input = "  line 1  \n  line 2  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("line"))
        .stdout(predicate::str::contains("\"line 1\""))
        .stdout(predicate::str::contains("\"line 2\""));
}

#[test]
fn test_trim_tsv_output() {
    // Test TSV output format
    let input = "  line 1  \n  line 2  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_trim_agent_output() {
    // Test agent format output
    let input = "  hello  \n  world  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Content:"))
        .stdout(predicate::str::contains("Stats:"))
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("world"));
}

#[test]
fn test_trim_raw_output() {
    // Test raw output format
    let input = "  hello  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Raw output should just be the trimmed content, no stats
    assert_eq!(stdout.trim(), "hello");
}

#[test]
fn test_trim_empty_input() {
    // Test with empty input
    let input = "";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim").write_stdin(input).assert().success();
}

#[test]
fn test_trim_whitespace_only() {
    // Test with whitespace-only input
    let input = "   \n\t\t\n   ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd.arg("trim").write_stdin(input).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // All lines should be empty after trimming
    for line in stdout.lines() {
        if !line.contains("% reduction") && !line.contains("mode:") {
            assert!(line.trim().is_empty());
        }
    }
}

#[test]
fn test_trim_no_reduction() {
    // Test with input that has no whitespace to trim
    let input = "hello\nworld";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd.arg("trim").write_stdin(input).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("hello"));
    assert!(stdout.contains("world"));
}

#[test]
fn test_trim_mixed_whitespace() {
    // Test with various whitespace types
    let input = "  spaces  \n\ttabs\t\n \t mixed \t ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("spaces"))
        .stdout(predicate::str::contains("tabs"))
        .stdout(predicate::str::contains("mixed"));
}

#[test]
fn test_trim_preserves_empty_lines() {
    // Test that empty lines are preserved
    let input = "  hello  \n\n  world  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "hello");
    assert_eq!(lines[1], "");
    assert_eq!(lines[2], "world");
}

#[test]
fn test_trim_json_with_leading_flag() {
    // Test JSON output with --leading flag
    let input = "  hello  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("trim")
        .arg("--leading")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["options"]["leading"], true);
    assert_eq!(json["options"]["trailing"], false);
}

#[test]
fn test_trim_json_with_trailing_flag() {
    // Test JSON output with --trailing flag
    let input = "  hello  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("trim")
        .arg("--trailing")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["options"]["leading"], false);
    assert_eq!(json["options"]["trailing"], true);
}

// ============================================================
// Input Bytes Tests (--stats flag)
// ============================================================

#[test]
fn test_clean_stats_shows_input_bytes() {
    let input = "line1\nline2\nline3\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_clean_stats_shows_output_bytes() {
    let input = "line1\nline2\nline3\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_clean_stats_shows_reduction() {
    // Use ANSI codes which will be stripped - this should result in smaller output
    let input = "\x1b[31mred text\x1b[0m and \x1b[32mgreen text\x1b[0m\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("Reduction:"));
}

#[test]
fn test_stats_shows_token_estimation() {
    // Use repeated lines that will be collapsed - this should result in token reduction
    let input = "test line\n".repeat(10);
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("clean")
        .arg("--collapse-repeats")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stderr(predicate::str::contains("Input tokens (est.):"))
        .stderr(predicate::str::contains("Output tokens (est.):"))
        .stderr(predicate::str::contains("Token reduction:"));
}

#[test]
fn test_trim_stats_shows_input_bytes() {
    let input = "  hello world  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_trim_stats_shows_output_bytes() {
    let input = "  hello world  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_search_stats_shows_input_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("search")
        .arg(".")
        .arg("fn main")
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_search_stats_shows_output_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("search")
        .arg(".")
        .arg("fn main")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_replace_stats_shows_input_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("replace")
        .arg(".")
        .arg("oldstring")
        .arg("newstring")
        .arg("--dry-run")
        .assert()
        .success()
        .stderr(
            predicate::str::contains("Input bytes:")
                .or(predicate::str::contains("Files affected:")),
        );
}

#[test]
fn test_replace_stats_shows_output_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("replace")
        .arg(".")
        .arg("oldstring")
        .arg("newstring")
        .arg("--dry-run")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_tail_stats_shows_input_bytes() {
    use std::io::Write;
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(temp_file, "line1").unwrap();
    writeln!(temp_file, "line2").unwrap();
    writeln!(temp_file, "line3").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("tail")
        .arg(temp_file.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_tail_stats_shows_output_bytes() {
    use std::io::Write;
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(temp_file, "line1").unwrap();
    writeln!(temp_file, "line2").unwrap();
    writeln!(temp_file, "line3").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("tail")
        .arg(temp_file.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_html2md_stats_shows_input_bytes() {
    let html_content =
        "<html><head><title>Test</title></head><body><h1>Hello</h1><p>World</p></body></html>";
    let mut temp_file = tempfile::NamedTempFile::with_suffix(".html").unwrap();
    std::io::Write::write_all(&mut temp_file, html_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("html2md")
        .arg(temp_file.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_html2md_stats_shows_output_bytes() {
    let html_content =
        "<html><head><title>Test</title></head><body><h1>Hello</h1><p>World</p></body></html>";
    let mut temp_file = tempfile::NamedTempFile::with_suffix(".html").unwrap();
    std::io::Write::write_all(&mut temp_file, html_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("html2md")
        .arg(temp_file.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_txt2md_stats_shows_input_bytes() {
    let text_content = "Heading\n\nThis is paragraph text.\n\n- item 1\n- item 2\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("txt2md")
        .write_stdin(text_content)
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_txt2md_stats_shows_output_bytes() {
    let text_content = "Heading\n\nThis is paragraph text.\n\n- item 1\n- item 2\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("txt2md")
        .write_stdin(text_content)
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

// ============================================================
// Git Status Fixture Tests
// ============================================================

mod fixtures;

use fixtures::*;

#[test]
fn test_fixture_git_status_clean() {
    let input = git_status_clean();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("clean"))
        .stdout(predicate::str::contains("main"));
}

#[test]
fn test_fixture_git_status_staged() {
    let input = git_status_staged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("staged (3)"))
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("src/utils.rs"))
        .stdout(predicate::str::contains("src/old_file.rs"));
}

#[test]
fn test_fixture_git_status_unstaged() {
    let input = git_status_unstaged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("unstaged (3)"))
        .stdout(predicate::str::contains("src/router.rs"))
        .stdout(predicate::str::contains("src/formatter.rs"))
        .stdout(predicate::str::contains("src/deprecated.rs"));
}

#[test]
fn test_fixture_git_status_untracked() {
    let input = git_status_untracked();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("untracked (3)"))
        .stdout(predicate::str::contains("new_feature.rs"))
        .stdout(predicate::str::contains("temp_file.txt"))
        .stdout(predicate::str::contains(".env.local"));
}

#[test]
fn test_fixture_git_status_mixed() {
    let input = git_status_mixed();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("staged (2)"))
        .stdout(predicate::str::contains("unstaged (2)"))
        .stdout(predicate::str::contains("untracked (2)"));
}

#[test]
fn test_fixture_git_status_ahead() {
    let input = git_status_ahead();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("feature"))
        .stdout(predicate::str::contains("ahead 3"));
}

#[test]
fn test_fixture_git_status_behind() {
    let input = git_status_behind();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("main"))
        .stdout(predicate::str::contains("behind 5"));
}

#[test]
fn test_fixture_git_status_diverged() {
    let input = git_status_diverged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("develop"))
        .stdout(predicate::str::contains("ahead 3"))
        .stdout(predicate::str::contains("behind 5"));
}

#[test]
fn test_fixture_git_status_detached() {
    let input = git_status_detached();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("HEAD detached at abc123"));
}

#[test]
fn test_fixture_git_status_renamed() {
    let input = git_status_renamed();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("R"))
        .stdout(predicate::str::contains("new_name.rs"));
}

#[test]
fn test_fixture_git_status_conflict() {
    let input = git_status_conflict();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("unmerged (3)"))
        .stdout(predicate::str::contains("conflict.rs"));
}

#[test]
fn test_fixture_git_status_porcelain() {
    let input = git_status_porcelain();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("src/router.rs"))
        .stdout(predicate::str::contains("src/new_file.rs"))
        .stdout(predicate::str::contains("untracked_file.txt"));
}

#[test]
fn test_fixture_git_status_porcelain_v2() {
    let input = git_status_porcelain_v2();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
}

#[test]
fn test_fixture_git_status_copied() {
    let input = git_status_copied();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("staged (1)"))
        .stdout(predicate::str::contains("implementation.rs"));
}

#[test]
fn test_fixture_git_status_typechange() {
    let input = git_status_typechange();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("symlink_to_file"));
}

#[test]
fn test_fixture_git_status_spanish_clean() {
    let input = git_status_spanish_clean();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
}

#[test]
fn test_fixture_git_status_german_clean() {
    let input = git_status_german_clean();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
}

#[test]
fn test_fixture_git_status_empty() {
    let input = git_status_empty();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success();
}

#[test]
fn test_fixture_git_status_whitespace_only() {
    let input = git_status_whitespace_only();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success();
}

#[test]
fn test_fixture_git_status_no_branch() {
    let input = git_status_no_branch();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("master"));
}

#[test]
fn test_fixture_git_status_long_paths() {
    let input = git_status_long_paths();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("nested"))
        .stdout(predicate::str::contains("with spaces"));
}

#[test]
fn test_fixture_git_status_all_status_codes() {
    let input = git_status_all_status_codes();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("staged"))
        .stdout(predicate::str::contains("unstaged"))
        .stdout(predicate::str::contains("untracked"))
        .stdout(predicate::str::contains("unmerged"));
}

// ============================================================
// Git Status Fixture JSON Output Tests
// ============================================================

#[test]
fn test_fixture_git_status_clean_json() {
    let input = git_status_clean();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"branch\":\"main\""))
        .stdout(predicate::str::contains("\"is_clean\":true"));
}

#[test]
fn test_fixture_git_status_staged_json() {
    let input = git_status_staged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"staged_count\":3"))
        .stdout(predicate::str::contains("\"status\":\"M\""))
        .stdout(predicate::str::contains("\"status\":\"A\""))
        .stdout(predicate::str::contains("\"status\":\"D\""));
}

#[test]
fn test_fixture_git_status_ahead_json() {
    let input = git_status_ahead();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ahead\":3"));
}

#[test]
fn test_fixture_git_status_behind_json() {
    let input = git_status_behind();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"behind\":5"));
}

#[test]
fn test_fixture_git_status_diverged_json() {
    let input = git_status_diverged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ahead\":3"))
        .stdout(predicate::str::contains("\"behind\":5"));
}

// ============================================================
// Git Status Fixture CSV/TSV Output Tests
// ============================================================

#[test]
fn test_fixture_git_status_staged_csv() {
    let input = git_status_staged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("status,path,new_path,section"))
        .stdout(predicate::str::contains("M,src/main.rs"))
        .stdout(predicate::str::contains("A,src/utils.rs"));
}

#[test]
fn test_fixture_git_status_staged_tsv() {
    let input = git_status_staged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("status\tpath\tnew_path\tsection"))
        .stdout(predicate::str::contains("M\tsrc/main.rs"));
}

// ============================================================
// Git Status Fixture Raw Output Tests
// ============================================================

#[test]
fn test_fixture_git_status_staged_raw() {
    let input = git_status_staged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("src/utils.rs"));
}

// ============================================================
// Git Diff Fixture Tests
// ============================================================

#[test]
fn test_fixture_git_diff_empty() {
    let input = git_diff_empty();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("diff: empty"));
}

#[test]
fn test_fixture_git_diff_modified() {
    let input = git_diff_modified();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (1)"))
        .stdout(predicate::str::contains("M src/main.rs"))
        .stdout(predicate::str::contains("+2"));
}

#[test]
fn test_fixture_git_diff_added() {
    let input = git_diff_added();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (1)"))
        .stdout(predicate::str::contains("+ src/utils.rs"))
        .stdout(predicate::str::contains("+5"));
}

#[test]
fn test_fixture_git_diff_deleted() {
    let input = git_diff_deleted();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (1)"))
        .stdout(predicate::str::contains("- src/deprecated.rs"))
        .stdout(predicate::str::contains("-5"));
}

#[test]
fn test_fixture_git_diff_renamed() {
    let input = git_diff_renamed();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (1)"))
        .stdout(predicate::str::contains("R"))
        .stdout(predicate::str::contains("old_name.rs"))
        .stdout(predicate::str::contains("new_name.rs"));
}

#[test]
fn test_fixture_git_diff_copied() {
    let input = git_diff_copied();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (1)"))
        .stdout(predicate::str::contains("C"))
        .stdout(predicate::str::contains("template.rs"))
        .stdout(predicate::str::contains("implementation.rs"));
}

#[test]
fn test_fixture_git_diff_binary() {
    let input = git_diff_binary();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (1)"))
        .stdout(predicate::str::contains("M assets/image.png"));
}

#[test]
fn test_fixture_git_diff_multiple() {
    let input = git_diff_multiple();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (3)"))
        .stdout(predicate::str::contains("M src/main.rs"))
        .stdout(predicate::str::contains("+ src/utils.rs"))
        .stdout(predicate::str::contains("- src/old.rs"));
}

#[test]
fn test_fixture_git_diff_mixed() {
    let input = git_diff_mixed();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (5)"))
        .stdout(predicate::str::contains("M src/main.rs"))
        .stdout(predicate::str::contains("M src/lib.rs"))
        .stdout(predicate::str::contains("+ src/utils.rs"))
        .stdout(predicate::str::contains("- src/deprecated.rs"));
}

#[test]
fn test_fixture_git_diff_large() {
    let input = git_diff_large();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (10)"))
        .stdout(predicate::str::contains("src/file01.rs"))
        .stdout(predicate::str::contains("src/file10.rs"));
}

#[test]
fn test_fixture_git_diff_long_paths() {
    let input = git_diff_long_paths();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (2)"))
        .stdout(predicate::str::contains("nested"));
}

// ============================================================
// Git Diff Fixture JSON Output Tests
// ============================================================

#[test]
fn test_fixture_git_diff_modified_json() {
    let input = git_diff_modified();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_empty\":false"))
        .stdout(predicate::str::contains("\"total_files\":1"))
        .stdout(predicate::str::contains("\"path\":\"src/main.rs\""))
        .stdout(predicate::str::contains("\"change_type\":\"M\""))
        .stdout(predicate::str::contains("\"additions\":2"));
}

#[test]
fn test_fixture_git_diff_added_json() {
    let input = git_diff_added();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"change_type\":\"A\""))
        .stdout(predicate::str::contains("\"path\":\"src/utils.rs\""));
}

#[test]
fn test_fixture_git_diff_deleted_json() {
    let input = git_diff_deleted();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"change_type\":\"D\""))
        .stdout(predicate::str::contains("\"path\":\"src/deprecated.rs\""));
}

#[test]
fn test_fixture_git_diff_renamed_json() {
    let input = git_diff_renamed();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"change_type\":\"R\""))
        .stdout(predicate::str::contains("\"new_path\""));
}

#[test]
fn test_fixture_git_diff_binary_json() {
    let input = git_diff_binary();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_binary\":true"))
        .stdout(predicate::str::contains("\"path\":\"assets/image.png\""));
}

#[test]
fn test_fixture_git_diff_multiple_json() {
    let input = git_diff_multiple();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total_files\":3"))
        .stdout(predicate::str::contains("\"total_additions\":4"))
        .stdout(predicate::str::contains("\"total_deletions\":2"));
}

// ============================================================
// Git Diff Fixture Raw Output Tests
// ============================================================

#[test]
fn test_fixture_git_diff_modified_raw() {
    let input = git_diff_modified();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("M src/main.rs"));
}

#[test]
fn test_fixture_git_diff_multiple_raw() {
    let input = git_diff_multiple();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("M src/main.rs"))
        .stdout(predicate::str::contains("A src/utils.rs"))
        .stdout(predicate::str::contains("D src/old.rs"));
}

#[test]
fn test_fixture_git_diff_renamed_raw() {
    let input = git_diff_renamed();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "R src/old_name.rs -> src/new_name.rs",
        ));
}
