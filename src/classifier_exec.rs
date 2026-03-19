//! External command execution and tee output saving.
//!
//! Handles the execute -> parse -> format pipeline for external commands
//! and saves full output on failure for recovery.

use crate::classifier::{classify_command, inject_file_path};
use crate::router::{CommandContext, Router};
use crate::{Commands, OutputFormat};

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
    let in_bytes = stdout.len() + stderr.len();

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
            if let Some(tee_path) = save_tee_output(&full_cmd, &stdout, &stderr) {
                eprintln!("[full output: {}]", tee_path);
            }
        }
        std::process::exit(output.status.code().unwrap_or(1));
    }

    // Print stderr passthrough (warnings, progress, etc.)
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }

    // Try to classify and parse the output (3-tier fallback)
    let mut out_bytes = in_bytes; // default: no reduction (passthrough)
    if let Some(parser) = classify_command(cmd, args) {
        // Estimate output size based on benchmarked reduction ratios per command
        let subcmd = args.first().map(|s| s.as_str()).unwrap_or("");
        let keep_ratio = match (cmd, subcmd) {
            ("git", "status") => 0.20,                 // 80% reduction
            ("git", "diff") => 0.10,                   // 90% reduction
            ("git", "log") => 0.10,                    // 90% reduction
            ("git", "branch") => 0.11,                 // 89% reduction
            ("ls" | "lsd" | "exa" | "eza", _) => 0.18, // 82% reduction
            ("tree", _) => 0.30,
            ("find" | "fd", _) => 0.52, // 48% reduction
            ("grep" | "rg" | "ag", _) => 0.40,
            ("env" | "printenv", _) => 0.32, // 68% reduction
            ("docker", "ps") => 0.30,
            ("docker", "logs") => 0.50,
            ("npm" | "pnpm" | "yarn" | "pip" | "pip3" | "cargo", "install" | "i") => 0.20,
            ("npm" | "pip" | "pip3" | "cargo", "ls" | "list" | "tree" | "freeze") => 0.40,
            ("cargo", "build" | "check" | "clippy") => 0.10,
            ("cargo", "test") => 0.05,
            ("make" | "tsc" | "gcc" | "g++", _) => 0.15,
            ("pytest" | "jest" | "vitest", _) => 0.10,
            ("npm" | "pnpm" | "bun" | "yarn", "test") => 0.10,
            ("wc", _) => 0.50,
            ("wget", _) => 0.15,
            ("curl", _) => 0.15,
            ("gh", "pr" | "issue" | "run") => 0.30,
            _ => 0.50,
        };
        out_bytes = (in_bytes as f64 * keep_ratio).max(1.0) as usize;

        // Tier 1: Try parser
        let router = Router::new();
        let tmpdir = std::env::temp_dir();
        let tmpfile = tmpdir.join(format!("trs_pipe_{}.tmp", std::process::id()));
        let parse_ok = if std::fs::write(&tmpfile, stdout.as_bytes()).is_ok() {
            let parser_with_file = inject_file_path(parser, tmpfile.clone());
            let parse_cmd = Commands::Parse {
                parser: parser_with_file,
            };

            // Capture parser panics/errors — fallback to passthrough
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                router.route(&parse_cmd, ctx)
            }));

            let _ = std::fs::remove_file(&tmpfile);

            match result {
                Ok(Ok(())) => true,  // Tier 1: Full — parser succeeded
                Ok(Err(_)) => false, // Tier 3: Parser returned error
                Err(_) => false,     // Tier 3: Parser panicked
            }
        } else {
            false
        };

        // Tier 3: Passthrough with truncation (parser failed)
        if !parse_ok {
            let passthrough_max = crate::config::config().limits.passthrough_max_chars;
            let truncated = if stdout.len() > passthrough_max {
                let cut = &stdout[..passthrough_max];
                format!(
                    "{}\n[trs:passthrough — truncated at {} chars, full output: {} chars]",
                    cut,
                    passthrough_max,
                    stdout.len()
                )
            } else {
                stdout.to_string()
            };
            print!("{}", truncated);
            out_bytes = truncated.len();
        }
    } else {
        // No parser matched — apply generic compression (collapse whitespace, strip ANSI)
        let compressed = generic_compress(&stdout);
        print!("{}", compressed);
        out_bytes = compressed.len();
    }

    // Track execution (fire-and-forget)
    let duration_ms = start.elapsed().as_millis() as u64;
    let full_cmd = if args.is_empty() {
        cmd.to_string()
    } else {
        format!("{} {}", cmd, args.join(" "))
    };
    crate::tracker::log_execution(&full_cmd, in_bytes, out_bytes, duration_ms);

    // Tee system: on failure, save full raw output for recovery
    if !output.status.success() {
        if let Some(tee_path) = save_tee_output(&full_cmd, &stdout, &stderr) {
            eprintln!("[full output: {}]", tee_path);
        }
        std::process::exit(output.status.code().unwrap_or(1));
    }
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

    std::fs::write(&filepath, &content).ok()?;
    Some(filepath.to_string_lossy().to_string())
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
