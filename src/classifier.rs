//! Command classifier and external command execution.
//!
//! Detects what parser to use for external commands and handles
//! the execute → parse → format pipeline.

use crate::{ParseCommands, TestRunner};

// keep_ratio moved to the unified command registry (command_registry.rs).
// Re-exported here so existing `crate::classifier::keep_ratio` callers keep
// working without churn.
pub(crate) use crate::command_registry::keep_ratio;

/// Build a full command string from command name and arguments.
pub(crate) fn full_cmd(cmd: &str, args: &[String]) -> String {
    if args.is_empty() {
        cmd.to_string()
    } else {
        format!("{} {}", cmd, args.join(" "))
    }
}

pub(crate) use crate::exec::build_command;

pub(crate) use crate::classifier_args::preprocess_tail_args;
use crate::classifier_args::{
    has_structured_output_flag, strip_git_global_opts, unwrap_shell_c, unwrap_timeout,
};

/// Classify an external command into the parser to pipe through, or None
/// for passthrough (generic compression).
pub(crate) fn classify_command(cmd: &str, args: &[String]) -> Option<ParseCommands> {
    // If user explicitly requests structured output, don't parse — passthrough
    if has_structured_output_flag(args) {
        return None;
    }

    // Route by basename so absolute/relative-path invocations
    // (`/opt/homebrew/bin/gh`, `./node_modules/.bin/eslint`) reach the same
    // parser as the bare name. Field data: `gh` invoked by absolute path
    // averaged 40 KB/cmd uncompressed before this.
    let cmd = cmd.rsplit(['/', '\\']).next().unwrap_or(cmd);

    // For git commands, strip global options before detecting subcommand
    let effective_args;
    let args_ref = if cmd == "git" {
        effective_args = strip_git_global_opts(args);
        &effective_args
    } else {
        args
    };

    let subcmd = args_ref.first().map(|s| s.as_str()).unwrap_or("");

    match cmd {
        // Git commands
        "git" => match subcmd {
            "status" => Some(ParseCommands::GitStatus {
                file: None,
                count: None,
            }),
            "diff" => Some(ParseCommands::GitDiff { file: None }),
            "log" => Some(ParseCommands::GitLog {
                file: None,
                truncate: None,
            }),
            "branch" => Some(ParseCommands::GitBranch { file: None }),
            // git show → commit header + diff; diff parser handles both
            "show" => Some(ParseCommands::GitDiff { file: None }),
            // git stash show -p → standard diff output
            // git stash pop/apply → diff + status after applying
            "stash" => {
                let stash_sub = args_ref.get(1).map(|s| s.as_str()).unwrap_or("");
                match stash_sub {
                    "show" if args_ref.iter().any(|a| a == "-p" || a == "--patch") => {
                        Some(ParseCommands::GitDiff { file: None })
                    }
                    "pop" | "apply" => Some(ParseCommands::GitDiff { file: None }),
                    _ => None,
                }
            }
            "pull" | "fetch" => Some(ParseCommands::GitPull { file: None }),
            // Field data: 192 cmds at 43% low compression before this route.
            "commit" => Some(ParseCommands::GitCommit { file: None }),
            "grep" => Some(ParseCommands::Grep { file: None }),
            // One path per line — same shape as find output (field data:
            // 100% low compression before this route).
            "ls-files" => Some(ParseCommands::Find { file: None }),
            _ => None,
        },

        // File listing
        "ls" | "lsd" | "exa" | "eza" => Some(ParseCommands::Ls { file: None }),
        "tree" => Some(ParseCommands::Tree { file: None }),

        // Search
        "grep" | "rg" | "ag" | "ack" => Some(ParseCommands::Grep { file: None }),
        "find" | "fd" => Some(ParseCommands::Find { file: None }),

        // Logs
        "tail" | "journalctl" => Some(ParseCommands::Logs { file: None }),

        // Docker
        "docker" => match subcmd {
            "ps" => Some(ParseCommands::DockerPs { file: None }),
            "logs" => Some(ParseCommands::DockerLogs { file: None }),
            "build" => Some(ParseCommands::Build { file: None }),
            _ => None,
        },

        // Test runners
        "pytest" => Some(ParseCommands::Test {
            runner: Some(TestRunner::Pytest),
            file: None,
        }),
        "jest" => Some(ParseCommands::Test {
            runner: Some(TestRunner::Jest),
            file: None,
        }),
        "vitest" => Some(ParseCommands::Test {
            runner: Some(TestRunner::Vitest),
            file: None,
        }),

        // Package managers — subcommand-aware
        "npm" => match subcmd {
            "test" => Some(ParseCommands::Test {
                runner: Some(TestRunner::Npm),
                file: None,
            }),
            "ls" | "list" => Some(ParseCommands::Deps { file: None }),
            "install" | "i" | "ci" => Some(ParseCommands::Install { file: None }),
            "audit" | "outdated" => Some(ParseCommands::Deps { file: None }),
            "run" => {
                let script = args_ref.get(1).map(|s| s.as_str()).unwrap_or("");
                match script {
                    s if s.starts_with("build") => Some(ParseCommands::Build { file: None }),
                    s if s.starts_with("test") => Some(ParseCommands::Test {
                        runner: Some(TestRunner::Npm),
                        file: None,
                    }),
                    s if s.starts_with("lint") => Some(ParseCommands::Lint { file: None }),
                    "type-check" | "typecheck" | "check" | "format" | "format:check" => {
                        Some(ParseCommands::Lint { file: None })
                    }
                    _ => None,
                }
            }
            _ => None,
        },
        "pnpm" => match subcmd {
            "test" => Some(ParseCommands::Test {
                runner: Some(TestRunner::Pnpm),
                file: None,
            }),
            "ls" | "list" | "audit" | "outdated" | "why" => {
                Some(ParseCommands::Deps { file: None })
            }
            "install" | "i" | "add" | "update" | "up" => {
                Some(ParseCommands::Install { file: None })
            }
            "run" => {
                let script = args_ref.get(1).map(|s| s.as_str()).unwrap_or("");
                match script {
                    s if s.starts_with("build") => Some(ParseCommands::Build { file: None }),
                    s if s.starts_with("test") => Some(ParseCommands::Test {
                        runner: Some(TestRunner::Pnpm),
                        file: None,
                    }),
                    s if s.starts_with("lint") => Some(ParseCommands::Lint { file: None }),
                    "type-check" | "typecheck" | "check" | "format" | "format:check" => {
                        Some(ParseCommands::Lint { file: None })
                    }
                    _ => None,
                }
            }
            // pnpm dlx <tool> — runs a one-off package. Route the
            // inner tool to its parser just like `npx <tool>` does.
            "dlx" | "exec" => {
                let inner = args_ref.get(1).map(|s| s.as_str()).unwrap_or("");
                match inner {
                    "tsc" | "eslint" | "biome" | "prettier" => {
                        Some(ParseCommands::Lint { file: None })
                    }
                    "jest" => Some(ParseCommands::Test {
                        runner: Some(TestRunner::Jest),
                        file: None,
                    }),
                    "vitest" => Some(ParseCommands::Test {
                        runner: Some(TestRunner::Vitest),
                        file: None,
                    }),
                    _ => None,
                }
            }
            _ => None,
        },
        "bun" => match subcmd {
            "test" => Some(ParseCommands::Test {
                runner: Some(TestRunner::Bun),
                file: None,
            }),
            "install" | "i" => Some(ParseCommands::Install { file: None }),
            "run" => {
                let script = args_ref.get(1).map(|s| s.as_str()).unwrap_or("");
                match script {
                    s if s.starts_with("build") => Some(ParseCommands::Build { file: None }),
                    s if s.starts_with("test") => Some(ParseCommands::Test {
                        runner: Some(TestRunner::Bun),
                        file: None,
                    }),
                    s if s.starts_with("lint") => Some(ParseCommands::Lint { file: None }),
                    "type-check" | "typecheck" | "check" | "format" | "format:check" => {
                        Some(ParseCommands::Lint { file: None })
                    }
                    _ => None,
                }
            }
            _ => None,
        },
        "yarn" => match subcmd {
            "test" => Some(ParseCommands::Test {
                runner: Some(TestRunner::Jest),
                file: None,
            }),
            "list" => Some(ParseCommands::Deps { file: None }),
            "install" | "add" => Some(ParseCommands::Install { file: None }),
            _ => None,
        },

        // Python package managers
        "pip" | "pip3" => match subcmd {
            "list" | "freeze" => Some(ParseCommands::Deps { file: None }),
            "install" => Some(ParseCommands::Install { file: None }),
            _ => None,
        },
        // uv — modern Python package manager (astral-sh/uv). Routes
        // every subcommand to the closest existing parser so uv
        // adopters get the same compression pip users have today.
        "uv" => match subcmd {
            "pip" => {
                let inner = args_ref.get(1).map(|s| s.as_str()).unwrap_or("");
                match inner {
                    "install" => Some(ParseCommands::Install { file: None }),
                    "list" | "freeze" => Some(ParseCommands::Deps { file: None }),
                    _ => None,
                }
            }
            "sync" | "add" | "remove" | "lock" => Some(ParseCommands::Install { file: None }),
            "tree" => Some(ParseCommands::Deps { file: None }),
            // `uv run <tool>` — same dispatch pattern as `npx <tool>`.
            "run" => {
                let inner = args_ref.get(1).map(|s| s.as_str()).unwrap_or("");
                match inner {
                    "pytest" => Some(ParseCommands::Test {
                        runner: Some(TestRunner::Pytest),
                        file: None,
                    }),
                    "ruff" | "mypy" | "pylint" => Some(ParseCommands::Lint { file: None }),
                    _ => None,
                }
            }
            _ => None,
        },
        // poetry — Python dependency manager. `poetry run <tool>` dispatches
        // the same way as `uv run <tool>` and `python3 -m <module>`.
        "poetry" => match subcmd {
            "install" | "add" | "update" | "remove" | "lock" => {
                Some(ParseCommands::Install { file: None })
            }
            "run" => {
                let inner = args_ref.get(1).map(|s| s.as_str()).unwrap_or("");
                match inner {
                    "pytest" => Some(ParseCommands::Test {
                        runner: Some(TestRunner::Pytest),
                        file: None,
                    }),
                    "ruff" | "mypy" | "pylint" | "flake8" | "black" | "isort" => {
                        Some(ParseCommands::Lint { file: None })
                    }
                    _ => None,
                }
            }
            _ => None,
        },

        // Build tools
        "cargo" => match subcmd {
            "clippy" => Some(ParseCommands::Lint { file: None }),
            "build" | "check" => Some(ParseCommands::Build { file: None }),
            "test" => Some(ParseCommands::CargoTest { file: None }),
            "tree" => Some(ParseCommands::Deps { file: None }),
            "install" => Some(ParseCommands::Install { file: None }),
            // Field data: 141 cmds at 89% low compression before this route.
            "fmt" => Some(ParseCommands::Fmt { file: None }),
            _ => None,
        },
        "make" | "cmake" => Some(ParseCommands::Build { file: None }),
        "tsc" => Some(ParseCommands::Lint { file: None }),
        "gcc" | "g++" | "clang" | "javac" => Some(ParseCommands::Build { file: None }),
        "go" => match subcmd {
            "build" => Some(ParseCommands::Build { file: None }),
            "test" => Some(ParseCommands::GoTest { file: None }),
            _ => None,
        },
        "swift" => match subcmd {
            "build" | "test" | "run" => Some(ParseCommands::Build { file: None }),
            _ => None,
        },
        // xcodebuild output is among the chattiest — compile command echoes,
        // Write auxiliary files, dependency checks — but we only need
        // warnings/errors/BUILD-SUCCEEDED|FAILED. handle_build does exactly
        // that via error:/warning: patterns + success sentinel matching.
        "xcodebuild" => Some(ParseCommands::Build { file: None }),

        // Network diagnostics
        "ping" => Some(ParseCommands::Ping { file: None }),

        // Process list — `ps aux` / `ps -ef` are the common forms.
        // Other ps invocations (e.g. `ps -o pid,cmd`) also route here;
        // the parser passes through when the header doesn't match.
        "ps" => Some(ParseCommands::Ps { file: None }),

        // `python3 -m <module>` — route to the module's dedicated
        // parser when we have one. `python3 -m pytest` is a very
        // common way to run pytest without having it on PATH; without
        // this dispatch it would fall through to the generic
        // traceback handler and miss the pytest-specific reduction.
        "python" | "python3" if subcmd == "-m" => {
            let module = args_ref.get(1).map(|s| s.as_str()).unwrap_or("");
            match module {
                "pytest" => Some(ParseCommands::Test {
                    runner: Some(TestRunner::Pytest),
                    file: None,
                }),
                "mypy" | "ruff" | "pylint" | "flake8" => Some(ParseCommands::Lint { file: None }),
                "unittest" => Some(ParseCommands::Test {
                    runner: Some(TestRunner::Pytest),
                    file: None,
                }),
                "build" | "pip" => Some(ParseCommands::Install { file: None }),
                // Unknown module — fall back to traceback handler so
                // Python errors still compress even without a module-
                // specific parser.
                _ => Some(ParseCommands::PythonTraceback { file: None }),
            }
        }
        // Python scripts — the python-traceback handler passes through
        // non-traceback output and compresses stack traces when they
        // appear. Matches the bare interpreter invocations
        // (`python`, `python3`, `python3.12`, …) rather than specific
        // tools like pytest which have their own parser.
        "python" | "python3" => Some(ParseCommands::PythonTraceback { file: None }),
        s if s.starts_with("python3.") => Some(ParseCommands::PythonTraceback { file: None }),

        // Homebrew install/upgrade/reinstall/uninstall
        "brew" => match subcmd {
            "install" | "upgrade" | "reinstall" | "uninstall" | "remove" => {
                Some(ParseCommands::Brew { file: None })
            }
            _ => None,
        },

        // GitHub CLI
        "gh" => match subcmd {
            "pr" => match args.get(1).map(|s| s.as_str()) {
                Some("list") => Some(ParseCommands::GhPr { file: None }),
                Some("view") => Some(ParseCommands::GhPrView { file: None }),
                Some("diff") => Some(ParseCommands::GitDiff { file: None }),
                Some("checks") => Some(ParseCommands::GhPrChecks { file: None }),
                _ => None,
            },
            "issue" if args.get(1).map(|s| s.as_str()) == Some("list") => {
                Some(ParseCommands::GhIssue { file: None })
            }
            "run" => match args.get(1).map(|s| s.as_str()) {
                Some("list") => Some(ParseCommands::GhRun { file: None }),
                Some("view") => Some(ParseCommands::GhRunView { file: None }),
                _ => None,
            },
            // `gh api <path>` returns raw GitHub JSON responses — route
            // to the download handler whose body compressor compacts
            // JSON and decodes base64-encoded contents payloads.
            "api" => Some(ParseCommands::Download { file: None }),
            _ => None,
        },

        // Database clients
        "psql" | "mysql" | "sqlite3" | "mariadb" => Some(ParseCommands::Db { file: None }),

        // Environment
        "env" | "printenv" => Some(ParseCommands::Env { file: None }),

        // Word count
        "wc" => Some(ParseCommands::Wc { file: None }),

        // Download tools + HTTP fetches. All curl invocations route
        // here; the handler distinguishes verbose HTTP protocol
        // output (headers) from plain response bodies and compresses
        // each appropriately (JSON → compact JSON, etc.).
        "wget" => Some(ParseCommands::Download { file: None }),
        "curl" => Some(ParseCommands::Download { file: None }),

        // npx <tool> — route to the underlying tool's parser so the
        // agent gets the same compression as when the tool runs
        // directly. Anything not in this list falls through to the
        // generic whitespace/ANSI fallback.
        // bash -c "<one simple command>" — classify the inner command so
        // its output reaches the right parser (field data: 306 cmds at 90%
        // low compression). Compound scripts fall through to generic.
        "bash" | "sh" | "zsh" | "dash" => {
            let inner = unwrap_shell_c(args_ref)?;
            classify_command(&inner[0], &inner[1..])
        }

        // `timeout [opts] DURATION cmd…` — unwrap and classify the inner
        // command (agents wrap long-running commands in it). Field data:
        // 9.9 KB/cmd uncompressed before this. `gtimeout` = coreutils on mac.
        "timeout" | "gtimeout" => {
            let inner = unwrap_timeout(args_ref)?;
            classify_command(&inner[0], &inner[1..])
        }

        "npx" => match subcmd {
            "jest" => Some(ParseCommands::Test {
                runner: Some(TestRunner::Jest),
                file: None,
            }),
            "vitest" => Some(ParseCommands::Test {
                runner: Some(TestRunner::Vitest),
                file: None,
            }),
            "tsc" | "eslint" | "biome" | "@biomejs/biome" | "prettier" => {
                Some(ParseCommands::Lint { file: None })
            }
            _ => None,
        },

        // Formatters run directly — same "reformatted N files" shape the
        // Fmt parser already handles for `cargo fmt`. Field data: black
        // 23×, isort present, both uncovered before this.
        "black" | "isort" | "autopep8" | "yapf" | "gofmt" => {
            Some(ParseCommands::Fmt { file: None })
        }

        // Linters run directly. flake8/mypy field data: 24× / present,
        // previously only routed via `python -m` / `poetry run`.
        "eslint" | "biome" | "ruff" | "pylint" | "golangci-lint" | "flake8" | "mypy" => {
            Some(ParseCommands::Lint { file: None })
        }

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn ls_files_routes_to_find() {
        assert!(matches!(
            classify_command("git", &argv("ls-files --others --exclude-standard")),
            Some(ParseCommands::Find { .. })
        ));
    }

    #[test]
    fn commit_routes_to_parser() {
        assert!(matches!(
            classify_command("git", &argv("commit -m msg")),
            Some(ParseCommands::GitCommit { .. })
        ));
        // Structured-output flags stay passthrough.
        assert!(classify_command("git", &argv("commit --porcelain")).is_none());
    }

    #[test]
    fn cargo_fmt_routes_to_parser() {
        assert!(matches!(
            classify_command("cargo", &argv("fmt --check")),
            Some(ParseCommands::Fmt { .. })
        ));
        assert!(matches!(
            classify_command("cargo", &argv("fmt")),
            Some(ParseCommands::Fmt { .. })
        ));
    }

    #[test]
    fn bash_c_simple_command_unwraps() {
        assert!(matches!(
            classify_command("bash", &["-c".into(), "git status".into()]),
            Some(ParseCommands::GitStatus { .. })
        ));
        assert!(matches!(
            classify_command("sh", &["-c".into(), "cargo test --lib".into()]),
            Some(ParseCommands::CargoTest { .. })
        ));
    }

    #[test]
    fn absolute_path_routes_by_basename() {
        // Field data: `/opt/homebrew/bin/gh` averaged 40 KB/cmd uncompressed
        // because the classifier matched the full path, not `gh`.
        assert!(matches!(
            classify_command("/opt/homebrew/bin/gh", &argv("pr list")),
            Some(ParseCommands::GhPr { .. })
        ));
        assert!(matches!(
            classify_command("/usr/bin/git", &argv("status")),
            Some(ParseCommands::GitStatus { .. })
        ));
        assert!(matches!(
            classify_command("./node_modules/.bin/eslint", &argv("src")),
            Some(ParseCommands::Lint { .. })
        ));
    }

    #[test]
    fn bare_python_linters_and_formatters_route() {
        for c in ["flake8", "mypy"] {
            assert!(
                matches!(
                    classify_command(c, &argv("src")),
                    Some(ParseCommands::Lint { .. })
                ),
                "{c} should route to Lint"
            );
        }
        for c in ["black", "isort"] {
            assert!(
                matches!(
                    classify_command(c, &argv(".")),
                    Some(ParseCommands::Fmt { .. })
                ),
                "{c} should route to Fmt"
            );
        }
    }

    #[test]
    fn timeout_unwraps_inner_command() {
        assert!(matches!(
            classify_command("timeout", &argv("30 cargo test")),
            Some(ParseCommands::CargoTest { .. })
        ));
        // Option with a separate value, then duration with a unit suffix.
        assert!(matches!(
            classify_command("timeout", &argv("-s KILL 5s git status")),
            Some(ParseCommands::GitStatus { .. })
        ));
        // No duration / no inner command → passthrough.
        assert!(classify_command("timeout", &argv("--help")).is_none());
        assert!(classify_command("timeout", &argv("30")).is_none());
    }

    #[test]
    fn bash_c_compound_or_quoted_stays_generic() {
        for script in [
            "echo a; git status",
            "git status | head",
            "ls && pwd",
            "echo \"hi\"",
            "node -e 'console.log(1)'",
            "VAR=$(date) printenv",
        ] {
            assert!(
                classify_command("bash", &["-c".into(), script.into()]).is_none(),
                "should stay generic: {script}"
            );
        }
    }
}
