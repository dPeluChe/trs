//! Argument-shape helpers for `main.rs`: deciding whether an invocation
//! bypasses clap entirely, and reading a human token budget. Split out to
//! keep `main.rs` about dispatch.

/// Check if args[1] is an external command (not a trs subcommand or flag).
/// This allows bypassing the full clap parser for the hot path.
pub(crate) fn is_external_fast_path(args: &[String]) -> bool {
    if args.len() < 2 {
        return false;
    }
    let first = args[1].as_str();
    // Skip if it's a flag
    if first.starts_with('-') {
        return false;
    }
    // Known trs subcommands (and aliases) that must go through clap
    !matches!(
        first,
        "parse"
            | "search"
            | "replace"
            | "run"
            | "tail"
            | "clean"
            | "trim"
            | "html2md"
            | "txt2md"
            | "is-clean"
            | "clean?"
            | "repo-clean"
            | "read"
            | "json"
            | "err"
            | "rewrite"
            | "discover"
            | "init"
            | "uninstall"
            | "doctor"
            | "benchmark"
            | "diff"
            | "ingest"
            | "audit-docs"
            | "output-saver"
            | "upgrade"
            | "debug-info"
            | "history"
            | "stats"
            | "raw"
            | "help"
            | "--help"
            | "-h"
            | "--version"
            | "-V"
    )
}

/// Parse token budget string: "128k" -> 128000, "64000" -> 64000
pub(crate) fn parse_token_budget(s: &str) -> usize {
    let s = s.trim().to_lowercase();
    if let Some(num) = s.strip_suffix('k') {
        num.parse::<f64>().unwrap_or(0.0) as usize * 1000
    } else if let Some(num) = s.strip_suffix('m') {
        num.parse::<f64>().unwrap_or(0.0) as usize * 1_000_000
    } else {
        s.parse::<usize>().unwrap_or(128_000)
    }
}
