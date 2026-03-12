use assert_cmd::Command;
use predicates::prelude::*;

// ============================================================
// Basic Help Tests
// ============================================================

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("TARS CLI"))
        .stdout(predicate::str::contains("Transform noisy terminal output"));
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("trs"));
}

// ============================================================
// Help System Tests
// ============================================================

#[test]
fn test_help_shows_output_format_flags() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("OUTPUT FORMAT FLAGS"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--csv"))
        .stdout(predicate::str::contains("--tsv"))
        .stdout(predicate::str::contains("--agent"))
        .stdout(predicate::str::contains("--compact"))
        .stdout(predicate::str::contains("--raw"));
}

#[test]
fn test_help_shows_global_flags() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("GLOBAL FLAGS"))
        .stdout(predicate::str::contains("--stats"));
}

#[test]
fn test_help_shows_examples() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("EXAMPLES"));
}

#[test]
fn test_help_shows_documentation_link() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Documentation"));
}

#[test]
fn test_help_shows_all_commands() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("parse"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("replace"))
        .stdout(predicate::str::contains("tail"))
        .stdout(predicate::str::contains("clean"))
        .stdout(predicate::str::contains("html2md"))
        .stdout(predicate::str::contains("txt2md"));
}

// ============================================================
// Command-Specific Help Tests
// ============================================================

#[test]
fn test_search_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Search for patterns"))
        .stdout(predicate::str::contains("ripgrep"))
        .stdout(predicate::str::contains("--extension"))
        .stdout(predicate::str::contains("--ignore-case"))
        .stdout(predicate::str::contains("--context"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn test_replace_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Search and replace"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn test_tail_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tail a file"))
        .stdout(predicate::str::contains("--errors"))
        .stdout(predicate::str::contains("--follow"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn test_clean_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Clean and format"))
        .stdout(predicate::str::contains("--no-ansi"))
        .stdout(predicate::str::contains("--collapse-blanks"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn test_parse_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse structured input"))
        .stdout(predicate::str::contains("git-status"))
        .stdout(predicate::str::contains("git-diff"))
        .stdout(predicate::str::contains("ls"))
        .stdout(predicate::str::contains("grep"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("logs"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn test_html2md_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Convert HTML to Markdown"))
        .stdout(predicate::str::contains("--metadata"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn test_txt2md_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Convert plain text to Markdown"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn test_run_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Execute a command"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

// ============================================================
// Parse Subcommand Help Tests
// ============================================================

#[test]
fn test_parse_git_status_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse git status"))
        .stdout(predicate::str::contains("branch info"));
}

#[test]
fn test_parse_git_diff_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse git diff"));
}

#[test]
fn test_parse_test_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("test")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse test runner"))
        .stdout(predicate::str::contains("pytest"));
}

// ============================================================
// Global Flags Tests
// ============================================================

#[test]
fn test_global_flags_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--raw"))
        .stdout(predicate::str::contains("--compact"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--csv"))
        .stdout(predicate::str::contains("--tsv"))
        .stdout(predicate::str::contains("--agent"))
        .stdout(predicate::str::contains("--stats"));
}

// ============================================================
// Command Execution Tests
// ============================================================

#[test]
fn test_search_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_search_with_options() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("/path/to/dir")
        .arg("pattern")
        .arg("--extension")
        .arg("rs")
        .arg("--ignore-case")
        .assert()
        .success();
}

#[test]
fn test_replace_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(".")
        .arg("old")
        .arg("new")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_replace_dry_run() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(".")
        .arg("old")
        .arg("new")
        .arg("--dry-run")
        .assert()
        .success();
}

#[test]
fn test_tail_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("/var/log/test.log")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_tail_with_lines() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("/var/log/test.log")
        .arg("--lines")
        .arg("20")
        .assert()
        .success();
}

#[test]
fn test_clean_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_clean_with_options() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .arg("--collapse-blanks")
        .arg("--trim")
        .assert()
        .success();
}

#[test]
fn test_parse_git_status() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .assert()
        .success()
        .stdout(predicate::str::contains("status: clean"));
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

#[test]
fn test_parse_grep() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse").arg("grep").assert().success();
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
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg("https://example.com")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_txt2md_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_global_json_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("search")
        .arg(".")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output format: Json"));
}

#[test]
fn test_global_csv_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("search")
        .arg(".")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output format: Csv"));
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
        .stderr(predicate::str::contains("Stats: enabled"));
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
    // When --capture-duration=true, duration_ms should be greater than 0
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
    // Parse JSON and check duration_ms > 0
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let duration_ms = json["duration_ms"].as_u64().unwrap();
    assert!(duration_ms > 0);
}

// ============================================================
// Command Routing Tests
// ============================================================

#[test]
fn test_router_search_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("pattern")
        .assert()
        .success()
        .stderr(predicate::str::contains("Search:"))
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_router_replace_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(".")
        .arg("old")
        .arg("new")
        .assert()
        .success()
        .stderr(predicate::str::contains("Replace:"))
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_router_tail_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("/var/log/test.log")
        .assert()
        .success()
        .stderr(predicate::str::contains("Tail:"))
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_router_clean_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .assert()
        .success()
        .stderr(predicate::str::contains("Clean:"))
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_router_html2md_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg("https://example.com")
        .assert()
        .success()
        .stderr(predicate::str::contains("Html2md:"))
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_router_txt2md_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .assert()
        .success()
        .stderr(predicate::str::contains("Txt2md:"))
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_router_parse_git_status_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .assert()
        .success()
        .stdout(predicate::str::contains("status: clean"));
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
        .stderr(predicate::str::contains("test"))
        .stdout(predicate::str::contains("not yet implemented"));
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
        .arg(".")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output format: Json"));
}

