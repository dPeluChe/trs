//! File system schema types (ls, find).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::SchemaVersion;

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
