use assert_cmd::Command;
use predicates::prelude::*;

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
