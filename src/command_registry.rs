//! Single source of truth for per-command behavior.
//!
//! Before this module existed, the knowledge of "which commands trs handles
//! and how" was duplicated across four sites that had to be kept in sync by
//! hand — and the code comments admitted as much ("keep loosely in sync with
//! classifier.rs"):
//!
//! - `rewrite_decide.rs`  → `REWRITE_PREFIXES`     (is the command rewrite-eligible?)
//! - `classifier.rs`      → `keep_ratio()`         (how much output survives compression?)
//! - `classifier_exec.rs` → `combine_stderr` match (does primary output go to stderr?)
//! - `stats_coverage.rs`  → `is_known_binary()`    (does trs compress this beyond ANSI?)
//!
//! Adding a single new command meant editing all four, and forgetting one was
//! easy. This table centralizes the four *flat* per-command facts so a new
//! command is one row.
//!
//! What intentionally stays OUT of this table: the subcommand → parser dispatch
//! (`classify_command` in `classifier.rs`). That logic is genuinely
//! command-specific control flow (`git stash show -p`, `npm run build|test|lint`,
//! `python -m <module>`), not a flat lookup, and forcing it into data would be
//! over-engineering. The invariant test below pins that every command with a
//! parser is at least *registered* here, so the two never silently drift.

/// Keep-ratio used for commands with no registry entry (generic compression).
pub(crate) const DEFAULT_KEEP_RATIO: f64 = 0.50;

/// stderr-stream policy: which subcommands write their primary output
/// (errors, warnings, results) to stderr instead of stdout, so the executor
/// must merge both streams before parsing.
#[derive(Clone, Copy)]
pub(crate) enum Stderr {
    /// Primary output is on stdout; stderr is passthrough (progress, warnings).
    Never,
    /// Primary output is on stderr for every invocation of this command.
    Always,
    /// Primary output is on stderr only for these subcommands.
    Subcmds(&'static [&'static str]),
}

impl Stderr {
    fn matches(&self, subcmd: &str) -> bool {
        match self {
            Stderr::Never => false,
            Stderr::Always => true,
            Stderr::Subcmds(list) => list.contains(&subcmd),
        }
    }
}

/// Benchmarked keep-ratio for a command: a default plus optional
/// per-subcommand overrides. The fraction of input that typically remains
/// after trs compression.
pub(crate) struct KeepRatio {
    /// Ratio applied to any subcommand without a specific override.
    pub default: f64,
    /// `(subcommand, ratio)` overrides, checked before the default.
    pub overrides: &'static [(&'static str, f64)],
}

impl KeepRatio {
    /// A single ratio that applies to every subcommand.
    const fn flat(r: f64) -> Self {
        KeepRatio {
            default: r,
            overrides: &[],
        }
    }

    fn lookup(&self, subcmd: &str) -> f64 {
        for (name, ratio) in self.overrides {
            if *name == subcmd {
                return *ratio;
            }
        }
        self.default
    }
}

/// Everything trs knows about one command (and its aliases).
pub(crate) struct CommandSpec {
    /// Canonical name plus aliases that share identical handling
    /// (e.g. `["ls", "lsd", "exa", "eza"]`).
    pub names: &'static [&'static str],
    /// Counted as a "known binary" in coverage stats — trs compresses its
    /// output beyond plain ANSI/whitespace stripping.
    pub known: bool,
    /// Estimated output size after compression.
    pub keep_ratio: KeepRatio,
    /// stderr-stream policy for the executor.
    pub stderr: Stderr,
}

