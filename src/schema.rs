//! Stable JSON schemas for TARS CLI reducers.
//!
//! This module provides stable, versioned JSON schemas for all reducer outputs.
//! These schemas are designed to be:
//!
//! - **Stable**: Breaking changes require a major version bump
//! - **Documented**: All fields have clear descriptions
//! - **Serializable**: All types implement `Serialize` and `Deserialize`
//! - **Versioned**: Schema version is included in output for compatibility
//!
//! # Schema Categories
//!
//! - **Git**: `GitStatusSchema`, `GitDiffSchema`, `RepositoryStateSchema`
//! - **File System**: `LsOutputSchema`, `FindOutputSchema`
//! - **Search**: `GrepOutputSchema`
//! - **Test Runners**: `TestOutputSchema` (unified for all runners)
//! - **Logs**: `LogsOutputSchema`
//! - **Process**: `ProcessOutputSchema`
//!
//! # Versioning
//!
//! All schemas include a `schema_version` field. The version follows semantic versioning:
//!
//! - **Major version**: Breaking changes (field removal, type changes)
//! - **Minor version**: Additive changes (new optional fields)
//! - **Patch version**: Documentation or internal changes
//!
//! Current schema version: 1.0.0

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// Schema Version
// ============================================================

/// Current schema version for all output types.
pub const SCHEMA_VERSION: &str = "1.0.0";

/// Version information included in all schema outputs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaVersion {
    /// Schema version string (semver).
    pub version: String,
    /// Schema type identifier.
    #[serde(rename = "type")]
    pub schema_type: String,
}

impl SchemaVersion {
    /// Create a new schema version for the given type.
    pub fn new(schema_type: &str) -> Self {
        Self {
            version: SCHEMA_VERSION.to_string(),
            schema_type: schema_type.to_string(),
        }
    }
}

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

// ============================================================
// LS Output Schema
// ============================================================

/// Schema for ls command output.
///
/// # Example JSON
///
/// ```json
/// {
///   "schema": { "version": "1.0.0", "type": "ls_output" },
///   "is_empty": false,
///   "entries": [
///     { "name": "src", "type": "directory", "is_hidden": false, "is_symlink": false }
///   ],
///   "directories": ["src", "tests"],
///   "files": ["Cargo.toml"],
///   "symlinks": [],
///   "hidden": [".gitignore"],
///   "generated": ["target"],
///   "counts": {
///     "total": 5,
///     "directories": 2,
///     "files": 1,
///     "symlinks": 0,
///     "hidden": 1,
///     "generated": 1
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LsOutputSchema {
    /// Schema version information.
    pub schema: SchemaVersion,
    /// Whether the output is empty.
    pub is_empty: bool,
    /// All entries.
    #[serde(default)]
    pub entries: Vec<LsEntry>,
    /// Directory names.
    #[serde(default)]
    pub directories: Vec<String>,
    /// File names.
    #[serde(default)]
    pub files: Vec<String>,
    /// Symlink names.
    #[serde(default)]
    pub symlinks: Vec<String>,
    /// Hidden entry names.
    #[serde(default)]
    pub hidden: Vec<String>,
    /// Generated directory names (build artifacts, dependencies).
    #[serde(default)]
    pub generated: Vec<String>,
    /// Error entries (permission denied, etc.).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<LsError>,
    /// Count summary.
    pub counts: LsCounts,
}

impl LsOutputSchema {
    /// Create a new ls output schema.
    pub fn new() -> Self {
        Self {
            schema: SchemaVersion::new("ls_output"),
            is_empty: true,
            entries: Vec::new(),
            directories: Vec::new(),
            files: Vec::new(),
            symlinks: Vec::new(),
            hidden: Vec::new(),
            generated: Vec::new(),
            errors: Vec::new(),
            counts: LsCounts::default(),
        }
    }
}

impl Default for LsOutputSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Entry type for ls output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LsEntryType {
    /// Regular file.
    #[default]
    File,
    /// Directory.
    Directory,
    /// Symbolic link.
    Symlink,
    /// Block device.
    BlockDevice,
    /// Character device.
    CharDevice,
    /// Socket.
    Socket,
    /// Pipe (FIFO).
    Pipe,
    /// Unknown or other type.
    Other,
}

