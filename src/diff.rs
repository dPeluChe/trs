//! `trs diff <cmd>` — show raw vs compact output and what trs dropped.
//!
//! Runs the command once raw, runs it again through the trs binary to get the
//! compacted output, then reports the byte/token delta and the lines present
//! in raw but absent from compact. Lets a user verify what the agent loses.

use std::collections::HashSet;
use std::process::{Command, Stdio};

use crate::classifier::full_cmd;

/// Rough GPT/Claude average; matches `benchmark.rs`.
const BYTES_PER_TOKEN: f64 = 4.0;

/// How many dropped lines to print before truncating (human output).
const MAX_DROPPED_SHOWN: usize = 40;

pub(crate) fn run_diff(command: &str, args: &[String], json: bool) {
    let raw = match capture_raw(command, args) {
        Some(r) => r,
        None => return, // error already printed
    };
    let compact = capture_compact(command, args).unwrap_or_default();

    let dropped = dropped_lines(&raw, &compact);
    let raw_bytes = raw.len();
    let compact_bytes = compact.len();

    if json {
        print_json(command, args, raw_bytes, compact_bytes, &dropped);
    } else {
        print_human(command, args, raw_bytes, compact_bytes, &compact, &dropped);
    }
}

/// Lines in `raw` that don't appear (trimmed) anywhere in `compact` — i.e.
/// what trs dropped or collapsed. Preserves raw order, skips blanks.
fn dropped_lines(raw: &str, compact: &str) -> Vec<String> {
    let kept: HashSet<&str> = compact.lines().map(str::trim).collect();
    raw.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !kept.contains(l))
        .map(|l| l.to_string())
        .collect()
}

fn tokens(bytes: usize) -> u64 {
    (bytes as f64 / BYTES_PER_TOKEN).round() as u64
}

fn reduction_pct(raw: usize, compact: usize) -> f64 {
    if raw == 0 {
        return 0.0;
    }
    ((raw as f64 - compact as f64) / raw as f64) * 100.0
}

/// Run the command directly and return combined stdout+stderr.
fn capture_raw(cmd: &str, args: &[String]) -> Option<String> {
    match Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(o) => {
            let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
            s.push_str(&String::from_utf8_lossy(&o.stderr));
            Some(s)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("Command not found: {}", cmd);
            None
        }
        Err(e) => {
            eprintln!("Failed to execute '{}': {}", cmd, e);
            None
        }
    }
}

/// Run `trs <cmd> [args]` (this binary) and return its compacted stdout+stderr.
fn capture_compact(cmd: &str, args: &[String]) -> Option<String> {
    let trs_bin = std::env::current_exe().ok()?;
    let mut trs_args: Vec<&str> = Vec::with_capacity(args.len() + 1);
    trs_args.push(cmd);
    trs_args.extend(args.iter().map(String::as_str));
    let o = Command::new(&trs_bin)
        .args(&trs_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .ok()?;
    let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    Some(s)
}

fn print_human(
    cmd: &str,
    args: &[String],
    raw_bytes: usize,
    compact_bytes: usize,
    compact: &str,
    dropped: &[String],
) {
    let saved = tokens(raw_bytes).saturating_sub(tokens(compact_bytes));
    println!();
    println!("trs diff: {}", full_cmd(cmd, args));
    println!("{}", "─".repeat(50));
    println!(
        "raw:     {:>8} B  ~{} tok",
        format_number(raw_bytes as u64),
        tokens(raw_bytes)
    );
    println!(
        "compact: {:>8} B  ~{} tok   ({:.0}% smaller, {} tok saved)",
        format_number(compact_bytes as u64),
        tokens(compact_bytes),
        reduction_pct(raw_bytes, compact_bytes),
        saved
    );
    println!("{}", "─".repeat(50));
    println!("compact output (what the agent sees):");
    if compact.trim().is_empty() {
        println!("  (empty)");
    } else {
        for line in compact.lines() {
            println!("  {}", line);
        }
    }
    println!("{}", "─".repeat(50));
    if dropped.is_empty() {
        println!("dropped / collapsed: none (nothing removed)");
    } else {
        println!(
            "dropped / collapsed ({} lines in raw, not in compact):",
            dropped.len()
        );
        for line in dropped.iter().take(MAX_DROPPED_SHOWN) {
            println!("  − {}", line);
        }
        if dropped.len() > MAX_DROPPED_SHOWN {
            println!("  … (+{} more)", dropped.len() - MAX_DROPPED_SHOWN);
        }
    }
}

fn print_json(
    cmd: &str,
    args: &[String],
    raw_bytes: usize,
    compact_bytes: usize,
    dropped: &[String],
) {
    let obj = serde_json::json!({
        "command": full_cmd(cmd, args),
        "raw_bytes": raw_bytes,
        "compact_bytes": compact_bytes,
        "raw_tokens": tokens(raw_bytes),
        "compact_tokens": tokens(compact_bytes),
        "saved_tokens": tokens(raw_bytes).saturating_sub(tokens(compact_bytes)),
        "reduction_pct": (reduction_pct(raw_bytes, compact_bytes) * 10.0).round() / 10.0,
        "dropped_line_count": dropped.len(),
        "dropped_lines": dropped,
    });
    println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
}

/// Thousands-separated number (matches benchmark's formatting).
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropped_lines_are_raw_minus_compact() {
        let raw = "keep me\ndrop me\n  keep me too \nnoise line";
        let compact = "keep me\nkeep me too";
        let dropped = dropped_lines(raw, compact);
        assert_eq!(dropped, vec!["drop me", "noise line"]);
    }

    #[test]
    fn nothing_dropped_when_compact_has_all_lines() {
        assert!(dropped_lines("a\nb", "a\nb\nextra").is_empty());
    }

    #[test]
    fn metrics_math() {
        assert_eq!(tokens(400), 100);
        assert!((reduction_pct(1000, 250) - 75.0).abs() < f64::EPSILON);
        assert_eq!(reduction_pct(0, 0), 0.0);
    }
}
