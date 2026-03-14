//! Comprehensive integration tests for the `run` command.
//!
//! This test module verifies command execution functionality through the CLI:
//! - Basic command execution
//! - Output format variations (JSON, CSV, TSV, Agent, Raw, Compact)
//! - Capture flags (stdout, stderr, exit_code, duration)
//! - Exit code propagation
//! - Error handling
//! - Working directory handling
//! - Environment variable handling
//! - Edge cases (large output, Unicode, ANSI codes, etc.)

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

// ============================================================
// Large Output Tests
// ============================================================

#[test]
fn test_run_large_output() {
    // Generate a large output (1000 lines)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("for i in $(seq 1 1000); do echo \"Line $i\"; done")
        .assert()
        .success()
        .stdout(predicate::str::contains("Line 1"))
        .stdout(predicate::str::contains("Line 500"))
        .stdout(predicate::str::contains("Line 1000"));
}

#[test]
fn test_run_long_lines() {
    // Generate output with very long lines
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("python3 -c \"print('x' * 10000)\"")
        .assert()
        .success();
}

// ============================================================
// Stats Output Tests
// ============================================================

#[test]
fn test_run_stats_shows_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Command:"));
}

#[test]
fn test_run_stats_shows_exit_code() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Exit code:"));
}

#[test]
fn test_run_stats_shows_duration() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Duration:"));
}

#[test]
fn test_run_stats_shows_output_mode() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output mode:"));
}

#[test]
fn test_run_stats_with_json_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output mode: json"));
}

// ============================================================
// Compact Format Tests
// ============================================================

#[test]
fn test_run_compact_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
fn test_run_compact_format_default() {
    // Compact is the default format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

// ============================================================
// Raw Format Tests
// ============================================================

#[test]
fn test_run_raw_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
fn test_run_raw_format_preserves_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("raw_output")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("raw_output\n") || stdout.contains("raw_output"));
}

// ============================================================
// Error Handling Tests
// ============================================================

#[test]
fn test_run_permission_denied() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // /etc is a directory, trying to execute it should fail
    cmd.arg("run")
        .arg("/etc")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Permission denied").or(predicate::str::contains("Error")),
        );
}

#[test]
fn test_run_shell_command_success() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo shell_test")
        .assert()
        .success()
        .stdout(predicate::str::contains("shell_test"));
}

#[test]
fn test_run_bash_command_success() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("bash")
        .arg("-c")
        .arg("echo bash_test")
        .assert()
        .success()
        .stdout(predicate::str::contains("bash_test"));
}

// ============================================================
// Stdout and stderr Combined Tests
// ============================================================

#[test]
fn test_run_stdout_only() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("stdout_only")
        .assert()
        .success()
        .stdout(predicate::str::contains("stdout_only"));
}

#[test]
fn test_run_stderr_only() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo stderr_only >&2")
        .assert()
        .success()
        .stdout(predicate::str::contains("stderr_only"));
}

#[test]
fn test_run_stdout_and_stderr() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo stdout_msg && echo stderr_msg >&2")
        .assert()
        .success()
        .stdout(predicate::str::contains("stdout_msg"))
        .stdout(predicate::str::contains("stderr_msg"));
}

// ============================================================
// Edge Cases Tests
// ============================================================

#[test]
fn test_run_empty_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("true")
        .assert()
        .success()
        .code(0);
}

#[test]
fn test_run_empty_args() {
    // echo with no args just prints a newline
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("echo").assert().success();
}

#[test]
fn test_run_special_characters_in_args() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("test-with-dashes")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-with-dashes"));
}

#[test]
fn test_run_spaces_in_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("multiple   spaces")
        .assert()
        .success()
        .stdout(predicate::str::contains("multiple"));
}

#[test]
fn test_run_newlines_in_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo 'line1'; echo 'line2'; echo 'line3'")
        .assert()
        .success()
        .stdout(predicate::str::contains("line1"))
        .stdout(predicate::str::contains("line2"))
        .stdout(predicate::str::contains("line3"));
}

#[test]
fn test_run_tab_characters() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("col1\tcol2")
        .assert()
        .success()
        .stdout(predicate::str::contains("col1"));
}

// ============================================================
// System Commands Tests
// ============================================================

#[test]
fn test_run_uname_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("uname")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Darwin").or(predicate::str::contains("Linux")),
        );
}

#[test]
fn test_run_whoami_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("whoami").assert().success();
}

#[test]
fn test_run_date_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("date").assert().success();
}

#[test]
fn test_run_cat_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("cat")
        .arg("/etc/hosts")
        .assert()
        .success()
        .stdout(predicate::str::contains("localhost"));
}

#[test]
fn test_run_ls_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("ls").arg("/tmp").assert().success();
}

#[test]
fn test_run_env_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("env")
        .assert()
        .success()
        .stdout(predicate::str::contains("PATH"));
}

// ============================================================
// Precedence Tests with Run Command
// ============================================================

#[test]
fn test_run_format_precedence_json_over_raw() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // JSON should win over raw - verify JSON structure is present
    cmd.arg("--json")
        .arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""command":"echo"#))
        .stdout(predicate::str::contains(r#""exit_code":0"#));
}

#[test]
fn test_run_format_precedence_json_over_compact() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // JSON should win over compact - verify JSON structure is present
    cmd.arg("--json")
        .arg("--compact")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""command":"echo"#))
        .stdout(predicate::str::contains(r#""exit_code":0"#));
}

#[test]
fn test_run_format_precedence_compact_over_raw() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Compact should win over raw, output should not be JSON
    cmd.arg("--compact")
        .arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

// ============================================================
// Multiple Capture Flags Tests
// ============================================================

#[test]
fn test_run_no_capture_all() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Capture flags must come before the command name
    cmd.arg("run")
        .arg("--capture-stdout=false")
        .arg("--capture-stderr=false")
        .arg("--capture-exit-code=false")
        .arg("--capture-duration=false")
        .arg("echo")
        .arg("test")
        .assert()
        .success();
}

#[test]
fn test_run_json_no_capture_all() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Capture flags must come before the command name
    cmd.arg("--json")
        .arg("run")
        .arg("--capture-stdout=false")
        .arg("--capture-stderr=false")
        .arg("--capture-exit-code=false")
        .arg("--capture-duration=false")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""exit_code":null"#))
        .stdout(predicate::str::contains(r#""duration_ms":0"#));
}

// ============================================================
// File Path as Command Tests
// ============================================================

#[test]
fn test_run_with_nonexistent_path() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("/nonexistent/path/to/command")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Command not found")
                .or(predicate::str::contains("No such file"))
                .or(predicate::str::contains("Error")),
        );
}
