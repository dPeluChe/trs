//! External command execution and tee output saving.
//!
//! Handles the execute -> parse -> format pipeline for external commands
//! and saves full output on failure for recovery.

use crate::classifier::{build_command, classify_command, full_cmd, keep_ratio};
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
    let in_bytes = stdout.len() + stderr.len();

    // Publish the real exit status so summarizing parsers can state the
    // verdict as fact instead of inferring it from text.
    crate::router::handlers::common::set_child_exit(output.status.code().unwrap_or(1));

    // Git push/pull/fetch: output goes to stderr, compact it inline
    let subcmd = args.first().map(|s| s.as_str()).unwrap_or("");
    if cmd == "git" && matches!(subcmd, "push" | "pull" | "fetch") {
        let combined = format!("{}{}", stdout, stderr);
        let compact = crate::classifier_transfer::compact_git_transfer(&combined, subcmd);
        print!("{}", compact);
        let duration_ms = start.elapsed().as_millis() as u64;
        let full_cmd = format!("{} {}", cmd, args.join(" "));
        crate::tracker::log_execution(&full_cmd, in_bytes, compact.len(), duration_ms);
        if !output.status.success() {
            emit_failure_footer(&output.status, &full_cmd, &stdout, &stderr);
        }
        std::process::exit(output.status.code().unwrap_or(1));
    }

    // Several commands write their primary output (errors, warnings, results)
    // to stderr rather than stdout. Combine both streams so the parser sees
    // everything. Build tools (cargo build, make, gcc) and linters all do this.
    // Notably excluded: cargo test — test results go to stdout and mixing in
    // cargo's stderr progress would confuse the test parser. The per-command
    // stderr policy lives in the unified command registry.
    let combine_stderr = crate::command_registry::combine_stderr(cmd, subcmd);
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

    // Min input guard: skip parsing entirely for tiny outputs
    let min_input = crate::config::config().limits.min_input_chars;
    if stdout_ref.len() < min_input {
        print!("{}", stdout_ref);
        out_bytes = stdout_ref.len();
        let duration_ms = start.elapsed().as_millis() as u64;
        let fcmd = full_cmd(cmd, args);
        crate::tracker::log_execution(&fcmd, in_bytes, out_bytes, duration_ms);
        if !output.status.success() {
            emit_failure_footer(&output.status, &fcmd, &stdout, &stderr);
        }
        return;
    }

    if let Some(parser) = classify_command(cmd, args) {
        // Estimate output size based on benchmarked reduction ratios per command
        let subcmd = args.first().map(|s| s.as_str()).unwrap_or("");
        let ratio = keep_ratio(cmd, subcmd);

        // Ratio gate: if parser is estimated to save < 10%, skip it and use generic
        // compression instead (avoids CPU cost for negligible gain).
        if ratio > 0.90 {
            let compressed = generic_compress(stdout_ref);
            print!("{}", compressed);
            out_bytes = compressed.len();
            let duration_ms = start.elapsed().as_millis() as u64;
            let fcmd = full_cmd(cmd, args);
            crate::tracker::log_execution(&fcmd, in_bytes, out_bytes, duration_ms);
            if !output.status.success() {
                emit_failure_footer(&output.status, &fcmd, &stdout, &stderr);
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
            if parsed.len() < stdout_ref.len() && !summary_hides_failure {
                print!("{}", parsed);
                out_bytes = parsed.len();
            } else {
                print!("{}", stdout_ref);
                out_bytes = stdout_ref.len();
            }
        } else {
            // Tier 3: Passthrough with truncation (parser failed)
            let passthrough_max = crate::config::config().limits.passthrough_max_chars;
            let truncated = if stdout_ref.len() > passthrough_max {
                let cut = &stdout_ref[..passthrough_max];
                format!(
                    "{}\n[trs:passthrough — truncated at {} chars, full output: {} chars]",
                    cut,
                    passthrough_max,
                    stdout_ref.len()
                )
            } else {
                stdout_ref.to_string()
            };
            print!("{}", truncated);
            out_bytes = truncated.len();
        }
    } else {
        // No parser matched — apply generic compression (collapse whitespace, strip ANSI)
        let compressed = generic_compress(stdout_ref);
        print!("{}", compressed);
        out_bytes = compressed.len();
    }

    // Track execution (fire-and-forget)
    let duration_ms = start.elapsed().as_millis() as u64;
    let fcmd = full_cmd(cmd, args);
    crate::tracker::log_execution(&fcmd, in_bytes, out_bytes, duration_ms);

    // Tee system: on failure, save full raw output for recovery
    if !output.status.success() {
        emit_failure_footer(&output.status, &fcmd, &stdout, &stderr);
    }
}

