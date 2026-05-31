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
    /// Eligible for explicit `trs` rewrite wrapping. (Unknown commands are
    /// still wrapped by the generic catch-all in `maybe_rewrite`; this flag
    /// records the documented intent and drives `is_rewrite_command`.)
    pub rewrite: bool,
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
        names: &["git"], rewrite: true, known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("status", 0.20), ("diff", 0.10), ("log", 0.10), ("branch", 0.11),
            ("show", 0.10), ("stash", 0.10), ("pull", 0.15), ("fetch", 0.15),
            ("grep", 0.40),
        ]},
        stderr: Stderr::Never,
    },

    // ---- File listing ----
    CommandSpec { names: &["ls", "lsd", "exa", "eza"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(0.18), stderr: Stderr::Never },
    CommandSpec { names: &["tree"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(0.30), stderr: Stderr::Never },

    // ---- Search ----
    CommandSpec { names: &["grep", "rg", "ag"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(0.40), stderr: Stderr::Never },
    // `ack` shares the Grep parser but historically had no keep_ratio entry,
    // so it keeps the default ratio — kept separate to preserve that.
    CommandSpec { names: &["ack"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },
    CommandSpec { names: &["find", "fd"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(0.52), stderr: Stderr::Never },

    // ---- Logs ----
    CommandSpec { names: &["tail"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },
    // `journalctl` shares the Logs parser but was never in the rewrite/known
    // lists — registered so the parser-coverage invariant holds.
    CommandSpec { names: &["journalctl"], rewrite: false, known: false,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- Rust / Cargo ----
    CommandSpec {
        names: &["cargo"], rewrite: true, known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("clippy", 0.15), ("build", 0.10), ("check", 0.10), ("test", 0.05),
            ("install", 0.20), ("i", 0.20),
            ("tree", 0.40), ("ls", 0.40), ("list", 0.40), ("freeze", 0.40),
        ]},
        stderr: Stderr::Subcmds(&["clippy", "build", "check"]),
    },

    // ---- JS package managers ----
    CommandSpec {
        names: &["npm"], rewrite: true, known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("install", 0.20), ("i", 0.20),
            ("ls", 0.40), ("list", 0.40), ("tree", 0.40), ("freeze", 0.40),
            ("test", 0.10), ("run", 0.15),
        ]},
        stderr: Stderr::Never,
    },
    CommandSpec {
        names: &["pnpm"], rewrite: true, known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("install", 0.20), ("i", 0.20), ("test", 0.10), ("run", 0.15),
        ]},
        stderr: Stderr::Never,
    },
    CommandSpec {
        names: &["bun"], rewrite: true, known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("test", 0.10), ("run", 0.15),
        ]},
        stderr: Stderr::Never,
    },
    CommandSpec {
        names: &["yarn"], rewrite: true, known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("install", 0.20), ("i", 0.20), ("test", 0.10), ("run", 0.15),
        ]},
        stderr: Stderr::Never,
    },
    CommandSpec { names: &["npx"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- Python package managers ----
    CommandSpec {
        names: &["pip", "pip3"], rewrite: true, known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("install", 0.20), ("i", 0.20),
            ("ls", 0.40), ("list", 0.40), ("tree", 0.40), ("freeze", 0.40),
        ]},
        stderr: Stderr::Never,
    },
    CommandSpec { names: &["uv"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },
    // `poetry` was never in the rewrite list (it is reached via `poetry run`
    // transparent-prefix handling) but is a known binary with ratio overrides.
    CommandSpec {
        names: &["poetry"], rewrite: false, known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("install", 0.20), ("add", 0.20), ("update", 0.20), ("run", 0.15),
        ]},
        stderr: Stderr::Never,
    },

    // ---- Test runners ----
    CommandSpec { names: &["pytest"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(0.10), stderr: Stderr::Never },
    CommandSpec { names: &["jest"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(0.10), stderr: Stderr::Never },
    CommandSpec { names: &["vitest"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(0.10), stderr: Stderr::Never },

    // ---- Build tools / compilers ----
    CommandSpec { names: &["make"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(0.15), stderr: Stderr::Always },
    // `cmake` merges stderr but had no keep_ratio entry (default ratio).
    CommandSpec { names: &["cmake"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Always },
    CommandSpec { names: &["tsc"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(0.15), stderr: Stderr::Never },
    CommandSpec { names: &["gcc", "g++"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(0.15), stderr: Stderr::Always },
    // `clang`/`javac` merge stderr but had no keep_ratio entry (default ratio).
    CommandSpec { names: &["clang", "javac"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Always },
    CommandSpec {
        names: &["go"], rewrite: false, known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[("test", 0.08)] },
        stderr: Stderr::Subcmds(&["build"]),
    },
    CommandSpec { names: &["swift"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Subcmds(&["build"]) },
    CommandSpec { names: &["xcodebuild"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Always },

    // ---- Linters ----
    CommandSpec { names: &["eslint", "biome", "ruff", "pylint", "golangci-lint"],
        rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(0.15), stderr: Stderr::Always },

    // ---- Containers / orchestration ----
    CommandSpec {
        names: &["docker"], rewrite: true, known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("ps", 0.30), ("logs", 0.50),
        ]},
        stderr: Stderr::Never,
    },
    CommandSpec { names: &["kubectl"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- GitHub CLI ----
    CommandSpec {
        names: &["gh"], rewrite: true, known: true,
        keep_ratio: KeepRatio { default: DEFAULT_KEEP_RATIO, overrides: &[
            ("pr", 0.30), ("issue", 0.30), ("run", 0.30),
        ]},
        stderr: Stderr::Never,
    },

    // ---- Environment / misc utilities ----
    CommandSpec { names: &["env", "printenv"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(0.32), stderr: Stderr::Never },
    CommandSpec { names: &["wc"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },
    CommandSpec { names: &["ps"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },
    CommandSpec { names: &["ping"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- Downloads ----
    CommandSpec { names: &["wget"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(0.15), stderr: Stderr::Never },
    CommandSpec { names: &["curl"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(0.15), stderr: Stderr::Never },

    // ---- Package managers (system) ----
    CommandSpec { names: &["brew"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- Python interpreter ----
    CommandSpec { names: &["python", "python3"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- LLM tooling ----
    CommandSpec { names: &["ollama"], rewrite: true, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- Database clients (parser exists; never in rewrite/known lists) ----
    CommandSpec { names: &["psql", "mysql", "sqlite3", "mariadb"], rewrite: false, known: false,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- Generic CLIs: rewrite-eligible for ANSI/whitespace compression,
    //      but no dedicated parser and not counted in coverage stats. ----
    CommandSpec { names: &["bash", "node", "awk", "du", "jq"], rewrite: true, known: false,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },

    // ---- Recognized-but-not-rewritten: intercepted in the fast path
    //      (cat/head/sed) or shell builtins. Counted as known for coverage. ----
    CommandSpec { names: &["cat", "head", "sed"], rewrite: false, known: true,
        keep_ratio: KeepRatio::flat(DEFAULT_KEEP_RATIO), stderr: Stderr::Never },
    CommandSpec { names: &["trs", "cd", "echo"], rewrite: false, known: true,
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

/// Whether the command is explicitly rewrite-eligible. Unknown commands are
/// still wrapped by the generic catch-all in `maybe_rewrite`; this records the
/// documented set trs is designed to compress.
pub(crate) fn is_rewrite_command(cmd: &str) -> bool {
    spec(cmd).map_or(false, |s| s.rewrite)
}

/// Binaries trs knows how to handle (compresses output beyond ANSI stripping).
/// Used by coverage stats to separate handled traffic from passthrough.
pub(crate) fn is_known_binary(cmd: &str) -> bool {
    spec(cmd).map_or(false, |s| s.known)
}

#[cfg(test)]
#[path = "command_registry_tests.rs"]
mod tests;
