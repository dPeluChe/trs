//! Validation tests for fixture loader functions.
//!
//! These tests verify that all fixture files exist, load correctly,
//! and contain expected content markers.

mod fixtures;
use fixtures::*;

// ============================================================
// Git Status Fixture Tests
// ============================================================

#[test]
fn test_fixtures_dir_exists() {
    let dir = fixtures_dir();
    assert!(dir.exists(), "Fixtures directory should exist: {:?}", dir);
}

#[test]
fn test_load_fixture_clean() {
    let content = git_status_clean();
    assert!(content.contains("On branch main"));
    assert!(content.contains("working tree clean"));
}

#[test]
fn test_load_fixture_staged() {
    let content = git_status_staged();
    assert!(content.contains("Changes to be committed"));
    assert!(content.contains("modified:"));
    assert!(content.contains("new file:"));
    assert!(content.contains("deleted:"));
}

#[test]
fn test_load_fixture_unstaged() {
    let content = git_status_unstaged();
    assert!(content.contains("Changes not staged for commit"));
    assert!(content.contains("modified:"));
    assert!(content.contains("deleted:"));
}

#[test]
fn test_load_fixture_untracked() {
    let content = git_status_untracked();
    assert!(content.contains("Untracked files"));
    assert!(content.contains("new_feature.rs"));
}

#[test]
fn test_load_fixture_mixed() {
    let content = git_status_mixed();
    assert!(content.contains("Changes to be committed"));
    assert!(content.contains("Changes not staged for commit"));
    assert!(content.contains("Untracked files"));
}

#[test]
fn test_load_fixture_ahead() {
    let content = git_status_ahead();
    assert!(content.contains("ahead of"));
    assert!(content.contains("by 3 commits"));
}

#[test]
fn test_load_fixture_behind() {
    let content = git_status_behind();
    assert!(content.contains("behind"));
    assert!(content.contains("by 5 commits"));
}

#[test]
fn test_load_fixture_diverged() {
    let content = git_status_diverged();
    assert!(content.contains("diverged"));
    assert!(content.contains("3 and 5 different commits"));
}

#[test]
fn test_load_fixture_detached() {
    let content = git_status_detached();
    assert!(content.contains("HEAD detached at"));
}

#[test]
fn test_load_fixture_renamed() {
    let content = git_status_renamed();
    assert!(content.contains("renamed:"));
    assert!(content.contains("->"));
}

#[test]
fn test_load_fixture_conflict() {
    let content = git_status_conflict();
    assert!(content.contains("Unmerged paths"));
    assert!(content.contains("both modified:"));
    assert!(content.contains("both added:"));
}

#[test]
fn test_load_fixture_porcelain() {
    let content = git_status_porcelain();
    assert!(content.contains(" M "));
    assert!(content.contains("A  "));
    assert!(content.contains("?? "));
}

#[test]
fn test_load_fixture_porcelain_v2() {
    let content = git_status_porcelain_v2();
    assert!(content.contains("# branch.head"));
    assert!(content.contains("# branch.ab"));
}

#[test]
fn test_load_fixture_copied() {
    let content = git_status_copied();
    assert!(content.contains("copied:"));
}

#[test]
fn test_load_fixture_typechange() {
    let content = git_status_typechange();
    assert!(content.contains("typechange:"));
}

#[test]
fn test_load_fixture_spanish_clean() {
    let content = git_status_spanish_clean();
    assert!(content.contains("En la rama"));
    assert!(content.contains("\u{00E1}rbol de trabajo limpio"));
}

#[test]
fn test_load_fixture_german_clean() {
    let content = git_status_german_clean();
    assert!(content.contains("Auf Branch"));
    assert!(content.contains("Arbeitsverzeichnis unver\u{00E4}ndert"));
}

#[test]
fn test_load_fixture_empty() {
    let content = git_status_empty();
    assert!(content.is_empty());
}

#[test]
fn test_load_fixture_whitespace_only() {
    let content = git_status_whitespace_only();
    assert!(content.trim().is_empty());
}

#[test]
fn test_load_fixture_no_branch() {
    let content = git_status_no_branch();
    assert!(content.contains("Initial commit"));
}

#[test]
fn test_load_fixture_long_paths() {
    let content = git_status_long_paths();
    assert!(content.contains("very/deeply/nested"));
    assert!(content.contains("path/with spaces"));
}

#[test]
fn test_load_fixture_all_status_codes() {
    let content = git_status_all_status_codes();
    assert!(content.contains("new file:"));
    assert!(content.contains("modified:"));
    assert!(content.contains("deleted:"));
    assert!(content.contains("renamed:"));
    assert!(content.contains("copied:"));
    assert!(content.contains("typechange:"));
    assert!(content.contains("both modified:"));
}

// ============================================================
// Git Diff Fixture Tests
// ============================================================

#[test]
fn test_load_git_diff_empty() {
    let content = git_diff_empty();
    assert!(content.trim().is_empty());
}

#[test]
fn test_load_git_diff_modified() {
    let content = git_diff_modified();
    assert!(content.contains("diff --git"));
    assert!(content.contains("src/main.rs"));
    assert!(content.contains("let x = 1"));
    assert!(content.contains("let y = 2"));
}

#[test]
fn test_load_git_diff_added() {
    let content = git_diff_added();
    assert!(content.contains("diff --git"));
    assert!(content.contains("new file mode"));
    assert!(content.contains("src/utils.rs"));
    assert!(content.contains("+pub fn helper()"));
}

