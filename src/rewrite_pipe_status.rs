//! Opt-in (`[hooks] pipe_status`): `cargo test 2>&1 | tail -8` exits 0 when the
//! tests fail, and 14.6% of the agent Bash calls measured here pipe a check.
//! The fix touches only the status: output stays byte-exact (#162 holds).
//! The guard makes shells without pipefail keep today's behavior.

use crate::rewrite_decide::unquoted_indices;
use crate::rewrite_permissions::strip_env;
use crate::text_util::cmd_basename;

const GUARD: &str = r#"[ -n "$BASH_VERSION$ZSH_VERSION" ] && set -o pipefail;"#;

/// `cmd` with pipefail on for its last statement, the only one whose status
/// the shell reports, or `None` when that status is not safe to change.
pub(super) fn wrap(cmd: &str) -> Option<String> {
    let cut = unquoted_indices(cmd)
        .filter_map(|i| match cmd.as_bytes()[i] {
            b';' | b'\n' => Some(i + 1),
            b' ' if cmd[i..].starts_with(" && ") => Some(i + 4),
            _ => None,
        })
        .last()
        .unwrap_or(0);
    let (head, last) = cmd.split_at(cut);
    // A continued line would glue the subshell to the statement before it.
    if head.trim_end_matches([' ', '\n']).ends_with('\\') {
        return None;
    }
    if last.contains(['\n', '`']) || last.contains("$(") {
        return None;
    }
    let (producer, sink) = single_pipe(last)?;
    (is_status_tool(producer) && sink_reads_everything(sink))
        .then(|| format!("{head}({GUARD} {})", last.trim()))
}

/// The two sides of a pipeline with exactly one plain `|` and no other shell
/// operator. A later `;` or `||` would see the new status, and a middle stage
/// like `grep -v` would add failures of its own.
fn single_pipe(seg: &str) -> Option<(&str, &str)> {
    let b = seg.as_bytes();
    let mut pipe = None;
    for i in unquoted_indices(seg) {
        match b[i] {
            b'|' if pipe.is_none() => pipe = Some(i),
            b'|' | b';' | b'(' | b')' | b'<' => return None,
            b'>' if !b[i.saturating_sub(1)..].starts_with(b"2>&1") => return None,
            b'&' if !b[i.saturating_sub(2)..].starts_with(b"2>&1") => return None,
            _ => {}
        }
    }
    let at = pipe?;
    Some((seg[..at].trim(), seg[at + 1..].trim()))
}

/// Tools whose exit status is the verdict. Others stay out on purpose: `grep`
/// finding nothing or `find` hitting one unreadable dir are not failures.
fn is_status_tool(producer: &str) -> bool {
    let mut words = strip_env(producer).split_whitespace();
    let program = cmd_basename(words.next().unwrap_or(""));
    let first = words.next().unwrap_or("");
    let second = words.next().unwrap_or("");
    let is_check = |s: &str| {
        ["test", "build", "lint", "check", "typecheck"]
            .iter()
            .any(|p| s.starts_with(p))
    };
    match program {
        "cargo" => matches!(first, "test" | "build" | "check" | "clippy" | "nextest"),
        "npm" | "pnpm" | "yarn" | "bun" if first == "run" => is_check(second),
        "npm" | "pnpm" | "yarn" | "bun" => is_check(first),
        "npx" | "bunx" => matches!(first, "tsc" | "jest" | "vitest" | "eslint" | "playwright"),
        "go" => matches!(first, "test" | "build" | "vet"),
        "python" | "python3" => first == "-m" && matches!(second, "pytest" | "unittest" | "mypy"),
        "ruff" => first == "check",
        "make" | "just" | "tsc" | "jest" | "vitest" | "eslint" | "pytest" | "mypy" => true,
        _ => false,
    }
}

