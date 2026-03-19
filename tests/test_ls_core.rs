//! Integration tests for the `ls` parse command - basic parsing, JSON output,
//! stats output, and raw vs reduced output size comparison tests.

use assert_cmd::Command;
use predicates::prelude::*;

// ============================================================
// Basic ls Parsing Tests
// ============================================================

#[test]
fn test_ls_simple() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin("src\nCargo.toml\nREADME.md\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("src"))
        .stdout(predicate::str::contains("Cargo.toml"))
        .stdout(predicate::str::contains("README.md"));
}

#[test]
fn test_ls_empty() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn test_ls_long_format() {
    let input = "total 32\ndrwxr-xr-x   3 user group 4096 Jan 15 10:30 src\n-rw-r--r--   1 user group  128 Jan 15 10:32 Cargo.toml\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("src"))
        .stdout(predicate::str::contains("Cargo.toml"));
}

#[test]
fn test_ls_with_directories() {
    let input = "src\ntarget\nCargo.toml\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("src"))
        .stdout(predicate::str::contains("target"));
}

#[test]
fn test_ls_with_hidden() {
    let input = ".git\n.gitignore\nsrc\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains(".git"))
        .stdout(predicate::str::contains(".gitignore"));
}

// ============================================================
// JSON Output Format Tests
// ============================================================

#[test]
fn test_ls_json_is_valid() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin("src\nCargo.toml\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
}

#[test]
fn test_ls_json_has_entries_field() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin("src\nCargo.toml\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""entries"#));
}

#[test]
fn test_ls_json_has_files_field() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin("Cargo.toml\nREADME.md\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""files"#));
}

#[test]
fn test_ls_json_has_directories_field() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin("src\ntarget\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""directories"#));
}

// ============================================================
// Stats Output Tests
// ============================================================

#[test]
fn test_ls_stats_shows_reducer() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("parse")
        .arg("ls")
        .write_stdin("src\nCargo.toml\n")
        .assert()
        .success()
        .stderr(predicate::str::contains("Reducer:"));
}

#[test]
fn test_ls_stats_shows_input_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("parse")
        .arg("ls")
        .write_stdin("src\nCargo.toml\n")
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_ls_stats_shows_output_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("parse")
        .arg("ls")
        .write_stdin("src\nCargo.toml\n")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_ls_stats_shows_output_mode() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("parse")
        .arg("ls")
        .write_stdin("src\nCargo.toml\n")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output mode:"));
}

// ============================================================
// Stats: Raw vs Reduced Output Size Comparison Tests
// ============================================================

#[test]
fn test_ls_stats_raw_output_same_size() {
    // When using --raw format with simple input (just filenames), input_bytes should equal output_bytes
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--raw")
        .arg("parse")
        .arg("ls")
        .write_stdin("src\nCargo.toml\nREADME.md\n")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // For raw output with simple input (just filenames), input and output bytes should be equal
    // because raw format just outputs the filenames again
    assert_eq!(
        input_bytes, output_bytes,
        "Raw output should have same input and output bytes"
    );

    // Also verify stdout length matches
    assert_eq!(
        output_bytes,
        Some(stdout.len()),
        "Output bytes should match stdout length"
    );
}

#[test]
fn test_ls_stats_json_output_larger_than_raw() {
    // When using --json format, output_bytes should be larger than raw input_bytes
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin("src\nCargo.toml\n")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // For JSON output, output should be larger than input (JSON adds metadata)
    assert!(
        output_bytes > input_bytes,
        "JSON output should be larger than raw input"
    );

    // Verify stdout length matches output bytes
    assert_eq!(
        output_bytes,
        Some(stdout.len()),
        "Output bytes should match stdout length"
    );
}

#[test]
fn test_ls_stats_compact_output_size() {
    // When using --compact format, verify proper byte counting
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--compact")
        .arg("parse")
        .arg("ls")
        .write_stdin("src\nCargo.toml\n")
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
fn test_ls_stats_long_format_comparison() {
    // Test with long format ls output - raw format extracts just filenames
    // so output will be smaller than input (strips metadata like permissions, size, dates)
    let input = "total 32\ndrwxr-xr-x   3 user group 4096 Jan 15 10:30 src\n-rw-r--r--   1 user group  128 Jan 15 10:32 Cargo.toml\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--raw")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // For long format input, raw output is smaller (extracts just filenames)
    // This demonstrates the reduction - metadata is stripped
    assert!(
        output_bytes < input_bytes,
        "Raw long format ls should have smaller output (just filenames)"
    );

    // Verify stdout contains the filenames
    assert!(stdout.contains("src"), "Output should contain 'src'");
    assert!(
        stdout.contains("Cargo.toml"),
        "Output should contain 'Cargo.toml'"
    );

    // Output bytes should match stdout length
    assert_eq!(
        output_bytes,
        Some(stdout.len()),
        "Output bytes should match stdout length"
    );
}

#[test]
fn test_ls_stats_long_format_json_larger() {
    // Test long format ls with JSON format - output should be larger due to JSON structure
    let input = "total 32\ndrwxr-xr-x   3 user group 4096 Jan 15 10:30 src\n-rw-r--r--   1 user group  128 Jan 15 10:32 Cargo.toml\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // JSON output should be larger than raw input
    assert!(
        output_bytes > input_bytes,
        "JSON long format ls output should be larger than raw input"
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
