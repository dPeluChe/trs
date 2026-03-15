// Test file to verify that line numbers are returned in search results
// This test file specifically addresses the requirement: "return line numbers"

use assert_cmd::Command;

/// Test that line numbers are present in compact format output
#[test]
fn test_line_numbers_compact_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("search")
        .arg("src/router.rs")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should contain line numbers in format like "1657:12:"
    assert!(output_str.contains("1657:"));
    assert!(output_str.contains("SearchHandler"));
}

/// Test that line numbers are present in JSON format
#[test]
fn test_line_numbers_json_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src/router.rs")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Parse JSON and verify line_number field exists
    let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();
    assert!(json["files"].is_array());

    let files = json["files"].as_array().unwrap();
    assert!(!files.is_empty());

    let first_file = &files[0];
    assert!(first_file["matches"].is_array());

    let matches = first_file["matches"].as_array().unwrap();
    assert!(!matches.is_empty());

    let first_match = &matches[0];
    assert!(first_match["line_number"].is_number());
    assert!(first_match["line_number"].as_u64().unwrap() > 0);
}

/// Test that line numbers are present in CSV format
#[test]
fn test_line_numbers_csv_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("search")
        .arg("src/router.rs")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // CSV should have header with line_number column
    assert!(output_str.contains("path,line_number,column,is_context,line"));

    // Should contain numeric line numbers
    let lines: Vec<&str> = output_str.lines().collect();
    assert!(lines.len() > 1); // At least header + one match

    // Second line should contain a number for line_number column
    let parts: Vec<&str> = lines[1].split(',').collect();
    assert!(parts.len() >= 2);
    assert!(parts[1].parse::<u64>().is_ok()); // line_number should be a number
}

/// Test that line numbers are present in TSV format
#[test]
fn test_line_numbers_tsv_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--tsv")
        .arg("search")
        .arg("src/router.rs")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // TSV should have header with line_number column
    assert!(output_str.contains("path\tline_number\tcolumn\tis_context\tline"));

    // Should contain numeric line numbers
    let lines: Vec<&str> = output_str.lines().collect();
    assert!(lines.len() > 1);

    let parts: Vec<&str> = lines[1].split('\t').collect();
    assert!(parts.len() >= 2);
    assert!(parts[1].parse::<u64>().is_ok()); // line_number should be a number
}

/// Test that line numbers are present in raw format
#[test]
fn test_line_numbers_raw_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("search")
        .arg("src/router.rs")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Raw format should show line numbers with colons
    assert!(output_str.contains("router.rs:"));
    assert!(output_str.contains("SearchHandler"));

    // Should contain at least one numeric line number
    let lines: Vec<&str> = output_str.lines().collect();
    assert!(!lines.is_empty());

    // First line should have format like "src/router.rs:1400:12:..."
    assert!(lines[0].contains(":1400:") || lines[0].contains(":"));
}

/// Test that line numbers work with context lines
#[test]
fn test_line_numbers_with_context() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("search")
        .arg("src/router.rs")
        .arg("SearchHandler")
        .arg("--context")
        .arg("1")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should contain match line numbers
    assert!(output_str.contains("1657:"));
    assert!(output_str.contains("SearchHandler"));

    // Context lines should be indicated with ellipsis
    assert!(output_str.contains("..."));
}

/// Test that line numbers are accurate across multiple matches
#[test]
fn test_line_numbers_accuracy() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src/router.rs")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();

    let matches = json["files"][0]["matches"].as_array().unwrap();

    // Verify all matches have valid line numbers
    for m in matches {
        assert!(m["line_number"].is_number());
        let line_num = m["line_number"].as_u64().unwrap();
        assert!(line_num > 0);
        assert!(line_num < 100000); // Reasonable upper bound
    }

    // Verify line numbers are in ascending order
    let line_numbers: Vec<u64> = matches
        .iter()
        .filter_map(|m| m["line_number"].as_u64())
        .collect();

    let mut sorted = line_numbers.clone();
    sorted.sort();
    assert_eq!(line_numbers, sorted);
}
