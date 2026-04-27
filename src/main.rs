#![allow(
    clippy::redundant_closure,
    clippy::manual_pattern_char_comparison,
    clippy::unnecessary_owned_empty_strings,
    clippy::useless_format,
    clippy::collapsible_if,
    clippy::result_large_err,
    clippy::type_complexity,
    clippy::double_ended_iterator_last,
    clippy::len_zero,
    clippy::match_like_matches_macro,
    clippy::manual_strip,
    clippy::manual_is_multiple_of,
    clippy::derivable_impls,
    clippy::if_same_then_else,
    clippy::unnecessary_map_or,
    clippy::needless_range_loop,
    clippy::print_with_newline,
    clippy::ptr_arg,
    clippy::get_first
)]
use clap::Parser;

mod audit_docs;
mod benchmark;
mod classifier;
mod classifier_exec;
mod classifier_transfer;
mod cli;
mod commands;
pub(crate) mod config;
mod debug_info;
mod discover;
mod doctor;
mod formatter;
mod help;
mod ingest;
mod init;
mod init_collision;
mod init_templates;
mod output_saver;
#[allow(dead_code)]
mod process;
#[allow(dead_code)]
mod reducer;
mod rewrite;
mod router;
#[allow(dead_code)]
mod schema;
pub(crate) mod tracker;
mod upgrade;

#[allow(unused_imports)]
pub(crate) use cli::format_precedence;
pub use cli::{Cli, OutputFormat};
pub use commands::{Commands, ParseCommands, TestRunner};

