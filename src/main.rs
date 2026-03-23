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

mod classifier;
mod classifier_exec;
mod classifier_transfer;
mod cli;
mod commands;
pub(crate) mod config;
mod discover;
mod formatter;
mod help;
mod init;
#[allow(dead_code)]
mod process;
#[allow(dead_code)]
mod reducer;
mod rewrite;
mod router;
#[allow(dead_code)]
mod schema;
pub(crate) mod tracker;

#[allow(unused_imports)]
pub(crate) use cli::format_precedence;
pub use cli::{Cli, OutputFormat};
pub use commands::{Commands, ParseCommands, TestRunner};

use classifier::preprocess_tail_args;
use classifier_exec::execute_and_parse;
use router::{CommandContext, Router};

fn main() {
    // Preprocess arguments to handle tail -N shorthand (e.g., -5 for last 5 lines)
    let args: Vec<String> = std::env::args().collect();
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
        Some(Commands::Init { tool, global, show }) => {
            if *show {
                init::show_status();
            } else if let Some(tool_name) = tool {
                match init::AiTool::from_str(tool_name) {
                    Some(t) => init::install_hook(&t, *global),
                    None => eprintln!(
                        "Unknown tool: '{}'. Supported: {}",
                        tool_name,
                        init::AiTool::all_names()
                    ),
                }
            } else {
                println!("Usage: trs init <tool> [--global]");
                println!("       trs init --show");
                println!("\nSupported tools: {}", init::AiTool::all_names());
            }
        }
        Some(Commands::Stats {
            history,
            project,
            json,
        }) => {
            use router::handlers::stats::{handle_stats, StatsInput};
            let input = StatsInput {
                history: *history,
                project: *project,
                json: *json,
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
            // Read from stdin when no command is provided
            use std::io::{self, Read};
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

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;
