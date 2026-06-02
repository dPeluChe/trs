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

/// Build the `Command` used to run an external program through trs.
///
/// On Windows, direct `Command::new(cmd)` only resolves `.exe` — it fails on
/// the `.cmd`/`.bat` PATHEXT shims that front most JS tooling (`npm`, `npx`,
/// `yarn`, `pnpm`, `tsc`, `eslint`, …), on `.ps1` scripts, and on shell
/// builtins. That's the deeper half of issue #53: even after the plugin stops
/// emitting a POSIX `VAR=value` prefix, `trs npm …` / `trs foo.ps1` would die
/// with "command not found". So on Windows we route through the shell the way
/// the user's own shell would: PowerShell for `.ps1`, `cmd /C` otherwise (which
/// honors PATHEXT and builtins). POSIX is unchanged — direct spawn.
pub(crate) fn build_command(cmd: &str, args: &[String]) -> std::process::Command {
    use std::process::Command;
    #[cfg(windows)]
    {
        if cmd.to_ascii_lowercase().ends_with(".ps1") {
            let mut c = Command::new("powershell");
            c.args(["-NoProfile", "-File", cmd]);
            c.args(args);
            return c;
        }
        let mut c = Command::new("cmd");
        c.arg("/C").arg(cmd).args(args);
        c
    }
    #[cfg(not(windows))]
    {
        let mut c = Command::new(cmd);
        c.args(args);
        c
    }
}

/// Preprocess arguments to handle tail -N shorthand (e.g., -5 for last 5 lines).
///
/// This function transforms arguments like:
/// - `trs tail -5 file.log` -> `trs tail -n 5 file.log`
/// - `trs tail -20 file.log` -> `trs tail -n 20 file.log`
pub(crate) fn preprocess_tail_args(args: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        // Check if we're in a tail command context
        if i > 0 && (args[i - 1] == "tail" || is_after_tail_subcommand(args, i)) {
            // Check if this is a -N argument (negative number like -5, -20, etc.)
            if let Some(number) = arg.strip_prefix('-') {
                if let Ok(n) = number.parse::<usize>() {
                    // Transform -N to -n N
                    result.push("-n".to_string());
                    result.push(n.to_string());
                    i += 1;
                    continue;
                }
            }
        }

        result.push(arg.clone());
        i += 1;
    }

    result
}

/// Check if the current position is after a tail subcommand (accounting for global flags).
pub(crate) fn is_after_tail_subcommand(args: &[String], pos: usize) -> bool {
    // Look backwards to find if we have a "tail" command
    for j in (0..pos).rev() {
        if args[j] == "tail" {
            return true;
        }
        // If we hit another subcommand, stop looking
        if j > 0 && !args[j].starts_with('-') && args[j - 1].starts_with('-') {
            break;
        }
    }
    false
}

/// Strip git global options that appear before the subcommand.
/// Returns the args with global options removed so the subcommand can be detected.
/// Global options: -C <path>, -c <key=val>, --git-dir=<path>, --work-tree=<path>,
/// --no-pager, --no-optional-locks, --bare, --literal-pathspecs
fn strip_git_global_opts(args: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            // Options that consume the next argument
            "-C" | "-c" | "--git-dir" | "--work-tree" => {
                i += 2; // skip flag + value
                continue;
            }
            // Options with = syntax
            a if a.starts_with("--git-dir=")
                || a.starts_with("--work-tree=")
                || a.starts_with("-c=") =>
            {
                i += 1;
                continue;
            }
            // Standalone flags
            "--no-pager"
            | "--no-optional-locks"
            | "--bare"
            | "--literal-pathspecs"
            | "--no-replace-objects"
            | "--no-lazy-fetch" => {
                i += 1;
                continue;
            }
            _ => {
                result.push(args[i].clone());
                i += 1;
            }
        }
    }
    result
}

/// Check if the command args contain flags that indicate structured output.
/// When the user explicitly requests JSON/structured output, we should passthrough.
fn has_structured_output_flag(args: &[String]) -> bool {
    args.iter().any(|a| {
        let s = a.as_str();
        s == "--json"
            || s == "--porcelain"
            || s == "--format=json"
            || s == "--output=json"
            || s == "-o=json"
            || s == "--format" && args.iter().any(|b| b == "json")
            || s.starts_with("--format=json")
            || s.starts_with("--output=json")
    })
}

/// Classify an external command and return the appropriate parser to pipe through.
/// Returns (command, args, parser) where parser is the ParseCommands variant to use,
/// or None if no parser matches (passthrough mode).
pub(crate) fn classify_command(cmd: &str, args: &[String]) -> Option<ParseCommands> {
    // If user explicitly requests structured output, don't parse — passthrough
    if has_structured_output_flag(args) {
        return None;
    }

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
            "grep" => Some(ParseCommands::Grep { file: None }),
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

        // Linters
        "eslint" | "biome" | "ruff" | "pylint" | "golangci-lint" => {
            Some(ParseCommands::Lint { file: None })
        }

        _ => None,
    }
}

#[cfg(test)]
mod build_command_tests {
    use super::build_command;

    #[test]
    #[cfg(not(windows))]
    fn posix_spawns_directly() {
        let c = build_command("npm", &["install".into(), "--save".into()]);
        assert_eq!(c.get_program(), "npm");
        let args: Vec<_> = c
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["install", "--save"]);
    }

    #[test]
    #[cfg(windows)]
    fn windows_routes_cmd_and_powershell() {
        // .cmd/.bat shims + builtins go through `cmd /C`.
        let c = build_command("npm", &["install".into()]);
        assert_eq!(c.get_program(), "cmd");
        let args: Vec<_> = c
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["/C", "npm", "install"]);

        // .ps1 scripts go through PowerShell -File.
        let c = build_command(r"C:\srv\start.ps1", &["--host".into(), "127.0.0.1".into()]);
        assert_eq!(c.get_program(), "powershell");
        let args: Vec<_> = c
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            vec![
                "-NoProfile",
                "-File",
                r"C:\srv\start.ps1",
                "--host",
                "127.0.0.1"
            ]
        );
    }
}
