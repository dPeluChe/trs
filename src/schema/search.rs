//! Search-related schema types (grep, replace).

use serde::{Deserialize, Serialize};

use super::SchemaVersion;

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
    /// Short excerpt of the matched text (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
}

impl GrepMatch {
    /// Create a new grep match.
    pub fn new(line: &str) -> Self {
        Self {
            line_number: None,
            column: None,
            line: line.to_string(),
            is_context: false,
            excerpt: None,
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
// Replace Output Schema
// ============================================================

/// Schema for replace command output.
///
/// # Example JSON
///
/// ```json
/// {
///   "schema": { "version": "1.0.0", "type": "replace_output" },
///   "dry_run": true,
///   "search_pattern": "old_function",
///   "replacement": "new_function",
///   "files": [
///     {
///       "path": "src/main.rs",
///       "matches": [
///         { "line_number": 10, "original": "old_function()", "replaced": "new_function()" }
///       ]
///     }
///   ],
///   "counts": {
///     "files_affected": 1,
///     "total_replacements": 3
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplaceOutputSchema {
    /// Schema version information.
    pub schema: SchemaVersion,
    /// Whether this was a dry run (preview mode).
    pub dry_run: bool,
    /// The search pattern used.
    pub search_pattern: String,
    /// The replacement string.
    pub replacement: String,
    /// List of files with replacements.
    #[serde(default)]
    pub files: Vec<ReplaceFile>,
    /// Count summary.
    pub counts: ReplaceCounts,
}

impl ReplaceOutputSchema {
    /// Create a new replace output schema.
    pub fn new(search_pattern: &str, replacement: &str, dry_run: bool) -> Self {
        Self {
            schema: SchemaVersion::new("replace_output"),
            dry_run,
            search_pattern: search_pattern.to_string(),
            replacement: replacement.to_string(),
            files: Vec::new(),
            counts: ReplaceCounts::default(),
        }
    }

    /// Set the files.
    pub fn with_files(mut self, files: Vec<ReplaceFile>) -> Self {
        self.files = files;
        self
    }

    /// Set the counts.
    pub fn with_counts(mut self, counts: ReplaceCounts) -> Self {
        self.counts = counts;
        self
    }
}

/// A file with replacements.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplaceFile {
    /// Path to the file.
    pub path: String,
    /// List of replacements in this file.
    #[serde(default)]
    pub matches: Vec<ReplaceMatch>,
}

impl ReplaceFile {
    /// Create a new replace file entry.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            matches: Vec::new(),
        }
    }
}

/// A single replacement match.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplaceMatch {
    /// Line number of the match.
    pub line_number: usize,
    /// Original line content (before replacement).
    pub original: String,
    /// Replaced line content (after replacement).
    pub replaced: String,
}

impl ReplaceMatch {
    /// Create a new replace match.
    pub fn new(line_number: usize, original: &str, replaced: &str) -> Self {
        Self {
            line_number,
            original: original.to_string(),
            replaced: replaced.to_string(),
        }
    }
}

/// Count summary for replace output.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplaceCounts {
    /// Number of files affected.
    pub files_affected: usize,
    /// Total number of replacements made.
    pub total_replacements: usize,
}
