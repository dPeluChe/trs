//! Command classifier and external command execution.
//!
//! Detects what parser to use for external commands and handles
//! the execute → parse → format pipeline.

use std::path::PathBuf;
use crate::{Commands, ParseCommands, TestRunner};
#[allow(unused_imports)]
use crate::OutputFormat;
use crate::router::{CommandContext, Router};

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

/// Classify an external command and return the appropriate parser to pipe through.
/// Returns (command, args, parser) where parser is the ParseCommands variant to use,
/// or None if no parser matches (passthrough mode).
pub(crate) fn classify_command(cmd: &str, args: &[String]) -> Option<ParseCommands> {
    let subcmd = args.first().map(|s| s.as_str()).unwrap_or("");

    match cmd {
        // Git commands
        "git" => match subcmd {
            "status" => Some(ParseCommands::GitStatus { file: None, count: None }),
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
        "pytest" => Some(ParseCommands::Test { runner: Some(TestRunner::Pytest), file: None }),
        "jest" => Some(ParseCommands::Test { runner: Some(TestRunner::Jest), file: None }),
        "vitest" => Some(ParseCommands::Test { runner: Some(TestRunner::Vitest), file: None }),

        // Package managers — subcommand-aware
        "npm" => match subcmd {
            "test" => Some(ParseCommands::Test { runner: Some(TestRunner::Npm), file: None }),
            "ls" | "list" => Some(ParseCommands::Deps { file: None }),
            "install" | "i" | "ci" => Some(ParseCommands::Install { file: None }),
            "audit" | "outdated" => Some(ParseCommands::Deps { file: None }),
            _ => None,
        },
        "pnpm" => match subcmd {
            "test" => Some(ParseCommands::Test { runner: Some(TestRunner::Pnpm), file: None }),
            "ls" | "list" => Some(ParseCommands::Deps { file: None }),
            "install" | "i" => Some(ParseCommands::Install { file: None }),
            _ => None,
        },
        "bun" => match subcmd {
            "test" => Some(ParseCommands::Test { runner: Some(TestRunner::Bun), file: None }),
            "install" | "i" => Some(ParseCommands::Install { file: None }),
            _ => None,
        },
        "yarn" => match subcmd {
            "test" => Some(ParseCommands::Test { runner: Some(TestRunner::Jest), file: None }),
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
            "test" => Some(ParseCommands::Test { runner: Some(TestRunner::Pytest), file: None }),
            "tree" => Some(ParseCommands::Deps { file: None }),
            "install" => Some(ParseCommands::Install { file: None }),
            _ => None,
        },
        "make" | "cmake" => Some(ParseCommands::Build { file: None }),
        "tsc" | "gcc" | "g++" | "clang" | "javac" | "go" if subcmd == "build" => {
            Some(ParseCommands::Build { file: None })
        }
        "tsc" => Some(ParseCommands::Build { file: None }),

        // Environment
        "env" | "printenv" => Some(ParseCommands::Env { file: None }),

        // npx with subcommands
        "npx" => match subcmd {
            "jest" => Some(ParseCommands::Test { runner: Some(TestRunner::Jest), file: None }),
            "vitest" => Some(ParseCommands::Test { runner: Some(TestRunner::Vitest), file: None }),
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
        ParseCommands::GitStatus { count, .. } => ParseCommands::GitStatus { file: Some(path), count },
        ParseCommands::GitDiff { .. } => ParseCommands::GitDiff { file: Some(path) },
        ParseCommands::GitLog { .. } => ParseCommands::GitLog { file: Some(path) },
        ParseCommands::GitBranch { .. } => ParseCommands::GitBranch { file: Some(path) },
        ParseCommands::Ls { .. } => ParseCommands::Ls { file: Some(path) },
        ParseCommands::Grep { .. } => ParseCommands::Grep { file: Some(path) },
        ParseCommands::Find { .. } => ParseCommands::Find { file: Some(path) },
        ParseCommands::Test { runner, .. } => ParseCommands::Test { runner, file: Some(path) },
        ParseCommands::Logs { .. } => ParseCommands::Logs { file: Some(path) },
        ParseCommands::Tree { .. } => ParseCommands::Tree { file: Some(path) },
        ParseCommands::DockerPs { .. } => ParseCommands::DockerPs { file: Some(path) },
        ParseCommands::DockerLogs { .. } => ParseCommands::DockerLogs { file: Some(path) },
        ParseCommands::Deps { .. } => ParseCommands::Deps { file: Some(path) },
        ParseCommands::Install { .. } => ParseCommands::Install { file: Some(path) },
        ParseCommands::Build { .. } => ParseCommands::Build { file: Some(path) },
        ParseCommands::Env { .. } => ParseCommands::Env { file: Some(path) },
    }
}

/// Execute an external command, optionally pipe through a parser, and print output.
pub(crate) fn execute_and_parse(cmd: &str, args: &[String], ctx: &CommandContext) {
    use std::process::{Command, Stdio};

    let start = std::time::Instant::now();

    // Execute the command
    let output = match Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                eprintln!("Command not found: {}", cmd);
            } else {
                eprintln!("Failed to execute '{}': {}", cmd, e);
            }
            std::process::exit(127);
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let in_bytes = stdout.len();

    // Print stderr passthrough (warnings, progress, etc.)
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }

    // Try to classify and parse the output
    let mut out_bytes = in_bytes; // default: no reduction (passthrough)
    if let Some(parser) = classify_command(cmd, args) {
        // Estimate output size based on benchmarked reduction ratios per command
        let subcmd = args.first().map(|s| s.as_str()).unwrap_or("");
        let keep_ratio = match (cmd, subcmd) {
            ("git", "status") => 0.20,   // 80% reduction
            ("git", "diff") => 0.10,     // 90% reduction
            ("git", "log") => 0.10,      // 90% reduction
            ("git", "branch") => 0.11,   // 89% reduction
            ("ls" | "lsd" | "exa" | "eza", _) => 0.18, // 82% reduction
            ("tree", _) => 0.30,
            ("find" | "fd", _) => 0.52,  // 48% reduction
            ("grep" | "rg" | "ag", _) => 0.40,
            ("env" | "printenv", _) => 0.32, // 68% reduction
            ("docker", "ps") => 0.30,
            ("docker", "logs") => 0.50,
            ("npm" | "pnpm" | "yarn" | "pip" | "pip3" | "cargo", "install" | "i") => 0.20,
            ("npm" | "pip" | "pip3" | "cargo", "ls" | "list" | "tree" | "freeze") => 0.40,
            ("cargo", "build" | "check" | "clippy") => 0.10,
            ("make" | "tsc" | "gcc" | "g++", _) => 0.15,
            ("pytest" | "jest" | "vitest", _) => 0.10,
            ("npm" | "pnpm" | "bun" | "yarn", "test") => 0.10,
            _ => 0.50,
        };
        out_bytes = (in_bytes as f64 * keep_ratio).max(1.0) as usize;

        let router = Router::new();
        let tmpdir = std::env::temp_dir();
        let tmpfile = tmpdir.join(format!("trs_pipe_{}.tmp", std::process::id()));
        if std::fs::write(&tmpfile, stdout.as_bytes()).is_ok() {
            let parser_with_file = inject_file_path(parser, tmpfile.clone());
            let parse_cmd = Commands::Parse { parser: parser_with_file };
            router.execute_and_print(&parse_cmd, ctx);
            let _ = std::fs::remove_file(&tmpfile);
        } else {
            print!("{}", stdout);
            out_bytes = in_bytes; // passthrough, no reduction
        }
    } else {
        // No parser matched — passthrough the output as-is
        print!("{}", stdout);
    }

    // Track execution (fire-and-forget)
    let duration_ms = start.elapsed().as_millis() as u64;
    let full_cmd = if args.is_empty() {
        cmd.to_string()
    } else {
        format!("{} {}", cmd, args.join(" "))
    };
    crate::tracker::log_execution(&full_cmd, in_bytes, out_bytes, duration_ms);

    // Propagate exit code
    if !output.status.success() {
        std::process::exit(output.status.code().unwrap_or(1));
    }
}
