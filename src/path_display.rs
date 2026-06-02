//! Forward-slash path output for consistent, agent-facing results.
//!
//! `Path::display()` / `to_string_lossy()` yield native separators, so on
//! Windows trs's real-filesystem commands (`search`, `find`) printed `\`
//! paths while everything else (and every other platform) used `/`. Agents
//! and downstream tooling want one stable shape, so trs normalizes textual
//! path output to `/` everywhere (issue #60). No-op on Unix.
//!
//! This is presentation only — it never touches the paths trs actually opens
//! or walks (Windows accepts `/` too, but we don't feed these strings back
//! into the filesystem APIs).

use std::path::Path;

/// Render `path` as a forward-slash string for output.
pub(crate) fn display_path(path: &Path) -> String {
    normalize(&path.to_string_lossy())
}

/// Normalize an already-stringified path's separators to `/`.
pub(crate) fn normalize(s: &str) -> String {
    #[cfg(windows)]
    {
        s.replace('\\', "/")
    }
    #[cfg(not(windows))]
    {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn normalizes_on_windows_only() {
        // On Unix, backslash is a valid filename char and must be preserved;
        // on Windows it's the separator and becomes `/`.
        let out = normalize(r"src\router\handlers");
        #[cfg(windows)]
        assert_eq!(out, "src/router/handlers");
        #[cfg(not(windows))]
        assert_eq!(out, r"src\router\handlers");
        // Forward slashes are untouched on every platform.
        assert_eq!(normalize("src/router/handlers"), "src/router/handlers");
    }
}
