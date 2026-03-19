use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_clean_collapse_blanks() {
    // Test blank line collapsing - multiple consecutive blanks become one
    let input = "line 1\n\n\n\nline 2\n\n\nline 3";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--collapse-blanks")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"))
        .stdout(predicate::str::contains("line 3"))
        // Verify no more than one consecutive blank line (no \n\n\n sequences)
        .stdout(predicate::function(|s: &str| !s.contains("\n\n\n")));
}

#[test]
fn test_clean_collapse_blanks_many_consecutive() {
    // Test with many consecutive blank lines
    let input = "start\n\n\n\n\n\n\n\n\n\nend";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--collapse-blanks")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| {
            // Should have at most one blank line between content lines
            // The output should be "start\n\nend" (one blank line between)
            // plus reduction stats at the end
            let lines: Vec<&str> = s.lines().collect();
            let mut consecutive_blank_count = 0;
            let mut max_consecutive_blanks = 0;

            for line in &lines {
                if line.trim().is_empty() {
                    consecutive_blank_count += 1;
                    max_consecutive_blanks = max_consecutive_blanks.max(consecutive_blank_count);
                } else {
                    // Reset counter on non-blank line (including stats line)
                    consecutive_blank_count = 0;
                }
            }
            // Max consecutive blank lines should be 1
            max_consecutive_blanks <= 1
        }));
}

#[test]
fn test_clean_collapse_blanks_whitespace_lines() {
    // Test that whitespace-only lines are treated as blank
    let input = "line 1\n   \n\t\t\n  \nline 2";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--collapse-blanks")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| {
            // No triple newlines should exist
            !s.contains("\n\n\n")
        }));
}

#[test]
fn test_clean_collapse_repeats() {
    // Test repeated line collapsing
    let input = "line 1\nline 1\nline 2\nline 2\nline 2\nline 3";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--collapse-repeats")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| {
            // Each line should appear only once
            let line1_count = s.matches("line 1").count();
            let line2_count = s.matches("line 2").count();
            let line3_count = s.matches("line 3").count();
            line1_count == 1 && line2_count == 1 && line3_count == 1
        }));
}

#[test]
fn test_clean_trim() {
    // Test whitespace trimming
    let input = "  line 1  \n\t\tline 2\t\t\n   line 3   ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--trim")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"))
        .stdout(predicate::str::contains("line 3"));
}

#[test]
fn test_clean_compact_output() {
    // Test compact format output
    let input = "  hello  \n\n\n  world  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("world"))
        .stdout(predicate::str::contains("reduction"));
}

#[test]
fn test_clean_raw_output() {
    // Test raw format output
    let input = "  line 1  \n  line 2  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_clean_agent_output() {
    // Test agent format output
    let input = "  hello  \n\n  world  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Content"))
        .stdout(predicate::str::contains("reduction"))
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("world"));
}

#[test]
fn test_clean_csv_output() {
    // Test CSV format output
    let input = "line 1\nline 2";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"line 1\""))
        .stdout(predicate::str::contains("\"line 2\""));
}

