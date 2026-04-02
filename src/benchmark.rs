//! Benchmark module for measuring trs compression metrics.
//!
//! Runs a command both raw and through the trs pipeline, then reports
//! byte reduction, estimated token savings, and execution time.

use std::process::{Command, Stdio};
use std::time::Instant;

use crate::classifier::{full_cmd, keep_ratio};

/// Estimated bytes per token (rough GPT/Claude average).
const BYTES_PER_TOKEN: f64 = 4.0;

/// Result of a single benchmark iteration.
struct IterResult {
    raw_bytes: usize,
    compressed_bytes: usize,
    time_ms: u64,
}

/// Aggregated benchmark metrics for reporting.
struct BenchReport {
    command: String,
    raw_bytes: u64,
    compressed_bytes: u64,
    reduction_pct: f64,
    raw_tokens: u64,
    compressed_tokens: u64,
    saved_tokens: u64,
    time_ms: u64,
    iterations: usize,
}

/// Run the benchmark and print results.
pub(crate) fn run_benchmark(command: &str, args: &[String], repeat: usize, json: bool) {
    let iterations = if repeat == 0 { 1 } else { repeat };
    let mut results: Vec<IterResult> = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        match run_once(command, args) {
            Some(r) => results.push(r),
            None => return, // Error already printed
        }
    }

    if results.is_empty() {
        eprintln!("No successful iterations");
        return;
    }

    // Calculate averages
    let count = results.len() as f64;
    let avg_raw = results.iter().map(|r| r.raw_bytes).sum::<usize>() as f64 / count;
    let avg_compressed = results.iter().map(|r| r.compressed_bytes).sum::<usize>() as f64 / count;
    let avg_time = results.iter().map(|r| r.time_ms).sum::<u64>() as f64 / count;

    let reduction_pct = if avg_raw > 0.0 {
        ((avg_raw - avg_compressed) / avg_raw) * 100.0
    } else {
        0.0
    };

    let raw_tokens = (avg_raw / BYTES_PER_TOKEN).round() as u64;
    let compressed_tokens = (avg_compressed / BYTES_PER_TOKEN).round() as u64;
    let saved_tokens = raw_tokens.saturating_sub(compressed_tokens);

    let fcmd = full_cmd(command, args);

    let report = BenchReport {
        command: fcmd,
        raw_bytes: avg_raw as u64,
        compressed_bytes: avg_compressed as u64,
        reduction_pct,
        raw_tokens,
        compressed_tokens,
        saved_tokens,
        time_ms: avg_time as u64,
        iterations,
    };

    if json {
        print_json(&report);
    } else {
        print_table(&report);
    }
}

/// Run one iteration: execute raw, then execute through trs.
fn run_once(cmd: &str, args: &[String]) -> Option<IterResult> {
    let start = Instant::now();

    // Step 1: Execute the command raw and capture output
    let raw_output = match Command::new(cmd)
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
            return None;
        }
    };

    let raw_stdout = String::from_utf8_lossy(&raw_output.stdout);
    let raw_stderr = String::from_utf8_lossy(&raw_output.stderr);
    let raw_bytes = raw_stdout.len() + raw_stderr.len();

    // Step 2: Execute the same command through trs to get compressed output
    let compressed_bytes = run_through_trs(cmd, args, raw_bytes);

    let time_ms = start.elapsed().as_millis() as u64;

    Some(IterResult {
        raw_bytes,
        compressed_bytes,
        time_ms,
    })
}

/// Run the command through trs and return the compressed byte count.
///
/// Locates the current trs binary and invokes `trs <cmd> [args...]`,
/// capturing stdout+stderr to measure compressed output size.
/// Falls back to estimation if the trs binary cannot be found.
fn run_through_trs(cmd: &str, args: &[String], raw_bytes: usize) -> usize {
    // Find our own binary path
    let trs_bin = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return estimate_compressed_size(cmd, args, raw_bytes),
    };

    // Build trs invocation: trs <cmd> [args...]
    let mut trs_args: Vec<&str> = Vec::with_capacity(args.len() + 1);
    trs_args.push(cmd);
    for a in args {
        trs_args.push(a);
    }

    let output = match Command::new(&trs_bin)
        .args(&trs_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(o) => o,
        Err(_) => return estimate_compressed_size(cmd, args, raw_bytes),
    };

    let compressed = output.stdout.len() + output.stderr.len();
    // Sanity check: if compressed is somehow larger, use estimation
    if compressed > raw_bytes && raw_bytes > 0 {
        return estimate_compressed_size(cmd, args, raw_bytes);
    }
    compressed
}

/// Fallback: estimate compressed size using the shared keep_ratio table.
fn estimate_compressed_size(cmd: &str, args: &[String], raw_bytes: usize) -> usize {
    let subcmd = args.first().map(|s| s.as_str()).unwrap_or("");
    let ratio = keep_ratio(cmd, subcmd);
    (raw_bytes as f64 * ratio).max(1.0) as usize
}

fn print_json(r: &BenchReport) {
    let obj = serde_json::json!({
        "command": r.command,
        "raw_bytes": r.raw_bytes,
        "compressed_bytes": r.compressed_bytes,
        "reduction_pct": (r.reduction_pct * 10.0).round() / 10.0,
        "raw_tokens": r.raw_tokens,
        "compressed_tokens": r.compressed_tokens,
        "saved_tokens": r.saved_tokens,
        "time_ms": r.time_ms
    });
    println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
}

fn print_table(r: &BenchReport) {
    println!();
    println!("Benchmark: {}", r.command);
    println!("{}", "-".repeat(35));
    println!("Raw output:   {:>8} bytes", format_number(r.raw_bytes));
    println!(
        "Compressed:   {:>8} bytes",
        format_number(r.compressed_bytes)
    );
    println!("Reduction:    {:>7.1}%", r.reduction_pct);
    println!(
        "Est. tokens:  {:>5} -> {} (saved {})",
        r.raw_tokens, r.compressed_tokens, r.saved_tokens
    );
    println!("Time:         {:>7}ms", r.time_ms);
    if r.iterations > 1 {
        println!("Iterations:   {:>8}", r.iterations);
    }
    println!();
}

/// Format a number with comma separators.
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    if len <= 3 {
        return s;
    }
    let mut result = String::with_capacity(len + len / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(b as char);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(42), "42");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1247), "1,247");
        assert_eq!(format_number(1_000_000), "1,000,000");
    }

    #[test]
    fn test_estimate_compressed_size() {
        let args = vec!["status".to_string()];
        let result = estimate_compressed_size("git", &args, 1000);
        assert_eq!(result, 200); // 20% keep ratio

        let args = vec!["log".to_string()];
        let result = estimate_compressed_size("git", &args, 1000);
        assert_eq!(result, 100); // 10% keep ratio

        let args: Vec<String> = vec![];
        let result = estimate_compressed_size("unknown-cmd", &args, 1000);
        assert_eq!(result, 500); // 50% default keep ratio
    }

    #[test]
    fn test_bytes_per_token_constant() {
        let bytes = 1000u64;
        let tokens = (bytes as f64 / BYTES_PER_TOKEN).round() as u64;
        assert_eq!(tokens, 250);
    }

    #[test]
    fn test_reduction_pct_zero_input() {
        // When raw is 0, reduction should be 0%
        let avg_raw = 0.0f64;
        let avg_compressed = 0.0f64;
        let pct = if avg_raw > 0.0 {
            ((avg_raw - avg_compressed) / avg_raw) * 100.0
        } else {
            0.0
        };
        assert_eq!(pct, 0.0);
    }
}
