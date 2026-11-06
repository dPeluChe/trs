//! External command execution and tee output saving.
//!
//! Handles the execute -> parse -> format pipeline for external commands
//! and saves full output on failure for recovery.

use crate::classifier::{build_command, classify_command, full_cmd, keep_ratio};
use crate::exec_verdict::{failure_is_visible, print_spawn_failure, JsonRun};
use crate::process::exit_code_of;
use crate::router::{CommandContext, Router};
use crate::Commands;

/// Execute an external command, optionally pipe through a parser, and print output.
pub(crate) fn execute_and_parse(cmd: &str, args: &[String], ctx: &CommandContext) {
    use std::process::Stdio;

    let start = std::time::Instant::now();

    // Execute the command. build_command routes through the platform shell on
    // Windows so .cmd/.bat shims, .ps1 scripts, and builtins resolve (issue #53).
    // stdin must be inherited: `Command::output()` defaults it to null, which
    // silently starves anything reading stdin — a heredoc (`python3 - <<EOF`)
    // would see EOF, run nothing, and still exit 0.
    let output = match build_command(cmd, args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            if matches!(ctx.format, crate::OutputFormat::Json) {
                print_spawn_failure(cmd, &e);
            }
            crate::process::exit_on_spawn_failure(cmd, &e)
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let in_bytes = stdout.len() + stderr.len();

    // Publish the real exit status so summarizing parsers can state the
    // verdict as fact instead of inferring it from text.
    let code = exit_code_of(&output.status);
    crate::router::handlers::common::set_child_exit(code);

    // Everything below routes by the program's name (`/usr/bin/git`, `./mvnw`)
    // and its real subcommand (`git -C dir push`); only the spawn needs `cmd`.
    let json_run = matches!(ctx.format, crate::OutputFormat::Json).then(|| JsonRun {
        fcmd: full_cmd(cmd, args),
        stdout: &stdout,
        stderr: &stderr,
        code,
        in_bytes,
        start,
        status: &output.status,
    });

    let name = crate::text_util::cmd_basename(cmd);
    let sub = crate::classifier_args::subcommand(name, args);
    let subcmd = sub.as_str();

    // An explicit machine format (`--json`, `--csv`, ...) is an answer the
    // caller asked for, however short the output or the parser's result is.
    let wants_format = matches!(
        ctx.format,
        crate::OutputFormat::Json
            | crate::OutputFormat::Csv
            | crate::OutputFormat::Tsv
            | crate::OutputFormat::Agent
    );

    // Git push/pull/fetch: output goes to stderr, compact it inline
    if name == "git" && matches!(subcmd, "push" | "pull" | "fetch") {
        let combined = format!("{}{}", stdout, stderr);
        let compact = crate::classifier_transfer::compact_git_transfer(&combined, subcmd);
        if let Some(run) = &json_run {
            run.finish(&compact, false);
        }
        print!("{}", compact);
        let full_cmd = full_cmd(cmd, args);
        track(&full_cmd, in_bytes, compact.len(), start, &output.status);
        if !output.status.success() {
            emit_failure_footer(code, &full_cmd, &stdout, &stderr);
        }
        std::process::exit(code);
    }

    // Several commands write their primary output (errors, warnings, results)
    // to stderr rather than stdout. Combine both streams so the parser sees
    // everything. Build tools (cargo build, make, gcc) and linters all do this.
    // Notably excluded: cargo test — test results go to stdout and mixing in
    // cargo's stderr progress would confuse the test parser. The per-command
    // stderr policy lives in the unified command registry.
    let (owner, owner_sub) = crate::classifier_args::stderr_owner(name, args);
    let combine_stderr = crate::command_registry::combine_stderr(owner, &owner_sub);
    let effective_stdout;
    let stdout_ref = if combine_stderr && !stderr.is_empty() {
        effective_stdout = format!("{}{}", stdout, stderr);
        &effective_stdout
    } else {
        // Print stderr passthrough (warnings, progress, etc.)
        if !stderr.is_empty() {
            eprint!("{}", stderr);
        }
        &*stdout
    };

    // Try to classify and parse the output (3-tier fallback)
    #[allow(unused_assignments)]
    let mut out_bytes = in_bytes; // default: no reduction (passthrough)

    // Min input guard: skip parsing entirely for tiny outputs. Verbatim
    // commands take the same exit at every size: their spacing is the data.
    // So does a requested machine format (`--json`, `--porcelain`).
    let min_input = crate::config::config().limits.min_input_chars;
    if (stdout_ref.len() < min_input && !wants_format)
        || crate::command_registry::is_verbatim_invocation(name, &args.join(" "))
        || crate::classifier_args::has_structured_output_flag(args)
    {
        if let Some(run) = &json_run {
            run.finish(stdout_ref, false);
        }
        print!("{}", stdout_ref);
        out_bytes = stdout_ref.len();
        let fcmd = full_cmd(cmd, args);
        track(&fcmd, in_bytes, out_bytes, start, &output.status);
        if !output.status.success() {
            emit_failure_footer(code, &fcmd, &stdout, &stderr);
        }
        return;
    }

    if let Some(parser) = classify_command(cmd, args) {
        // Estimate output size based on benchmarked reduction ratios per command
        let ratio = keep_ratio(name, subcmd);

        // Ratio gate: if parser is estimated to save < 10%, skip it and use generic
        // compression instead (avoids CPU cost for negligible gain).
        if ratio > 0.90 && !wants_format {
            let compressed = generic_compress(stdout_ref);
            print!("{}", compressed);
            out_bytes =
                compressed.len() + point_to_raw(&output.status, cmd, args, &stdout, &stderr);
            let fcmd = full_cmd(cmd, args);
            track(&fcmd, in_bytes, out_bytes, start, &output.status);
            if !output.status.success() {
                emit_failure_footer(code, &fcmd, &stdout, &stderr);
            }
            return;
        }

        // Tier 1: Try parser. Its output is captured (not streamed) so we can
        // measure it and enforce the never-worse guard below.
        let router = Router::new();
        let tmpdir = std::env::temp_dir();
        let tmpfile = tmpdir.join(format!("trs_pipe_{}.tmp", std::process::id()));
        let mut parsed = String::new();
        let parse_ok = if std::fs::write(&tmpfile, stdout_ref.as_bytes()).is_ok() {
            let parser_with_file = parser.with_file(tmpfile.clone());
            let parse_cmd = Commands::Parse {
                parser: parser_with_file,
            };

            // Capture parser panics/errors — fallback to passthrough
            let mut ok = false;
            parsed = crate::parse_out::capture(|| {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    router.route(&parse_cmd, ctx)
                }));
                ok = matches!(result, Ok(Ok(())));
            });

            let _ = std::fs::remove_file(&tmpfile);
            ok
        } else {
            false
        };

        if parse_ok {
            // A summary that dropped most of a large output must say where the
            // rest is, or the only way back is re-running the command.
            let dropped = crate::parse_out::take_dropped();
            let mostly_cut =
                stdout_ref.len() >= RECOVERABLE_MIN_BYTES && parsed.len() * 10 <= stdout_ref.len();
            let cut =
                dropped || mostly_cut || crate::anchor_guard::anchors_lost(stdout_ref, &parsed);
            // Nothing is never a summary of something: a parser that misread
            // its input and printed nothing would otherwise "win" on size.
            let parsed_nothing = parsed.trim().is_empty() && !stdout_ref.trim().is_empty();
            if let Some(run) = &json_run {
                run.finish(if parsed_nothing { stdout_ref } else { &parsed }, cut);
            }
            // Truth guard: a summary must never stand in for a failed command
            // unless it actually shows the failure. Parsers infer success from
            // text patterns, so an unrecognized error format (e.g. tsc's
            // `error TS2322:`) yields a clean-looking summary for a non-zero
            // exit — a false claim, not lossy compression. Emit the raw output
            // instead so the real error survives.
            let summary_hides_failure = !output.status.success() && !failure_is_visible(&parsed);
            // Never-worse guard: a parser must never make output larger than
            // the raw command output. If it somehow did (degenerate/tiny
            // input, header overhead), emit the raw instead. Ties go to raw —
            // no point spending a parse when it didn't save anything.
            let smaller = wants_format || parsed.len() < stdout_ref.len();
            if smaller && !summary_hides_failure && !parsed_nothing {
                print!("{}", parsed);
                out_bytes = parsed.len();
                // Failures already get this from the footer below.
                if output.status.success() && cut {
                    if let Some(path) = save_tee_output(&full_cmd(cmd, args), &stdout, &stderr) {
                        let line = format!("[trs] full output: {}\n", path);
                        print!("{}", line);
                        out_bytes += line.len();
                    }
                }
            } else {
                print!("{}", stdout_ref);
                out_bytes = stdout_ref.len();
            }
        } else {
            // Tier 3: Passthrough with truncation (parser failed)
            let passthrough_max = crate::config::config().limits.passthrough_max_chars;
            if let Some(run) = &json_run {
                match crate::passthrough_cut::cut(stdout_ref, passthrough_max) {
                    Some(cut) => run.finish(&cut, true),
                    None => run.finish(stdout_ref, false),
                }
            }
            let truncated = match crate::passthrough_cut::cut(stdout_ref, passthrough_max) {
                Some(mut cut) => {
                    let saved = save_tee_output(&full_cmd(cmd, args), &stdout, &stderr)
                        .map(|path| format!(", full output: {path}"))
                        .unwrap_or_default();
                    cut.push_str(&format!(
                        "\n[trs:passthrough cut to {} of {} chars{}]",
                        passthrough_max,
                        stdout_ref.len(),
                        saved
                    ));
                    cut
                }
                None => stdout_ref.to_string(),
            };
            print!("{}", truncated);
            out_bytes = truncated.len();
        }
    } else {
        // No parser matched — apply generic compression (collapse whitespace, strip ANSI)
        let compressed = generic_compress(stdout_ref);
        if let Some(run) = &json_run {
            run.finish(&compressed, crate::parse_out::take_dropped());
        }
        print!("{}", compressed);
        out_bytes = compressed.len() + point_to_raw(&output.status, cmd, args, &stdout, &stderr);
    }

    // Track execution (fire-and-forget)
    let fcmd = full_cmd(cmd, args);
    track(&fcmd, in_bytes, out_bytes, start, &output.status);

    // Tee system: on failure, save full raw output for recovery
    if !output.status.success() {
        emit_failure_footer(code, &fcmd, &stdout, &stderr);
    }
}

