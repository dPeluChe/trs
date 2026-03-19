use super::load_fixture;

// ============================================================
// Clean Status Fixtures
// ============================================================

/// Returns a clean git status output (no changes).
pub fn git_status_clean() -> String {
    load_fixture("git_status_clean.txt")
}

/// Returns empty git status output.
pub fn git_status_empty() -> String {
    load_fixture("git_status_empty.txt")
}

/// Returns whitespace-only git status output.
pub fn git_status_whitespace_only() -> String {
    load_fixture("git_status_whitespace_only.txt")
}

// ============================================================
// Staged Changes Fixtures
// ============================================================

/// Returns git status with staged changes only.
pub fn git_status_staged() -> String {
    load_fixture("git_status_staged.txt")
}

/// Returns git status with staged renamed files.
pub fn git_status_renamed() -> String {
    load_fixture("git_status_renamed.txt")
}

/// Returns git status with staged copied files.
pub fn git_status_copied() -> String {
    load_fixture("git_status_copied.txt")
}

// ============================================================
// Unstaged Changes Fixtures
// ============================================================

/// Returns git status with unstaged changes only.
pub fn git_status_unstaged() -> String {
    load_fixture("git_status_unstaged.txt")
}

/// Returns git status with typechange (symlink to file).
pub fn git_status_typechange() -> String {
    load_fixture("git_status_typechange.txt")
}

// ============================================================
// Untracked Files Fixtures
// ============================================================

/// Returns git status with untracked files only.
pub fn git_status_untracked() -> String {
    load_fixture("git_status_untracked.txt")
}

// ============================================================
// Mixed Changes Fixtures
// ============================================================

/// Returns git status with staged, unstaged, and untracked changes.
pub fn git_status_mixed() -> String {
    load_fixture("git_status_mixed.txt")
}

/// Returns git status with all possible status codes.
pub fn git_status_all_status_codes() -> String {
    load_fixture("git_status_all_status_codes.txt")
}

// ============================================================
// Branch Status Fixtures
// ============================================================

/// Returns git status where branch is ahead of remote.
pub fn git_status_ahead() -> String {
    load_fixture("git_status_ahead.txt")
}

/// Returns git status where branch is behind remote.
pub fn git_status_behind() -> String {
    load_fixture("git_status_behind.txt")
}

/// Returns git status where branch has diverged from remote.
pub fn git_status_diverged() -> String {
    load_fixture("git_status_diverged.txt")
}

/// Returns git status in detached HEAD state.
pub fn git_status_detached() -> String {
    load_fixture("git_status_detached.txt")
}

/// Returns git status for initial commit (no upstream).
pub fn git_status_no_branch() -> String {
    load_fixture("git_status_no_branch.txt")
}

// ============================================================
// Conflict Fixtures
// ============================================================

/// Returns git status with merge conflicts.
pub fn git_status_conflict() -> String {
    load_fixture("git_status_conflict.txt")
}

// ============================================================
// Porcelain Format Fixtures
// ============================================================

/// Returns git status in porcelain format (v1).
pub fn git_status_porcelain() -> String {
    load_fixture("git_status_porcelain.txt")
}

/// Returns git status in porcelain format (v2).
pub fn git_status_porcelain_v2() -> String {
    load_fixture("git_status_porcelain_v2.txt")
}

// ============================================================
// Localized Fixtures
// ============================================================

/// Returns git status in Spanish (clean).
pub fn git_status_spanish_clean() -> String {
    load_fixture("git_status_spanish_clean.txt")
}

/// Returns git status in Spanish (with changes).
#[allow(dead_code)]
pub fn git_status_spanish_staged() -> String {
    load_fixture("git_status_spanish_staged.txt")
}

/// Returns git status in German (clean).
pub fn git_status_german_clean() -> String {
    load_fixture("git_status_german_clean.txt")
}

// ============================================================
// Edge Cases
// ============================================================

/// Returns git status with very long file paths.
pub fn git_status_long_paths() -> String {
    load_fixture("git_status_long_paths.txt")
}
