//! Machine formats for commands that print text: run them again without the
//! format flag, capture what they print and wrap it as the one document
//! `--json` promises or the one-column `line` table `--csv` and `--tsv` do.

use std::ffi::OsString;
use std::process::{Output, Stdio};

use crate::commands::Commands;
use crate::OutputFormat;

/// Whether `command` prints its own JSON document and its own table. No `_`
/// arm: a new command must say which formats it handles itself.
fn own_formats(command: &Commands) -> (bool, bool) {
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
        | Commands::AuditDocs { .. } => (false, false),
        Commands::Doctor { .. }
        | Commands::Benchmark { .. }
        | Commands::Diff { .. }
        | Commands::Stats { .. }
        | Commands::Html2md { .. }
        | Commands::Txt2md { .. }
        | Commands::IsClean { .. }
        | Commands::Read { .. }
        | Commands::Json { .. } => (true, false),
        // Raw is the program's own output and rewrite a hook's protocol;
        // parse and programs pick their table where they run.
        Commands::Run { .. }
        | Commands::Rewrite { .. }
        | Commands::Raw { .. }
        | Commands::Parse { .. }
        | Commands::Search { .. }
        | Commands::Replace { .. }
        | Commands::Tail { .. }
        | Commands::Clean { .. }
        | Commands::Trim { .. }
        | Commands::Err { .. }
        | Commands::External(_) => (true, true),
    }
}

/// Whether `format` asks `command` for something it only prints as text.
pub(crate) fn wraps(command: &Commands, format: OutputFormat) -> bool {
    let (json, table) = own_formats(command);
    (format == OutputFormat::Json && !json) || (format.is_table() && !table)
}

/// Rerun this invocation without its format flag, its output captured.
/// Stdin reaches only the commands that read data from it; for the rest it
/// is closed, so a prompt reads no answer and nothing runs without `--yes`.
pub(crate) fn run_wrapped(command: &Commands, format: OutputFormat) -> ! {
    let flag = match format {
        OutputFormat::Csv => "--csv",
        OutputFormat::Tsv => "--tsv",
        _ => "--json",
    };
    let mut args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let end = args.iter().position(|a| a == "--").unwrap_or(args.len());
    let mut i = 0;
    args.retain(|a| {
        i += 1;
        i > end || a != flag
    });
    let reads_stdin = matches!(
        command,
        Commands::Html2md { .. }
            | Commands::Txt2md { .. }
            | Commands::Json { .. }
            | Commands::Read { .. }
    );
    let stdin = if reads_stdin {
        Stdio::inherit()
    } else {
        Stdio::null()
    };
    let out =
        crate::classifier::trs_command().and_then(|mut trs| trs.args(&args).stdin(stdin).output());
    match out {
        Ok(out) if format.is_table() => finish_table(format, &out),
        Ok(out) => finish("trs", &out),
        Err(e) => crate::exec_verdict::exit_on_spawn_failure(format, "trs", &e),
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

/// Print `out` as a `line` table, stderr where it was, and exit with its code.
fn finish_table(format: OutputFormat, out: &Output) -> ! {
    let stdout = String::from_utf8_lossy(&out.stdout);
    print!(
        "{}",
        crate::router::handlers::common::line_table(format, &stdout)
    );
    eprint!("{}", String::from_utf8_lossy(&out.stderr));
    std::process::exit(crate::process::exit_code_of(&out.status));
}
