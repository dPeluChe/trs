#![allow(dead_code)]
use super::load_fixture;

// ============================================================
// Git Diff - Empty/Clean Fixtures
// ============================================================

/// Returns empty git diff output.
pub fn git_diff_empty() -> String {
    load_fixture("git_diff_empty.txt")
}

// ============================================================
// Git Diff - Basic Change Type Fixtures
// ============================================================

/// Returns git diff with a modified file.
pub fn git_diff_modified() -> String {
    load_fixture("git_diff_modified.txt")
}

/// Returns git diff with a new file added.
pub fn git_diff_added() -> String {
    load_fixture("git_diff_added.txt")
}

/// Returns git diff with a deleted file.
pub fn git_diff_deleted() -> String {
    load_fixture("git_diff_deleted.txt")
}

/// Returns git diff with a renamed file.
pub fn git_diff_renamed() -> String {
    load_fixture("git_diff_renamed.txt")
}

/// Returns git diff with a copied file.
pub fn git_diff_copied() -> String {
    load_fixture("git_diff_copied.txt")
}

// ============================================================
// Git Diff - Binary Files
// ============================================================

/// Returns git diff with a binary file.
pub fn git_diff_binary() -> String {
    load_fixture("git_diff_binary.txt")
}

// ============================================================
// Git Diff - Multiple Files
// ============================================================

/// Returns git diff with multiple files (modified, added, deleted).
pub fn git_diff_multiple() -> String {
    load_fixture("git_diff_multiple.txt")
}

/// Returns git diff with mixed changes (multiple types).
pub fn git_diff_mixed() -> String {
    load_fixture("git_diff_mixed.txt")
}

// ============================================================
// Git Diff - Edge Cases
// ============================================================

/// Returns git diff with many files (for testing truncation).
pub fn git_diff_large() -> String {
    load_fixture("git_diff_large.txt")
}

/// Returns git diff with very long file paths.
pub fn git_diff_long_paths() -> String {
    load_fixture("git_diff_long_paths.txt")
}
