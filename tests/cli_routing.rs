use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

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
