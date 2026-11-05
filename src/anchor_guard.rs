//! Runtime twin of the literal anchor score in `docs/development/benchmarks/truth.py`:
//! when a summary cuts 90% or more and keeps under 70% of the raw output's
//! anchors (paths, `file:line` refs, commit hashes), the caller points at the
//! saved raw output instead of leaving the loss silent.

use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

/// Below this raw size the check is not worth its latency.
const MIN_RAW_BYTES: usize = 2 * 1024;
const MIN_CUT_PCT: usize = 90;
const MIN_KEPT_PCT: usize = 70;

/// Raw output beyond this is not scanned: bounded latency on huge outputs.
const SCAN_BYTES: usize = 256 * 1024;

/// Paths, with an optional `:line`, so `file:line` counts as the one anchor it is.
static PATH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[\w./-]+\.(?:rs|ts|tsx|js|py|go|md|json|toml|sh|swift|lock)\b(?::\d+)?").unwrap()
});
static HASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b[0-9a-f]{7,40}\b").unwrap());

/// A hex run is a commit hash only if it mixes digits and letters.
pub(crate) fn looks_like_hash(h: &str) -> bool {
    h.bytes().any(|b| b.is_ascii_digit()) && h.bytes().any(|b| b.is_ascii_lowercase())
}

/// True when the summary cut at least 90% of a sizeable output and under 70%
/// of its anchors survive in it. A hash survives as its 7-char prefix.
pub(crate) fn anchors_lost(raw: &str, summary: &str) -> bool {
    if raw.len() < MIN_RAW_BYTES || summary.len() * 100 > raw.len() * (100 - MIN_CUT_PCT) {
        return false;
    }
    let mut end = raw.len().min(SCAN_BYTES);
    while !raw.is_char_boundary(end) {
        end -= 1;
    }
    let raw = &raw[..end];
    let paths: HashSet<&str> = PATH.find_iter(raw).map(|m| m.as_str()).collect();
    let hashes: HashSet<&str> = HASH
        .find_iter(raw)
        .map(|m| m.as_str())
        .filter(|h| looks_like_hash(h))
        .collect();
    let total = paths.len() + hashes.len();
    if total == 0 {
        return false;
    }
    let kept = paths.iter().filter(|p| summary.contains(*p)).count()
        + hashes.iter().filter(|h| summary.contains(&h[..7])).count();
    kept * 100 < total * MIN_KEPT_PCT
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_listing() -> String {
        (0..80)
            .map(|i| format!(" src/module_{i}.rs | {i} {}\n", "+".repeat(180)))
            .collect()
    }

    #[test]
    fn a_summary_that_drops_the_paths_is_flagged() {
        assert!(anchors_lost(&raw_listing(), "80 files changed"));
    }

    #[test]
    fn a_summary_that_keeps_the_paths_is_not() {
        let summary: String = (0..80).map(|i| format!("src/module_{i}.rs\n")).collect();
        assert!(!anchors_lost(&raw_listing(), &summary));
    }

    #[test]
    fn small_or_anchorless_output_is_never_flagged() {
        assert!(!anchors_lost("src/a.rs\n", "x"));
        assert!(!anchors_lost(&"plain words\n".repeat(400), "x"));
    }

    #[test]
    fn a_short_hash_prefix_counts_as_kept() {
        let raw = format!(
            "{}commit 0123456789abcdef0123456789abcdef01234567\n",
            " \n".repeat(2500)
        );
        assert!(!anchors_lost(&raw, "0123456"));
    }
}
