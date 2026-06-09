//! Quality harness — measures what compression PRESERVES, not just bytes.
//!
//! Every case runs a fixture through its parser (compact format, what agents
//! see) and asserts signal-preservation invariants against the raw input:
//!
//!   Codes   — compiler/test error codes (E0xxx, TSxxxx) must survive
//!   Paths   — file basenames on failure lines must survive
//!   Marker  — if raw shows failures, compact must say so
//!
//! All cases also report their compression ratio (run with
//! `cargo test --test quality_harness -- --nocapture` for the table).
//! A case opting OUT of a check is visible debt: it means that parser
//! cannot yet guarantee that invariant — fix the parser, then tighten.

use std::collections::BTreeSet;
use std::process::Command;

#[derive(Clone, Copy, PartialEq)]
enum Check {
    Codes,
    Paths,
    Marker,
}

struct Case {
    name: &'static str,
    /// argv after `trs` — fixture path appended as `--file <fixture>`.
    args: &'static [&'static str],
    fixture: &'static str,
    checks: &'static [Check],
}

const ALL: &[Check] = &[Check::Codes, Check::Paths, Check::Marker];
const RATIO_ONLY: &[Check] = &[];

const CASES: &[Case] = &[
    Case {
        name: "pytest mixed",
        args: &["parse", "test", "--runner", "pytest"],
        fixture: "pytest_mixed.txt",
        checks: ALL,
    },
    Case {
        name: "pytest single failed",
        args: &["parse", "test", "--runner", "pytest"],
        fixture: "pytest_single_failed.txt",
        checks: ALL,
    },
    Case {
        name: "jest mixed",
        args: &["parse", "test", "--runner", "jest"],
        fixture: "jest_mixed.txt",
        checks: ALL,
    },
    Case {
        name: "vitest mixed",
        args: &["parse", "test", "--runner", "vitest"],
        fixture: "vitest_mixed.txt",
        checks: ALL,
    },
    Case {
        name: "bun mixed",
        args: &["parse", "test", "--runner", "bun"],
        fixture: "bun_test_mixed.txt",
        checks: ALL,
    },
    Case {
        name: "npm single failed",
        args: &["parse", "test", "--runner", "npm"],
        fixture: "npm_test_single_failed.txt",
        checks: ALL,
    },
    Case {
        name: "pnpm mixed",
        args: &["parse", "test", "--runner", "pnpm"],
        fixture: "pnpm_test_mixed.txt",
        checks: ALL,
    },
    Case {
        name: "logs with exceptions",
        args: &["parse", "logs"],
        fixture: "logs_with_exceptions.txt",
        checks: &[Check::Marker],
    },
    Case {
        name: "logs mixed format",
        args: &["parse", "logs"],
        fixture: "logs_mixed_format.txt",
        checks: &[Check::Marker],
    },
    Case {
        name: "git status mixed",
        args: &["parse", "git-status"],
        fixture: "git_status_mixed.txt",
        checks: RATIO_ONLY,
    },
    Case {
        name: "git diff mixed",
        args: &["parse", "git-diff"],
        fixture: "git_diff_mixed.txt",
        checks: RATIO_ONLY,
    },
    Case {
        name: "grep mixed",
        args: &["parse", "grep"],
        fixture: "grep_mixed.txt",
        checks: RATIO_ONLY,
    },
];

fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("tests/fixture_data").join(name)
}

fn run_compact(case: &Case) -> String {
    let bin = env!("CARGO_BIN_EXE_trs");
    let out = Command::new(bin)
        .args(case.args)
        .arg("--file")
        .arg(fixture_path(case.fixture))
        .output()
        .expect("run trs");
    assert!(
        out.status.success(),
        "{}: trs exited {:?}",
        case.name,
        out.status.code()
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// Lines that carry failure signal in raw output.
fn is_signal_line(line: &str) -> bool {
    line.contains("error[")
        || line.contains("error:")
        || line.contains("Error:")
        || line.contains("ERROR")
        || line.contains("FAILED")
        || line.contains("FAIL ")
        || line.contains("✗")
        || line.contains("✘")
        || line.contains("panicked")
        || line.contains("Exception")
}

/// Compiler/test error codes: rustc `E0xxx`, TypeScript `TSxxxx(x)`.
fn error_codes(raw: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let bytes = raw.as_bytes();
    for (i, w) in raw.char_indices() {
        let rest = &raw[i..];
        let code_len = |prefix: &str, digits: std::ops::RangeInclusive<usize>| -> Option<usize> {
            let r = rest.strip_prefix(prefix)?;
            let n = r.chars().take_while(|c| c.is_ascii_digit()).count();
            digits.contains(&n).then_some(prefix.len() + n)
        };
        // Word boundary: previous byte must not be alphanumeric.
        if i > 0 && bytes[i - 1].is_ascii_alphanumeric() {
            continue;
        }
        let len = match w {
            'E' => code_len("E", 4..=4),
            'T' => code_len("TS", 4..=5),
            _ => None,
        };
        if let Some(len) = len {
            out.insert(rest[..len].to_string());
        }
    }
    out
}

/// File basenames mentioned on failure lines (path-looking tokens only).
fn signal_file_basenames(raw: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in raw.lines().filter(|l| is_signal_line(l)) {
        for token in line.split_whitespace() {
            let token = token.trim_matches(|c: char| "()[]<>\"',;".contains(c));
            // `path::test_name` (pytest) → keep the path part.
            let token = token.split("::").next().unwrap_or(token);
            // Strip :line:col suffixes.
            let token = token.split(':').next().unwrap_or(token);
            let looks_like_path = token.contains('/')
                || std::path::Path::new(token)
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| {
                        matches!(e, "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "java")
                    });
            if !looks_like_path || token.starts_with('-') {
                continue;
            }
            if let Some(base) = token.rsplit('/').next() {
                if base.contains('.') && base.len() > 3 {
                    out.insert(base.to_string());
                }
            }
        }
    }
    out
}

fn has_failure_marker(compact: &str) -> bool {
    let l = compact.to_lowercase();
    l.contains("fail") || l.contains("error") || l.contains("fatal") || l.contains("✗")
}

#[test]
fn signal_preservation() {
    let mut failures: Vec<String> = Vec::new();
    println!(
        "\n{:<22} {:>8} {:>8} {:>6}  checks",
        "case", "raw", "compact", "saved"
    );
    for case in CASES {
        let raw = std::fs::read_to_string(fixture_path(case.fixture)).expect("fixture");
        let compact = run_compact(case);
        let saved = if raw.is_empty() {
            0
        } else {
            100 - (compact.len() * 100 / raw.len().max(1)).min(100)
        };
        println!(
            "{:<22} {:>8} {:>8} {:>5}%  {}",
            case.name,
            raw.len(),
            compact.len(),
            saved,
            case.checks.len()
        );

        if case.checks.contains(&Check::Codes) {
            for code in error_codes(&raw) {
                if !compact.contains(&code) {
                    failures.push(format!("{}: error code {} lost", case.name, code));
                }
            }
        }
        if case.checks.contains(&Check::Paths) {
            for base in signal_file_basenames(&raw) {
                if !compact.contains(&base) {
                    failures.push(format!("{}: failing file {} lost", case.name, base));
                }
            }
        }
        if case.checks.contains(&Check::Marker) {
            let raw_has_failures = raw.lines().any(is_signal_line);
            if raw_has_failures && !has_failure_marker(&compact) {
                failures.push(format!(
                    "{}: raw shows failures but compact has no failure marker",
                    case.name
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "\nsignal lost in compression:\n  {}",
        failures.join("\n  ")
    );
}
