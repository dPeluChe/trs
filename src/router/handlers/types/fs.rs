//! Filesystem-related data structures (ls, find) for command handlers.

use std::collections::HashMap;

// ============================================================
// LS Data Structures
// ============================================================

/// Entry type for ls output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LsEntryType {
    /// Regular file.
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

impl Default for LsEntryType {
    fn default() -> Self {
        LsEntryType::File
    }
}

/// A single entry in ls output.
#[derive(Debug, Clone, Default)]
pub(crate) struct LsEntry {
    /// Name of the file or directory.
    pub(crate) name: String,
    /// Type of entry (file, directory, etc.).
    pub(crate) entry_type: LsEntryType,
    /// Whether this is a hidden file (starts with .).
    pub(crate) is_hidden: bool,
    /// File size in bytes (if available).
    #[allow(dead_code)]
    pub(crate) size: Option<u64>,
    /// File permissions (if available).
    #[allow(dead_code)]
    pub(crate) permissions: Option<String>,
    /// Number of hard links (if available).
    pub(crate) links: Option<u64>,
    /// Owner user name (if available).
    pub(crate) owner: Option<String>,
    /// Owner group name (if available).
    pub(crate) group: Option<String>,
    /// Last modification time (if available).
    pub(crate) modified: Option<String>,
    /// Symlink target (if this is a symlink).
    pub(crate) symlink_target: Option<String>,
    /// Whether the symlink is broken (target doesn't exist).
    pub(crate) is_broken_symlink: bool,
}

// ============================================================
// Find Data Structures
// ============================================================

/// A single entry in find output.
#[derive(Debug, Clone, Default)]
pub(crate) struct FindEntry {
    /// Path to the file or directory.
    pub(crate) path: String,
    /// Whether this is a directory.
    pub(crate) is_directory: bool,
    /// Whether this is a hidden file/directory.
    pub(crate) is_hidden: bool,
    /// File extension (if available).
    pub(crate) extension: Option<String>,
    /// Depth of the path (number of path separators).
    pub(crate) depth: usize,
}

/// A permission denied or error entry from find output.
#[derive(Debug, Clone, Default)]
pub(crate) struct FindError {
    /// The path that was denied access.
    pub(crate) path: String,
    /// The error message.
    pub(crate) message: String,
}

/// Parsed find output.
#[derive(Debug, Clone, Default)]
pub(crate) struct FindOutput {
    /// List of all entries.
    pub(crate) entries: Vec<FindEntry>,
    /// Directory paths.
    pub(crate) directories: Vec<String>,
    /// File paths.
    pub(crate) files: Vec<String>,
    /// Hidden entries.
    pub(crate) hidden: Vec<String>,
    /// File extensions with counts.
    pub(crate) extensions: HashMap<String, usize>,
    /// Permission denied or error entries.
    pub(crate) errors: Vec<FindError>,
    /// Total count of entries (excluding errors).
    pub(crate) total_count: usize,
    /// Whether the output is empty.
    pub(crate) is_empty: bool,
}

/// Common generated directory names that are typically build artifacts or dependencies.
pub(crate) const COMMON_GENERATED_DIRS: &[&str] = &[
    // JavaScript/TypeScript
    "node_modules",
    "dist",
    "build",
    "out",
    ".next",
    ".nuxt",
    ".output",
    // Python
    "__pycache__",
    ".venv",
    "venv",
    "env",
    ".tox",
    ".nox",
    "htmlcov",
    ".eggs",
    "eggs",
    "sdist",
    "wheelhouse",
    // Rust
    "target",
    // Java/Kotlin
    "target", // Maven
    "build",  // Gradle
    "out",    // IntelliJ
    ".gradle",
    // Go
    "vendor",
    // Ruby
    "vendor",
    ".bundle",
    // PHP
    "vendor",
    // .NET/C#
    "bin",
    "obj",
    // Swift/Objective-C
    "DerivedData",
    "Pods",
    ".build",
    // Elixir/Erlang
    "_build",
    "deps",
    // Haskell
    "dist-newstyle",
    ".stack-work",
    // Scala
    ".bloop",
    ".metals",
    // Docker
    ".docker",
    // Cache directories
    ".cache",
    ".npm",
    ".yarn",
    ".pnpm-store",
    // IDE/Editor
    ".idea",
    ".vscode",
    ".vs",
    // Misc
    "tmp",
    "temp",
];

/// Check if a directory name is a common generated directory.
pub(crate) fn is_generated_directory(name: &str) -> bool {
    // Strip trailing slash if present (common in ls output)
    let name = name.strip_suffix('/').unwrap_or(name);
    let name_lower = name.to_lowercase();
    COMMON_GENERATED_DIRS.contains(&name_lower.as_str())
}

/// A permission denied or error entry.
#[derive(Debug, Clone, Default)]
pub(crate) struct LsError {
    /// The path that was denied access.
    pub(crate) path: String,
    /// The error message.
    pub(crate) message: String,
}

/// Parsed ls output.
#[derive(Debug, Clone, Default)]
pub(crate) struct LsOutput {
    /// List of all entries.
    pub(crate) entries: Vec<LsEntry>,
    /// Directory entries.
    pub(crate) directories: Vec<LsEntry>,
    /// File entries.
    pub(crate) files: Vec<LsEntry>,
    /// Symlink entries.
    pub(crate) symlinks: Vec<LsEntry>,
    /// Hidden entries.
    pub(crate) hidden: Vec<LsEntry>,
    /// Generated directory entries (build artifacts, dependencies, etc.).
    pub(crate) generated: Vec<LsEntry>,
    /// Permission denied or error entries.
    pub(crate) errors: Vec<LsError>,
    /// Total count of entries (excluding errors).
    pub(crate) total_count: usize,
    /// Whether the output is empty.
    pub(crate) is_empty: bool,
}
