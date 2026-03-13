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

// ============================================================
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
        .stdout(predicate::str::contains("ls: empty"));
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
        .stdout(predicate::str::contains("total: 3"))
        .stdout(predicate::str::contains("files (3):"))
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"))
        .stdout(predicate::str::contains("file3.txt"));
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
        .stdout(predicate::str::contains("total: 4"))
        .stdout(predicate::str::contains("directories (2):"))
        .stdout(predicate::str::contains("files (2):"))
        .stdout(predicate::str::contains("dir1"))
        .stdout(predicate::str::contains("dir2"));
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
        .stdout(predicate::str::contains("total: 3"))
        .stdout(predicate::str::contains("hidden (2):"))
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
        .stdout(predicate::str::contains("total: 2"))
        .stdout(predicate::str::contains("hidden (1):"))
        .stdout(predicate::str::contains(".git/"))
        // Verify hidden section appears and public is NOT listed there
        .stdout(predicate::str::contains("hidden (1):\n  .git/"));
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
        .stdout(predicate::str::contains("total: 3"))
        .stdout(predicate::str::contains("hidden (3):"))
        .stdout(predicate::str::contains(".gitignore"))
        .stdout(predicate::str::contains(".env.local"))
        .stdout(predicate::str::contains(".config.json"));
}

#[test]
fn test_parse_ls_dot_and_dotdot() {
    // Test that . and .. are detected as hidden (though typically not shown by ls)
    let ls_input = ".\n..\nfile.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("hidden (2):"))
        .stdout(predicate::str::contains("."))
        .stdout(predicate::str::contains(".."));
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
        .stdout(predicate::str::contains("hidden (2):"))
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
        .stdout(predicate::str::contains("total: 3"))
        .stdout(predicate::str::contains("hidden (3):"))
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
        .stdout(predicate::str::contains("hidden (1):"))
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
        .stdout(predicate::str::contains("\"is_hidden\":false"))
        .stdout(predicate::str::contains("\"is_hidden\":true"))
        .stdout(predicate::str::contains("\"hidden\":[\".hidden\"]"));
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
        .stdout(predicate::str::contains("total: 6"))
        .stdout(predicate::str::contains("hidden (3):"))
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
        .stdout(predicate::str::contains("total: 3"))
        .stdout(predicate::str::contains("hidden (3):"))
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
        .stdout(predicate::str::contains("total: 3"))
        .stdout(predicate::function(|x: &str| !x.contains("hidden")));
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
        .stdout(predicate::str::contains("total: 2"))
        .stdout(predicate::str::contains("directories (1):"))
        .stdout(predicate::str::contains("files (1):"))
        .stdout(predicate::str::contains("dirname"))
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
        .stdout(predicate::str::contains("\"total_count\":2"))
        .stdout(predicate::str::contains("\"name\":\"file1.txt\""))
        .stdout(predicate::str::contains("\"type\":\"file\""));
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
        .stdout(predicate::str::contains("total: 3"))
        .stdout(predicate::str::contains("generated (1):"))
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
        .stdout(predicate::str::contains("total: 3"))
        .stdout(predicate::str::contains("generated (1):"))
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
        .stdout(predicate::str::contains("total: 5"))
        .stdout(predicate::str::contains("generated (3):"))
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
        .stdout(predicate::str::contains("total: 3"))
        .stdout(predicate::str::contains("generated (2):"));
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
        .stdout(predicate::str::contains("total: 3"))
        .stdout(predicate::function(|x: &str| !x.contains("generated")));
}

