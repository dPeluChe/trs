//! Codex CLI version awareness.
//!
//! Codex grew a `PreToolUse`/`PostToolUse` hook framework in v0.117.0
//! (Mar 2026), and the `updatedInput.command` rewrite trs needs landed in
//! PR openai/codex#20527 (merged 2026-05-12), shipping in the 0.13x line.
//! Verified end-to-end against codex-cli 0.136.0: a `PreToolUse` hook
//! returning `{"hookSpecificOutput":{"permissionDecision":"allow",
//! "updatedInput":{"command":"…"}}}` rewrites the executed command.
//!
//! (Note: pre-0.134 builds — and `codex exec` non-interactive mode, which
//! doesn't dispatch `PreToolUse` at all — reject/ignore the rewrite. The
//! `permissionDecision:"allow"` field is mandatory; without it the runtime
//! errors "unsupported updatedInput". The broader openai/codex#18491 stays
//! open for `read_file`/`grep` dispatch, which trs doesn't use.)
//!
//! [`REWRITE_HOOK_MIN_VERSION`] gates the hook install on version so older
//! installs fall back to rules-only.

/// First `codex-cli` version known to implement `updatedInput` command
/// rewrite in `PreToolUse` hooks. Conservative: PR #20527 merged 2026-05-12
/// and 0.134.0 (2026-05-26) is the first release we can confirm postdates
/// it; pre-0.134 builds fall back to rules-only. `None` would disable the
/// hook entirely.
pub(crate) const REWRITE_HOOK_MIN_VERSION: Option<(u32, u32, u32)> = Some((0, 134, 0));

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
/// Drives whether `trs init codex` installs a real PreToolUse hook
/// (vs rules-only). Shells out to `codex --version`.
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
    fn gate_reflects_min_version() {
        // Pre-0.134 falls back to rules-only; 0.134+ gets the hook.
        assert!(!rewrite_hook_supported((0, 130, 0)));
        assert!(!rewrite_hook_supported((0, 133, 9)));
        assert!(rewrite_hook_supported((0, 134, 0)));
        assert!(rewrite_hook_supported((0, 136, 0)));
        assert!(rewrite_hook_supported((1, 0, 0)));
    }
}
