//! Grep-related data structures for command handlers.

// ============================================================
// Grep Data Structures
// ============================================================

/// A single match in grep output.
#[derive(Debug, Clone, Default)]
pub(crate) struct GrepMatch {
    /// Line number (if available with -n flag).
    pub(crate) line_number: Option<usize>,
    /// Column number (if available with --column flag).
    pub(crate) column: Option<usize>,
    /// The matched line content.
    pub(crate) line: String,
    /// Whether this is a context line (not a direct match).
    pub(crate) is_context: bool,
    /// Short excerpt of the matched text.
    pub(crate) excerpt: Option<String>,
}

/// A file with grep matches.
#[derive(Debug, Clone, Default)]
pub(crate) struct GrepFile {
    /// Path to the file.
    pub(crate) path: String,
    /// List of matches in this file.
    pub(crate) matches: Vec<GrepMatch>,
}

/// Parsed grep output.
#[derive(Debug, Clone, Default)]
pub(crate) struct GrepOutput {
    /// List of files with matches (limited if truncated).
    pub(crate) files: Vec<GrepFile>,
    /// Total number of files with matches.
    pub(crate) file_count: usize,
    /// Total number of matches across all files.
    pub(crate) match_count: usize,
    /// Whether the output is empty (no matches).
    pub(crate) is_empty: bool,
    /// Whether the output was truncated.
    pub(crate) is_truncated: bool,
    /// Total number of files available before truncation.
    pub(crate) total_files: usize,
    /// Total number of matches available before truncation.
    pub(crate) total_matches: usize,
    /// Number of files shown after truncation.
    pub(crate) files_shown: usize,
    /// Number of matches shown after truncation.
    pub(crate) matches_shown: usize,
    /// Total bytes of all matched lines (original output size).
    pub(crate) input_bytes: usize,
}