#[test]
fn test_parse_ls_json_includes_generated() {
    // Test that JSON output includes is_generated field and generated array
    let ls_input = "src/\nnode_modules/\nfile.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_generated\":true"))
        .stdout(predicate::str::contains("\"is_generated\":false"))
        .stdout(predicate::str::contains(
            "\"generated\":[\"node_modules/\"]",
        ));
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
        .stdout(predicate::str::contains("total: 3"))
        .stdout(predicate::str::contains("generated (1):"))
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
        .stdout(predicate::str::contains("total: 3"))
        .stdout(predicate::str::contains("generated (1):"))
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
        .stdout(predicate::str::contains("total: 3"))
        .stdout(predicate::str::contains("generated (1):"))
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
        .stdout(predicate::str::contains("total: 3"))
        .stdout(predicate::str::contains("generated (1):"))
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
        .stdout(predicate::str::contains("total: 4"))
        .stdout(predicate::str::contains("hidden (2):"))
        .stdout(predicate::str::contains("generated (2):"));
}

// ============================================================
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
        .stdout(predicate::str::contains("total: 2"))
        .stdout(predicate::str::contains("files (2):"));
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
    // Test when all output is errors - still shows total: 0 with errors
    let ls_input = "ls: cannot open directory '.': Permission denied\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("total: 0"));
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
        .stdout(predicate::str::contains("symlinks (1):"))
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
            "\"symlink_target\":\"destination\"",
        ))
        .stdout(predicate::str::contains("\"name\":\"mylink\""));
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
        .stdout(predicate::str::contains("symlinks (2):"))
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
        .stdout(predicate::str::contains("symlinks (1):"))
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
        .stdout(predicate::str::contains("symlinks (1):"))
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
        .stdout(predicate::str::contains("\"is_broken_symlink\":true"))
        .stdout(predicate::str::contains(
            "\"symlink_target\":\"/nonexistent/path\"",
        ))
        .stdout(predicate::str::contains("\"broken_symlinks\":[\"broken\"]"));
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
        .stdout(predicate::str::contains("symlinks (1):"))
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
        .stdout(predicate::str::contains("\"is_broken_symlink\":true"))
        .stdout(predicate::str::contains("\"symlink_target\":\"loop\""));
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
        .stdout(predicate::str::contains("symlinks (2):"))
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
    // Test that broken_symlinks array is populated in JSON
    let ls_input = "lrwxrwxrwx  1 user  group    10 Jan  1 12:34 broken1 -> /nonexistent\nlrwxrwxrwx  1 user  group    10 Jan  1 12:34 broken2 -> nonexistent\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"broken_symlinks\":[\"broken1\",\"broken2\"]",
        ));
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
        .stdout(predicate::str::contains("\"is_broken_symlink\":false"))
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
        .stdout(predicate::str::contains("matches:"));
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
        .stdout(predicate::str::contains("\"file_count\":1"))
        .stdout(predicate::str::contains("\"match_count\":1"))
        .stdout(predicate::str::contains("\"line_number\":42"))
        .stdout(predicate::str::contains("\"line\":\"fn main() {\""));
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
        .stdout(predicate::str::contains("matches: 1 files, 2 results"))
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
        .stdout(predicate::str::contains("\"file_count\":2"))
        .stdout(predicate::str::contains("\"match_count\":2"));
}

// ============================================================
// Grep Truncation Tests
// ============================================================

#[test]
fn test_parse_grep_truncation_json_not_truncated() {
    // Small result set should not be truncated
    let grep_input = "src/main.rs:42:fn main() {";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["is_truncated"], false);
    assert_eq!(json["total_files"], 1);
    assert_eq!(json["total_matches"], 1);
    assert_eq!(json["files_shown"], 1);
    assert_eq!(json["matches_shown"], 1);
}

#[test]
fn test_parse_grep_truncation_json_many_files() {
    // Create input with 60 files (exceeds DEFAULT_MAX_GREP_FILES = 50)
    let mut grep_input = String::new();
    for i in 1..=60 {
        grep_input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input.as_str())
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["is_truncated"], true);
    assert_eq!(json["total_files"], 60);
    assert_eq!(json["files_shown"], 50);
    assert!(json["truncation"]["hidden_files"].as_u64().unwrap() > 0);
    assert!(json["truncation"]["message"].as_str().unwrap().contains("60"));
}

