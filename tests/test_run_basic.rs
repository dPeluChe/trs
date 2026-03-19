//! Integration tests for the `run` command: basic execution, exit codes, JSON output,
//! capture flags, working directory, environment variables, unicode, ANSI, and large output.

use assert_cmd::Command;
use predicates::prelude::*;

// ============================================================
// Basic Command Execution Tests
// ============================================================

#[test]
fn test_run_echo_simple() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
}

#[test]
fn test_run_echo_multiple_args() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("one")
        .arg("two")
        .arg("three")
        .assert()
        .success()
        .stdout(predicate::str::contains("one"))
        .stdout(predicate::str::contains("two"))
        .stdout(predicate::str::contains("three"));
}

#[test]
fn test_run_true_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("true").assert().success().code(0);
}

#[test]
fn test_run_false_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("false").assert().code(1);
}

#[test]
fn test_run_pwd_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("pwd")
        .assert()
        .success()
        .stdout(predicate::str::contains("/"));
}

// ============================================================
// Exit Code Propagation Tests
// ============================================================

#[test]
fn test_run_exit_code_zero() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("true").assert().success().code(0);
}

#[test]
fn test_run_exit_code_one() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("false").assert().code(1);
}

#[test]
fn test_run_exit_code_custom() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("exit 42")
        .assert()
        .code(42);
}

#[test]
fn test_run_exit_code_max() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("exit 255")
        .assert()
        .code(255);
}

#[test]
fn test_run_command_not_found() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("nonexistent_command_xyz123")
        .assert()
        .code(127)
        .stderr(predicate::str::contains("Command not found"));
}

// ============================================================
// JSON Output Format Tests
// ============================================================

#[test]
fn test_run_json_is_valid() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
}

#[test]
fn test_run_json_has_command_field() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // The run command JSON output has a "command" field with the command name
    cmd.arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""command":"echo"#));
}

#[test]
fn test_run_json_has_args_field() {
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
fn test_run_json_has_exit_code_field() {
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
fn test_run_json_has_stdout_field() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test_output")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""stdout":"test_output\n"#));
}

#[test]
fn test_run_json_has_stderr_field() {
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
fn test_run_json_has_duration_field() {
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
fn test_run_json_has_timed_out_field() {
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
fn test_run_json_exit_code_non_zero() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("sh")
        .arg("-c")
        .arg("exit 42")
        .assert()
        .code(42)
        .stdout(predicate::str::contains(r#""exit_code":42"#));
}

// ============================================================
// Capture Flags Tests
// ============================================================

#[test]
fn test_run_capture_stdout_default() {
    // By default, stdout is captured
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("captured")
        .assert()
        .success()
        .stdout(predicate::str::contains("captured"));
}

#[test]
fn test_run_no_capture_stdout() {
    // When --capture-stdout=false, output is not in the processed output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Capture flag must come before the command name
    cmd.arg("run")
        .arg("--capture-stdout=false")
        .arg("echo")
        .arg("hello")
        .assert()
        .success();
    // stdout goes directly to terminal when not captured
}

#[test]
fn test_run_capture_stderr_default() {
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
fn test_run_no_capture_stderr() {
    // When --capture-stderr=false, stderr goes directly to terminal
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Capture flag must come before the command name
    cmd.arg("run")
        .arg("--capture-stderr=false")
        .arg("sh")
        .arg("-c")
        .arg("echo stderr_test >&2")
        .assert()
        .success();
}

#[test]
fn test_run_capture_exit_code_default() {
    // By default, exit code is captured
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
fn test_run_no_capture_exit_code() {
    // When --capture-exit-code=false, exit_code is null in JSON
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("--capture-exit-code=false")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""exit_code":null"#));
}

#[test]
fn test_run_capture_duration_default() {
    // By default, duration is captured
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json["duration_ms"].as_u64().unwrap() > 0);
}

#[test]
fn test_run_no_capture_duration() {
    // When --capture-duration=false, duration_ms is 0
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("--capture-duration=false")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""duration_ms":0"#));
}

// ============================================================
// Working Directory Tests
// ============================================================

#[test]
fn test_run_with_working_dir_tmp() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("run")
        .arg("pwd")
        .current_dir("/tmp")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // On macOS, /tmp is a symlink to /private/tmp
    assert!(stdout.contains("/tmp") || stdout.contains("private/tmp"));
}

// ============================================================
// Environment Variable Tests
// ============================================================

#[test]
fn test_run_with_env_var() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo $TEST_VAR")
        .env("TEST_VAR", "test_value")
        .assert()
        .success()
        .stdout(predicate::str::contains("test_value"));
}

#[test]
fn test_run_inherits_parent_env() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("env")
        .assert()
        .success()
        .stdout(predicate::str::contains("PATH"));
}

// ============================================================
// Unicode Handling Tests
// ============================================================

#[test]
fn test_run_unicode_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("Hello 世界 🌍")
        .assert()
        .success()
        .stdout(predicate::str::contains("世界"))
        .stdout(predicate::str::contains("🌍"));
}

#[test]
fn test_run_json_unicode_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("run")
        .arg("echo")
        .arg("Привет мир")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json["stdout"].as_str().unwrap().contains("Привет"));
}

#[test]
fn test_run_emoji_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("🚀 🎉 ✅")
        .assert()
        .success()
        .stdout(predicate::str::contains("🚀"));
}

// ============================================================
// ANSI Code Handling Tests
// ============================================================

#[test]
fn test_run_ansi_codes_in_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Commands may output ANSI codes; the CLI should handle them
    cmd.arg("run")
        .arg("echo")
        .arg("\x1b[31mred\x1b[0m")
        .assert()
        .success()
        .stdout(predicate::str::contains("red"));
}

#[test]
fn test_run_ansi_codes_json_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("run")
        .arg("echo")
        .arg("\x1b[32mgreen\x1b[0m")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // The output should contain "green" (possibly with ANSI codes)
    assert!(json["stdout"].as_str().unwrap().contains("green"));
}
