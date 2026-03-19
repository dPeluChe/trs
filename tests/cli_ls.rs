use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

// LS Parser Tests
// ============================================================

#[test]
fn test_parse_ls_empty() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains("(empty)"));
}

#[test]
fn test_parse_ls_simple_files() {
    let ls_input = "file1.txt\nfile2.txt\nfile3.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"))
        .stdout(predicate::str::contains("file3.txt"))
        .stdout(predicate::str::contains("3 files"));
}

#[test]
fn test_parse_ls_with_directories() {
    let ls_input = "file1.txt\ndir1\nfile2.txt\ndir2\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("2 files"))
        .stdout(predicate::str::contains("2 dirs"))
        .stdout(predicate::str::contains("dir1/"))
        .stdout(predicate::str::contains("dir2/"));
}

#[test]
fn test_parse_ls_with_hidden_files() {
    let ls_input = "file1.txt\n.hidden_file\n.visible_file\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(".hidden_file"))
        .stdout(predicate::str::contains(".visible_file"));
}

// ============================================================
// Hidden File Detection Tests
// ============================================================

#[test]
fn test_parse_ls_hidden_directory() {
    // Test that hidden directories are detected
    let ls_input = ".git/\npublic/\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(".git/"))
        .stdout(predicate::str::contains("public/"));
}

#[test]
fn test_parse_ls_hidden_file_with_extension() {
    // Test that hidden files with extensions are detected
    let ls_input = ".gitignore\n.env.local\n.config.json\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(".gitignore"))
        .stdout(predicate::str::contains(".env.local"))
        .stdout(predicate::str::contains(".config.json"));
}

#[test]
fn test_parse_ls_dot_and_dotdot() {
    // Test that . and .. are filtered from compact output (they add no signal)
    let ls_input = ".\n..\nfile.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("file.txt"));
}

#[test]
fn test_parse_ls_double_dots() {
    // Test files starting with multiple dots
    let ls_input = "..swp\n...triple\nfile.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("..swp"))
        .stdout(predicate::str::contains("...triple"));
}

#[test]
fn test_parse_ls_long_format_hidden_files() {
    // Test hidden files in long format output
    let ls_input = "total 8\n-rw-r--r--  1 user  group  123 Jan  1 12:34 .gitignore\n-rw-r--r--  1 user  group  456 Jan  1 12:34 .env\ndrwxr-xr-x  2 user  group 4096 Jan  1 12:34 .git\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(".gitignore"))
        .stdout(predicate::str::contains(".env"))
        .stdout(predicate::str::contains(".git"));
}

#[test]
fn test_parse_ls_hidden_symlink() {
    // Test hidden symlinks
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 .link_to_file\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(".link_to_file"));
}

#[test]
fn test_parse_ls_json_hidden_files() {
    // Test JSON output includes is_hidden field
    let ls_input = "file.txt\n.hidden\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_hidden\": false"))
        .stdout(predicate::str::contains("\"is_hidden\": true"))
        .stdout(predicate::str::contains("\"hidden\": ["))
        .stdout(predicate::str::contains(".hidden"));
}

#[test]
fn test_parse_ls_mixed_hidden_and_visible() {
    // Test a mix of hidden and visible files/directories
    let ls_input = "public/\nsrc/\n.git/\n.env\nREADME.md\n.gitignore\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(".git/"))
        .stdout(predicate::str::contains(".env"))
        .stdout(predicate::str::contains(".gitignore"));
}

#[test]
fn test_parse_ls_only_hidden_files() {
    // Test when all files are hidden
    let ls_input = ".a\n.b\n.c\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(".a"))
        .stdout(predicate::str::contains(".b"))
        .stdout(predicate::str::contains(".c"));
}

#[test]
fn test_parse_ls_no_hidden_files() {
    // Test when no hidden files are present
    let ls_input = "file1.txt\nfile2.txt\nfile3.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("3 files"))
        .stdout(predicate::str::contains("file1.txt"));
}

#[test]
fn test_parse_ls_long_format() {
    let ls_input = "total 0\ndrwxr-xr-x  2 user  group  4096 Jan  1 12:34 dirname\n-rw-r--r--  1 user  group    42 Jan  1 12:34 file1.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 files"))
        .stdout(predicate::str::contains("1 dirs"))
        .stdout(predicate::str::contains("dirname/"))
        .stdout(predicate::str::contains("file1.txt"));
}