pub(crate) fn track(
    cmd: &str,
    in_bytes: usize,
    out_bytes: usize,
    start: std::time::Instant,
    status: &std::process::ExitStatus,
) {
    let ms = start.elapsed().as_millis() as u64;
    crate::tracker::log_run(cmd, in_bytes, out_bytes, ms, status.success());
}

/// When the generic pass cut a dense line from a successful command, save the
/// raw and print where; failures get this from the footer. Returns bytes added.
fn point_to_raw(
    status: &std::process::ExitStatus,
    cmd: &str,
    args: &[String],
    stdout: &str,
    stderr: &str,
) -> usize {
    if !crate::parse_out::take_dropped() || !status.success() {
        return 0;
    }
    match save_tee_output(&full_cmd(cmd, args), stdout, stderr) {
        Some(path) => {
            let line = format!("[trs] full output: {}\n", path);
            print!("{}", line);
            line.len()
        }
        None => 0,
    }
}

/// Raw size from which a >=90% cut also saves the raw output and prints its
/// path, for parsers that summarize without saying so. Ones that drop content
/// on purpose call `parse_out::mark_dropped` and get the pointer at any size.
const RECOVERABLE_MIN_BYTES: usize = 16 * 1024;

/// Terminal failure footer, printed on STDOUT and never returning.
///
/// stdout, not stderr: agents run commands with `2>/dev/null` to keep their
/// context clean, which silently drops a stderr-only notice exactly when it
/// matters most. The exit code makes the status verifiable instead of
/// inferred, and the tee path is the escape hatch to the full raw output.
fn emit_failure_footer(code: i32, fcmd: &str, stdout: &str, stderr: &str) -> ! {
    match save_tee_output(fcmd, stdout, stderr) {
        Some(tee_path) => println!("[trs] exit {} · full output: {}", code, tee_path),
        None => println!("[trs] exit {}", code),
    }
    std::process::exit(code);
}