/// A single entry in ls output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LsEntry {
    /// Name of the file or directory.
    pub name: String,
    /// Type of entry.
    #[serde(rename = "type")]
    pub entry_type: LsEntryType,
    /// Whether this is a hidden file (starts with .).
    #[serde(default)]
    pub is_hidden: bool,
    /// Whether this is a symlink.
    #[serde(default)]
    pub is_symlink: bool,
    /// Symlink target (if this is a symlink).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symlink_target: Option<String>,
    /// Whether the symlink is broken.
    #[serde(default)]
    pub is_broken_symlink: bool,
    /// File size in bytes (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// File permissions (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<String>,
    /// Owner user name (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Owner group name (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Last modification time (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
}

impl LsEntry {
    /// Create a new ls entry.
    pub fn new(name: &str, entry_type: LsEntryType) -> Self {
        Self {
            name: name.to_string(),
            entry_type,
            is_hidden: name.starts_with('.'),
            is_symlink: entry_type == LsEntryType::Symlink,
            symlink_target: None,
            is_broken_symlink: false,
            size: None,
            permissions: None,
            owner: None,
            group: None,
            modified: None,
        }
    }
}

/// An error entry from ls output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LsError {
    /// The path that had an error.
    pub path: String,
    /// The error message.
    pub message: String,
}

/// Count summary for ls output.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LsCounts {
    /// Total count of entries (excluding errors).
    pub total: usize,
    /// Number of directories.
    pub directories: usize,
    /// Number of files.
    pub files: usize,
    /// Number of symlinks.
    pub symlinks: usize,
    /// Number of hidden entries.
    pub hidden: usize,
    /// Number of generated directories.
    pub generated: usize,
}

// ============================================================
// Find Output Schema
// ============================================================

/// Schema for find command output.
///
/// # Example JSON
///
/// ```json
/// {
///   "schema": { "version": "1.0.0", "type": "find_output" },
///   "is_empty": false,
///   "entries": [
///     { "path": "./src/main.rs", "is_directory": false, "is_hidden": false, "extension": "rs", "depth": 1 }
///   ],
///   "directories": ["./src", "./tests"],
///   "files": ["./src/main.rs", "./Cargo.toml"],
///   "hidden": ["./.gitignore"],
///   "extensions": { "rs": 2, "toml": 1 },
///   "counts": {
///     "total": 5,
///     "directories": 2,
///     "files": 3
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FindOutputSchema {
    /// Schema version information.
    pub schema: SchemaVersion,
    /// Whether the output is empty.
    pub is_empty: bool,
    /// All entries.
    #[serde(default)]
    pub entries: Vec<FindEntry>,
    /// Directory paths.
    #[serde(default)]
    pub directories: Vec<String>,
    /// File paths.
    #[serde(default)]
    pub files: Vec<String>,
    /// Hidden entry paths.
    #[serde(default)]
    pub hidden: Vec<String>,
    /// File extensions with counts.
    #[serde(default)]
    pub extensions: HashMap<String, usize>,
    /// Error entries (permission denied, etc.).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<FindError>,
    /// Count summary.
    pub counts: FindCounts,
}

impl FindOutputSchema {
    /// Create a new find output schema.
    pub fn new() -> Self {
        Self {
            schema: SchemaVersion::new("find_output"),
            is_empty: true,
            entries: Vec::new(),
            directories: Vec::new(),
            files: Vec::new(),
            hidden: Vec::new(),
            extensions: HashMap::new(),
            errors: Vec::new(),
            counts: FindCounts::default(),
        }
    }
}

impl Default for FindOutputSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// A single entry in find output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FindEntry {
    /// Path to the file or directory.
    pub path: String,
    /// Whether this is a directory.
    #[serde(default)]
    pub is_directory: bool,
    /// Whether this is a hidden file/directory.
    #[serde(default)]
    pub is_hidden: bool,
    /// File extension (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension: Option<String>,
    /// Depth of the path (number of path separators).
    #[serde(default)]
    pub depth: usize,
}

impl FindEntry {
    /// Create a new find entry.
    pub fn new(path: &str) -> Self {
        let is_hidden = path.split('/').any(|p| p.starts_with('.'));
        let depth = path.matches('/').count();
        let extension = path.rsplit('.').next().map(|s| s.to_string());

        Self {
            path: path.to_string(),
            is_directory: false,
            is_hidden,
            extension,
            depth,
        }
    }
}