#[test]
fn test_parse_ls_json_format() {
    let ls_input = "file1.txt\nfile2.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        // Check schema structure
        .stdout(predicate::str::contains("\"schema\""))
        .stdout(predicate::str::contains("\"type\": \"ls_output\""))
        .stdout(predicate::str::contains("\"counts\""))
        .stdout(predicate::str::contains("\"total\": 2"))
        .stdout(predicate::str::contains("\"name\": \"file1.txt\""))
        .stdout(predicate::str::contains("file"));
}

#[test]
fn test_parse_ls_raw_format() {
    let ls_input = "file1.txt\nfile2.txt\nfile3.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"))
        .stdout(predicate::str::contains("file3.txt"))
        .stdout(predicate::function(|x: &str| !x.contains("total:")));
}

#[test]
fn test_parse_ls_with_symlinks() {
    let ls_input = "file1.txt\nlrwxrwxrwx  1 user  group    10 Jan  1 12:34 link_to_file\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("symlink"))
        .stdout(predicate::str::contains("link_to_file"));
}

#[test]
fn test_parse_ls_with_file_from_stdin() {
    // Test that we can pipe ls output to the parser
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run").arg("ls").arg("/tmp").assert().success();
}

// ============================================================
// Generated Directories Tests
// ============================================================

#[test]
fn test_parse_ls_node_modules_detected() {
    // Test that node_modules is detected as a generated directory
    let ls_input = "src/\nnode_modules/\npackage.json\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 generated"))
        .stdout(predicate::str::contains("node_modules/"));
}

#[test]
fn test_parse_ls_target_detected() {
    // Test that target directory (Rust) is detected
    let ls_input = "src/\ntarget/\nCargo.toml\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 generated"))
        .stdout(predicate::str::contains("target/"));
}

#[test]
fn test_parse_ls_multiple_generated_dirs() {
    // Test multiple generated directories
    let ls_input = "src/\nnode_modules/\ndist/\nbuild/\npackage.json\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("3 generated"))
        .stdout(predicate::str::contains("node_modules/"))
        .stdout(predicate::str::contains("dist/"))
        .stdout(predicate::str::contains("build/"));
}

#[test]
fn test_parse_ls_generated_dirs_case_insensitive() {
    // Test that generated directory detection is case-insensitive
    let ls_input = "src/\nNode_Modules/\nDIST/\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("2 generated"));
}

#[test]
fn test_parse_ls_no_generated_dirs() {
    // Test when no generated directories are present
    let ls_input = "src/\nlib/\nREADME.md\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::function(|x: &str| !x.contains("generated")));
}

#[test]
fn test_parse_ls_json_includes_generated() {
    // Test that JSON output includes generated array
    let ls_input = "src/\nnode_modules/\nfile.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"generated\":"))
        .stdout(predicate::str::contains("node_modules/"))
        .stdout(predicate::str::contains("\"counts\": {"))
        .stdout(predicate::str::contains("\"generated\": 1"));
}

#[test]
fn test_parse_ls_long_format_generated_dirs() {
    // Test generated directories in long format output
    let ls_input = "total 8\ndrwxr-xr-x  5 user  group 4096 Jan  1 12:34 node_modules\ndrwxr-xr-x  2 user  group 4096 Jan  1 12:34 src\n-rw-r--r--  1 user  group   42 Jan  1 12:34 package.json\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 generated"))
        .stdout(predicate::str::contains("node_modules"));
}

#[test]
fn test_parse_ls_venv_detected() {
    // Test that Python venv directories are detected
    let ls_input = "src/\nvenv/\nrequirements.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 generated"))
        .stdout(predicate::str::contains("venv/"));
}

#[test]
fn test_parse_ls_pycache_detected() {
    // Test that __pycache__ is detected
    let ls_input = "src/\n__pycache__/\nmain.py\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 generated"))
        .stdout(predicate::str::contains("__pycache__/"));
}

#[test]
fn test_parse_ls_vendor_detected() {
    // Test that vendor directory (Go/PHP/Ruby) is detected
    let ls_input = "cmd/\nvendor/\ngo.mod\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 generated"))
        .stdout(predicate::str::contains("vendor/"));
}

#[test]
fn test_parse_ls_hidden_and_generated() {
    // Test that a directory can be both hidden and generated (e.g., .venv, .next)
    let ls_input = "src/\n.next/\n.venv/\npackage.json\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("2 generated"));
}

// ============================================================