/// The registry. One row per command/alias-family. Order is irrelevant: each
/// (command, subcommand) maps to exactly one spec, so lookups are unambiguous.
#[rustfmt::skip]
pub(crate) static REGISTRY: &[CommandSpec] = &[
    // ---- Git ----
    CommandSpec {
        names: &["git"], known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("status", 0.20), ("diff", 0.10), ("log", 0.10), ("branch", 0.11),
            ("show", 0.10), ("stash", 0.10), ("pull", 0.15), ("fetch", 0.15),
            ("grep", 0.40), ("commit", 0.20),
        ]},
        stderr: Stderr::Never,
    },

    // ---- File listing ----
    CommandSpec { names: &["ls", "lsd", "exa", "eza"], known: true,
        keep_ratio: KeepRatio::flat(0.18), stderr: Stderr::Never },
    CommandSpec { names: &["tree"], known: true,
        keep_ratio: KeepRatio::flat(0.30), stderr: Stderr::Never },

    // ---- Search ----
    CommandSpec { names: &["grep", "rg", "ag"], known: true,
        keep_ratio: KeepRatio::flat(0.40), stderr: Stderr::Never },
    // `ack` shares the Grep parser but historically had no keep_ratio entry,
    // so it keeps the default ratio — kept separate to preserve that.
    CommandSpec { names: &["ack"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },
    CommandSpec { names: &["find", "fd"], known: true,
        keep_ratio: KeepRatio::flat(0.52), stderr: Stderr::Never },

    // ---- Logs ----
    CommandSpec { names: &["tail"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },
    // `journalctl` shares the Logs parser. `known: true` because it is
    // parsed: `is_known_binary` feeds only `stats --coverage`, and reporting
    // a parsed command as a gap sends someone to write one that exists.
    CommandSpec { names: &["journalctl"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- Rust / Cargo ----
    CommandSpec {
        names: &["cargo"], known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("clippy", 0.15), ("build", 0.10), ("check", 0.10), ("test", 0.05),
            ("install", 0.20), ("i", 0.20), ("fmt", 0.10),
            ("tree", 0.40), ("ls", 0.40), ("list", 0.40), ("freeze", 0.40),
        ]},
        stderr: Stderr::Subcmds(&["clippy", "build", "check"]),
    },

    // ---- JS package managers ----
    CommandSpec {
        names: &["npm"], known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("install", 0.20), ("i", 0.20),
            ("ls", 0.40), ("list", 0.40), ("tree", 0.40), ("freeze", 0.40),
            ("test", 0.10), ("run", 0.15),
        ]},
        stderr: Stderr::Never,
    },
    CommandSpec {
        names: &["pnpm"], known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("install", 0.20), ("i", 0.20), ("test", 0.10), ("run", 0.15),
        ]},
        stderr: Stderr::Never,
    },
    CommandSpec {
        names: &["bun"], known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("test", 0.10), ("run", 0.15),
        ]},
        stderr: Stderr::Never,
    },
    CommandSpec {
        names: &["yarn"], known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("install", 0.20), ("i", 0.20), ("test", 0.10), ("run", 0.15),
        ]},
        stderr: Stderr::Never,
    },
    CommandSpec { names: &["npx", "bunx"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- Python package managers ----
    CommandSpec {
        names: &["pip", "pip3"], known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("install", 0.20), ("i", 0.20),
            ("ls", 0.40), ("list", 0.40), ("tree", 0.40), ("freeze", 0.40),
        ]},
        stderr: Stderr::Never,
    },
    CommandSpec { names: &["uv"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },
    // `poetry` was never in the rewrite list (it is reached via `poetry run`
    // transparent-prefix handling) but is a known binary with ratio overrides.
    CommandSpec {
        names: &["poetry"], known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("install", 0.20), ("add", 0.20), ("update", 0.20), ("run", 0.15),
        ]},
        stderr: Stderr::Never,
    },

    // ---- Test runners ----
    CommandSpec { names: &["pytest"], known: true,
        keep_ratio: KeepRatio::flat(0.10), stderr: Stderr::Never },
    CommandSpec { names: &["jest"], known: true,
        keep_ratio: KeepRatio::flat(0.10), stderr: Stderr::Never },
    CommandSpec { names: &["vitest"], known: true,
        keep_ratio: KeepRatio::flat(0.10), stderr: Stderr::Never },

    // ---- Build tools / compilers ----
    CommandSpec { names: &["make"], known: true,
        keep_ratio: KeepRatio::flat(0.15), stderr: Stderr::Always },
    // `cmake` merges stderr but had no keep_ratio entry (default ratio).
    CommandSpec { names: &["cmake"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Always },
    CommandSpec { names: &["tsc"], known: true,
        keep_ratio: KeepRatio::flat(0.15), stderr: Stderr::Never },
    CommandSpec { names: &["gcc", "g++"], known: true,
        keep_ratio: KeepRatio::flat(0.15), stderr: Stderr::Always },
    // `clang`/`javac` merge stderr but had no keep_ratio entry (default ratio).
    CommandSpec { names: &["clang", "javac"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Always },
    CommandSpec {
        names: &["go"], known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[("test", 0.08)] },
        stderr: Stderr::Subcmds(&["build"]),
    },
    CommandSpec { names: &["swift"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Subcmds(&["build"]) },
    CommandSpec { names: &["xcodebuild"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Always },

    // ---- Linters ----
    CommandSpec { names: &["eslint", "biome", "ruff", "pylint", "golangci-lint"],
        known: true,
        keep_ratio: KeepRatio::flat(0.15), stderr: Stderr::Always },

    // ---- Containers / orchestration ----
    CommandSpec {
        names: &["docker"], known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("ps", 0.30), ("logs", 0.50),
        ]},
        stderr: Stderr::Never,
    },
    // `known: false` on purpose: no dedicated parser, only the generic
    // reducer. Claiming otherwise hides a real gap from `stats --coverage`.
    CommandSpec { names: &["kubectl"], known: false,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- GitHub CLI ----
    CommandSpec {
        names: &["gh"], known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("pr", 0.30), ("issue", 0.30), ("run", 0.30),
            // Measured over pulls/repos/commits/issues responses: the link
            // boilerplate is a consistent ~62% of a GitHub REST body.
            ("api", 0.38),
        ]},
        stderr: Stderr::Never,
    },

    // ---- Cloud CLIs ----
    // Recursive s3 output is ~1 line per object: the most compressible shape
    // trs sees, hence the very low keep ratio.
    CommandSpec {
        names: &["aws"], known: true,
        keep_ratio: KeepRatio { default: 0.30, overrides: &[("s3", 0.01)] },
        stderr: Stderr::Always,
    },

    // ---- Environment / misc utilities ----
    CommandSpec { names: &["env", "printenv"], known: true,
        keep_ratio: KeepRatio::flat(0.32), stderr: Stderr::Never },
    CommandSpec { names: &["wc"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },
    CommandSpec { names: &["ps"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },
    CommandSpec { names: &["ping"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- Downloads ----
    CommandSpec { names: &["wget"], known: true,
        keep_ratio: KeepRatio::flat(0.15), stderr: Stderr::Never },
    CommandSpec { names: &["curl"], known: true,
        keep_ratio: KeepRatio::flat(0.15), stderr: Stderr::Never },

    // ---- Package managers (system) ----
    CommandSpec { names: &["brew"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- Python interpreter ----
    CommandSpec { names: &["python", "python3"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- LLM tooling ----
    // `list`/`ps`/`pull` are parsed; `run`/`show`/`serve` fall to generic.
    CommandSpec { names: &["ollama"], known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("list", 0.45), ("ls", 0.45), ("ps", 0.45), ("pull", 0.15),
        ]},
        stderr: Stderr::Never },

    // ---- Database clients: routed to the Db parser, so `known: true`. ----
    CommandSpec { names: &["psql", "mysql", "sqlite3", "mariadb"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- Generic CLIs: rewrite-eligible for ANSI/whitespace compression,
    //      but no dedicated parser and not counted in coverage stats. ----
    CommandSpec { names: &["bash", "node"], known: false,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- System inventory: many rows, few columns that matter. ----
    CommandSpec { names: &["du", "lsof", "pgrep"], known: true,
        keep_ratio: KeepRatio::flat(0.35), stderr: Stderr::Never },

    // ---- Verbatim: output is the payload, never compressed. ----
    CommandSpec { names: VERBATIM_COMMANDS, known: true,
        keep_ratio: KeepRatio::flat(1.0), stderr: Stderr::Never },

    // ---- Recognized-but-not-rewritten: intercepted in the fast path
    //      (cat/head) or shell builtins. Counted as known for coverage.
    //      `sed` moved to VERBATIM_COMMANDS: read_intercept only catches
    //      `sed -n X,Yp FILE`, so every transform form fell through to the
    //      generic reducer and had its indentation flattened. ----
    CommandSpec { names: &["cat", "head"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },
    CommandSpec { names: &["trs", "cd", "echo"], known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },
];

/// Look up the spec for a command name (matches canonical name or any alias).
pub(crate) fn spec(cmd: &str) -> Option<&'static CommandSpec> {
    REGISTRY.iter().find(|s| s.names.contains(&cmd))
}

/// Benchmarked keep-ratio for estimating compressed output size per command.
/// Returns the fraction of input that typically remains after trs compression.
pub(crate) fn keep_ratio(cmd: &str, subcmd: &str) -> f64 {
    match spec(cmd) {
        Some(s) => s.keep_ratio.lookup(subcmd),
        None => DEFAULT_KEEP_RATIO,
    }
}

/// Whether the executor should merge stderr into stdout before parsing.
/// Several commands write their primary output (errors, warnings, results)
/// to stderr; for those the parser must see both streams combined.
pub(crate) fn combine_stderr(cmd: &str, subcmd: &str) -> bool {
    match spec(cmd) {
        Some(s) => s.stderr.matches(subcmd),
        None => false,
    }
}

/// Commands whose stdout IS the payload: a byte-level transform or re-layout
/// of their input, where runs of spaces and blank lines carry meaning. Generic
/// compression collapses exactly those, so `awk 'NR<=4' x.py` came back with
/// every indent flattened to one space, which is broken Python, not a terser
/// rendering of it. trs passes these through untouched: no rewrite, and no
/// compression when invoked directly. Same reasoning that already keeps
/// `cat`/`head`/`echo` out of the hook's rewrite list.
///
/// Referenced by the `REGISTRY` row above, so the list lives in one place.
pub(crate) const VERBATIM_COMMANDS: &[&str] = &[
    "awk", "base64", "basenc", "column", "comm", "cut", "expand", "fold", "hexdump", "iconv",
    "join", "jq", "nl", "od", "paste", "printf", "rev", "sed", "sort", "strings", "tac", "tr",
    "unexpand", "uniq", "xxd", "yq",
];

/// Whether trs must hand this command's output back byte for byte.
pub(crate) fn is_verbatim_command(cmd: &str) -> bool {
    VERBATIM_COMMANDS.contains(&cmd)
}

/// Flags by which a caller says "give me exactly these fields and nothing
/// else". Their output is already the answer, so a parser that prunes it
/// removes something the caller named on purpose: `gh api --jq '{name, url}'`
/// returns valid JSON that the gh-api pruner would strip `url` out of.
///
/// A table rather than an inline `cmd == "gh"` check, so the next tool with a
/// selector is one row and not another special case in a shared predicate.
const FIELD_SELECTOR_FLAGS: &[(&str, &[&str])] = &[("gh", &["--jq", "-q", "--template", "-t"])];

/// Whether the caller pre-selected their fields on this command line.
fn caller_selected_fields(cmd: &str, rest: &str) -> bool {
    FIELD_SELECTOR_FLAGS
        .iter()
        .find(|(bin, _)| *bin == cmd)
        .is_some_and(|(_, flags)| rest.split_whitespace().any(|t| flags.contains(&t)))
}

/// Same question, seeing through a shell wrapper: `bash -c "column -t x"`
/// must be left alone exactly like a bare `column -t x`. `rest` is whatever
/// follows the binary, either the raw remainder of the line (the hook, which
/// has not tokenized yet) or the joined argv (the executor, which has).
///
/// Deliberately looser than `unwrap_shell_c`: that gate has to be strict
/// because it picks a parser, and the wrong pick produces garbage. Here the
/// only decision is "leave the bytes alone", where a false positive costs
/// some compression and a false negative corrupts output.
///
/// Known limit: a compound script (`cd x && awk …`) reports its first token,
/// so an inner verbatim command later in the chain is not seen.
pub(crate) fn is_verbatim_invocation(cmd: &str, rest: &str) -> bool {
    if is_verbatim_command(cmd) || caller_selected_fields(cmd, rest) {
        return true;
    }
    if !matches!(cmd, "bash" | "sh" | "zsh" | "dash") {
        return false;
    }
    let mut toks = rest.split_whitespace();
    while let Some(t) = toks.next() {
        if t.starts_with('-') && t.ends_with('c') {
            return toks
                .next()
                .map(|s| s.trim_start_matches(['"', '\'']))
                .and_then(|s| s.split_whitespace().next())
                .is_some_and(is_verbatim_command);
        }
    }
    false
}

/// Binaries trs knows how to handle (compresses output beyond ANSI stripping).
/// Used by coverage stats to separate handled traffic from passthrough.
pub(crate) fn is_known_binary(cmd: &str) -> bool {
    spec(cmd).map_or(false, |s| s.known)
}

#[cfg(test)]
#[path = "command_registry_tests.rs"]
mod tests;