/// An error entry from find output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FindError {
    /// The path that was denied access.
    pub path: String,
    /// The error message.
    pub message: String,
}

/// Count summary for find output.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FindCounts {
    /// Total count of entries (excluding errors).
    pub total: usize,
    /// Number of directories.
    pub directories: usize,
    /// Number of files.
    pub files: usize,
}

// ============================================================
// Grep Output Schema
// ============================================================

/// Schema for grep/ripgrep output.
///
/// # Example JSON
///
/// ```json
/// {
///   "schema": { "version": "1.0.0", "type": "grep_output" },
///   "is_empty": false,
///   "is_truncated": false,
///   "files": [
///     {
///       "path": "src/main.rs",
///       "matches": [
///         { "line_number": 10, "column": null, "line": "fn main() {", "is_context": false }
///       ]
///     }
///   ],
///   "counts": {
///     "files": 1,
///     "matches": 1,
///     "total_files": 1,
///     "total_matches": 1,
///     "files_shown": 1,
///     "matches_shown": 1
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrepOutputSchema {
    /// Schema version information.
    pub schema: SchemaVersion,
    /// Whether the output is empty (no matches).
    pub is_empty: bool,
    /// Whether the output was truncated.
    #[serde(default)]
    pub is_truncated: bool,
    /// List of files with matches (limited if truncated).
    #[serde(default)]
    pub files: Vec<GrepFile>,
    /// Count summary.
    pub counts: GrepCounts,
}

impl GrepOutputSchema {
    /// Create a new grep output schema.
    pub fn new() -> Self {
        Self {
            schema: SchemaVersion::new("grep_output"),
            is_empty: true,
            is_truncated: false,
            files: Vec::new(),
            counts: GrepCounts::default(),
        }
    }
}

impl Default for GrepOutputSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// A file with grep matches.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrepFile {
    /// Path to the file.
    pub path: String,
    /// List of matches in this file.
    #[serde(default)]
    pub matches: Vec<GrepMatch>,
}

impl GrepFile {
    /// Create a new grep file entry.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            matches: Vec::new(),
        }
    }
}

/// A single match in grep output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrepMatch {
    /// Line number (if available with -n flag).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<usize>,
    /// Column number (if available with --column flag).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    /// The matched line content.
    pub line: String,
    /// Whether this is a context line (not a direct match).
    #[serde(default)]
    pub is_context: bool,
}

impl GrepMatch {
    /// Create a new grep match.
    pub fn new(line: &str) -> Self {
        Self {
            line_number: None,
            column: None,
            line: line.to_string(),
            is_context: false,
        }
    }
}

/// Count summary for grep output.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrepCounts {
    /// Number of files with matches shown.
    pub files: usize,
    /// Number of matches shown.
    pub matches: usize,
    /// Total number of files before truncation.
    #[serde(default)]
    pub total_files: usize,
    /// Total number of matches before truncation.
    #[serde(default)]
    pub total_matches: usize,
    /// Number of files shown after truncation.
    #[serde(default)]
    pub files_shown: usize,
    /// Number of matches shown after truncation.
    #[serde(default)]
    pub matches_shown: usize,
}

// ============================================================
// Test Output Schema (Unified)
// ============================================================

/// Schema for test runner output (unified across all runners).
///
/// Supports: pytest, jest, vitest, npm test, pnpm test, bun test.
///
/// # Example JSON
///
/// ```json
/// {
///   "schema": { "version": "1.0.0", "type": "test_output" },
///   "runner": "pytest",
///   "is_empty": false,
///   "success": true,
///   "test_suites": [
///     {
///       "file": "tests/test_main.py",
///       "passed": true,
///       "duration_ms": 150,
///       "tests": [
///         { "name": "test_example", "status": "passed", "duration_ms": 10 }
///       ]
///     }
///   ],
///   "summary": {
///     "total": 10,
///     "passed": 8,
///     "failed": 2,
///     "skipped": 0,
///     "duration_ms": 1500
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestOutputSchema {
    /// Schema version information.
    pub schema: SchemaVersion,
    /// Test runner type.
    pub runner: TestRunnerType,
    /// Whether the output is empty.
    pub is_empty: bool,
    /// Whether all tests passed.
    pub success: bool,
    /// List of test suites (files).
    #[serde(default)]
    pub test_suites: Vec<TestSuite>,
    /// Summary statistics.
    pub summary: TestSummary,
    /// Runner version (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runner_version: Option<String>,
    /// Platform info (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    /// Working directory (rootdir for pytest, cwd for others).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
}

