use assert_cmd::Command;
use predicates::prelude::*;

// LS Parser: Permission Denied Tests
// ============================================================

#[test]
fn test_parse_ls_permission_denied() {
    // Test that permission denied entries are detected and not treated as files
    let ls_input = "file1.txt\nls: cannot open directory '/root': Permission denied\nfile2.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("Permission denied"))
        .stdout(predicate::str::contains("2 files"));
}

#[test]
fn test_parse_ls_permission_denied_json() {
    // Test JSON output includes errors array
    let ls_input = "file.txt\nls: cannot access 'missing': No such file or directory\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"errors\":"))
        .stdout(predicate::str::contains("No such file or directory"));
}

#[test]
fn test_parse_ls_only_errors() {
    // Test when all output is errors
    let ls_input = "ls: cannot open directory '.': Permission denied\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("Permission denied"));
}

// ============================================================
// LS Parser: Symlink Target Tests
// ============================================================

#[test]
fn test_parse_ls_symlink_with_target() {
    // Test that symlink targets are displayed in compact format
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 link_to_file -> /path/to/target\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("link_to_file -> /path/to/target"));
}

#[test]
fn test_parse_ls_symlink_target_json() {
    // Test that JSON output includes symlink_target field
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 mylink -> destination\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"symlink_target\": \"destination\"",
        ))
        .stdout(predicate::str::contains("\"name\": \"mylink\""));
}

#[test]
fn test_parse_ls_multiple_symlinks_with_targets() {
    // Test multiple symlinks with different targets
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 link1 -> target1\nlrwxrwxrwx  1 user  group    10 Jan  1 12:34 link2 -> target2\n-rw-r--r--  1 user  group   42 Jan  1 12:34 file.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("link1 -> target1"))
        .stdout(predicate::str::contains("link2 -> target2"));
}

#[test]
fn test_parse_ls_symlink_no_target() {
    // Test symlink without target (should still work)
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 link_no_target\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("link_no_target"));
}

// ============================================================
// LS Parser: Broken Symlink Tests
// ============================================================

#[test]
fn test_parse_ls_broken_symlink_compact() {
    // Test that broken symlinks are marked in compact output
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 broken_link -> /nonexistent\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "broken_link -> /nonexistent [broken]",
        ));
}

#[test]
fn test_parse_ls_broken_symlink_json() {
    // Test that JSON output includes is_broken_symlink field
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 broken -> /nonexistent/path\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_broken_symlink\": true"))
        .stdout(predicate::str::contains(
            "\"symlink_target\": \"/nonexistent/path\"",
        ));
}

#[test]
fn test_parse_ls_circular_symlink() {
    // Test circular symlinks (self-referencing)
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 circular -> circular\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("circular -> circular [broken]"));
}

#[test]
fn test_parse_ls_circular_symlink_json() {
    // Test circular symlinks in JSON
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 loop -> loop\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_broken_symlink\": true"))
        .stdout(predicate::str::contains("\"symlink_target\": \"loop\""));
}

#[test]
fn test_parse_ls_mixed_broken_and_valid_symlinks() {
    // Test mix of broken and valid symlinks
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 good_link -> existing_file\nlrwxrwxrwx  1 user  group    10 Jan  1 12:34 bad_link -> /nonexistent\n-rw-r--r--  1 user  group   42 Jan  1 12:34 existing_file\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("good_link -> existing_file"))
        .stdout(predicate::str::contains(
            "bad_link -> /nonexistent [broken]",
        ))
        .stdout(predicate::function(|x: &str| {
            // Check that good_link line does NOT contain [broken]
            let lines: Vec<&str> = x.lines().collect();
            let good_link_line = lines
                .iter()
                .find(|l| l.contains("good_link") && l.contains("->"));
            match good_link_line {
                Some(line) => !line.contains("[broken]"),
                None => false,
            }
        }));
}

#[test]
fn test_parse_ls_broken_symlink_json_has_broken_array() {
    // Test that broken symlinks are detected and marked with is_broken_symlink
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 broken1 -> /nonexistent\nlrwxrwxrwx  1 user  group    10 Jan  1 12:34 broken2 -> nonexistent\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_broken_symlink\": true"))
        .stdout(predicate::str::contains("broken1"))
        .stdout(predicate::str::contains("broken2"));
}

