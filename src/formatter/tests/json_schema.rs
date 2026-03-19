use super::*;

// ============================================================
// JsonFormatter Schema Formatting Tests
// ============================================================

#[test]
fn test_json_format_git_status_clean() {
    use crate::schema::{GitStatusCounts, GitStatusSchema};
    let mut status = GitStatusSchema::new("main");
    status.is_clean = true;
    status.counts = GitStatusCounts::default();
    let output = JsonFormatter::format_git_status(&status);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "main");
    assert_eq!(json["is_clean"], true);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "git_status");
}

#[test]
fn test_json_format_git_status_dirty() {
    use crate::schema::{GitFileEntry, GitStatusCounts, GitStatusSchema};
    let mut status = GitStatusSchema::new("feature");
    status.is_clean = false;
    status.ahead = Some(3);
    status.behind = Some(1);
    status.staged.push(GitFileEntry::new("M", "src/main.rs"));
    status.unstaged.push(GitFileEntry::new("M", "src/lib.rs"));
    status
        .untracked
        .push(GitFileEntry::new("??", "new_file.txt"));
    status.counts = GitStatusCounts {
        staged: 1,
        unstaged: 1,
        untracked: 1,
        unmerged: 0,
    };
    let output = JsonFormatter::format_git_status(&status);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "feature");
    assert_eq!(json["is_clean"], false);
    assert_eq!(json["ahead"], 3);
    assert_eq!(json["behind"], 1);
    assert!(json["staged"].is_array());
    assert!(json["unstaged"].is_array());
    assert!(json["untracked"].is_array());
    assert_eq!(json["counts"]["staged"], 1);
    assert_eq!(json["counts"]["unstaged"], 1);
    assert_eq!(json["counts"]["untracked"], 1);
}

#[test]
fn test_json_format_git_status_renamed() {
    use crate::schema::{GitFileEntry, GitStatusSchema};
    let mut status = GitStatusSchema::new("main");
    status.is_clean = false;
    status
        .staged
        .push(GitFileEntry::renamed("R", "old.rs", "new.rs"));
    status.counts.staged = 1;
    let output = JsonFormatter::format_git_status(&status);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["staged"][0]["status"], "R");
    assert_eq!(json["staged"][0]["path"], "new.rs");
    assert_eq!(json["staged"][0]["old_path"], "old.rs");
}

#[test]
fn test_json_format_git_diff_empty() {
    use crate::schema::GitDiffSchema;
    let diff = GitDiffSchema::new();
    let output = JsonFormatter::format_git_diff(&diff);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], true);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "git_diff");
}

#[test]
fn test_json_format_git_diff_with_files() {
    use crate::schema::{GitDiffCounts, GitDiffEntry, GitDiffSchema};
    let mut diff = GitDiffSchema::new();
    diff.is_empty = false;
    let mut entry = GitDiffEntry::new("src/main.rs", "M");
    entry.additions = 10;
    entry.deletions = 5;
    diff.files.push(entry);
    diff.total_additions = 10;
    diff.total_deletions = 5;
    diff.counts = GitDiffCounts {
        total_files: 1,
        files_shown: 1,
    };
    let output = JsonFormatter::format_git_diff(&diff);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], false);
    assert!(json["files"].is_array());
    assert_eq!(json["files"][0]["path"], "src/main.rs");
    assert_eq!(json["files"][0]["change_type"], "M");
    assert_eq!(json["files"][0]["additions"], 10);
    assert_eq!(json["files"][0]["deletions"], 5);
    assert_eq!(json["total_additions"], 10);
    assert_eq!(json["total_deletions"], 5);
}

#[test]
fn test_json_format_git_diff_truncated() {
    use crate::schema::{GitDiffCounts, GitDiffEntry, GitDiffSchema};
    let mut diff = GitDiffSchema::new();
    diff.is_empty = false;
    diff.is_truncated = true;
    let mut entry = GitDiffEntry::new("src/main.rs", "M");
    entry.additions = 10;
    entry.deletions = 5;
    diff.files.push(entry);
    diff.total_additions = 10;
    diff.total_deletions = 5;
    diff.counts = GitDiffCounts {
        total_files: 10,
        files_shown: 1,
    };
    let output = JsonFormatter::format_git_diff(&diff);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_truncated"], true);
    assert_eq!(json["counts"]["total_files"], 10);
    assert_eq!(json["counts"]["files_shown"], 1);
}

#[test]
fn test_json_format_ls_empty() {
    use crate::schema::LsOutputSchema;
    let ls = LsOutputSchema::new();
    let output = JsonFormatter::format_ls(&ls);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], true);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "ls_output");
}

#[test]
fn test_json_format_ls_with_entries() {
    use crate::schema::{LsCounts, LsOutputSchema};
    let mut ls = LsOutputSchema::new();
    ls.is_empty = false;
    ls.directories.push("src".to_string());
    ls.files.push("main.rs".to_string());
    ls.hidden.push(".gitignore".to_string());
    ls.counts = LsCounts {
        total: 3,
        directories: 1,
        files: 1,
        symlinks: 0,
        hidden: 1,
        generated: 0,
    };
    let output = JsonFormatter::format_ls(&ls);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], false);
    assert!(json["directories"].is_array());
    assert!(json["files"].is_array());
    assert!(json["hidden"].is_array());
    assert_eq!(json["counts"]["directories"], 1);
    assert_eq!(json["counts"]["files"], 1);
    assert_eq!(json["counts"]["hidden"], 1);
}