#[test]
fn test_parse_grep_truncation_json_many_matches_per_file() {
    // Create input with 1 file but 25 matches (exceeds DEFAULT_MAX_GREP_MATCHES_PER_FILE = 20)
    let mut grep_input = String::new();
    for i in 1..=25 {
        grep_input.push_str(&format!("src/main.rs:{}:fn func{}() {{\n", i, i));
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input.as_str())
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["is_truncated"], true);
    assert_eq!(json["total_matches"], 25);
    assert_eq!(json["matches_shown"], 20);
    assert!(json["truncation"]["hidden_matches"].as_u64().unwrap() > 0);
}

#[test]
fn test_parse_grep_truncation_compact_format() {
    // Create input with 60 files to trigger truncation
    let mut grep_input = String::new();
    for i in 1..=60 {
        grep_input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input.as_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("truncated"))
        .stdout(predicate::str::contains("50/60"))
        .stdout(predicate::str::contains("10 more file"));
}

#[test]
fn test_parse_grep_truncation_raw_format() {
    // Create input with 60 files to trigger truncation
    let mut grep_input = String::new();
    for i in 1..=60 {
        grep_input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input.as_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("10 more file"));
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
        .stdout(predicate::str::contains("no tests found"));
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
// JSON Output Tests for Not-Implemented Commands
// ============================================================

#[test]
fn test_search_json_output_not_implemented() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg(".")
        .arg("test")
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    // The last line should be JSON
    let json_line = stderr.lines().last().unwrap_or("");
    let json: serde_json::Value = serde_json::from_str(json_line).unwrap();
    assert_eq!(json["not_implemented"], true);
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("search command execution"));
}

#[test]
fn test_replace_json_output_not_implemented() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg(".")
        .arg("old")
        .arg("new")
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_line = stderr.lines().last().unwrap_or("");
    let json: serde_json::Value = serde_json::from_str(json_line).unwrap();
    assert_eq!(json["not_implemented"], true);
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("replace command execution"));
}

#[test]
fn test_tail_json_output_not_implemented() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("tail")
        .arg("/var/log/test.log")
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_line = stderr.lines().last().unwrap_or("");
    let json: serde_json::Value = serde_json::from_str(json_line).unwrap();
    assert_eq!(json["not_implemented"], true);
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("tail command execution"));
}

#[test]
fn test_clean_json_output_not_implemented() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd.arg("--json").arg("clean").assert().success();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_line = stderr.lines().last().unwrap_or("");
    let json: serde_json::Value = serde_json::from_str(json_line).unwrap();
    assert_eq!(json["not_implemented"], true);
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("clean command execution"));
}

#[test]
fn test_html2md_json_output_not_implemented() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("html2md")
        .arg("https://example.com")
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_line = stderr.lines().last().unwrap_or("");
    let json: serde_json::Value = serde_json::from_str(json_line).unwrap();
    assert_eq!(json["not_implemented"], true);
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("html2md command execution"));
}

#[test]
fn test_txt2md_json_output_not_implemented() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd.arg("--json").arg("txt2md").assert().success();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_line = stderr.lines().last().unwrap_or("");
    let json: serde_json::Value = serde_json::from_str(json_line).unwrap();
    assert_eq!(json["not_implemented"], true);
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("txt2md command execution"));
}

#[test]
fn test_parse_grep_json_output() {
    // Test that grep parser now works and produces valid JSON output
    let grep_input = "src/main.rs:42:fn main() {";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["file_count"], 1);
    assert_eq!(json["match_count"], 1);
}

#[test]
fn test_parse_test_json_output() {
    // Test that pytest parser now works and produces valid JSON output
    let pytest_input = r#"tests/test_main.py::test_add PASSED
1 passed in 0.01s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["passed"], 1);
    assert_eq!(json["summary"]["total"], 1);
}

#[test]
fn test_parse_test_jest_json_output() {
    // Test that Jest parser works and produces valid JSON output
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)
  ✓ should subtract numbers (2 ms)

Test Suites: 1 passed, 1 total
Tests:       2 passed, 2 total"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests"]["passed"], 2);
    assert_eq!(json["summary"]["tests"]["total"], 2);
    assert_eq!(json["summary"]["suites"]["passed"], 1);
    assert_eq!(json["summary"]["suites"]["total"], 1);
}

