use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

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
