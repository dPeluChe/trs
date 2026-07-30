use assert_cmd::Command;
use predicates::prelude::*;

// Stats Output Tests for Command Execution
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
fn test_run_stats_shows_stdout_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Stdout bytes:"));
}

#[test]
fn test_run_stats_shows_stderr_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Stderr bytes:"));
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
fn test_run_stats_shows_output_mode_json() {
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

#[test]
fn test_run_stats_shows_output_mode_raw() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output mode: raw"));
}

#[test]
fn test_run_stats_shows_output_mode_compact() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("--compact")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output mode: compact"));
}

// ============================================================
// Error Handling Tests for Command Execution
// ============================================================

#[test]
fn test_run_permission_denied() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // /etc is a directory, trying to execute it should fail
    cmd.arg("run").arg("/etc").assert().failure().stderr(
        predicate::str::contains("Permission denied").or(predicate::str::contains("Error")),
    );
}

#[test]
fn test_run_empty_args() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // echo with no args just prints a newline
    cmd.arg("run").arg("echo").assert().success();
}

// ============================================================
// Exit Code Propagation Tests
// ============================================================

#[test]
fn test_exit_code_zero_success() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("true").assert().success().code(0);
}

#[test]
fn test_exit_code_one_propagated() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("false").assert().code(1);
}

#[test]
fn test_exit_code_42_propagated() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("exit 42")
        .assert()
        .code(42);
}

#[test]
fn test_exit_code_255_propagated() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("exit 255")
        .assert()
        .code(255);
}

#[test]
fn test_exit_code_command_not_found_is_127() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("nonexistent_command_xyz123");
    let assert = cmd.assert();
    // POSIX: direct spawn → 127 + "Command not found". Windows routes through
    // `cmd /C`, which signals a missing command via its own non-zero exit (#53).
    #[cfg(not(windows))]
    assert
        .code(127)
        .stderr(predicate::str::contains("Command not found"));
    #[cfg(windows)]
    assert.failure();
}

#[test]
fn test_command_not_found_json_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("nonexistent_command_xyz123");
    #[cfg(not(windows))]
    {
        let output = cmd.assert().code(127);
        // Error output goes to stderr when using JSON format.
        let stderr = String::from_utf8_lossy(&output.get_output().stderr);
        let json: serde_json::Value = serde_json::from_str(&stderr).unwrap();
        assert_eq!(json["error"], true);
        assert_eq!(json["exit_code"], 127);
        assert!(json["message"]
            .as_str()
            .unwrap()
            .contains("Command not found"));
    }
    #[cfg(windows)]
    cmd.assert().failure();
}

#[test]
fn test_exit_code_permission_denied_is_126() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // 126 ("not executable") is a POSIX convention; `/etc/passwd` is Unix-only.
    // Windows has no equivalent — just assert the run fails.
    #[cfg(not(windows))]
    cmd.arg("run").arg("/etc/passwd").assert().code(126);
    #[cfg(windows)]
    cmd.arg("run").arg("/etc/passwd").assert().failure();
}

#[test]
fn test_exit_code_no_capture_still_propagates() {
    // Even when exit code is not captured, the CLI should still fail
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("false")
        .arg("--capture-stdout=false")
        .arg("--capture-stderr=false")
        .assert()
        .code(1);
}

// ============================================================
// Find Parser: Permission Denied Tests
// ============================================================

#[test]
fn test_parse_find_permission_denied() {
    // Test that permission denied entries are detected and not treated as files
    let find_input = "./src/main.rs\nfind: '/root': Permission denied\n./src/lib.rs\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("Permission denied"))
        .stdout(predicate::str::contains("main"))
        .stdout(predicate::str::contains("lib"));
}

#[test]
fn test_parse_find_permission_denied_json() {
    // Test JSON output includes errors array
    let find_input = "./file.txt\nfind: '/secure': Permission denied\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"errors\":"))
        .stdout(predicate::str::contains("Permission denied"));
}