/// A sink that stops early kills the producer with SIGPIPE, which pipefail then
/// reports as a failure (141). So no `head`, `tail -f`, `grep -q/-m/-l`.
fn sink_reads_everything(sink: &str) -> bool {
    let mut words = sink.split_whitespace();
    let program = words.next().unwrap_or("");
    let mut flags = words.take_while(|w| w.starts_with('-'));
    let short_has =
        |w: &str, set: &str| !w.starts_with("--") && w.chars().skip(1).any(|c| set.contains(c));
    match program {
        "tail" => !flags.any(|w| short_has(w, "fF") || w.starts_with("--follow")),
        "grep" => !flags.any(|w| {
            short_has(w, "qmlL")
                || ["--quiet", "--silent", "--max-count", "--files-with"]
                    .iter()
                    .any(|l| w.starts_with(l))
        }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrapped(pipeline: &str) -> String {
        format!("({GUARD} {pipeline})")
    }

    #[test]
    fn a_check_piped_to_a_full_reader_gets_the_real_status() {
        for cmd in [
            "cargo test 2>&1 | tail -30",
            "cargo clippy --all-targets 2>&1 | tail -8",
            "npm run build | tail -5",
            "pnpm test 2>&1 | grep -E 'FAIL|passed'",
            "npx tsc --noEmit | tail",
            "CI=1 pytest -x 2>&1 | tail -20",
            "python3 -m pytest | grep -c FAILED",
            "make check 2>&1 | tail -3",
            "./node_modules/.bin/jest | tail",
        ] {
            assert_eq!(wrap(cmd), Some(wrapped(cmd)), "{cmd}");
        }
    }

    #[test]
    fn earlier_chain_segments_stay_outside() {
        assert_eq!(
            wrap("cd sub && cargo test | tail -5"),
            Some(format!("cd sub && {}", wrapped("cargo test | tail -5")))
        );
    }

    #[test]
    fn only_the_last_statement_changes() {
        assert_eq!(
            wrap("cd sub\ncargo fmt\ncargo test | tail -5"),
            Some(format!(
                "cd sub\ncargo fmt\n{}",
                wrapped("cargo test | tail -5")
            ))
        );
        assert_eq!(
            wrap("cargo fmt | tail -1; make | tail -2"),
            Some(format!("cargo fmt | tail -1;{}", wrapped("make | tail -2")))
        );
    }

    #[test]
    fn multi_line_text_is_never_mistaken_for_the_last_statement() {
        for cmd in [
            "echo \"a\ncargo test | tail\"",
            "cat <<EOF\ncargo test | tail\nEOF",
            "FOO=1 \\\n  make | tail",
            "x=$(a\nmake | tail)",
        ] {
            assert_eq!(wrap(cmd), None, "{cmd}");
        }
    }

    #[test]
    fn a_status_something_else_reads_is_left_alone() {
        for cmd in [
            "cargo test | tail -5 && git push",
            "cargo test | tail -5 || echo failed",
            "cargo test | tail -5; echo done",
            "for t in a b; do cargo test | tail -5; done",
            "{ make | tail; }",
            "if cargo test | grep ok; then echo y; fi",
            "cargo test | tail -5 &",
            "(cargo test | tail -5)",
        ] {
            assert_eq!(wrap(cmd), None, "{cmd}");
        }
    }

    #[test]
    fn a_sink_that_stops_early_would_report_a_false_failure() {
        for cmd in [
            "cargo test | head -5",
            "cargo test | tail -f",
            "make run | tail -F",
            "cargo test | tail --follow",
            "cargo test | grep -q ok",
            "cargo test | grep -m1 FAILED",
            "cargo test | grep -lE x",
            "cargo test | less",
            "cargo test | tee out.log",
        ] {
            assert_eq!(wrap(cmd), None, "{cmd}");
        }
    }

    #[test]
    fn tools_whose_status_is_not_a_verdict_are_left_alone() {
        for cmd in [
            "grep -r foo . | tail",
            "find . | tail",
            "git log | tail -3",
            "npm run dev | tail",
            "cargo run | tail",
        ] {
            assert_eq!(wrap(cmd), None, "{cmd}");
        }
    }

    #[test]
    fn anything_that_is_not_one_flat_pipe_is_left_alone() {
        for cmd in [
            "cargo test",
            "cargo test | grep -v x | tail",
            "cargo test |& tail",
            "cargo test > out.log | tail",
            "cargo test $(cat args) | tail",
            "cargo test 'a|b'",
            "pytest é>x | tail",
            "cat <<EOF | tail\nx\nEOF",
        ] {
            assert_eq!(wrap(cmd), None, "{cmd}");
        }
    }
}