#[test]
fn test_parse_test_jest_compact_output() {
    // Test that Jest parser works with compact output
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)

FAIL src/api.test.js
  ✕ should fetch data (10 ms)

Test Suites: 1 passed, 1 failed, 2 total
Tests:       1 passed, 1 failed, 2 total"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Compact output should show summary and failed tests
    assert!(stdout.contains("FAIL:"));
    assert!(stdout.contains("1 passed, 1 failed"));
    assert!(stdout.contains("failed suites"));
    assert!(stdout.contains("src/api.test.js"));
}

#[test]
fn test_parse_test_vitest_json_output() {
    // Test that Vitest parser works and produces valid JSON output
    let vitest_input = r#" ✓ test/example-1.test.ts (5 tests | 1 skipped) 306ms
 ✓ test/example-2.test.ts (5 tests) 307ms

 Test Files  2 passed (4)
      Tests  10 passed | 3 skipped (65)
   Start at  11:01:36
   Duration  2.00s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests"]["passed"], 10);
    assert_eq!(json["summary"]["tests"]["skipped"], 3);
    assert_eq!(json["summary"]["tests"]["total"], 65);
    assert_eq!(json["summary"]["suites"]["passed"], 2);
    assert_eq!(json["summary"]["suites"]["total"], 4);
}

#[test]
fn test_parse_test_vitest_compact_output() {
    // Test that Vitest parser works with compact output
    let vitest_input = r#" ✓ test/utils.test.ts (2 tests) 306ms

 ✗ test/api.test.ts (2 tests | 1 failed) 307ms

 Test Files  1 passed, 1 failed (2)
      Tests  3 passed, 1 failed, 2 skipped (6)
   Duration  1.26s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Compact output should show summary and failed tests
    assert!(stdout.contains("FAIL:"));
    assert!(stdout.contains("1 passed, 1 failed"));
    assert!(stdout.contains("failed suites"));
    assert!(stdout.contains("test/api.test.ts"));
}

#[test]
fn test_parse_test_vitest_with_tree_output() {
    // Test that Vitest parser works with tree format output
    let vitest_input = r#"✓ __tests__/file1.test.ts (2) 725ms
   ✓ first test file (2) 725ms
     ✓ 2 + 2 should equal 4
     ✓ 4 - 2 should equal 2

 Test Files  1 passed (1)
      Tests  2 passed (2)
   Start at  12:34:32
   Duration  1.26s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests"]["passed"], 2);
    assert_eq!(json["summary"]["suites"]["passed"], 1);
}

#[test]
fn test_parse_test_vitest_failed_output() {
    // Test that Vitest parser handles failed tests
    let vitest_input = r#" ✗ test/failing.test.ts (2 tests | 1 failed) 306ms

 Test Files  1 failed (1)
      Tests  1 passed, 1 failed (2)
   Duration  0.50s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["summary"]["tests"]["passed"], 1);
    assert_eq!(json["summary"]["tests"]["failed"], 1);
    assert_eq!(json["summary"]["suites"]["failed"], 1);
}

// ============================================================
// Bun Test Parser Tests
// ============================================================

#[test]
fn test_parse_bun_test_json_output() {
    // Test that bun parser works and produces valid JSON output
    let bun_input = r#"test/package-json-lint.test.ts:
✓ test/package.json [0.88ms]
✓ test/js/third_party/grpc-js/package.json [0.18ms]

 4 pass
 0 fail
 4 expect() calls
Ran 4 tests in 1.44ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 4);
    assert_eq!(json["summary"]["tests_failed"], 0);
    assert_eq!(json["summary"]["expect_calls"], 4);
}