impl TestOutputSchema {
    /// Create a new test output schema.
    pub fn new(runner: TestRunnerType) -> Self {
        Self {
            schema: SchemaVersion::new("test_output"),
            runner,
            is_empty: true,
            success: true,
            test_suites: Vec::new(),
            summary: TestSummary::default(),
            runner_version: None,
            platform: None,
            working_directory: None,
        }
    }
}

/// Supported test runner types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestRunnerType {
    /// Python pytest.
    Pytest,
    /// JavaScript Jest.
    Jest,
    /// JavaScript Vitest.
    Vitest,
    /// npm test (Node.js built-in test runner).
    Npm,
    /// pnpm test.
    Pnpm,
    /// bun test.
    Bun,
}

impl std::fmt::Display for TestRunnerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestRunnerType::Pytest => write!(f, "pytest"),
            TestRunnerType::Jest => write!(f, "jest"),
            TestRunnerType::Vitest => write!(f, "vitest"),
            TestRunnerType::Npm => write!(f, "npm"),
            TestRunnerType::Pnpm => write!(f, "pnpm"),
            TestRunnerType::Bun => write!(f, "bun"),
        }
    }
}

/// A test suite (typically a file).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestSuite {
    /// Test file path.
    pub file: String,
    /// Whether the suite passed.
    pub passed: bool,
    /// Execution time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Number of tests in suite.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_count: Option<usize>,
    /// List of test results in this suite.
    #[serde(default)]
    pub tests: Vec<TestResult>,
}

impl TestSuite {
    /// Create a new test suite.
    pub fn new(file: &str) -> Self {
        Self {
            file: file.to_string(),
            passed: true,
            duration_ms: None,
            test_count: None,
            tests: Vec::new(),
        }
    }
}

/// A single test result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestResult {
    /// Full test name (module::test_name or file::test_name).
    pub name: String,
    /// Test name only (last part).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_name: Option<String>,
    /// Ancestor names (describe blocks).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ancestors: Vec<String>,
    /// Status of the test.
    pub status: TestStatus,
    /// Duration in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Error message (for failed tests).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// File path (if different from suite).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Line number (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

impl TestResult {
    /// Create a new test result.
    pub fn new(name: &str, status: TestStatus) -> Self {
        Self {
            name: name.to_string(),
            test_name: None,
            ancestors: Vec::new(),
            status,
            duration_ms: None,
            error_message: None,
            file: None,
            line: None,
        }
    }
}

/// Status of a test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    /// Test passed.
    Passed,
    /// Test failed.
    Failed,
    /// Test was skipped.
    Skipped,
    /// Test expected to fail (xfail).
    XFailed,
    /// Test expected to fail but passed (xpass).
    XPassed,
    /// Test encountered an error.
    Error,
    /// Test was todo.
    Todo,
}

/// Test summary statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestSummary {
    /// Total number of tests.
    pub total: usize,
    /// Number of passed tests.
    pub passed: usize,
    /// Number of failed tests.
    pub failed: usize,
    /// Number of skipped tests.
    #[serde(default)]
    pub skipped: usize,
    /// Number of xfailed tests.
    #[serde(default)]
    pub xfailed: usize,
    /// Number of xpassed tests.
    #[serde(default)]
    pub xpassed: usize,
    /// Number of error tests.
    #[serde(default)]
    pub errors: usize,
    /// Number of todo tests.
    #[serde(default)]
    pub todo: usize,
    /// Number of test suites passed.
    #[serde(default)]
    pub suites_passed: usize,
    /// Number of test suites failed.
    #[serde(default)]
    pub suites_failed: usize,
    /// Number of test suites total.
    #[serde(default)]
    pub suites_total: usize,
    /// Total duration in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

// ============================================================
// Logs Output Schema
// ============================================================

