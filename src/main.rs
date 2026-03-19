use clap::Parser;

mod classifier;
mod classifier_transfer;
mod cli;
mod commands;
pub(crate) mod config;
mod formatter;
mod help;
#[allow(dead_code)]
mod process;
#[allow(dead_code)]
mod reducer;
mod router;
#[allow(dead_code)]
mod schema;
pub(crate) mod tracker;

pub use cli::{Cli, OutputFormat};
#[allow(unused_imports)]
pub(crate) use cli::format_precedence;
pub use commands::{Commands, ParseCommands, TestRunner};

use classifier::{execute_and_parse, preprocess_tail_args};
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
        Some(Commands::Stats { history, project, json }) => {
            use router::handlers::stats::{StatsInput, handle_stats};
            let input = StatsInput {
                history: *history,
                project: *project,
                json: *json,
            };
            handle_stats(&input);
        }
        Some(Commands::Parse { .. }
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
            | Commands::Err { .. }) => {
            router.execute_and_print(cli.command.as_ref().unwrap(), &ctx);
        }
        Some(Commands::External(ext_args)) => {
            // External command: classify, execute, and parse
            // Extract trs flags (--json, --csv, etc.) from the external args
            // so users can write: trs git status --json
            let trs_flags = ["--json", "--csv", "--tsv", "--agent", "--compact", "--raw", "--stats"];
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