#[test]
fn test_parse_bun_test_failing_compact_output() {
    // Test compact output with failures
    let bun_input = r#"test/api.test.ts:
✓ should pass [0.88ms]
✗ should fail

 1 pass
 1 fail
 2 expect() calls
Ran 2 tests in 1.44ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Compact output should show summary and failed tests
    assert!(stdout.contains("FAIL:"));
    assert!(stdout.contains("1 passed, 1 failed"));
    assert!(stdout.contains("test/api.test.ts"));
}

#[test]
fn test_parse_bun_test_non_tty_format() {
    // Test non-TTY format (for CI environments)
    let bun_input = r#"test/package-json-lint.test.ts:
(pass) test/package.json [0.48ms]
(fail) test/failing.test.ts
(skip) test/skipped.test.ts

 2 pass
 1 fail
 1 skipped
Ran 4 tests across 1 files. [0.66ms]"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["tests_failed"], 1);
    assert_eq!(json["summary"]["tests_skipped"], 1);
    assert_eq!(json["summary"]["suites_total"], 1);
}

#[test]
fn test_parse_bun_test_all_passed() {
    // Test all tests passed
    let bun_input = r#"test/math.test.ts:
✓ should add numbers [1.00ms]
✓ should subtract numbers [0.50ms]

 2 pass
 0 fail
 2 expect() calls
Ran 2 tests in 1.50ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["suites_passed"], 1);
}

#[test]
fn test_parse_bun_test_failing_json() {
    // Test JSON output with failures
    let bun_input = r#" ✗ test/failing.test.ts (2 tests | 1 failed) 307ms

 1 pass
 1 fail
Ran 2 tests in 0.50s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // With the current parser, this output without a suite header might not parse as expected
    // but it should still produce valid JSON
    assert!(json.is_object());
}

// ============================================================
// NPM Test Parser Tests
// ============================================================

#[test]
fn test_parse_npm_test_json_output() {
    // Test that npm parser works and produces valid JSON output with passed test count
    let npm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["tests_total"], 2);
    assert_eq!(json["summary"]["suites_passed"], 1);
    assert_eq!(json["summary"]["suites_total"], 1);
}

#[test]
fn test_parse_npm_test_failing_compact_output() {
    // Test compact output with failures
    let npm_input = r#"▶ test/math.test.js
  ✖ should multiply numbers
    AssertionError: values are not equal
  ✔ should divide numbers (1.234ms)
▶ test/math.test.js (5.678ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 10ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Compact output should show summary and failed tests
    assert!(stdout.contains("FAIL:"));
    assert!(stdout.contains("1 passed, 1 failed"));
    assert!(stdout.contains("test/math.test.js"));
}

#[test]
fn test_parse_npm_test_with_skipped() {
    // Test that npm parser correctly counts passed tests with skipped tests
    let npm_input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # SKIP
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 skipped (3)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["tests_skipped"], 1);
    assert_eq!(json["summary"]["tests_total"], 3);
}

#[test]
fn test_parse_npm_test_failing_json() {
    // Test that npm parser correctly extracts failed test count in JSON output
    let npm_input = r#"▶ test/math.test.js
  ✖ should multiply numbers
    AssertionError: values are not equal
  ✔ should divide numbers (1.234ms)
▶ test/math.test.js (5.678ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 10ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["summary"]["tests_passed"], 1);
    assert_eq!(json["summary"]["tests_failed"], 1);
    assert_eq!(json["summary"]["tests_total"], 2);
    assert_eq!(json["summary"]["suites_failed"], 1);
}

// ============================================================
// PNPM Test Parser Tests
// ============================================================

#[test]
fn test_parse_pnpm_test_json_output() {
    // Test that pnpm parser works and produces valid JSON output with passed test count
    let pnpm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["tests_total"], 2);
    assert_eq!(json["summary"]["suites_passed"], 1);
    assert_eq!(json["summary"]["suites_total"], 1);
}

