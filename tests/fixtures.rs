//! Git status test fixtures module.
//!
//! This module provides access to various git status output fixtures
//! for testing the git status parser.

use std::path::PathBuf;

/// Returns the path to the fixtures directory.
pub fn fixtures_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures");
    path
}

/// Loads a fixture file by name and returns its contents.
///
/// # Panics
///
/// Panics if the fixture file cannot be read.
pub fn load_fixture(name: &str) -> String {
    let path = fixtures_dir().join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture '{}': {}", name, e))
}

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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(content.contains("árbol de trabajo limpio"));
    }

    #[test]
    fn test_load_fixture_german_clean() {
        let content = git_status_german_clean();
        assert!(content.contains("Auf Branch"));
        assert!(content.contains("Arbeitsverzeichnis unverändert"));
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
}