#[test]
fn test_context_agent_format_routing() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("search")
        .arg(".")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output format: Agent"));
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
        .stderr(predicate::str::contains("Stats: enabled"));
}

#[test]
fn test_context_combined_flags_routing() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("--stats")
        .arg("search")
        .arg(".")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output format: Json"))
        .stderr(predicate::str::contains("Stats: enabled"));
}

// ============================================================
// System Command Execution Tests
// ============================================================

#[test]
fn test_run_pwd_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("pwd")
        .assert()
        .success()
        .stdout(predicate::str::contains("/"));
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
fn test_run_uname_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("uname")
        .assert()
        .success()
        .stdout(predicate::str::contains("Darwin").or(predicate::str::contains("Linux")));
}

#[test]
fn test_run_shell_command() {
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
fn test_run_bash_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("bash")
        .arg("-c")
        .arg("echo bash_test")
        .assert()
        .success()
        .stdout(predicate::str::contains("bash_test"));
}

#[test]
fn test_run_command_with_multiple_args() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("arg1")
        .arg("arg2")
        .arg("arg3")
        .assert()
        .success()
        .stdout(predicate::str::contains("arg1"))
        .stdout(predicate::str::contains("arg2"))
        .stdout(predicate::str::contains("arg3"));
}

#[test]
fn test_run_command_with_stderr() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo stderr_test >&2")
        .assert()
        .success()
        .stdout(predicate::str::contains("stderr_test"));
}

#[test]
fn test_run_command_with_stdout_and_stderr() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo stdout_test && echo stderr_test >&2")
        .assert()
        .success()
        .stdout(predicate::str::contains("stdout_test"))
        .stdout(predicate::str::contains("stderr_test"));
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
    cmd.arg("run").arg("env").assert().success();
}

#[test]
fn test_run_true_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("true").assert().success();
}

#[test]
fn test_run_exit_code_propagation() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("sh")
        .arg("-c")
        .arg("exit 42")
        .assert()
        .code(42); // Exit code 42 is now propagated correctly
}

// ============================================================
// JSON Output Tests for Command Execution
// ============================================================

#[test]
fn test_run_json_output_has_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""command":"echo"#));
}

#[test]
fn test_run_json_output_has_args() {
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
fn test_run_json_output_has_exit_code() {
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
fn test_run_json_output_has_stdout() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("echo")
        .arg("hello_world")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""stdout":"hello_world\n"#));
}

#[test]
fn test_run_json_output_has_duration() {
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
fn test_run_json_output_has_stderr() {
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
fn test_run_json_output_timed_out() {
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
fn test_run_json_parsable() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Verify it's valid JSON
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
}

// ============================================================
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
    cmd.arg("run")
        .arg("nonexistent_command_xyz123")
        .assert()
        .code(127) // Standard "command not found" exit code
        .stderr(predicate::str::contains("Command not found"));
}

#[test]
fn test_command_not_found_json_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("run")
        .arg("nonexistent_command_xyz123")
        .assert()
        .code(127);

    // Error output goes to stderr when using JSON format
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json: serde_json::Value = serde_json::from_str(&stderr).unwrap();

    assert_eq!(json["error"], true);
    assert_eq!(json["exit_code"], 127);
    assert!(json["message"].as_str().unwrap().contains("Command not found"));
}

#[test]
fn test_exit_code_permission_denied_is_126() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("/etc/passwd") // A file that exists but isn't executable
        .assert()
        .code(126); // Standard "permission denied" exit code
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
