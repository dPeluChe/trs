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

// ============================================================
// Grep - Edge Cases
// ============================================================

/// Returns grep output with special characters in filenames.
pub fn grep_special_chars() -> String {
    load_fixture("grep_special_chars.txt")
}

// ============================================================
// Grep - Large Output
// ============================================================

/// Returns grep output with many files and matches (for testing truncation).
pub fn grep_large() -> String {
    load_fixture("grep_large.txt")
}

// ============================================================
// Grep - Ripgrep Heading Format
// ============================================================

/// Returns grep output in ripgrep heading format (--heading).
pub fn grep_ripgrep_heading() -> String {
    load_fixture("grep_ripgrep_heading.txt")
}
