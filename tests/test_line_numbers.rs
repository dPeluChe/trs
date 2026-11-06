// Test file to verify that line numbers are returned in search results
// This test file specifically addresses the requirement: "return line numbers"

mod support;

/// Test that line numbers are present in compact format output
#[test]
fn test_line_numbers_compact_format() {
    let mut cmd = support::search_cmd();
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

    // Should contain line numbers in format like "NNN:" and the search term
    assert!(output_str.contains("SearchHandler"));
    // Verify at least one line number pattern (digits followed by colon)
    let has_line_number = output_str
        .lines()
        .any(|line| line.chars().any(|c| c.is_ascii_digit()) && line.contains(':'));
    assert!(has_line_number, "Output should contain line numbers");
}

/// Test that line numbers are present in JSON format
#[test]
fn test_line_numbers_json_format() {
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src/router/")
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
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("--csv")
        .arg("search")
        .arg("src/router/")
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
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("--tsv")
        .arg("search")
        .arg("src/router/")
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
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("--raw")
        .arg("search")
        .arg("src/router/")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Raw format should show line numbers with colons
    assert!(output_str.contains("router/"));
    assert!(output_str.contains("SearchHandler"));

    assert!(output_str.contains("search.rs:1:12:"), "{output_str}");
}

/// Test that line numbers work with context lines
#[test]
fn test_line_numbers_with_context() {
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("search")
        .arg("src/router/")
        .arg("SearchHandler")
        .arg("--context")
        .arg("1")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should contain match line numbers and search term
    assert!(output_str.contains("SearchHandler"));
    let has_line_number = output_str
        .lines()
        .any(|line| line.chars().any(|c| c.is_ascii_digit()) && line.contains(':'));
    assert!(has_line_number, "Output should contain line numbers");

    // Requested context lines print, marked `N-` the way grep marks them
    let has_context = output_str.lines().any(|l| {
        l.trim_start()
            .split_once("- ")
            .is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
    });
    assert!(
        has_context,
        "context lines should print with a `-` marker:\n{output_str}"
    );
}

/// Test that line numbers are accurate across multiple matches
#[test]
fn test_line_numbers_accuracy() {
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src/router/")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();

    let file = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"].as_str().unwrap().ends_with("handlers/search.rs"))
        .unwrap();
    let line_numbers: Vec<u64> = file["matches"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["line_number"].as_u64())
        .collect();
    assert_eq!(line_numbers, [1, 3, 5]);
}
