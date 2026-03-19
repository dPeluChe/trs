//! Command classifier and external command execution.
//!
//! Detects what parser to use for external commands and handles
//! the execute → parse → format pipeline.

use crate::{ParseCommands, TestRunner};
use std::path::PathBuf;

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
            "log" => Some(ParseCommands::GitLog { file: None }),
            "branch" => Some(ParseCommands::GitBranch { file: None }),
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
            _ => None,
        },
        "pnpm" => match subcmd {
            "test" => Some(ParseCommands::Test {
                runner: Some(TestRunner::Pnpm),
                file: None,
            }),
            "ls" | "list" => Some(ParseCommands::Deps { file: None }),
            "install" | "i" => Some(ParseCommands::Install { file: None }),
            _ => None,
        },
        "bun" => match subcmd {
            "test" => Some(ParseCommands::Test {
                runner: Some(TestRunner::Bun),
                file: None,
            }),
            "install" | "i" => Some(ParseCommands::Install { file: None }),
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

        // Build tools
        "cargo" => match subcmd {
            "build" | "check" | "clippy" => Some(ParseCommands::Build { file: None }),
            "test" => Some(ParseCommands::CargoTest { file: None }),
            "tree" => Some(ParseCommands::Deps { file: None }),
            "install" => Some(ParseCommands::Install { file: None }),
            _ => None,
        },
        "make" | "cmake" => Some(ParseCommands::Build { file: None }),
        "tsc" | "gcc" | "g++" | "clang" | "javac" | "go" if subcmd == "build" => {
            Some(ParseCommands::Build { file: None })
        }
        "tsc" => Some(ParseCommands::Build { file: None }),

        // GitHub CLI
        "gh" => match subcmd {
            "pr" if args.get(1).map(|s| s.as_str()) == Some("list") => {
                Some(ParseCommands::GhPr { file: None })
            }
            "issue" if args.get(1).map(|s| s.as_str()) == Some("list") => {
                Some(ParseCommands::GhIssue { file: None })
            }
            "run" if args.get(1).map(|s| s.as_str()) == Some("list") => {
                Some(ParseCommands::GhRun { file: None })
            }
            _ => None,
        },

        // Environment
        "env" | "printenv" => Some(ParseCommands::Env { file: None }),

        // Word count
        "wc" => Some(ParseCommands::Wc { file: None }),

        // Download tools
        "wget" => Some(ParseCommands::Download { file: None }),
        "curl"
            if args
                .iter()
                .any(|a| a == "-v" || a == "--verbose" || a == "-I" || a == "--head") =>
        {
            Some(ParseCommands::Download { file: None })
        }

        // npx with subcommands
        "npx" => match subcmd {
            "jest" => Some(ParseCommands::Test {
                runner: Some(TestRunner::Jest),
                file: None,
            }),
            "vitest" => Some(ParseCommands::Test {
                runner: Some(TestRunner::Vitest),
                file: None,
            }),
            "tsc" => Some(ParseCommands::Build { file: None }),
            _ => None,
        },

        _ => None,
    }
}

/// Inject a file path into a ParseCommands variant.
/// This replaces the `file: None` with `file: Some(path)` for all variants.
pub(crate) fn inject_file_path(parser: ParseCommands, path: PathBuf) -> ParseCommands {
    match parser {
        ParseCommands::GitStatus { count, .. } => ParseCommands::GitStatus {
            file: Some(path),
            count,
        },
        ParseCommands::GitDiff { .. } => ParseCommands::GitDiff { file: Some(path) },
        ParseCommands::GitLog { .. } => ParseCommands::GitLog { file: Some(path) },
        ParseCommands::GitBranch { .. } => ParseCommands::GitBranch { file: Some(path) },
        ParseCommands::Ls { .. } => ParseCommands::Ls { file: Some(path) },
        ParseCommands::Grep { .. } => ParseCommands::Grep { file: Some(path) },
        ParseCommands::Find { .. } => ParseCommands::Find { file: Some(path) },
        ParseCommands::Test { runner, .. } => ParseCommands::Test {
            runner,
            file: Some(path),
        },
        ParseCommands::Logs { .. } => ParseCommands::Logs { file: Some(path) },
        ParseCommands::Tree { .. } => ParseCommands::Tree { file: Some(path) },
        ParseCommands::DockerPs { .. } => ParseCommands::DockerPs { file: Some(path) },
        ParseCommands::DockerLogs { .. } => ParseCommands::DockerLogs { file: Some(path) },
        ParseCommands::Deps { .. } => ParseCommands::Deps { file: Some(path) },
        ParseCommands::Install { .. } => ParseCommands::Install { file: Some(path) },
        ParseCommands::Build { .. } => ParseCommands::Build { file: Some(path) },
        ParseCommands::Env { .. } => ParseCommands::Env { file: Some(path) },
        ParseCommands::Wc { .. } => ParseCommands::Wc { file: Some(path) },
        ParseCommands::Download { .. } => ParseCommands::Download { file: Some(path) },
        ParseCommands::GhPr { .. } => ParseCommands::GhPr { file: Some(path) },
        ParseCommands::GhIssue { .. } => ParseCommands::GhIssue { file: Some(path) },
        ParseCommands::GhRun { .. } => ParseCommands::GhRun { file: Some(path) },
        ParseCommands::CargoTest { .. } => ParseCommands::CargoTest { file: Some(path) },
    }
}

