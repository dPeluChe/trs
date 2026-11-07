//! `--json` for commands that print text: run them again without `--json`,
//! capture what they print and wrap it as the one document `--json` promises.

use std::ffi::OsString;
use std::process::{Output, Stdio};

use crate::commands::Commands;
use crate::OutputFormat;

/// Commands whose own output is text, so `--json` wraps it. No `_` arm: a
/// new command must say whether it prints its own document.
pub(crate) fn wraps(command: &Commands) -> bool {
    match command {
        Commands::Init { .. }
        | Commands::Uninstall { .. }
        | Commands::Upgrade { .. }
        | Commands::Report { .. }
        | Commands::OutputSaver { .. }
        | Commands::DebugInfo { .. }
        | Commands::History { .. }
        | Commands::Ingest { .. }
        | Commands::Discover { .. }
        | Commands::AuditDocs { .. } => true,
        Commands::Run { .. }
        | Commands::Rewrite { .. }
        | Commands::Doctor { .. }
        | Commands::Benchmark { .. }
        | Commands::Diff { .. }
        | Commands::Raw { .. }
        | Commands::Parse { .. }
        | Commands::Search { .. }
        | Commands::Replace { .. }
        | Commands::Tail { .. }
        | Commands::Clean { .. }
        | Commands::Html2md { .. }
        | Commands::Txt2md { .. }
        | Commands::Trim { .. }
        | Commands::IsClean { .. }
        | Commands::Stats { .. }
        | Commands::Read { .. }
        | Commands::Json { .. }
        | Commands::Err { .. }
        | Commands::External(_) => false,
    }
}

/// Rerun this invocation without `--json`, its output captured. Stdin is
/// closed, so a prompt reads no answer and nothing runs without `--yes`.
pub(crate) fn run_wrapped() -> ! {
    let mut args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let end = args.iter().position(|a| a == "--").unwrap_or(args.len());
    let mut i = 0;
    args.retain(|a| {
        i += 1;
        i > end || a != "--json"
    });
    let out = crate::classifier::trs_command()
        .and_then(|mut trs| trs.args(&args).stdin(Stdio::null()).output());
    match out {
        Ok(out) => finish("trs", &out),
        Err(e) => crate::exec_verdict::exit_on_spawn_failure(OutputFormat::Json, "trs", &e),
    }
}

/// Print `out` as one document with its exit status and stderr, and exit.
/// Output past the passthrough limit is cut, with the rest in `full_output`.
pub(crate) fn finish(cmd: &str, out: &Output) -> ! {
    let code = crate::process::exit_code_of(&out.status);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let max = crate::config::config().limits.passthrough_max_chars;
    let cut = crate::passthrough_cut::cut(&stdout, max);
    let raw = || {
        cut.as_ref()
            .and_then(|_| crate::classifier_exec::save_tee_output(cmd, &stdout, &stderr))
    };
    let text = cut.as_deref().unwrap_or(&stdout);
    let document = crate::exec_verdict::json_document(text, code, raw);
    println!("{}", crate::exec_verdict::with_stderr(document, &stderr));
    std::process::exit(code);
}
