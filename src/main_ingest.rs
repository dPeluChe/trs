//! The `trs ingest` dispatch arm, lifted out of `main.rs`.
//!
//! The variant binds twenty fields, so it takes the `Commands` value whole
//! and re-destructures rather than threading twenty parameters through a
//! signature. The `else` arm returns instead of panicking: `main.rs` only
//! calls this for the matching variant, and an unreachable panic in a CLI
//! entry point is worse than a no-op.

use crate::commands::Commands;
use crate::ingest;
use crate::main_args::parse_token_budget;
use crate::router::handlers::common::CommandContext;
use crate::OutputFormat;

pub(crate) fn run(cmd: &Commands, ctx: &CommandContext) {
    let Commands::Ingest {
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
        html,
        max_loc,
        tmp,
    } = cmd
    else {
        return;
    };

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
        let agent_mode = ctx.format == OutputFormat::Agent;
        // Agent-first ergonomics: `--agent` emits the digest to stdout
        // (implicit --print) instead of just the saved path — an agent
        // asked for the content, not a file to re-read. Skipped when the
        // caller explicitly writes a file with `-o`.
        let print_content = *print || (agent_mode && output.is_none());
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
            print_content,
            warn_at_tokens: {
                let n = parse_token_budget(warn_at);
                if n == 0 {
                    None
                } else {
                    Some(n)
                }
            },
            symbols_index: *symbols,
            html: *html,
            max_loc: max_loc.unwrap_or(500),
            agent_mode,
        };
        ingest::run_ingest(&config);
    }
}
