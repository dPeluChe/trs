//! Git-related schema types.

use serde::{Deserialize, Serialize};

use super::SchemaVersion;

// ============================================================
// Git Status Schema
// ============================================================

/// Schema for git status output.
///
/// Represents the state of a git working directory including
/// branch information, staged/unstaged changes, and untracked files.
///
/// # Example JSON
///
/// ```json
/// {
///   "schema": { "version": "1.0.0", "type": "git_status" },
///   "branch": "main",
///   "is_clean": false,
///   "ahead": 2,
///   "behind": null,
///   "staged": [
///     { "status": "M", "path": "src/main.rs", "old_path": null }
///   ],
///   "unstaged": [],
///   "untracked": ["new_file.txt"],
///   "unmerged": [],
///   "counts": {
///     "staged": 1,
///     "unstaged": 0,
///     "untracked": 1,
///     "unmerged": 0
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitStatusSchema {
    /// Schema version information.
    pub schema: SchemaVersion,
    /// Current branch name.
    pub branch: String,
    /// Whether the working tree is clean.
    pub is_clean: bool,
    /// Number of commits ahead of upstream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ahead: Option<usize>,
    /// Number of commits behind upstream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behind: Option<usize>,
    /// Staged changes (to be committed).
    #[serde(default)]
    pub staged: Vec<GitFileEntry>,
    /// Unstaged changes (not staged for commit).
    #[serde(default)]
    pub unstaged: Vec<GitFileEntry>,
    /// Untracked files.
    #[serde(default)]
    pub untracked: Vec<GitFileEntry>,
    /// Unmerged paths (merge conflicts).
    #[serde(default)]
    pub unmerged: Vec<GitFileEntry>,
    /// Count summary.
    pub counts: GitStatusCounts,
}

impl GitStatusSchema {
    /// Create a new git status schema.
    pub fn new(branch: &str) -> Self {
        Self {
            schema: SchemaVersion::new("git_status"),
            branch: branch.to_string(),
            is_clean: true,
            ahead: None,
            behind: None,
            staged: Vec::new(),
            unstaged: Vec::new(),
            untracked: Vec::new(),
            unmerged: Vec::new(),
            counts: GitStatusCounts::default(),
        }
    }
}

/// A single file entry in git status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitFileEntry {
    /// Status code (e.g., "M" for modified, "A" for added, "D" for deleted, "R" for renamed).
    pub status: String,
    /// Path to the file.
    pub path: String,
    /// Original path for renamed files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_path: Option<String>,
}

impl GitFileEntry {
    /// Create a new file entry.
    pub fn new(status: &str, path: &str) -> Self {
        Self {
            status: status.to_string(),
            path: path.to_string(),
            old_path: None,
        }
    }

    /// Create a renamed file entry.
    pub fn renamed(status: &str, old_path: &str, new_path: &str) -> Self {
        Self {
            status: status.to_string(),
            path: new_path.to_string(),
            old_path: Some(old_path.to_string()),
        }
    }
}

/// Count summary for git status.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitStatusCounts {
    /// Number of staged files.
    pub staged: usize,
    /// Number of unstaged files.
    pub unstaged: usize,
    /// Number of untracked files.
    pub untracked: usize,
    /// Number of unmerged files.
    pub unmerged: usize,
}

// ============================================================
// Git Diff Schema
// ============================================================

/// Schema for git diff output.
///
/// Represents the differences between commits, commit and working tree, etc.
///
/// # Example JSON
///
/// ```json
/// {
///   "schema": { "version": "1.0.0", "type": "git_diff" },
///   "is_empty": false,
///   "is_truncated": false,
///   "files": [
///     {
///       "path": "src/main.rs",
///       "old_path": null,
///       "change_type": "M",
///       "additions": 10,
///       "deletions": 5,
///       "is_binary": false
///     }
///   ],
///   "total_additions": 10,
///   "total_deletions": 5,
///   "counts": {
///     "total_files": 1,
///     "files_shown": 1
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitDiffSchema {
    /// Schema version information.
    pub schema: SchemaVersion,
    /// Whether the diff is empty.
    pub is_empty: bool,
    /// Whether the output was truncated.
    #[serde(default)]
    pub is_truncated: bool,
    /// List of file entries (limited if truncated).
    #[serde(default)]
    pub files: Vec<GitDiffEntry>,
    /// Total lines added across all files.
    #[serde(default)]
    pub total_additions: usize,
    /// Total lines deleted across all files.
    #[serde(default)]
    pub total_deletions: usize,
    /// Count summary.
    pub counts: GitDiffCounts,
}

impl GitDiffSchema {
    /// Create a new git diff schema.
    pub fn new() -> Self {
        Self {
            schema: SchemaVersion::new("git_diff"),
            is_empty: true,
            is_truncated: false,
            files: Vec::new(),
            total_additions: 0,
            total_deletions: 0,
            counts: GitDiffCounts::default(),
        }
    }
}

impl Default for GitDiffSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// A single file entry in git diff.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitDiffEntry {
    /// Path to the file (new path for renamed files).
    pub path: String,
    /// Original path for renamed files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_path: Option<String>,
    /// Change type: "M" (modified), "A" (added), "D" (deleted), "R" (renamed), "C" (copied).
    pub change_type: String,
    /// Number of lines added.
    #[serde(default)]
    pub additions: usize,
    /// Number of lines deleted.
    #[serde(default)]
    pub deletions: usize,
    /// Whether this is a binary file.
    #[serde(default)]
    pub is_binary: bool,
}

impl GitDiffEntry {
    /// Create a new diff entry.
    pub fn new(path: &str, change_type: &str) -> Self {
        Self {
            path: path.to_string(),
            old_path: None,
            change_type: change_type.to_string(),
            additions: 0,
            deletions: 0,
            is_binary: false,
        }
    }
}

/// Count summary for git diff.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitDiffCounts {
    /// Total number of files before truncation.
    #[serde(default)]
    pub total_files: usize,
    /// Number of files shown (after truncation).
    #[serde(default)]
    pub files_shown: usize,
}

// ============================================================
// Repository State Schema
// ============================================================

/// Schema for repository state (is-clean command).
///
/// # Example JSON
///
/// ```json
/// {
///   "schema": { "version": "1.0.0", "type": "repository_state" },
///   "is_git_repo": true,
///   "is_clean": false,
///   "is_detached": false,
///   "branch": "feature/test",
///   "counts": {
///     "staged": 1,
///     "unstaged": 2,
///     "untracked": 3,
///     "unmerged": 0
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryStateSchema {
    /// Schema version information.
    pub schema: SchemaVersion,
    /// Whether this is a git repository.
    pub is_git_repo: bool,
    /// Whether the repository is clean (no changes).
    pub is_clean: bool,
    /// Whether the repository is in a detached HEAD state.
    #[serde(default)]
    pub is_detached: bool,
    /// The current branch name (or commit hash if detached).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Count summary.
    pub counts: GitStatusCounts,
}

impl RepositoryStateSchema {
    /// Create a new repository state schema.
    pub fn new() -> Self {
        Self {
            schema: SchemaVersion::new("repository_state"),
            is_git_repo: true,
            is_clean: true,
            is_detached: false,
            branch: None,
            counts: GitStatusCounts::default(),
        }
    }
}

impl Default for RepositoryStateSchema {
    fn default() -> Self {
        Self::new()
    }
}
