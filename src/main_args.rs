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
    // Known trs subcommands (and aliases) that must go through clap.
    // `every_clap_subcommand_takes_the_clap_path` pins this against the clap
    // definition: a name missing here does not error, it silently becomes an
    // external command lookup and the subcommand appears not to exist.
    !matches!(
        first,
        "parse"
            | "search"
            | "replace"
            | "run"
            | "tail"
            | "clean"
            | "trim"
            | "report"
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

/// Exit status of a tool whose reader went away (128 + SIGPIPE), the same a
/// shell reports for `git log | head`.
pub(crate) const CLOSED_PIPE_EXIT: i32 = 141;

/// Whether a panic message is `print!` failing because the other end of the
/// pipe closed: `trs git log | head` must end quietly, not with a backtrace
/// hint and status 101.
pub(crate) fn is_closed_pipe_panic(msg: &str) -> bool {
    msg.starts_with("failed printing to std")
        && (msg.contains("Broken pipe") || msg.contains("pipe is being closed"))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_closed_output_pipe_is_quiet() {
        assert!(is_closed_pipe_panic(
            "failed printing to stdout: Broken pipe (os error 32)"
        ));
        assert!(is_closed_pipe_panic(
            "failed printing to stderr: Broken pipe (os error 32)"
        ));
        assert!(is_closed_pipe_panic(
            "failed printing to stdout: The pipe is being closed. (os error 232)"
        ));
        assert!(!is_closed_pipe_panic(
            "failed printing to stdout: No space left on device"
        ));
        assert!(!is_closed_pipe_panic("index out of bounds: the len is 3"));
        assert!(!is_closed_pipe_panic(
            "called `Result::unwrap()` on an `Err` value: Broken pipe"
        ));
    }
}