/// Save full command output to ~/.trs/tee/ for failure recovery.
/// Returns the path to the saved file, or None if saving failed.
pub(crate) fn save_tee_output(cmd: &str, stdout: &str, stderr: &str) -> Option<String> {
    let tee_dir = crate::tracker::trs_dir()?.join("tee");

    // Create tee directory if needed
    std::fs::create_dir_all(&tee_dir).ok()?;

    // Clean old tee files (keep last N from config)
    let max_files = crate::config::config().limits.tee_max_files;
    if let Ok(entries) = std::fs::read_dir(&tee_dir) {
        let mut files: Vec<std::path::PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().map_or(false, |e| e == "log"))
            .collect();
        files.sort();
        if files.len() > max_files {
            for old in &files[..files.len() - max_files] {
                let _ = std::fs::remove_file(old);
            }
        }
    }

    // Generate filename: timestamp_command.log
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let safe_cmd = cmd
        .chars()
        .take(40)
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let filename = format!("{}_{}.log", timestamp, safe_cmd);
    let filepath = tee_dir.join(&filename);

    // Write stdout + stderr
    let mut content = String::new();
    if !stdout.is_empty() {
        content.push_str(stdout);
    }
    if !stderr.is_empty() {
        if !content.is_empty() {
            content.push_str("\n--- stderr ---\n");
        }
        content.push_str(stderr);
    }

    // Truncate if exceeds max size
    let max_bytes = crate::config::config().limits.tee_max_bytes;
    if max_bytes > 0 && content.len() > max_bytes {
        content.truncate(max_bytes);
        content.push_str(&format!("\n--- truncated at {} bytes ---", max_bytes));
    }

    // mode 0600: tee/*.log can carry response bodies that include
    // credentials or session tokens — user-only readable on Unix.
    write_user_only(&filepath, content.as_bytes()).ok()?;
    crate::pointer_recall::note_pointer();
    Some(filepath.to_string_lossy().to_string())
}

