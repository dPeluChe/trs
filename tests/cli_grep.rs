use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

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
    // When --capture-duration=true, duration_ms should be present and non-negative
    // Note: on fast CI runners, echo can complete in <1ms so duration_ms may be 0
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
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json["duration_ms"].is_u64());
}

// ============================================================
