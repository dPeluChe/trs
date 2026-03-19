use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

// Input Bytes Tests (--stats flag)
// ============================================================

#[test]
fn test_clean_stats_shows_input_bytes() {
    let input = "line1\nline2\nline3\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_clean_stats_shows_output_bytes() {
    let input = "line1\nline2\nline3\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_clean_stats_shows_reduction() {
    // Use ANSI codes which will be stripped - this should result in smaller output
    let input = "\x1b[31mred text\x1b[0m and \x1b[32mgreen text\x1b[0m\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("Reduction:"));
}

#[test]
fn test_stats_shows_token_estimation() {
    // Use repeated lines that will be collapsed - this should result in token reduction
    let input = "test line\n".repeat(10);
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("clean")
        .arg("--collapse-repeats")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stderr(predicate::str::contains("Input tokens (est.):"))
        .stderr(predicate::str::contains("Output tokens (est.):"))
        .stderr(predicate::str::contains("Token reduction:"));
}

#[test]
fn test_trim_stats_shows_input_bytes() {
    let input = "  hello world  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_trim_stats_shows_output_bytes() {
    let input = "  hello world  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_search_stats_shows_input_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("search")
        .arg(".")
        .arg("fn main")
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_search_stats_shows_output_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("search")
        .arg(".")
        .arg("fn main")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_replace_stats_shows_input_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("replace")
        .arg(".")
        .arg("oldstring")
        .arg("newstring")
        .arg("--dry-run")
        .assert()
        .success()
        .stderr(
            predicate::str::contains("Input bytes:")
                .or(predicate::str::contains("Files affected:")),
        );
}

#[test]
fn test_replace_stats_shows_output_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("replace")
        .arg(".")
        .arg("oldstring")
        .arg("newstring")
        .arg("--dry-run")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_tail_stats_shows_input_bytes() {
    use std::io::Write;
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(temp_file, "line1").unwrap();
    writeln!(temp_file, "line2").unwrap();
    writeln!(temp_file, "line3").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("tail")
        .arg(temp_file.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_tail_stats_shows_output_bytes() {
    use std::io::Write;
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(temp_file, "line1").unwrap();
    writeln!(temp_file, "line2").unwrap();
    writeln!(temp_file, "line3").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("tail")
        .arg(temp_file.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_html2md_stats_shows_input_bytes() {
    let html_content =
        "<html><head><title>Test</title></head><body><h1>Hello</h1><p>World</p></body></html>";
    let mut temp_file = tempfile::NamedTempFile::with_suffix(".html").unwrap();
    std::io::Write::write_all(&mut temp_file, html_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("html2md")
        .arg(temp_file.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_html2md_stats_shows_output_bytes() {
    let html_content =
        "<html><head><title>Test</title></head><body><h1>Hello</h1><p>World</p></body></html>";
    let mut temp_file = tempfile::NamedTempFile::with_suffix(".html").unwrap();
    std::io::Write::write_all(&mut temp_file, html_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("html2md")
        .arg(temp_file.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_txt2md_stats_shows_input_bytes() {
    let text_content = "Heading\n\nThis is paragraph text.\n\n- item 1\n- item 2\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("txt2md")
        .write_stdin(text_content)
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_txt2md_stats_shows_output_bytes() {
    let text_content = "Heading\n\nThis is paragraph text.\n\n- item 1\n- item 2\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("txt2md")
        .write_stdin(text_content)
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

// ============================================================