#[test]
fn test_clean_tsv_output() {
    // Test TSV format output
    let input = "line 1\nline 2";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_html2md_json_output() {
    // Test with a local HTML file and JSON output
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_html2md_json.html");

    let html_content = r#"<!DOCTYPE html>
<html>
<head><title>JSON Test</title></head>
<body>
<h1>JSON Test</h1>
</body>
</html>"#;

    let mut file = std::fs::File::create(&html_path).unwrap();
    file.write_all(html_content.as_bytes()).unwrap();
    drop(file);

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("html2md")
        .arg(&html_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json["markdown"].as_str().unwrap().contains("JSON Test"));

    // Cleanup
    let _ = std::fs::remove_file(&html_path);
}

#[test]
fn test_html2md_json_output_includes_metadata() {
    // Test that JSON output automatically includes metadata without --metadata flag
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_html2md_json_meta.html");

    let html_content = r#"<!DOCTYPE html>
<html>
<head>
<title>Metadata Test Page</title>
<meta name="description" content="A test page for metadata extraction">
</head>
<body>
<h1>Test Content</h1>
<p>This is test content.</p>
</body>
</html>"#;

    let mut file = std::fs::File::create(&html_path).unwrap();
    file.write_all(html_content.as_bytes()).unwrap();
    drop(file);

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("html2md")
        .arg(&html_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify markdown content is present
    assert!(json["markdown"].as_str().unwrap().contains("Test Content"));

    // Verify metadata is automatically included in JSON output
    assert!(
        json["metadata"].is_object(),
        "metadata should be present in JSON output"
    );
    assert_eq!(
        json["metadata"]["title"].as_str().unwrap(),
        "Metadata Test Page"
    );
    assert_eq!(
        json["metadata"]["description"].as_str().unwrap(),
        "A test page for metadata extraction"
    );
    assert_eq!(json["metadata"]["type"].as_str().unwrap(), "file");
    assert!(json["metadata"]["source"]
        .as_str()
        .unwrap()
        .contains("test_html2md_json_meta.html"));

    // Cleanup
    let _ = std::fs::remove_file(&html_path);
}

#[test]
fn test_txt2md_json_output() {
    // Test that txt2md produces valid JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("txt2md")
        .write_stdin("TITLE\n\nParagraph text.")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify markdown content is present
    assert!(json["markdown"].as_str().unwrap().contains("# Title"));

    // Verify metadata is present
    assert!(json["metadata"].is_object());
    assert_eq!(json["metadata"]["type"].as_str().unwrap(), "stdin");
    assert!(json["metadata"]["title"]
        .as_str()
        .unwrap()
        .contains("TITLE"));
}

#[test]
fn test_txt2md_stdin_input() {
    // Test explicit stdin input (no file specified)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("MY DOCUMENT\n\nThis is content from stdin.\n\n- Item 1\n- Item 2")
        .assert()
        .success()
        .stdout(predicate::str::contains("# My Document"))
        .stdout(predicate::str::contains("This is content from stdin."))
        .stdout(predicate::str::contains("- Item 1"))
        .stdout(predicate::str::contains("- Item 2"));
}

#[test]
fn test_txt2md_stdin_empty() {
    // Test stdin with empty input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md").write_stdin("").assert().success();
}

#[test]
fn test_txt2md_stdin_with_output_flag() {
    // Test stdin input with output to file
    #[allow(unused_imports)]
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join("test_txt2md_stdin_output.md");

    // Clean up any existing file
    let _ = std::fs::remove_file(&output_path);

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .arg("--output")
        .arg(&output_path)
        .write_stdin("SECTION\n\nContent here.")
        .assert()
        .success();

    // Verify file was created
    assert!(output_path.exists());

    // Verify content
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("# Section"));

    // Cleanup
    let _ = std::fs::remove_file(&output_path);
}

#[test]
fn test_txt2md_file_input() {
    // Test with file input
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("test_txt2md_input.txt");

    let mut file = std::fs::File::create(&input_path).unwrap();
    writeln!(file, "DOCUMENT TITLE").unwrap();
    writeln!(file).unwrap();
    writeln!(file, "This is a paragraph.").unwrap();
    drop(file);

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .arg("--input")
        .arg(&input_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("# Document Title"));

    // Cleanup
    let _ = std::fs::remove_file(&input_path);
}

#[test]
fn test_txt2md_file_output() {
    // Test with file output
    #[allow(unused_imports)]
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join("test_txt2md_output.md");

    // Clean up any existing file
    let _ = std::fs::remove_file(&output_path);

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .arg("--output")
        .arg(&output_path)
        .write_stdin("SECTION HEADING\n\nSome content.")
        .assert()
        .success();

    // Verify file was created
    assert!(output_path.exists());

    // Verify content
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("# Section Heading"));

    // Cleanup
    let _ = std::fs::remove_file(&output_path);
}
