use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
    /// Parse `cargo test` output into pass/fail/ignore/suite summary.
    pub(crate) fn handle_cargo_test(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        let mut passed = 0usize;
        let mut failed = 0usize;
        let mut ignored = 0usize;
        let mut filtered = 0usize;
        let mut failed_names: Vec<String> = Vec::new();
        let mut duration = String::new();
        let mut compile_errors: Vec<String> = Vec::new();
        let mut suites = 0usize;
        // Per-failure panic summary: "<file:line:col>: <message>" captured
        // from the `---- name stdout ----` block — the location and reason
        // are the actionable signal, not just the test name.
        let mut panic_info: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut panic_test: Option<String> = None;
        let mut awaiting_panic_msg = false;

        for line in input.lines() {
            let trimmed = line.trim();

            // "---- tests::test_add stdout ----" opens a failure detail block.
            if let Some(rest) = trimmed.strip_prefix("---- ") {
                if let Some(name) = rest.strip_suffix(" stdout ----") {
                    panic_test = Some(name.to_string());
                    awaiting_panic_msg = false;
                    continue;
                }
            }
            if let Some(name) = &panic_test {
                // "thread 'X' panicked at src/math.rs:42:9:"
                if let Some(pos) = trimmed.find("panicked at ") {
                    let loc = trimmed[pos + "panicked at ".len()..]
                        .trim_end_matches(':')
                        .to_string();
                    panic_info.insert(name.clone(), loc);
                    awaiting_panic_msg = true;
                    continue;
                }
                // First line after "panicked at" is the panic message.
                if awaiting_panic_msg && !trimmed.is_empty() && !trimmed.starts_with("note:") {
                    if let Some(entry) = panic_info.get_mut(name) {
                        entry.push_str(": ");
                        entry.push_str(trimmed);
                    }
                    awaiting_panic_msg = false;
                    continue;
                }
            }

            // Summary line: "test result: ok. X passed; Y failed; Z ignored; ..."
            if trimmed.starts_with("test result:") {
                suites += 1;
                // Remove "test result: ok." or "test result: FAILED." prefix
                let summary_part = trimmed
                    .strip_prefix("test result:")
                    .unwrap_or(trimmed)
                    .trim()
                    .trim_start_matches("ok.")
                    .trim_start_matches("FAILED.")
                    .trim();
                for part in summary_part.split(';') {
                    let t = part.trim();
                    if t.contains("passed") {
                        passed += t
                            .split_whitespace()
                            .next()
                            .and_then(|n| n.parse::<usize>().ok())
                            .unwrap_or(0);
                    } else if t.contains("failed") {
                        let n = t
                            .split_whitespace()
                            .next()
                            .and_then(|n| n.parse::<usize>().ok())
                            .unwrap_or(0);
                        failed += n;
                    } else if t.contains("ignored") {
                        ignored += t
                            .split_whitespace()
                            .next()
                            .and_then(|n| n.parse::<usize>().ok())
                            .unwrap_or(0);
                    } else if t.contains("filtered out") {
                        filtered += t
                            .split_whitespace()
                            .next()
                            .and_then(|n| n.parse::<usize>().ok())
                            .unwrap_or(0);
                    } else if t.contains("finished in") {
                        if let Some(pos) = t.find("finished in ") {
                            duration = t[pos + 12..].to_string();
                        }
                    }
                }
                continue;
            }

            // Individual test result: "test name ... FAILED"
            if trimmed.starts_with("test ") && trimmed.contains(" ... ") {
                if trimmed.ends_with("FAILED") {
                    let name = trimmed
                        .strip_prefix("test ")
                        .unwrap_or("")
                        .split(" ... ")
                        .next()
                        .unwrap_or("")
                        .to_string();
                    failed_names.push(name);
                }
                continue;
            }

            // Compile errors (before tests run)
            if trimmed.starts_with("error[") || trimmed.starts_with("error:") {
                compile_errors.push(trimmed.to_string());
            }
        }

        let total = passed + failed + ignored;
        let success = failed == 0 && compile_errors.is_empty();

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({
                "success": success,
                "passed": passed,
                "failed": failed,
                "ignored": ignored,
                "filtered": filtered,
                "total": total,
                "suites": suites,
                "duration": duration,
                "failed_tests": failed_names,
                "panics": panic_info,
                "compile_errors": compile_errors,
            })
            .to_string(),
            _ => {
                let mut out = String::new();
                if !compile_errors.is_empty() {
                    out.push_str(&format!("compile errors ({}):\n", compile_errors.len()));
                    for err in compile_errors.iter().take(10) {
                        out.push_str(&format!("  {}\n", err));
                    }
                    if compile_errors.len() > 10 {
                        out.push_str(&format!("  ...+{} more\n", compile_errors.len() - 10));
                    }
                }
                let status = if success { "ok" } else { "FAILED" };
                out.push_str(&format!("cargo test: {} ({} passed", status, passed));
                if failed > 0 {
                    out.push_str(&format!(", {} failed", failed));
                }
                if ignored > 0 {
                    out.push_str(&format!(", {} ignored", ignored));
                }
                if filtered > 0 {
                    out.push_str(&format!(", {} filtered", filtered));
                }
                if suites > 1 {
                    out.push_str(&format!(", {} suites", suites));
                }
                if !duration.is_empty() {
                    out.push_str(&format!(", {}", duration));
                }
                out.push_str(")\n");

                if !failed_names.is_empty() {
                    out.push_str(&format!("failures ({}):\n", failed_names.len()));
                    for name in &failed_names {
                        match panic_info.get(name) {
                            Some(info) => out.push_str(&format!("  {} — {}\n", name, info)),
                            None => out.push_str(&format!("  {}\n", name)),
                        }
                    }
                }
                out
            }
        };

        crate::parse_out::emit(&output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("cargo-test")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(total)
                .print();
        }
        Ok(())
    }
}