fn write_user_only(path: &std::path::Path, content: &[u8]) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    let mut opts = OpenOptions::new();
    opts.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts.open(path)?;
    f.write_all(content)
}

/// Generic compression for commands without a dedicated parser.
/// Collapses consecutive whitespace in tabular output, strips ANSI codes,
/// removes carriage returns (progress bars), and collapses blank lines.
fn generic_compress(input: &str) -> String {
    use crate::router::handlers::common::strip_ansi_codes;

    // Whitespace collapsing and folds would change the text inside the values.
    if crate::safe_folds::is_json(input) {
        return input.to_string();
    }

    let cleaned = strip_ansi_codes(input);
    let mut result = String::with_capacity(cleaned.len());
    let mut prev_blank = false;

    for line in cleaned.lines() {
        // Skip carriage-return progress lines
        if line.contains('\r') {
            if let Some(last) = line.rsplit('\r').next() {
                let trimmed = last.trim();
                if !trimmed.is_empty() {
                    result.push_str(trimmed);
                    result.push('\n');
                }
            }
            prev_blank = false;
            continue;
        }

        let trimmed = line.trim_end();

        // Collapse consecutive blank lines
        if trimmed.is_empty() {
            if !prev_blank {
                result.push('\n');
            }
            prev_blank = true;
            continue;
        }
        prev_blank = false;

        // Collapse runs of 2+ spaces to single space (tabular padding)
        let compressed = collapse_whitespace(trimmed);
        result.push_str(&compressed);
        result.push('\n');
    }

    // Trim trailing whitespace
    while result.ends_with('\n') && result.len() > 1 && result[..result.len() - 1].ends_with('\n') {
        result.pop();
    }

    // Repeats fold to `line (xN)`, checked by undoing it; dense lines keep a
    // head and tail and flag the output so the raw is saved.
    let (result, dense_cut) = crate::safe_folds::elide_dense(&result);
    let (result, times_cut) = crate::safe_folds::fold_timestamped(&result);
    let cut = dense_cut || times_cut;
    let result = crate::safe_folds::fold_repeats(&result);

    // Ratio threshold: if compression savings are below the configured minimum,
    // return the original input instead (compression not worth the fidelity loss).
    let min_pct = crate::config::config().limits.min_compression_pct;
    let threshold = input.len() * (100 - min_pct) / 100;
    if result.len() > threshold {
        return input.to_string();
    }
    if cut {
        crate::parse_out::mark_dropped();
    }
    result
}

/// Collapse runs of 2+ whitespace chars to a single space.
fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_spaces = false;
    for ch in s.chars() {
        if ch == ' ' || ch == '\t' {
            if !in_spaces {
                result.push(' ');
                in_spaces = true;
            }
        } else {
            result.push(ch);
            in_spaces = false;
        }
    }
    result
}