/// Does this compressed output actually show that something went wrong?
/// Backstop for parsers that summarize without consulting the exit status —
/// when the answer is "no" we fall back to raw, so a miss costs tokens, never
/// truth.
///
/// Counts of zero don't count: "0 errors" contains "error" while asserting the
/// opposite, and reading it as failure evidence is what lets a clean summary
/// stand in for a failed command.
fn failure_is_visible(parsed: &str) -> bool {
    if parsed.contains('✗') {
        return true;
    }
    let lower = parsed.to_ascii_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    const MARKERS: &[&str] = &["fail", "error", "panic", "fatal", "abort", "refus"];
    words.iter().enumerate().any(|(i, w)| {
        if !MARKERS.iter().any(|m| w.starts_with(m)) {
            return false;
        }
        !matches!(
            i.checked_sub(1).and_then(|p| words.get(p)),
            Some(&"0") | Some(&"no")
        )
    })
}

/// Terminal failure footer, printed on STDOUT and never returning.
///
/// stdout, not stderr: agents run commands with `2>/dev/null` to keep their
/// context clean, which silently drops a stderr-only notice exactly when it
/// matters most. The exit code makes the status verifiable instead of
/// inferred, and the tee path is the escape hatch to the full raw output.
fn emit_failure_footer(
    status: &std::process::ExitStatus,
    fcmd: &str,
    stdout: &str,
    stderr: &str,
) -> ! {
    let code = status.code().unwrap_or(1);
    match save_tee_output(fcmd, stdout, stderr) {
        Some(tee_path) => println!("[trs] exit {} · full output: {}", code, tee_path),
        None => println!("[trs] exit {}", code),
    }
    std::process::exit(code);
}

/// Save full command output to ~/.trs/tee/ for failure recovery.
/// Returns the path to the saved file, or None if saving failed.
fn save_tee_output(cmd: &str, stdout: &str, stderr: &str) -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let tee_dir = std::path::Path::new(&home).join(".trs").join("tee");

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

    // Collapse consecutive identical lines (e.g., repeated log entries)
    let result = collapse_repeated_lines(&result);

    // Ratio threshold: if compression savings are below the configured minimum,
    // return the original input instead (compression not worth the fidelity loss).
    let min_pct = crate::config::config().limits.min_compression_pct;
    let threshold = input.len() * (100 - min_pct) / 100;
    if result.len() > threshold {
        return input.to_string();
    }

    result
}

/// Collapse consecutive identical lines into `line\n  ...(N more identical lines)`.
/// Minimum 3 consecutive identical lines to trigger collapse.
pub(crate) fn collapse_repeated_lines(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    if lines.len() < 3 {
        return input.to_string();
    }

    let mut result = String::with_capacity(input.len());
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let mut count = 1;

        // Count consecutive identical lines
        while i + count < lines.len() && lines[i + count] == line {
            count += 1;
        }

        if count >= 3 {
            // Show the line once, then a collapsed marker
            result.push_str(line);
            result.push('\n');
            result.push_str(&format!("  ...({} more identical lines)\n", count - 1));
            i += count;
        } else {
            result.push_str(line);
            result.push('\n');
            i += 1;
        }
    }

    // Remove trailing newline if input didn't end with one
    if !input.ends_with('\n') && result.ends_with('\n') {
        result.pop();
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
