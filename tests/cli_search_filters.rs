use assert_cmd::Command;

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