#[test]
fn test_parse_find_only_errors() {
    // Test when all output is errors - still shows total: 0 with errors
    let find_input = "find: '.': Permission denied\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("Permission denied"));
}

#[test]
fn test_parse_find_no_such_file() {
    // Test "No such file or directory" error handling
    let find_input = "./exists.txt\nfind: 'missing': No such file or directory\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("No such file or directory"))
        .stdout(predicate::str::contains("exists"));
}

#[test]
fn test_parse_find_cannot_open_directory() {
    // Test "cannot open directory" error handling
    let find_input =
        "./file.rs\nfind: cannot open directory '/root': Permission denied\n./another.rs\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("cannot open directory"))
        .stdout(predicate::str::contains("file"))
        .stdout(predicate::str::contains("another"));
}

#[test]
fn test_parse_find_multiple_errors() {
    // Test multiple error messages
    let find_input =
        "find: '/root': Permission denied\n./file.txt\nfind: '/var': Permission denied\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("/root"))
        .stdout(predicate::str::contains("/var"))
        .stdout(predicate::str::contains("file"));
}

// ============================================================
// IsClean Command Tests
// ============================================================

#[test]
fn test_is_clean_in_git_repo() {
    // This test verifies the is-clean command works in a git repo
    // The repo may be clean or dirty, so we just verify the command runs
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // The command should exit with 0 (clean) or 1 (dirty)
    cmd.arg("is-clean")
        .assert()
        .stdout(predicate::str::contains("clean").or(predicate::str::contains("dirty")));
}

#[test]
fn test_is_clean_json_format() {
    // Test JSON output format includes is_clean field
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("is-clean")
        .assert()
        // JSON should contain is_clean field (true or false)
        .stdout(
            predicate::str::contains("\"is_clean\":true")
                .or(predicate::str::contains("\"is_clean\":false")),
        )
        // JSON should contain is_git_repo field
        .stdout(predicate::str::contains("\"is_git_repo\":true"));
}

#[test]
fn test_is_clean_compact_format() {
    // Test compact output format shows status
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("is-clean")
        .assert()
        // Compact should show either clean or dirty with counts
        .stdout(predicate::str::contains("clean").or(predicate::str::contains("dirty")));
}

#[test]
fn test_is_clean_raw_format() {
    // Test raw output format shows clean or dirty
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("is-clean")
        .assert()
        // Raw should show just clean or dirty
        .stdout(predicate::str::contains("clean").or(predicate::str::contains("dirty")));
}

// ============================================================

/// `find` output is tiered by result size: full names for small results,
/// capped names in the middle, counts only for thousands of matches. The small
/// case must stay lossless — a short find IS the answer, not a summary of it.
#[test]
fn find_tiers_by_result_size() {
    use assert_cmd::Command as C;

    let run = |body: String| -> String {
        let out = C::cargo_bin("trs")
            .unwrap()
            .args(["parse", "find"])
            .write_stdin(body)
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).to_string()
    };

    // Small: every name present, no truncation marker.
    let small: String = (0..5).map(|i| format!("src/a/f{}.rs\n", i)).collect();
    let got = run(small);
    for i in 0..5 {
        assert!(got.contains(&format!("f{}", i)), "lost a name: {}", got);
    }
    assert!(
        !got.contains("more"),
        "small result must not be capped: {}",
        got
    );

    // Middle (>200 entries, one crowded dir): capped with a marker, and the
    // count of what was dropped has to be there.
    let mid: String = (0..300).map(|i| format!("src/a/f{}.rs\n", i)).collect();
    let got = run(mid);
    assert!(got.contains("more"), "middle tier should cap: {}", got);

    // Large (>1000): directory map with counts, no individual names.
    let big: String = (0..1200)
        .map(|i| format!("src/d{}/f{}.rs\n", i % 40, i))
        .collect();
    let got = run(big);
    assert!(
        got.contains("(30)"),
        "large tier should show counts: {}",
        got
    );
}