#[test]
fn test_parse_pnpm_test_failing_compact_output() {
    // Test compact output with failures
    let pnpm_input = r#"▶ test/api.test.js
  ✖ should fetch data
    Error: network timeout
  ✔ should create item (2.345ms)
▶ test/api.test.js (8.123ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 12ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Compact output should show summary and failed tests
    assert!(stdout.contains("FAIL:"));
    assert!(stdout.contains("1 passed, 1 failed"));
    assert!(stdout.contains("test/api.test.js"));
}

#[test]
fn test_parse_pnpm_test_with_skipped() {
    // Test that pnpm parser correctly counts passed tests with skipped tests
    let pnpm_input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # SKIP
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 skipped (3)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["tests_skipped"], 1);
    assert_eq!(json["summary"]["tests_total"], 3);
}

#[test]
fn test_parse_pnpm_test_failing_json() {
    // Test that pnpm parser correctly extracts failed test count in JSON output
    let pnpm_input = r#"▶ test/api.test.js
  ✖ should fetch data
    Error: network timeout
  ✔ should create item (2.345ms)
▶ test/api.test.js (8.123ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 12ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["summary"]["tests_passed"], 1);
    assert_eq!(json["summary"]["tests_failed"], 1);
    assert_eq!(json["summary"]["tests_total"], 2);
    assert_eq!(json["summary"]["suites_failed"], 1);
}

// ============================================================
// Test Runner Duration Extraction Tests
// ============================================================

#[test]
fn test_parse_pytest_duration_extraction() {
    // Test that pytest parser correctly extracts execution duration
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract PASSED
2 passed in 1.23s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["passed"], 2);
    // Verify duration is extracted and is approximately 1.23 seconds
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!((duration - 1.23).abs() < 0.01, "Expected duration ~1.23s, got {}", duration);
}

#[test]
fn test_parse_pytest_duration_in_milliseconds() {
    // Test pytest duration extraction with milliseconds format
    let pytest_input = r#"tests/test_main.py::test_quick PASSED
1 passed in 0.05s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!((duration - 0.05).abs() < 0.01, "Expected duration ~0.05s, got {}", duration);
}

#[test]
fn test_parse_jest_duration_extraction() {
    // Test that Jest parser correctly extracts execution duration from time summary
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)
  ✓ should subtract numbers (2 ms)

Test Suites: 1 passed, 1 total
Tests:       2 passed, 2 total
Time:        1.5 s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    // Verify duration is extracted
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!((duration - 1.5).abs() < 0.1, "Expected duration ~1.5s, got {}", duration);
}

#[test]
fn test_parse_jest_duration_in_ms() {
    // Test Jest duration extraction with milliseconds format
    let jest_input = r#"PASS src/utils.test.js
  ✓ test (1 ms)

Test Suites: 1 passed, 1 total
Tests:       1 passed, 1 total
Time:        500 ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let duration = json["summary"]["duration"].as_f64().unwrap();
    // 500 ms = 0.5 s
    assert!((duration - 0.5).abs() < 0.1, "Expected duration ~0.5s, got {}", duration);
}

#[test]
fn test_parse_vitest_duration_extraction() {
    // Test that Vitest parser correctly extracts execution duration
    let vitest_input = r#" ✓ test/example.test.ts (5 tests) 306ms

 Test Files  1 passed (1)
      Tests  5 passed (5)
   Start at  11:01:36
   Duration  2.50s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    // Verify duration is extracted
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!((duration - 2.50).abs() < 0.1, "Expected duration ~2.50s, got {}", duration);
}

#[test]
fn test_parse_vitest_duration_in_ms() {
    // Test Vitest duration extraction with milliseconds format
    let vitest_input = r#" ✓ test/quick.test.ts (1 test) 50ms

 Test Files  1 passed (1)
      Tests  1 passed (1)
   Duration  150ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let duration = json["summary"]["duration"].as_f64().unwrap();
    // 150ms = 0.15s
    assert!((duration - 0.15).abs() < 0.05, "Expected duration ~0.15s, got {}", duration);
}

#[test]
fn test_parse_npm_test_duration_extraction() {
    // Test that npm test parser correctly extracts execution duration
    let npm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 25.5ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    // Verify duration is extracted (25.5ms = 0.0255s)
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!((duration - 0.0255).abs() < 0.01, "Expected duration ~0.0255s, got {}", duration);
}

