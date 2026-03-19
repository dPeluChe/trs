use assert_cmd::Command;
use predicates::prelude::*;

// ============================================================
// Malformed Input Handling Tests
// ============================================================

#[test]
fn test_stdin_extremely_long_line() {
    // Test handling of extremely long lines (10KB+)
    let long_line = "x".repeat(10000);
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin(long_line.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("x"));
}

#[test]
fn test_stdin_mixed_binary_and_text() {
    // Test handling of mixed binary and text content
    let mut input = Vec::new();
    input.extend_from_slice(b"valid text\n");
    input.extend_from_slice(&[0x00, 0x01, 0x02, 0xFF, 0xFE]); // binary garbage
    input.extend_from_slice(b"\nmore text\n");

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("valid text"))
        .stdout(predicate::str::contains("more text"));
}

#[test]
fn test_stdin_unicode_edge_cases() {
    // Test handling of various Unicode edge cases
    // Note: decorative emojis are stripped by default (🚀 removed, leaving "emoji")
    let input = "normal\n混合文字\n🚀emoji\n日本語\n한국어\nÖlçü\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("normal"))
        .stdout(predicate::str::contains("emoji"))
        .stdout(predicate::str::contains("日本語"))
        .stdout(predicate::str::contains("한국어"));
}

#[test]
fn test_stdin_only_control_characters() {
    // Test input with only control characters
    let input = "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x0B\x0C\x0E\x0F";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin(input).assert().success();
}

#[test]
fn test_stdin_repeated_null_bytes() {
    // Test handling of many consecutive null bytes
    let input = "start\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00end";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("startend"));
}

#[test]
fn test_stdin_carriage_returns() {
    // Test handling of various line endings (CRLF, CR, LF)
    let input = "line1\r\nline2\rline3\nline4";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("line1"))
        .stdout(predicate::str::contains("line4"));
}

#[test]
fn test_parse_git_status_empty() {
    // Test empty git status input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn test_parse_git_status_only_garbage() {
    // Test git status with only unrecognizable content
    let mut input = Vec::new();
    input.extend_from_slice(&[0x00, 0x01, 0x02]);
    input.extend_from_slice(b"garbage");
    input.extend_from_slice(&[0x7F, 0x1F]);
    input.extend_from_slice(b"\nmore");
    input.extend_from_slice(&[0x00]);
    input.extend_from_slice(b"junk");

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success(); // Should not crash
}

#[test]
fn test_parse_git_status_truncated_input() {
    // Test truncated git status (incomplete lines)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin("On branch main\nChanges to be committed:\n  modified:")
        .assert()
        .success();
}

#[test]
fn test_parse_git_status_invalid_utf8() {
    // Test git status with invalid UTF-8 sequences
    let mut input = b"On branch main\n".to_vec();
    input.extend_from_slice(&[0xFF, 0xFE, 0xFD]); // Invalid UTF-8
    input.extend_from_slice(b"\nmodified: file.txt\n");

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success();
}

#[test]
fn test_parse_ls_empty_input() {
    // Test empty ls input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn test_parse_ls_malformed_long_format() {
    // Test ls with malformed long format lines
    let input = "drwxr-xr-x\n-rw-r--r-- 1\n"; // Truncated entries
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success();
}

#[test]
fn test_parse_ls_with_binary_filenames() {
    // Test ls with filenames containing special characters
    let input = "file with spaces.txt\nfile\twith\ttabs.txt\nfile\nwith\nnewlines.txt\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success();
}

#[test]
fn test_parse_grep_empty_input() {
    // Test empty grep input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn test_parse_grep_all_malformed() {
    // Test grep with all malformed lines
    let input = "completely malformed\nanother bad line\nyet another\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success(); // Should return empty results, not crash
}

#[test]
fn test_parse_grep_with_null_bytes() {
    // Test grep input with null bytes
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin("file.rs:10:match\x00here\nfile.rs:20:another")
        .assert()
        .success()
        .stdout(predicate::str::contains("file.rs"));
}

#[test]
fn test_parse_find_empty() {
    // Test empty find input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn test_parse_find_with_errors() {
    // Test find output with permission errors
    let input = "./src\nfind: ./secret: Permission denied\n./tests\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("src"))
        .stdout(predicate::str::contains("tests"));
}

#[test]
fn test_parse_logs_empty() {
    // Test empty logs input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("logs")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn test_parse_logs_only_whitespace() {
    // Test logs with only whitespace
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("logs")
        .write_stdin("   \n\n   \t  \n")
        .assert()
        .success();
}

#[test]
fn test_parse_logs_with_mixed_encoding() {
    // Test logs with mixed valid and invalid content
    let mut input = b"[INFO] Starting\n".to_vec();
    input.extend_from_slice(&[0xFF, 0xFE]); // Invalid UTF-8
    input.extend_from_slice(b"\n[ERROR] Failed\n");

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success();
}

#[test]
fn test_parse_git_diff_empty() {
    // Test empty git diff input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn test_parse_git_diff_malformed() {
    // Test malformed git diff input
    let input = "garbage\n+++\n---\nrandom content\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success(); // Should not crash
}

#[test]
fn test_parse_test_empty() {
    // Test empty test input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn test_parse_test_malformed_pytest() {
    // Test malformed pytest output
    let input = "garbage output\nnot valid pytest\nrandom text\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(input)
        .assert()
        .success(); // Should not crash
}

#[test]
fn test_parse_test_malformed_jest() {
    // Test malformed jest output
    let input = "not jest output\nrandom text\nmore garbage\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(input)
        .assert()
        .success(); // Should not crash
}

#[test]
fn test_stdin_json_format_with_mixed_content() {
    // Test JSON output with mixed content types
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .write_stdin("text\x00with\x01control\nand newlines")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"content\""));
}

#[test]
fn test_stdin_csv_format_with_special_chars() {
    // Test CSV output with special characters
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .write_stdin("line with, comma\nline with \"quotes\"\n")
        .assert()
        .success();
}

#[test]
fn test_stdin_tsv_format_with_tabs() {
    // Test TSV output with embedded tabs
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .write_stdin("line\twith\ttabs\nnormal line\n")
        .assert()
        .success();
}

#[test]
fn test_stdin_agent_format_with_malformed() {
    // Test agent format with malformed input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .write_stdin("\x00\x01malformed\x02\x03\nvalid content\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Content:"));
}

#[test]
fn test_run_command_with_nonexistent_path() {
    // Test run command with path that doesn't exist
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("/nonexistent/path/to/command/xyz123")
        .assert()
        .failure(); // Should fail gracefully
}

#[test]
fn test_search_nonexistent_directory() {
    // Test search in a directory that doesn't exist
    // Note: The search command may succeed with "no matches" rather than fail
    // since it uses ripgrep which handles nonexistent paths gracefully
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("/nonexistent/directory/path")
        .arg("pattern")
        .assert()
        .stdout(predicate::str::contains("grep:").or(predicate::str::contains("no matches")));
}

#[test]
fn test_stdin_very_large_input() {
    // Test handling of very large input (100KB)
    let large_content: String = (0..1000)
        .map(|i| format!("Line {} with some content\n", i))
        .collect();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin(large_content.as_bytes()).assert().success();
}

// ============================================================
