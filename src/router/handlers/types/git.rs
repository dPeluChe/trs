//! Git-related data structures for command handlers.

// ============================================================
// Git Status Data Structures
// ============================================================

/// Section of git status output being parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitStatusSection {
    /// Not in any specific section.
    None,
    /// Staged changes section.
    Staged,
    /// Unstaged changes section.
    Unstaged,
    /// Untracked files section.
    Untracked,
    /// Unmerged paths section.
    Unmerged,
}

/// A single file entry in git status.
#[derive(Debug, Clone, Default)]
pub(crate) struct GitStatusEntry {
    /// Status code (e.g., "M", "A", "D", "??").
    pub(crate) status: String,
    /// Path to the file.
    pub(crate) path: String,
    /// Original path for renamed files.
    pub(crate) new_path: Option<String>,
}

/// Parsed git status output.
#[derive(Debug, Clone, Default)]
pub(crate) struct GitStatus {
    /// Current branch name.
    pub(crate) branch: String,
    /// Whether the working tree is clean.
    pub(crate) is_clean: bool,
    /// Number of commits ahead of upstream.
    pub(crate) ahead: Option<usize>,
    /// Number of commits behind upstream.
    pub(crate) behind: Option<usize>,
    /// Staged changes (to be committed).
    pub(crate) staged: Vec<GitStatusEntry>,
    /// Unstaged changes (not staged for commit).
    pub(crate) unstaged: Vec<GitStatusEntry>,
    /// Untracked files.
    pub(crate) untracked: Vec<GitStatusEntry>,
    /// Unmerged paths (merge conflicts).
    pub(crate) unmerged: Vec<GitStatusEntry>,
    /// Number of staged files.
    pub(crate) staged_count: usize,
    /// Number of unstaged files.
    pub(crate) unstaged_count: usize,
    /// Number of untracked files.
    pub(crate) untracked_count: usize,
    /// Number of unmerged files.
    pub(crate) unmerged_count: usize,
}

// ============================================================
// Git Diff Data Structures
// ============================================================

/// A single hunk in a git diff file entry.
#[derive(Debug, Clone, Default)]
pub(crate) struct GitDiffHunk {
    /// The hunk header line (e.g., "@@ -10,6 +10,8 @@ fn main()").
    pub(crate) header: String,
    /// Lines within the hunk: context (starts with ' '), additions ('+'), deletions ('-').
    pub(crate) lines: Vec<String>,
}

/// A single file entry in git diff output.
#[derive(Debug, Clone, Default)]
pub(crate) struct GitDiffEntry {
    /// Path to the file (new path for renamed files).
    pub(crate) path: String,
    /// Original path for renamed files.
    pub(crate) new_path: Option<String>,
    /// Change type (M=modified, A=added, D=deleted, R=renamed, C=copied).
    pub(crate) change_type: String,
    /// Number of lines added.
    pub(crate) additions: usize,
    /// Number of lines deleted.
    pub(crate) deletions: usize,
    /// Binary file flag.
    pub(crate) is_binary: bool,
    /// Parsed hunks with their lines (for compact output with context compression).
    pub(crate) hunks: Vec<GitDiffHunk>,
}

/// Parsed git diff output.
#[derive(Debug, Clone, Default)]
pub(crate) struct GitDiff {
    /// List of file entries (limited if truncated).
    pub(crate) files: Vec<GitDiffEntry>,
    /// Total lines added across all files.
    pub(crate) total_additions: usize,
    /// Total lines deleted across all files.
    pub(crate) total_deletions: usize,
    /// Whether the diff is empty.
    pub(crate) is_empty: bool,
    /// Whether the output was truncated.
    pub(crate) is_truncated: bool,
    /// Total number of files available before truncation.
    pub(crate) total_files: usize,
    /// Number of files shown after truncation.
    pub(crate) files_shown: usize,
}