#[test]
fn test_parse_ls_valid_symlink_not_marked_broken() {
    // Test that valid symlinks are NOT marked as broken
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 valid_link -> some_file\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_broken_symlink\": false"))
        .stdout(predicate::function(|x: &str| !x.contains("[broken]")));
}

#[test]
fn test_parse_grep() {
    let grep_input = "src/main.rs:42:fn main() {";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"));
}

#[test]
fn test_parse_grep_empty() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains("grep: no matches"));
}

#[test]
fn test_parse_grep_json() {
    let grep_input = "src/main.rs:42:fn main() {";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"files\": 1"))
        .stdout(predicate::str::contains("\"matches\": 1"))
        .stdout(predicate::str::contains("\"line_number\": 42"))
        .stdout(predicate::str::contains("\"line\": \"fn main() {\""));
}

#[test]
fn test_parse_grep_compact() {
    let grep_input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:    println!";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs (2):"));
}

#[test]
fn test_parse_grep_compact_preserves_line_numbers() {
    // Test that line numbers are preserved in compact format
    let grep_input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:    println!";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        // Verify line numbers are present in output
        .stdout(predicate::str::contains("42: fn main() {"))
        .stdout(predicate::str::contains("45:     println!"));
}

#[test]
fn test_parse_grep_compact_line_numbers_multiple_files() {
    // Test line numbers are preserved across multiple files
    let grep_input = "src/main.rs:10:line one\nsrc/lib.rs:25:line two\nsrc/main.rs:30:line three";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        // Verify line numbers for each file
        .stdout(predicate::str::contains("10: line one"))
        .stdout(predicate::str::contains("25: line two"))
        .stdout(predicate::str::contains("30: line three"));
}

#[test]
fn test_parse_grep_groups_interleaved_files() {
    // Test that interleaved matches from same file are grouped together
    let grep_input = "src/main.rs:10:line one\nsrc/lib.rs:25:line two\nsrc/main.rs:30:line three";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        // Should show 2 files, not 3 (main.rs appears twice but grouped)
        .stdout(predicate::str::contains("matches: 2 files, 3 results"))
        // main.rs should show both matches grouped (2)
        .stdout(predicate::str::contains("src/main.rs (2):"))
        // lib.rs should show 1 match
        .stdout(predicate::str::contains("src/lib.rs (1):"));
}

#[test]
fn test_parse_grep_groups_interleaved_files_json() {
    // Test that interleaved matches from same file are grouped together in JSON output
    let grep_input = "src/main.rs:10:line one\nsrc/lib.rs:25:line two\nsrc/main.rs:30:line three";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();

    // Should have 2 files
    assert_eq!(json["counts"]["files"], 2);
    assert_eq!(json["counts"]["matches"], 3);

    let files = json["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);

    // First file should be main.rs with 2 matches
    assert_eq!(files[0]["path"], "src/main.rs");
    assert_eq!(files[0]["matches"].as_array().unwrap().len(), 2);

    // Second file should be lib.rs with 1 match
    assert_eq!(files[1]["path"], "src/lib.rs");
    assert_eq!(files[1]["matches"].as_array().unwrap().len(), 1);
}

#[test]
fn test_parse_grep_csv() {
    let grep_input = "src/main.rs:42:fn main() {";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "path,line_number,column,is_context,line",
        ))
        .stdout(predicate::str::contains("src/main.rs,42,,false,"));
}

#[test]
fn test_parse_grep_tsv() {
    let grep_input = "src/main.rs:42:fn main() {";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "path\tline_number\tcolumn\tis_context\tline",
        ))
        .stdout(predicate::str::contains("src/main.rs\t42\t\tfalse\t"));
}

#[test]
fn test_parse_grep_raw() {
    let grep_input = "src/main.rs:42:fn main() {";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs:42:fn main() {"));
}

#[test]
fn test_parse_grep_multiple_files() {
    let grep_input = "src/main.rs:42:fn main() {\nsrc/lib.rs:10:pub fn helper()";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"files\": 2"))
        .stdout(predicate::str::contains("\"matches\": 2"));
}

// ============================================================