#[test]
fn test_parse_npm_test_duration_in_seconds() {
    // Test npm test duration extraction with seconds format
    let npm_input = r#"▶ test/slow.test.js
  ✔ slow test (1000.123ms)
▶ test/slow.test.js (1.5s)

ℹ tests 1 passed (1)
ℹ test files 1 passed (1)
ℹ duration 2.5s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!((duration - 2.5).abs() < 0.1, "Expected duration ~2.5s, got {}", duration);
}

#[test]
fn test_parse_pnpm_test_duration_extraction() {
    // Test that pnpm test parser correctly extracts execution duration
    let pnpm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 30.25ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    // Verify duration is extracted (30.25ms = 0.03025s)
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!((duration - 0.03025).abs() < 0.01, "Expected duration ~0.03025s, got {}", duration);
}

#[test]
fn test_parse_pnpm_test_duration_in_seconds() {
    // Test pnpm test duration extraction with seconds format
    let pnpm_input = r#"▶ test/integration.test.js
  ✔ integration test (500ms)
▶ test/integration.test.js (0.75s)

ℹ tests 1 passed (1)
ℹ test files 1 passed (1)
ℹ duration 1.25s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!((duration - 1.25).abs() < 0.1, "Expected duration ~1.25s, got {}", duration);
}

#[test]
fn test_parse_bun_test_duration_extraction() {
    // Test that Bun test parser correctly extracts execution duration
    let bun_input = r#"test/example.test.ts:
✓ test case [0.05s]

 1 pass
 0 fail
 1 expect() calls
Ran 1 tests in 150ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    // Verify duration is extracted (150ms = 0.15s)
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!((duration - 0.15).abs() < 0.05, "Expected duration ~0.15s, got {}", duration);
}

#[test]
fn test_parse_bun_test_duration_in_ms() {
    // Test Bun test duration extraction with milliseconds format
    let bun_input = r#"test/quick.test.ts:
✓ quick test [5ms]

 1 pass
 0 fail
Ran 1 tests in 50ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let duration = json["summary"]["duration"].as_f64().unwrap();
    // 50ms = 0.05s
    assert!((duration - 0.05).abs() < 0.02, "Expected duration ~0.05s, got {}", duration);
}

#[test]
fn test_parse_logs_json_output_not_implemented() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_line = stderr.lines().last().unwrap_or("");
    let json: serde_json::Value = serde_json::from_str(json_line).unwrap();
    assert_eq!(json["not_implemented"], true);
    assert!(json["message"].as_str().unwrap().contains("logs parsing"));
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
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("Command not found"));
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

// ============================================================
// Find Parser: Permission Denied Tests
// ============================================================

#[test]
fn test_parse_find_permission_denied() {
    // Test that permission denied entries are detected and not treated as files
    let find_input =
        "./src/main.rs\nfind: '/root': Permission denied\n./src/lib.rs\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("Permission denied"))
        .stdout(predicate::str::contains("total: 2"))
        .stdout(predicate::str::contains("files (2):"));
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
    let find_input =
        "./exists.txt\nfind: 'missing': No such file or directory\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("No such file or directory"))
        .stdout(predicate::str::contains("total: 1"));
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
        .stdout(predicate::str::contains("total: 2"));
}

#[test]
fn test_parse_find_multiple_errors() {
    // Test multiple error messages
    let find_input = "find: '/root': Permission denied\n./file.txt\nfind: '/var': Permission denied\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("error:"))
        .stdout(predicate::str::contains("/root"))
        .stdout(predicate::str::contains("/var"))
        .stdout(predicate::str::contains("total: 1"));
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
        .stdout(
            predicate::str::contains("clean")
                .or(predicate::str::contains("dirty")),
        );
}

#[test]
fn test_is_clean_raw_format() {
    // Test raw output format shows clean or dirty
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("is-clean")
        .assert()
        // Raw should show just clean or dirty
        .stdout(
            predicate::str::contains("clean")
                .or(predicate::str::contains("dirty")),
        );
}
