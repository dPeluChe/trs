use super::load_fixture;

// ============================================================
// Ls - Empty/Clean Fixtures
// ============================================================

/// Returns empty ls output.
pub fn ls_empty() -> String {
    load_fixture("ls_empty.txt")
}

// ============================================================
// Ls - Simple Format Fixtures
// ============================================================

/// Returns simple ls output (just filenames).
pub fn ls_simple() -> String {
    load_fixture("ls_simple.txt")
}

/// Returns ls output with directories.
pub fn ls_with_directories() -> String {
    load_fixture("ls_with_directories.txt")
}

/// Returns ls output with hidden files.
pub fn ls_with_hidden() -> String {
    load_fixture("ls_with_hidden.txt")
}

// ============================================================
// Ls - Long Format Fixtures
// ============================================================

/// Returns ls -l output (long format).
pub fn ls_long_format() -> String {
    load_fixture("ls_long_format.txt")
}

/// Returns ls -l output with symlinks.
pub fn ls_long_format_with_symlinks() -> String {
    load_fixture("ls_long_format_with_symlinks.txt")
}

/// Returns ls -l output with broken symlinks.
pub fn ls_broken_symlink() -> String {
    load_fixture("ls_broken_symlink.txt")
}

// ============================================================
// Ls - Error Fixtures
// ============================================================

/// Returns ls output with permission denied errors.
pub fn ls_permission_denied() -> String {
    load_fixture("ls_permission_denied.txt")
}

// ============================================================
// Ls - Mixed Fixtures
// ============================================================

/// Returns ls -l output with mixed content (dirs, files, symlinks, hidden, errors).
pub fn ls_mixed() -> String {
    load_fixture("ls_mixed.txt")
}

/// Returns ls output with generated directories (node_modules, target, etc.).
pub fn ls_generated_dirs() -> String {
    load_fixture("ls_generated_dirs.txt")
}

// ============================================================
// Ls - Edge Cases
// ============================================================

/// Returns ls output with special characters in filenames.
pub fn ls_special_chars() -> String {
    load_fixture("ls_special_chars.txt")
}

/// Returns ls output with long file paths.
pub fn ls_long_paths() -> String {
    load_fixture("ls_long_paths.txt")
}