#[test]
fn test_load_git_diff_deleted() {
    let content = git_diff_deleted();
    assert!(content.contains("diff --git"));
    assert!(content.contains("deleted file mode"));
    assert!(content.contains("src/deprecated.rs"));
    assert!(content.contains("-pub fn old_function()"));
}

#[test]
fn test_load_git_diff_renamed() {
    let content = git_diff_renamed();
    assert!(content.contains("diff --git"));
    assert!(content.contains("rename from"));
    assert!(content.contains("rename to"));
    assert!(content.contains("src/old_name.rs"));
    assert!(content.contains("src/new_name.rs"));
}

#[test]
fn test_load_git_diff_copied() {
    let content = git_diff_copied();
    assert!(content.contains("diff --git"));
    assert!(content.contains("copy from"));
    assert!(content.contains("copy to"));
    assert!(content.contains("src/template.rs"));
    assert!(content.contains("src/implementation.rs"));
}

#[test]
fn test_load_git_diff_binary() {
    let content = git_diff_binary();
    assert!(content.contains("diff --git"));
    assert!(content.contains("Binary files"));
    assert!(content.contains("differ"));
    assert!(content.contains("assets/image.png"));
}

#[test]
fn test_load_git_diff_multiple() {
    let content = git_diff_multiple();
    assert!(content.contains("src/main.rs"));
    assert!(content.contains("src/utils.rs"));
    assert!(content.contains("src/old.rs"));
    // Check for different change types
    assert!(content.contains("new file mode"));
    assert!(content.contains("deleted file mode"));
}

#[test]
fn test_load_git_diff_mixed() {
    let content = git_diff_mixed();
    // Check for multiple files
    assert!(content.contains("src/main.rs"));
    assert!(content.contains("src/lib.rs"));
    assert!(content.contains("src/utils.rs"));
    assert!(content.contains("src/deprecated.rs"));
    assert!(content.contains("assets/logo.png"));
    // Check for binary diff
    assert!(content.contains("Binary files"));
}

#[test]
fn test_load_git_diff_large() {
    let content = git_diff_large();
    // Check that we have 10 files
    assert!(content.contains("src/file01.rs"));
    assert!(content.contains("src/file10.rs"));
    // All should be new files
    let new_file_count = content.matches("new file mode").count();
    assert_eq!(new_file_count, 10);
}

#[test]
fn test_load_git_diff_long_paths() {
    let content = git_diff_long_paths();
    assert!(content.contains("very/deeply/nested"));
    assert!(content.contains("file with spaces"));
    assert!(content.contains("special chars"));
}

// ============================================================
// Ls Fixture Tests
// ============================================================

#[test]
fn test_load_ls_empty() {
    let content = ls_empty();
    assert!(content.is_empty());
}

#[test]
fn test_load_ls_simple() {
    let content = ls_simple();
    assert!(content.contains("src"));
    assert!(content.contains("Cargo.toml"));
    assert!(content.contains("README.md"));
}

#[test]
fn test_load_ls_with_directories() {
    let content = ls_with_directories();
    assert!(content.contains("src"));
    assert!(content.contains("tests"));
    assert!(content.contains("target"));
    assert!(content.contains("node_modules"));
}

#[test]
fn test_load_ls_with_hidden() {
    let content = ls_with_hidden();
    assert!(content.contains(".git"));
    assert!(content.contains(".gitignore"));
    assert!(content.contains(".cargo"));
    assert!(content.contains(".hidden_file"));
}

#[test]
fn test_load_ls_long_format() {
    let content = ls_long_format();
    assert!(content.contains("total 32"));
    assert!(content.contains("drwxr-xr-x"));
    assert!(content.contains("-rw-r--r--"));
}

#[test]
fn test_load_ls_long_format_with_symlinks() {
    let content = ls_long_format_with_symlinks();
    assert!(content.contains("lrwxr-xr-x"));
    assert!(content.contains("->"));
    assert!(content.contains("link_to_src"));
    assert!(content.contains("link_to_file"));
}

#[test]
fn test_load_ls_broken_symlink() {
    let content = ls_broken_symlink();
    assert!(content.contains("broken_link"));
    assert!(content.contains("old_link"));
    assert!(content.contains("->"));
}

#[test]
fn test_load_ls_permission_denied() {
    let content = ls_permission_denied();
    assert!(content.contains("ls:"));
    assert!(content.contains("Permission denied"));
    assert!(content.contains("No such file or directory"));
}

#[test]
fn test_load_ls_mixed() {
    let content = ls_mixed();
    assert!(content.contains("total 48"));
    assert!(content.contains("drwxr-xr-x"));
    assert!(content.contains("lrwxr-xr-x"));
    assert!(content.contains("-rw-r--r--"));
    assert!(content.contains(".git"));
    assert!(content.contains("ls:"));
    assert!(content.contains("Permission denied"));
}

#[test]
fn test_load_ls_generated_dirs() {
    let content = ls_generated_dirs();
    assert!(content.contains("node_modules"));
    assert!(content.contains("target"));
    assert!(content.contains("dist"));
    assert!(content.contains("build"));
    assert!(content.contains("__pycache__"));
}

#[test]
fn test_load_ls_special_chars() {
    let content = ls_special_chars();
    assert!(content.contains("file with spaces.txt"));
    assert!(content.contains("special[1].txt"));
    assert!(content.contains("bracket(2).txt"));
    assert!(content.contains("unicode_\u{00F1}ame.txt"));
}

#[test]
fn test_load_ls_long_paths() {
    let content = ls_long_paths();
    assert!(content.contains("very/deeply/nested"));
    assert!(content.contains("another/long/path"));
    assert!(content.contains("project/submodule/src"));
}

