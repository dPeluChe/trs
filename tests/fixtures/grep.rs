#![allow(dead_code)]
use super::load_fixture;

// ============================================================
// Grep - Empty/Clean Fixtures
// ============================================================

/// Returns empty grep output.
pub fn grep_empty() -> String {
    load_fixture("grep_empty.txt")
}

// ============================================================
// Grep - Simple Format Fixtures
// ============================================================

/// Returns simple grep output (single match).
pub fn grep_simple() -> String {
    load_fixture("grep_simple.txt")
}

/// Returns grep output with multiple matches in a single file.
pub fn grep_single_file_multiple_matches() -> String {
    load_fixture("grep_single_file_multiple_matches.txt")
}

/// Returns grep output with matches in multiple files.
pub fn grep_multiple_files() -> String {
    load_fixture("grep_multiple_files.txt")
}

// ============================================================
// Grep - With Column Numbers
// ============================================================

/// Returns grep output with column numbers (path:line:col:content).
pub fn grep_with_column() -> String {
    load_fixture("grep_with_column.txt")
}

/// Returns grep output without line numbers (path:content).
pub fn grep_without_line_numbers() -> String {
    load_fixture("grep_without_line_numbers.txt")
}

// ============================================================
// Grep - Binary Files
// ============================================================

/// Returns grep output with a binary file match.
pub fn grep_binary_file() -> String {
    load_fixture("grep_binary_file.txt")
}

// ============================================================
// Grep - Context Lines
// ============================================================

/// Returns grep output with context lines (before and after).
pub fn grep_context_lines() -> String {
    load_fixture("grep_context_lines.txt")
}

/// Returns grep output with context lines before matches.
pub fn grep_context_before() -> String {
    load_fixture("grep_context_before.txt")
}

/// Returns grep output with context lines after matches.
pub fn grep_context_after() -> String {
    load_fixture("grep_context_after.txt")
}

// ============================================================
// Grep - Edge Cases
// ============================================================

/// Returns grep output with long file paths.
pub fn grep_long_paths() -> String {
    load_fixture("grep_long_paths.txt")
}

/// Returns grep output with special characters in filenames.
pub fn grep_special_chars() -> String {
    load_fixture("grep_special_chars.txt")
}

/// Returns grep output with colon in content.
pub fn grep_with_colon_in_content() -> String {
    load_fixture("grep_with_colon_in_content.txt")
}

// ============================================================
// Grep - Large Output
// ============================================================

/// Returns grep output with many files and matches (for testing truncation).
pub fn grep_large() -> String {
    load_fixture("grep_large.txt")
}

// ============================================================
// Grep - Mixed Fixtures
// ============================================================

/// Returns grep output with mixed content (multiple files, context, binary).
pub fn grep_mixed() -> String {
    load_fixture("grep_mixed.txt")
}

// ============================================================
// Grep - Ripgrep Heading Format
// ============================================================

/// Returns grep output in ripgrep heading format (--heading).
pub fn grep_ripgrep_heading() -> String {
    load_fixture("grep_ripgrep_heading.txt")
}