/// Schema for log/tail output.
///
/// # Example JSON
///
/// ```json
/// {
///   "schema": { "version": "1.0.0", "type": "logs_output" },
///   "is_empty": false,
///   "entries": [
///     {
///       "line": "2024-01-15 10:30:00 [INFO] Application started",
///       "level": "info",
///       "timestamp": "2024-01-15 10:30:00",
///       "source": null,
///       "message": "Application started",
///       "line_number": 1
///     }
///   ],
///   "counts": {
///     "total_lines": 100,
///     "debug": 10,
///     "info": 50,
///     "warning": 5,
///     "error": 3,
///     "fatal": 0,
///     "unknown": 32
///   },
///   "recent_critical": [],
///   "repeated_lines": []
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogsOutputSchema {
    /// Schema version information.
    pub schema: SchemaVersion,
    /// Whether the output is empty.
    pub is_empty: bool,
    /// All log entries.
    #[serde(default)]
    pub entries: Vec<LogEntry>,
    /// Count summary.
    pub counts: LogCounts,
    /// Most recent critical lines (ERROR and FATAL level entries).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent_critical: Vec<LogEntry>,
    /// Repeated lines (collapsed).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repeated_lines: Vec<RepeatedLine>,
}

impl LogsOutputSchema {
    /// Create a new logs output schema.
    pub fn new() -> Self {
        Self {
            schema: SchemaVersion::new("logs_output"),
            is_empty: true,
            entries: Vec::new(),
            counts: LogCounts::default(),
            recent_critical: Vec::new(),
            repeated_lines: Vec::new(),
        }
    }
}

impl Default for LogsOutputSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Log level classification.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Debug level.
    Debug,
    /// Info level.
    Info,
    /// Warning level.
    Warning,
    /// Error level.
    Error,
    /// Fatal/Critical level.
    Fatal,
    /// Unknown or unclassified level.
    #[default]
    Unknown,
}

/// A single parsed log line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogEntry {
    /// Original line content.
    pub line: String,
    /// Detected log level.
    pub level: LogLevel,
    /// Timestamp (if detected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Source/logger name (if detected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Message content (without timestamp/level prefix).
    pub message: String,
    /// Line number in the input.
    pub line_number: usize,
}

impl LogEntry {
    /// Create a new log entry.
    pub fn new(line: &str, line_number: usize) -> Self {
        Self {
            line: line.to_string(),
            level: LogLevel::Unknown,
            timestamp: None,
            source: None,
            message: line.to_string(),
            line_number,
        }
    }
}

/// Statistics for repeated lines.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepeatedLine {
    /// The repeated line content.
    pub line: String,
    /// Number of occurrences.
    pub count: usize,
    /// First occurrence line number.
    pub first_line: usize,
    /// Last occurrence line number.
    pub last_line: usize,
}

/// Count summary for log output.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogCounts {
    /// Total line count.
    pub total_lines: usize,
    /// Number of debug level lines.
    #[serde(default)]
    pub debug: usize,
    /// Number of info level lines.
    #[serde(default)]
    pub info: usize,
    /// Number of warning level lines.
    #[serde(default)]
    pub warning: usize,
    /// Number of error level lines.
    #[serde(default)]
    pub error: usize,
    /// Number of fatal level lines.
    #[serde(default)]
    pub fatal: usize,
    /// Number of unknown level lines.
    #[serde(default)]
    pub unknown: usize,
}

// ============================================================
// Process Output Schema
// ============================================================

/// Schema for process/command output.
///
/// # Example JSON
///
/// ```json
/// {
///   "schema": { "version": "1.0.0", "type": "process_output" },
///   "command": "echo",
///   "args": ["hello", "world"],
///   "stdout": "hello world\n",
///   "stderr": "",
///   "exit_code": 0,
///   "duration_ms": 5,
///   "timed_out": false,
///   "success": true
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessOutputSchema {
    /// Schema version information.
    pub schema: SchemaVersion,
    /// The command that was executed.
    pub command: String,
    /// Arguments passed to the command.
    #[serde(default)]
    pub args: Vec<String>,
    /// Standard output.
    #[serde(default)]
    pub stdout: String,
    /// Standard error.
    #[serde(default)]
    pub stderr: String,
    /// Exit code (if captured).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Duration in milliseconds.
    #[serde(default)]
    pub duration_ms: u64,
    /// Whether the command timed out.
    #[serde(default)]
    pub timed_out: bool,
    /// Whether the command succeeded (exit code 0).
    #[serde(default)]
    pub success: bool,
}

