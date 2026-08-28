//! Build-output compression (make, cmake, gcc/clang, swift, xcodebuild): keep error/warning lines and the success sentinel, drop the compile-command echoes. Split out of extra_system.rs, where it was a third of the file.

use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
    pub(crate) fn handle_build(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        // Fail-open: if the build subprocess clearly crashed (panic, traceback,
        // fatal), don't let our signature/error extraction drop context the
        // user needs to debug. Pass through verbatim.
        if super::super::common::output_has_failure_signal(&input) {
            return Self::emit_compressed(&input, None, "build", ctx);
        }

        let mut errors: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();
        let mut creds: Vec<String> = Vec::new();
        let mut info_last = String::new();
        let mut success = true;
        // Which list got the previous line — rustc prints the location on a
        // separate ` --> src/file.rs:7:18` line right after the diagnostic;
        // dropping it loses the file:line an agent needs to act.
        enum Prev {
            Err,
            Warn,
            Other,
        }
        let mut prev = Prev::Other;

        for line in input.lines() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            // Preserve any line that looks credential-bearing. It stays in
            // the output even when nothing else about the line would keep it.
            if super::super::common::contains_credential(t) {
                creds.push(t.to_string());
            }
            if let Some(loc) = t.strip_prefix("--> ") {
                match prev {
                    Prev::Err => {
                        if let Some(e) = errors.last_mut() {
                            e.push_str(&format!(" ({})", loc.trim()));
                        }
                    }
                    Prev::Warn => {
                        if let Some(w) = warnings.last_mut() {
                            w.push_str(&format!(" ({})", loc.trim()));
                        }
                    }
                    Prev::Other => {}
                }
                continue;
            }
            if super::super::common::is_error_line(t) {
                errors.push(t.to_string());
                success = false;
                prev = Prev::Err;
            } else if super::super::common::is_warning_line(t) {
                warnings.push(t.to_string());
                prev = Prev::Warn;
            } else {
                let lower = t.to_ascii_lowercase();
                if lower.starts_with("compiling ") || lower.starts_with("finished ") {
                    info_last = t.to_string();
                    prev = Prev::Other;
                } else if lower.starts_with("build complete!")
                    || lower.starts_with("** build succeeded **")
                    || lower.starts_with("** build failed **")
                    || lower.starts_with("build succeeded")
                {
                    // Swift (swift build) and xcodebuild success/failure sentinels.
                    info_last = t.to_string();
                    if lower.contains("failed") {
                        success = false;
                    }
                }
            }
        }
        warnings.dedup();
        creds.dedup();

        // Hard rule: a non-zero exit means the build failed, whatever the text
        // looked like. Heuristics can't know every tool's error dialect (tsc
        // writes `error TS2322:`, not `error:`), and a summary that says "ok"
        // for a failed build is a false claim, not compression.
        let exit_code = super::super::common::child_exit_code();
        if super::super::common::child_failed() {
            success = false;
        }

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({
                "success": success,
                "exit_code": exit_code,
                "errors": errors,
                "error_count": errors.len(),
                "warnings": warnings,
                "warning_count": warnings.len(),
                "credentials_preserved": creds,
            })
            .to_string(),
            _ => {
                // The exit code rides along so the verdict is verifiable
                // rather than inferred.
                let exit_note = match exit_code {
                    Some(c) => format!(", exit {}", c),
                    None => String::new(),
                };
                let mut out = format!(
                    "build: {} ({} errors, {} warnings{})\n",
                    if success { "ok" } else { "FAILED" },
                    errors.len(),
                    warnings.len(),
                    exit_note
                );
                if !errors.is_empty() {
                    out.push_str(&format!("errors ({}):\n", errors.len()));
                    for e in errors.iter().take(20) {
                        out.push_str(&format!("  {}\n", e));
                    }
                    if errors.len() > 20 {
                        out.push_str(&format!("  ...+{} more\n", errors.len() - 20));
                    }
                }
                if !warnings.is_empty() {
                    out.push_str(&format!("warnings ({}):\n", warnings.len()));
                    for w in warnings.iter().take(10) {
                        out.push_str(&format!("  {}\n", w));
                    }
                    if warnings.len() > 10 {
                        out.push_str(&format!("  ...+{} more\n", warnings.len() - 10));
                    }
                }
                if !info_last.is_empty() {
                    out.push_str(&format!("{}\n", info_last));
                }
                if !creds.is_empty() {
                    out.push_str(&format!(
                        "preserved ({} credential-bearing):\n",
                        creds.len()
                    ));
                    for c in &creds {
                        out.push_str(&format!("  {}\n", c));
                    }
                }
                out
            }
        };
        crate::parse_out::emit(&output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("build")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }

    pub(crate) fn handle_wc(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut entries: Vec<(String, u64, u64, u64)> = Vec::new(); // (name, lines, words, bytes)

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            match parts.len() {
                // wc with file: lines words bytes filename
                4 => {
                    let lines = parts[0].parse::<u64>().unwrap_or(0);
                    let words = parts[1].parse::<u64>().unwrap_or(0);
                    let bytes = parts[2].parse::<u64>().unwrap_or(0);
                    let name = parts[3].to_string();
                    entries.push((name, lines, words, bytes));
                }
                // wc from stdin (no filename): lines words bytes
                3 => {
                    let first = parts[0].parse::<u64>().unwrap_or(0);
                    let second = parts[1].parse::<u64>().unwrap_or(0);
                    let third = parts[2].parse::<u64>().unwrap_or(0);
                    // Could be "lines words bytes" (stdin) or "count count filename"
                    if parts[2].parse::<u64>().is_ok() {
                        entries.push((String::new(), first, second, third));
                    } else {
                        // "count filename" with extra column — treat as partial
                        entries.push((parts[2].to_string(), first, second, 0));
                    }
                }
                // wc -l/-w/-c: single count + filename (e.g. "22 file.ts")
                2 => {
                    let count = parts[0].parse::<u64>().unwrap_or(0);
                    let name = parts[1].to_string();
                    // We don't know which flag was used, show as lines
                    entries.push((name, count, 0, 0));
                }
                // wc -l from stdin: just a number
                1 => {
                    if let Ok(count) = parts[0].parse::<u64>() {
                        entries.push((String::new(), count, 0, 0));
                    }
                }
                _ => continue,
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => {
                let jv: Vec<serde_json::Value> = entries.iter().map(|(name, lines, words, bytes)| {
                    serde_json::json!({"file": name, "lines": lines, "words": words, "bytes": bytes})
                }).collect();
                serde_json::json!({"entries": jv, "count": entries.len()}).to_string()
            }
            _ => {
                let mut out = String::new();
                let has_full_stats = entries.iter().any(|(_, _, w, b)| *w > 0 || *b > 0);
                for (name, lines, words, bytes) in &entries {
                    let stats = if has_full_stats {
                        format!("{}L {}W {}B", lines, words, bytes)
                    } else {
                        format!("{}L", lines)
                    };
                    if name.is_empty() {
                        out.push_str(&format!("{}\n", stats));
                    } else if name == "total" {
                        out.push_str(&format!("total: {}\n", stats));
                    } else {
                        out.push_str(&format!("{} {}\n", name, stats));
                    }
                }
                out
            }
        };
        crate::parse_out::emit(&output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("wc")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(entries.len())
                .print();
        }
        Ok(())
    }
}