#[test]
fn test_json_format_ls_with_symlinks() {
    use crate::schema::{LsCounts, LsEntry, LsEntryType, LsOutputSchema};
    let mut ls = LsOutputSchema::new();
    ls.is_empty = false;
    let mut entry = LsEntry::new("link", LsEntryType::Symlink);
    entry.symlink_target = Some("target".to_string());
    entry.is_broken_symlink = false;
    ls.entries.push(entry);
    ls.symlinks.push("link".to_string());
    ls.counts = LsCounts {
        total: 1,
        directories: 0,
        files: 0,
        symlinks: 1,
        hidden: 0,
        generated: 0,
    };
    let output = JsonFormatter::format_ls(&ls);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(json["symlinks"].is_array());
    assert!(json["entries"][0]["symlink_target"].is_string());
}

#[test]
fn test_json_format_ls_broken_symlink() {
    use crate::schema::{LsCounts, LsEntry, LsEntryType, LsOutputSchema};
    let mut ls = LsOutputSchema::new();
    ls.is_empty = false;
    let mut entry = LsEntry::new("broken_link", LsEntryType::Symlink);
    entry.symlink_target = Some("missing".to_string());
    entry.is_broken_symlink = true;
    ls.entries.push(entry);
    ls.symlinks.push("broken_link".to_string());
    ls.counts = LsCounts {
        total: 1,
        directories: 0,
        files: 0,
        symlinks: 1,
        hidden: 0,
        generated: 0,
    };
    let output = JsonFormatter::format_ls(&ls);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["entries"][0]["is_broken_symlink"], true);
}

#[test]
fn test_json_format_grep_empty() {
    use crate::schema::GrepOutputSchema;
    let grep = GrepOutputSchema::new();
    let output = JsonFormatter::format_grep(&grep);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], true);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "grep_output");
}

#[test]
fn test_json_format_grep_with_matches() {
    use crate::schema::{GrepCounts, GrepFile, GrepMatch, GrepOutputSchema};
    let mut grep = GrepOutputSchema::new();
    grep.is_empty = false;
    let mut file = GrepFile::new("src/main.rs");
    let mut m = GrepMatch::new("fn main()");
    m.line_number = Some(10);
    file.matches.push(m);
    grep.files.push(file);
    grep.counts = GrepCounts {
        files: 1,
        matches: 1,
        total_files: 1,
        total_matches: 1,
        files_shown: 1,
        matches_shown: 1,
    };
    let output = JsonFormatter::format_grep(&grep);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], false);
    assert!(json["files"].is_array());
    assert_eq!(json["files"][0]["path"], "src/main.rs");
    assert_eq!(json["files"][0]["matches"][0]["line"], "fn main()");
    assert_eq!(json["files"][0]["matches"][0]["line_number"], 10);
    assert_eq!(json["counts"]["files"], 1);
    assert_eq!(json["counts"]["matches"], 1);
}

#[test]
fn test_json_format_grep_truncated() {
    use crate::schema::{GrepCounts, GrepFile, GrepMatch, GrepOutputSchema};
    let mut grep = GrepOutputSchema::new();
    grep.is_empty = false;
    grep.is_truncated = true;
    let mut file = GrepFile::new("src/main.rs");
    let mut m = GrepMatch::new("fn main()");
    m.line_number = Some(10);
    file.matches.push(m);
    grep.files.push(file);
    grep.counts = GrepCounts {
        files: 1,
        matches: 1,
        total_files: 5,
        total_matches: 10,
        files_shown: 1,
        matches_shown: 1,
    };
    let output = JsonFormatter::format_grep(&grep);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_truncated"], true);
    assert_eq!(json["counts"]["total_files"], 5);
    assert_eq!(json["counts"]["total_matches"], 10);
}

#[test]
fn test_json_format_find_empty() {
    use crate::schema::FindOutputSchema;
    let find = FindOutputSchema::new();
    let output = JsonFormatter::format_find(&find);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], true);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "find_output");
}

#[test]
fn test_json_format_find_with_entries() {
    use crate::schema::{FindCounts, FindOutputSchema};
    let mut find = FindOutputSchema::new();
    find.is_empty = false;
    find.directories.push("./src".to_string());
    find.files.push("./main.rs".to_string());
    find.counts = FindCounts {
        total: 2,
        directories: 1,
        files: 1,
    };
    let output = JsonFormatter::format_find(&find);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], false);
    assert!(json["directories"].is_array());
    assert!(json["files"].is_array());
    assert_eq!(json["counts"]["total"], 2);
    assert_eq!(json["counts"]["directories"], 1);
    assert_eq!(json["counts"]["files"], 1);
}