impl ProcessOutputSchema {
    /// Create a new process output schema.
    pub fn new(command: &str) -> Self {
        Self {
            schema: SchemaVersion::new("process_output"),
            command: command.to_string(),
            args: Vec::new(),
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            duration_ms: 0,
            timed_out: false,
            success: true,
        }
    }
}

// ============================================================
// Error Schema
// ============================================================

/// Schema for error responses.
///
/// # Example JSON
///
/// ```json
/// {
///   "schema": { "version": "1.0.0", "type": "error" },
///   "error": true,
///   "message": "Command not found: foo",
///   "error_type": "command_not_found",
///   "exit_code": 127
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorSchema {
    /// Schema version information.
    pub schema: SchemaVersion,
    /// Always true for error responses.
    pub error: bool,
    /// Human-readable error message.
    pub message: String,
    /// Error type classification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    /// Exit code (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Additional context.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub context: HashMap<String, String>,
}

impl ErrorSchema {
    /// Create a new error schema.
    pub fn new(message: &str) -> Self {
        Self {
            schema: SchemaVersion::new("error"),
            error: true,
            message: message.to_string(),
            error_type: None,
            exit_code: None,
            context: HashMap::new(),
        }
    }

    /// Create an error with a specific type.
    pub fn with_type(message: &str, error_type: &str) -> Self {
        Self {
            schema: SchemaVersion::new("error"),
            error: true,
            message: message.to_string(),
            error_type: Some(error_type.to_string()),
            exit_code: None,
            context: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Schema Version Tests
    // ============================================================

    #[test]
    fn test_schema_version() {
        let version = SchemaVersion::new("test_type");
        assert_eq!(version.version, SCHEMA_VERSION);
        assert_eq!(version.schema_type, "test_type");
    }

    #[test]
    fn test_schema_version_serialization() {
        let version = SchemaVersion::new("git_status");
        let json = serde_json::to_string(&version).unwrap();
        assert!(json.contains("\"version\":\"1.0.0\""));
        assert!(json.contains("\"type\":\"git_status\""));
    }

    // ============================================================
    // Git Status Schema Tests
    // ============================================================

    #[test]
    fn test_git_status_schema_new() {
        let schema = GitStatusSchema::new("main");
        assert_eq!(schema.branch, "main");
        assert!(schema.is_clean);
        assert!(schema.staged.is_empty());
        assert!(schema.unstaged.is_empty());
        assert!(schema.untracked.is_empty());
        assert!(schema.unmerged.is_empty());
    }

    #[test]
    fn test_git_status_schema_serialization() {
        let schema = GitStatusSchema::new("main");
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("\"branch\":\"main\""));
        assert!(json.contains("\"is_clean\":true"));
        assert!(json.contains("\"type\":\"git_status\""));
    }

    #[test]
    fn test_git_file_entry_new() {
        let entry = GitFileEntry::new("M", "src/main.rs");
        assert_eq!(entry.status, "M");
        assert_eq!(entry.path, "src/main.rs");
        assert!(entry.old_path.is_none());
    }

    #[test]
    fn test_git_file_entry_renamed() {
        let entry = GitFileEntry::renamed("R", "old.rs", "new.rs");
        assert_eq!(entry.status, "R");
        assert_eq!(entry.path, "new.rs");
        assert_eq!(entry.old_path, Some("old.rs".to_string()));
    }

    #[test]
    fn test_git_status_counts_default() {
        let counts = GitStatusCounts::default();
        assert_eq!(counts.staged, 0);
        assert_eq!(counts.unstaged, 0);
        assert_eq!(counts.untracked, 0);
        assert_eq!(counts.unmerged, 0);
    }

    // ============================================================
    // Git Diff Schema Tests
    // ============================================================

    #[test]
    fn test_git_diff_schema_new() {
        let schema = GitDiffSchema::new();
        assert!(schema.is_empty);
        assert!(!schema.is_truncated);
        assert!(schema.files.is_empty());
    }

    #[test]
    fn test_git_diff_entry_new() {
        let entry = GitDiffEntry::new("src/main.rs", "M");
        assert_eq!(entry.path, "src/main.rs");
        assert_eq!(entry.change_type, "M");
        assert!(entry.old_path.is_none());
    }

    // ============================================================
    // Repository State Schema Tests
    // ============================================================

    #[test]
    fn test_repository_state_schema_new() {
        let schema = RepositoryStateSchema::new();
        assert!(schema.is_git_repo);
        assert!(schema.is_clean);
        assert!(!schema.is_detached);
        assert!(schema.branch.is_none());
    }

    // ============================================================
    // LS Output Schema Tests
    // ============================================================

    #[test]
    fn test_ls_output_schema_new() {
        let schema = LsOutputSchema::new();
        assert!(schema.is_empty);
        assert!(schema.entries.is_empty());
        assert!(schema.directories.is_empty());
        assert!(schema.files.is_empty());
    }

    #[test]
    fn test_ls_entry_new() {
        let entry = LsEntry::new("src", LsEntryType::Directory);
        assert_eq!(entry.name, "src");
        assert_eq!(entry.entry_type, LsEntryType::Directory);
        assert!(!entry.is_hidden);
    }

    #[test]
    fn test_ls_entry_hidden() {
        let entry = LsEntry::new(".gitignore", LsEntryType::File);
        assert!(entry.is_hidden);
    }

    #[test]
    fn test_ls_entry_type_serialization() {
        let entry = LsEntry::new("link", LsEntryType::Symlink);
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"type\":\"symlink\""));
    }

    // ============================================================
    // Find Output Schema Tests
    // ============================================================

    #[test]
    fn test_find_output_schema_new() {
        let schema = FindOutputSchema::new();
        assert!(schema.is_empty);
        assert!(schema.entries.is_empty());
        assert!(schema.directories.is_empty());
        assert!(schema.files.is_empty());
    }

    #[test]
    fn test_find_entry_new() {
        let entry = FindEntry::new("./src/main.rs");
        assert_eq!(entry.path, "./src/main.rs");
        assert!(!entry.is_directory);
    }

    #[test]
    fn test_find_entry_hidden_detection() {
        let entry = FindEntry::new("./.git/config");
        assert!(entry.is_hidden);
    }

    // ============================================================
    // Grep Output Schema Tests
    // ============================================================

    #[test]
    fn test_grep_output_schema_new() {
        let schema = GrepOutputSchema::new();
        assert!(schema.is_empty);
        assert!(!schema.is_truncated);
        assert!(schema.files.is_empty());
    }

    #[test]
    fn test_grep_file_new() {
        let file = GrepFile::new("src/main.rs");
        assert_eq!(file.path, "src/main.rs");
        assert!(file.matches.is_empty());
    }

    #[test]
    fn test_grep_match_new() {
        let m = GrepMatch::new("fn main() {");
        assert_eq!(m.line, "fn main() {");
        assert!(m.line_number.is_none());
        assert!(!m.is_context);
    }

    // ============================================================
    // Test Output Schema Tests
    // ============================================================

    #[test]
    fn test_test_output_schema_new() {
        let schema = TestOutputSchema::new(TestRunnerType::Pytest);
        assert_eq!(schema.runner, TestRunnerType::Pytest);
        assert!(schema.is_empty);
        assert!(schema.success);
        assert!(schema.test_suites.is_empty());
    }

    #[test]
    fn test_test_suite_new() {
        let suite = TestSuite::new("tests/test_main.py");
        assert_eq!(suite.file, "tests/test_main.py");
        assert!(suite.passed);
        assert!(suite.tests.is_empty());
    }

    #[test]
    fn test_test_result_new() {
        let result = TestResult::new("test_example", TestStatus::Passed);
        assert_eq!(result.name, "test_example");
        assert_eq!(result.status, TestStatus::Passed);
    }

    #[test]
    fn test_test_runner_type_display() {
        assert_eq!(TestRunnerType::Pytest.to_string(), "pytest");
        assert_eq!(TestRunnerType::Jest.to_string(), "jest");
        assert_eq!(TestRunnerType::Vitest.to_string(), "vitest");
        assert_eq!(TestRunnerType::Npm.to_string(), "npm");
        assert_eq!(TestRunnerType::Pnpm.to_string(), "pnpm");
        assert_eq!(TestRunnerType::Bun.to_string(), "bun");
    }

    #[test]
    fn test_test_status_serialization() {
        let result = TestResult::new("test", TestStatus::Passed);
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"status\":\"passed\""));
    }

    // ============================================================
    // Logs Output Schema Tests
    // ============================================================

    #[test]
    fn test_logs_output_schema_new() {
        let schema = LogsOutputSchema::new();
        assert!(schema.is_empty);
        assert!(schema.entries.is_empty());
        assert!(schema.recent_critical.is_empty());
    }

    #[test]
    fn test_log_entry_new() {
        let entry = LogEntry::new("[INFO] Application started", 1);
        assert_eq!(entry.line, "[INFO] Application started");
        assert_eq!(entry.line_number, 1);
        assert_eq!(entry.level, LogLevel::Unknown);
    }

    #[test]
    fn test_log_level_serialization() {
        let entry = LogEntry {
            line: "test".to_string(),
            level: LogLevel::Error,
            timestamp: None,
            source: None,
            message: "test".to_string(),
            line_number: 1,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"level\":\"error\""));
    }

    // ============================================================
    // Process Output Schema Tests
    // ============================================================

    #[test]
    fn test_process_output_schema_new() {
        let schema = ProcessOutputSchema::new("echo");
        assert_eq!(schema.command, "echo");
        assert!(schema.args.is_empty());
        assert!(schema.stdout.is_empty());
        assert!(schema.stderr.is_empty());
        assert!(schema.exit_code.is_none());
        assert!(schema.success);
    }

    #[test]
    fn test_process_output_schema_serialization() {
        let schema = ProcessOutputSchema::new("echo");
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("\"command\":\"echo\""));
        assert!(json.contains("\"type\":\"process_output\""));
    }

    // ============================================================
    // Error Schema Tests
    // ============================================================

    #[test]
    fn test_error_schema_new() {
        let error = ErrorSchema::new("Something went wrong");
        assert!(error.error);
        assert_eq!(error.message, "Something went wrong");
        assert!(error.error_type.is_none());
    }

    #[test]
    fn test_error_schema_with_type() {
        let error = ErrorSchema::with_type("Command not found", "command_not_found");
        assert!(error.error);
        assert_eq!(error.message, "Command not found");
        assert_eq!(error.error_type, Some("command_not_found".to_string()));
    }

    #[test]
    fn test_error_schema_serialization() {
        let error = ErrorSchema::new("Test error");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"error\":true"));
        assert!(json.contains("\"message\":\"Test error\""));
        assert!(json.contains("\"type\":\"error\""));
    }

    // ============================================================
    // Deserialization Tests
    // ============================================================

    #[test]
    fn test_git_status_schema_deserialization() {
        let json = r#"{
            "schema": {"version": "1.0.0", "type": "git_status"},
            "branch": "main",
            "is_clean": true,
            "staged": [],
            "unstaged": [],
            "untracked": [],
            "unmerged": [],
            "counts": {"staged": 0, "unstaged": 0, "untracked": 0, "unmerged": 0}
        }"#;
        let schema: GitStatusSchema = serde_json::from_str(json).unwrap();
        assert_eq!(schema.branch, "main");
        assert!(schema.is_clean);
    }

    #[test]
    fn test_ls_entry_type_deserialization() {
        let json = r#"{"name": "src", "type": "directory", "is_hidden": false, "is_symlink": false}"#;
        let entry: LsEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.name, "src");
        assert_eq!(entry.entry_type, LsEntryType::Directory);
    }

    #[test]
    fn test_test_status_deserialization() {
        let json = r#"{"name": "test", "status": "failed"}"#;
        let result: TestResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.status, TestStatus::Failed);
    }

    // ============================================================
    // Round-trip Tests
    // ============================================================

    #[test]
    fn test_git_status_round_trip() {
        let original = GitStatusSchema::new("feature/test");
        let json = serde_json::to_string(&original).unwrap();
        let parsed: GitStatusSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_ls_output_round_trip() {
        let mut original = LsOutputSchema::new();
        original.is_empty = false;
        original.entries.push(LsEntry::new("src", LsEntryType::Directory));
        original.directories.push("src".to_string());
        original.counts.total = 1;
        original.counts.directories = 1;

        let json = serde_json::to_string(&original).unwrap();
        let parsed: LsOutputSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_test_output_round_trip() {
        let original = TestOutputSchema::new(TestRunnerType::Jest);
        let json = serde_json::to_string(&original).unwrap();
        let parsed: TestOutputSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }
}
