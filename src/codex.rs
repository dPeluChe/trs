//! Codex CLI version awareness.
//!
//! Codex grew a `PreToolUse`/`PostToolUse` hook framework in v0.117.0
//! (Mar 2026), and v0.123.0 added `apply_patch` events + the `tool_name`
//! field. The piece trs needs to route shell output through a hook —
//! returning `updatedInput.command` from `PreToolUse` — is *documented*
//! (<https://developers.openai.com/codex/hooks>) but **not implemented in
//! the runtime**: it rejects the payload with "PreToolUse hook returned
//! unsupported updatedInput". Tracking: <https://github.com/openai/codex/issues/18491>.
//!
//! Until that lands, Codex stays rules-only (AGENTS.md prefix guidance).
//! This module centralizes the version gate so flipping it on later is a
//! one-line change: set [`REWRITE_HOOK_MIN_VERSION`] to the first release
//! that implements the rewrite and the install path turns on by version.

/// First `codex-cli` version that implements `updatedInput` command rewrite
/// in `PreToolUse` hooks. `None` = not yet implemented in any release
/// (tracked in openai/codex#18491). When OpenAI ships it, set
/// `Some((major, minor, patch))` — [`rewrite_hook_supported`] then returns
/// true for that version onward and the hook install path can engage.
pub(crate) const REWRITE_HOOK_MIN_VERSION: Option<(u32, u32, u32)> = None;

/// Parse a `codex --version` line into `(major, minor, patch)`.
/// Accepts shapes like `codex-cli 0.130.0`, `codex 0.130.0`, or a bare
/// `0.130.0`. Pre-release/build suffixes on the patch (`0.130.0-rc1`) are
/// tolerated — only the leading integer is read.
pub(crate) fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let token = s.split_whitespace().find(|t| {
        t.split('.')
            .next()
            .is_some_and(|h| h.chars().all(|c| c.is_ascii_digit()))
            && t.contains('.')
    })?;
    let mut parts = token.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    // Patch may carry a `-rc1`/`+build` suffix; read the leading digits.
    let patch_raw = parts.next().unwrap_or("0");
    let patch_digits: String = patch_raw
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let patch = patch_digits.parse().unwrap_or(0);
    Some((major, minor, patch))
}

/// Best-effort detection of the installed `codex-cli` version by shelling
/// out to `codex --version`. Returns `None` when codex isn't on `PATH` or
/// the output can't be parsed.
pub(crate) fn detect_version() -> Option<(u32, u32, u32)> {
    let out = std::process::Command::new("codex")
        .arg("--version")
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    parse_version(&String::from_utf8_lossy(&out.stdout))
}

/// Whether the given codex version implements the `updatedInput` rewrite
/// hook trs needs. Always false while [`REWRITE_HOOK_MIN_VERSION`] is
/// `None` (no released version supports it yet).
pub(crate) fn rewrite_hook_supported(version: (u32, u32, u32)) -> bool {
    match REWRITE_HOOK_MIN_VERSION {
        Some(min) => version >= min,
        None => false,
    }
}

/// Whether the installed codex (if any) supports the rewrite hook today.
/// Drives whether `trs init codex` could install a real hook instead of
/// rules-only. Currently always false (see [`REWRITE_HOOK_MIN_VERSION`]).
#[allow(dead_code)] // wired into the install path when the gate flips on.
pub(crate) fn rewrite_hook_available() -> bool {
    detect_version().is_some_and(rewrite_hook_supported)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_codex_cli_prefix() {
        assert_eq!(parse_version("codex-cli 0.130.0"), Some((0, 130, 0)));
        assert_eq!(parse_version("codex 0.117.3\n"), Some((0, 117, 3)));
        assert_eq!(parse_version("0.123.0"), Some((0, 123, 0)));
    }

    #[test]
    fn tolerates_prerelease_patch() {
        assert_eq!(parse_version("codex-cli 0.135.0-rc1"), Some((0, 135, 0)));
        assert_eq!(parse_version("1.2.5+build7"), Some((1, 2, 5)));
    }

    #[test]
    fn rejects_unparseable() {
        assert_eq!(parse_version(""), None);
        assert_eq!(parse_version("codex unknown"), None);
        assert_eq!(parse_version("no version here"), None);
    }

    #[test]
    fn gate_is_off_until_min_version_set() {
        // While REWRITE_HOOK_MIN_VERSION is None, every version is unsupported.
        assert!(!rewrite_hook_supported((0, 130, 0)));
        assert!(!rewrite_hook_supported((99, 0, 0)));
    }

    #[test]
    fn gate_compares_when_min_set() {
        // Mirror of the prod check with a hypothetical threshold, so the
        // comparison logic is covered even while the real gate is None.
        let supported = |v: (u32, u32, u32), min: (u32, u32, u32)| v >= min;
        assert!(supported((0, 140, 0), (0, 140, 0)));
        assert!(supported((0, 141, 0), (0, 140, 0)));
        assert!(!supported((0, 139, 9), (0, 140, 0)));
    }
}
