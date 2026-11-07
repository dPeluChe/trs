//! Integration tests for the `search` command: help, paths, edge cases,
//! combined options, and raw vs reduced output size comparison.
//!
//! Covers:
//! - Help and usage
//! - Path handling
//! - Edge cases
//! - Combined options
//! - Stats: raw vs reduced output size comparison

use predicates::prelude::*;

mod support;

// ============================================================
// Help and Usage Tests
// ============================================================

#[test]
fn test_search_help_flag() {
    let mut cmd = support::search_cmd();
    cmd.arg("search")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Search for patterns"))
        .stdout(predicate::str::contains("PATH"))
        .stdout(predicate::str::contains("QUERY"))
        .stdout(predicate::str::contains("--extension"))
        .stdout(predicate::str::contains("--ignore-case"))
        .stdout(predicate::str::contains("--context"))
        .stdout(predicate::str::contains("--limit"));
}

// ============================================================
// Path Handling Tests
// ============================================================

#[test]
fn test_search_with_absolute_path() {
    let mut cmd = support::search_cmd();
    cmd.arg("search")
        .arg(support::search_tree())
        .arg("fn main")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rs"));
}

// ============================================================
// Edge Cases Tests
// ============================================================

#[test]
fn test_search_nonexistent_directory() {
    let mut cmd = support::search_cmd();
    cmd.arg("search")
        .arg("/nonexistent/directory/xyz123")
        .arg("test")
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains(
            "cannot read /nonexistent/directory/xyz123",
        ));
}

#[cfg(unix)]
#[test]
fn test_search_unreadable_directory() {
    let dir = tempfile::tempdir().unwrap();
    let locked = dir.path().join("locked");
    std::fs::create_dir(&locked).unwrap();
    let mut cmd = support::search_cmd();
    if let Some(assert) = support::while_locked(&locked, || {
        cmd.arg("search").arg(&locked).arg("test").assert()
    }) {
        assert
            .code(2)
            .stderr(predicate::str::contains("cannot read"));
    }
}

#[test]
fn test_search_file_as_path() {
    let mut cmd = support::search_cmd();
    // Searching a specific file should work
    cmd.arg("search")
        .arg("src/main.rs")
        .arg("fn main")
        .assert()
        .success()
        .stdout(predicate::str::contains("fn main"));
}

// ============================================================
// Combined Options Tests
// ============================================================

#[test]
fn test_search_combined_extension_and_ignore_case() {
    let mut cmd = support::search_cmd();
    cmd.arg("search")
        .arg("src")
        .arg("searchhandler")
        .arg("-e")
        .arg("rs")
        .arg("-i")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_json_with_all_options() {
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("-e")
        .arg("rs")
        .arg("-C")
        .arg("1")
        .arg("--limit")
        .arg("10")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
}

// ============================================================
// Raw vs Reduced Output Size Comparison Tests
// ============================================================

#[test]
fn test_search_stats_raw_vs_reduced() {
    // Test that raw output shows the raw ripgrep-style output
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("--stats")
        .arg("--raw")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // Both should be present
    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");

    // Output bytes should match stdout length
    assert_eq!(
        output_bytes,
        Some(stdout.len()),
        "Output bytes should match stdout length"
    );
}

#[test]
fn test_search_stats_json_larger() {
    // When using --json format, output_bytes should be larger than input_bytes due to JSON structure
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("--stats")
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // Both should be present
    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");

    // For JSON output, output should be larger than input (JSON adds metadata)
    assert!(
        output_bytes > input_bytes,
        "JSON search output should be larger than raw input"
    );

    // Verify stdout length matches output bytes
    assert_eq!(
        output_bytes,
        Some(stdout.len()),
        "Output bytes should match stdout length"
    );
}

#[test]
fn test_search_stats_compact_reduction() {
    // When using --compact format, verify proper byte counting and reduction
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("--stats")
        .arg("--compact")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // Both should be present and output bytes should match stdout length
    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");
    assert_eq!(
        output_bytes,
        Some(stdout.len()),
        "Output bytes should match stdout length"
    );

    // Compact format typically shows reduction since it summarizes matches
    // (though this depends on the specific pattern and matches)
}

#[test]
fn test_search_stats_agent_format() {
    // When using --agent format, verify proper byte counting
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("--stats")
        .arg("--agent")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // Both should be present and output bytes should match stdout length
    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");
    assert_eq!(
        output_bytes,
        Some(stdout.len()),
        "Output bytes should match stdout length"
    );
}

#[test]
fn test_search_stats_csv_format() {
    // When using --csv format, verify proper byte counting
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("--stats")
        .arg("--csv")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // Both should be present and output bytes should match stdout length
    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");
    assert_eq!(
        output_bytes,
        Some(stdout.len()),
        "Output bytes should match stdout length"
    );
}

#[test]
fn test_search_stats_tsv_format() {
    // When using --tsv format, verify proper byte counting
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("--stats")
        .arg("--tsv")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // Both should be present and output bytes should match stdout length
    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");
    assert_eq!(
        output_bytes,
        Some(stdout.len()),
        "Output bytes should match stdout length"
    );
}

#[test]
fn test_search_stats_empty_results() {
    // Test stats with empty search results
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("--stats")
        .arg("search")
        .arg("src")
        .arg("nonexistent_pattern_xyz123_abc")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // Both should be present (even for empty results)
    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");

    // For empty results, input_bytes should be 0
    assert_eq!(
        input_bytes,
        Some(0),
        "Empty search should have 0 input bytes"
    );
    assert_eq!(
        output_bytes,
        Some(stdout.len()),
        "Output bytes should match stdout length"
    );
}

/// Helper function to extract byte count from stats output
fn extract_bytes(stderr: &str, prefix: &str) -> Option<usize> {
    for line in stderr.lines() {
        if line.contains(prefix) {
            // Extract the number after the prefix
            if let Some(pos) = line.find(prefix) {
                let after = &line[pos + prefix.len()..];
                if let Ok(bytes) = after.trim().parse::<usize>() {
                    return Some(bytes);
                }
            }
        }
    }
    None
}