use classifier::preprocess_tail_args;
use classifier_exec::execute_and_parse;
mod fast_find;
mod read_intercept;
use router::{CommandContext, Router};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Fast path: bypass clap for external commands (saves ~2-4ms)
    if args.len() >= 2 && is_external_fast_path(&args) {
        let mut ctx = CommandContext::default_compact();
        let mut cmd_args: Vec<String> = Vec::new();
        for arg in &args[1..] {
            match arg.as_str() {
                "--json" => ctx.format = OutputFormat::Json,
                "--csv" => ctx.format = OutputFormat::Csv,
                "--tsv" => ctx.format = OutputFormat::Tsv,
                "--agent" => ctx.format = OutputFormat::Agent,
                "--compact" => ctx.format = OutputFormat::Compact,
                "--raw" => ctx.format = OutputFormat::Raw,
                "--stats" => ctx.stats = true,
                _ => cmd_args.push(arg.clone()),
            }
        }
        if let Some((cmd, rest)) = cmd_args.split_first() {
            // Fast find: trs find --gitignore . -name "*.rs"
            // Uses internal walker (respects .gitignore) instead of spawning find
            if (cmd == "find" || cmd == "fd") && rest.iter().any(|a| a == "--gitignore") {
                let find_args: Vec<&str> = rest
                    .iter()
                    .filter(|a| a.as_str() != "--gitignore")
                    .map(|a| a.as_str())
                    .collect();
                fast_find::run(&find_args, &ctx);
                return;
            }
            // File-read intercepts: cat/head/sed -n X,Yp apply filter_minimal
            // instead of spawning a subprocess and getting 0% compression.
            if cmd == "cat" && read_intercept::try_cat(rest, &ctx) {
                return;
            }
            if cmd == "head" && read_intercept::try_head(rest, &ctx) {
                return;
            }
            if cmd == "sed" && read_intercept::try_sed(rest, &ctx) {
                return;
            }
            execute_and_parse(cmd, rest, &ctx);
        }
        return;
    }

    // Preprocess arguments to handle tail -N shorthand (e.g., -5 for last 5 lines)
    let processed_args = preprocess_tail_args(&args);

    let cli = Cli::parse_from(&processed_args);

    // Create command context from global CLI options
    let ctx = CommandContext::from_cli(&cli);

    // Create router and execute the command
    let router = Router::new();

    match &cli.command {
        Some(Commands::Rewrite) => {
            rewrite::run_rewrite();
        }
        Some(Commands::Discover { all, since }) => {
            discover::run_discover(*all, *since);
        }
        Some(Commands::Init {
            tool,
            global,
            show,
            all,
            replace,
            force,
        }) => {
            let opts = init::InstallOpts {
                global: *global,
                replace: *replace,
                force: *force,
            };
            if *show {
                init::show_status();
            } else if *all {
                init::install_all(opts);
            } else if let Some(tool_name) = tool {
                match init::AiTool::from_str(tool_name) {
                    Some(t) => init::install_hook(&t, opts),
                    None => eprintln!(
                        "Unknown tool: '{}'. Supported: {}",
                        tool_name,
                        init::AiTool::all_names()
                    ),
                }
            } else {
                // No args: show current status + usage hint combined.
                init::show_status_and_usage();
            }
        }
        Some(Commands::Doctor { json }) => {
            let checks = doctor::run_checks();
            if *json {
                doctor::print_report_json(&checks);
            } else {
                doctor::print_report(&checks);
            }
            let has_fail = checks.iter().any(|c| c.status == doctor::CheckStatus::Fail);
            if has_fail {
                std::process::exit(1);
            }
        }
        Some(Commands::OutputSaver {
            tool,
            install,
            remove,
            print,
            refresh,
        }) => {
            output_saver::run(tool.as_deref(), *install, *remove, *print, *refresh);
        }
        Some(Commands::Upgrade {
            check,
            yes,
            binary_only,
        }) => {
            upgrade::run_upgrade(*check, *yes, *binary_only);
        }
        Some(Commands::DebugInfo { output }) => {
            debug_info::run(output.as_deref());
        }
        Some(Commands::AuditDocs { path }) => {
            audit_docs::run_audit_docs(std::path::Path::new(path));
        }
        Some(Commands::Benchmark {
            command,
            args,
            repeat,
            json,
        }) => {
            benchmark::run_benchmark(command, args, *repeat, *json);
        }
        Some(Commands::Ingest {
            path,
            list,
            read,
            level,
            budget,
            changed,
            since,
            exclude,
            output,
            ollama,
            deps,
            since_last,
            fresh,
            force,
            print,
            warn_at,
            symbols,
            tmp,
        }) => {
            if *list {
                ingest::list_ingests();
            } else if let Some(read_name) = read {
                let project_path = std::path::Path::new(path);
                ingest::read_digest(read_name.as_deref(), project_path);
            } else {
                // Resolve remote inputs (URLs / owner/repo shorthands) to a
                // local path. The guard keeps ephemeral clones alive until
                // run_ingest finishes.
                let remote_source = if ingest::is_remote_ref(path) {
                    let mode = if *tmp {
                        ingest::TmpMode::Force
                    } else {
                        ingest::TmpMode::Auto
                    };
                    match ingest::resolve_remote(path, mode) {
                        Ok(src) => Some(src),
                        Err(e) => {
                            eprintln!("trs ingest: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    None
                };
                let input_path: std::path::PathBuf = match &remote_source {
                    Some(src) => src.path.clone(),
                    None => std::path::PathBuf::from(path),
                };
                let root = match ingest::resolve_project_root(&input_path) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("trs ingest: {}", e);
                        std::process::exit(1);
                    }
                };
                let budget_tokens = budget.as_ref().map(|b| parse_token_budget(b));
                let config = ingest::IngestConfig {
                    root,
                    level: ingest::IngestLevel::from_str(level),
                    budget_tokens,
                    changed_only: *changed,
                    since: since.clone(),
                    exclude: exclude.clone(),
                    output_file: output.as_ref().map(std::path::PathBuf::from),
                    ollama_model: ollama.clone(),
                    deps_only: *deps,
                    since_last: *since_last,
                    fresh_check: *fresh,
                    force: *force,
                    print_content: *print,
                    warn_at_tokens: {
                        let n = parse_token_budget(warn_at);
                        if n == 0 {
                            None
                        } else {
                            Some(n)
                        }
                    },
                    symbols_index: *symbols,
                };
                ingest::run_ingest(&config);
            }
        }
        Some(Commands::Stats {
            history,
            project,
            json,
            by_agent,
            by_command,
            limit,
        }) => {
            use router::handlers::stats::{handle_stats, StatsInput};
            let input = StatsInput {
                history: *history,
                project: *project,
                json: *json,
                by_agent: *by_agent,
                by_command: *by_command,
                limit: *limit,
            };
            handle_stats(&input);
        }
        Some(Commands::Raw { args }) => {
            // Execute command without filtering but track usage
            use std::process::{Command, Stdio};
            let start = std::time::Instant::now();
            let cmd = &args[0];
            let cmd_args = &args[1..];
            let output = Command::new(cmd)
                .args(cmd_args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output();
            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    print!("{}", stdout);
                    if !stderr.is_empty() {
                        eprint!("{}", stderr);
                    }
                    let total = stdout.len() + stderr.len();
                    let full_cmd = args.join(" ");
                    let ms = start.elapsed().as_millis() as u64;
                    crate::tracker::log_execution(&full_cmd, total, total, ms);
                    std::process::exit(out.status.code().unwrap_or(1));
                }
                Err(e) => {
                    eprintln!("Failed to execute '{}': {}", cmd, e);
                    std::process::exit(127);
                }
            }
        }
        Some(
            Commands::Parse { .. }
            | Commands::Search { .. }
            | Commands::Replace { .. }
            | Commands::Run { .. }
            | Commands::Tail { .. }
            | Commands::Clean { .. }
            | Commands::Trim { .. }
            | Commands::Html2md { .. }
            | Commands::Txt2md { .. }
            | Commands::IsClean { .. }
            | Commands::Read { .. }
            | Commands::Json { .. }
            | Commands::Err { .. },
        ) => {
            router.execute_and_print(cli.command.as_ref().unwrap(), &ctx);
        }
        Some(Commands::External(ext_args)) => {
            // External command: classify, execute, and parse
            // Extract trs flags (--json, --csv, etc.) from the external args
            // so users can write: trs git status --json
            let trs_flags = [
                "--json",
                "--csv",
                "--tsv",
                "--agent",
                "--compact",
                "--raw",
                "--stats",
            ];
            let mut cmd_args: Vec<String> = Vec::new();
            let mut ctx = ctx;
            for arg in ext_args {
                if trs_flags.contains(&arg.as_str()) {
                    match arg.as_str() {
                        "--json" => ctx.format = OutputFormat::Json,
                        "--csv" => ctx.format = OutputFormat::Csv,
                        "--tsv" => ctx.format = OutputFormat::Tsv,
                        "--agent" => ctx.format = OutputFormat::Agent,
                        "--compact" => ctx.format = OutputFormat::Compact,
                        "--raw" => ctx.format = OutputFormat::Raw,
                        "--stats" => ctx.stats = true,
                        _ => {}
                    }
                } else {
                    cmd_args.push(arg.clone());
                }
            }
            if let Some((cmd, args)) = cmd_args.split_first() {
                execute_and_parse(cmd, args, &ctx);
            }
        }
        None => {
            use std::io::{self, IsTerminal, Read};

            // If stdin is a terminal (no pipe), show help instead of blocking
            if io::stdin().is_terminal() {
                Cli::parse_from(["trs", "--help"]);
                return;
            }

            // Read from stdin when piped
            let mut buffer = Vec::new();
            if let Err(e) = io::stdin().read_to_end(&mut buffer) {
                eprintln!("Error reading from stdin: {}", e);
                std::process::exit(1);
            }

            let input = String::from_utf8_lossy(&buffer);

            match router.process_stdin(&input, &ctx) {
                Ok(output) => print!("{}", output),
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(e.exit_code().unwrap_or(1));
                }
            }
        }
    }
}

/// Check if args[1] is an external command (not a trs subcommand or flag).
/// This allows bypassing the full clap parser for the hot path.
fn is_external_fast_path(args: &[String]) -> bool {
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
            | "doctor"
            | "benchmark"
            | "ingest"
            | "audit-docs"
            | "output-saver"
            | "upgrade"
            | "debug-info"
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
fn parse_token_budget(s: &str) -> usize {
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
#[path = "main_tests.rs"]
mod tests;
