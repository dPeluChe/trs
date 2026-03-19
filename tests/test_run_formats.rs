//! Integration tests for the `run` command: output formats (stats, compact, raw),
//! error handling, stdout/stderr, edge cases, system commands, precedence, and capture flags.

use assert_cmd::Command;
use predicates::prelude::*;

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
