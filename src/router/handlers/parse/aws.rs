//! `aws` CLI output compression.
//!
//! Field data (16k executions, one month): `aws` was the single largest
//! uncompressed source at ~367 MB and 1% savings, dominated by two recursive
//! `s3` calls that print ONE LINE PER OBJECT:
//!
//! ```text
//! delete: s3://bucket/logs/2026/01/01/a.log
//! delete: s3://bucket/logs/2026/01/01/b.log
//! … ×400,000
//! ```
//!
//! Per-object lines are a receipt, not information: the agent needs the verb,
//! the count, the bucket prefixes and — above all — anything that failed.
//! Errors and warnings are always preserved verbatim.

use std::collections::BTreeMap;

use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

/// Distinct prefixes listed before collapsing to a count.
const PREFIXES_SHOWN: usize = 3;

/// s3 progress verbs, each emitted once per object.
const VERBS: &[&str] = &[
    "delete",
    "upload",
    "download",
    "copy",
    "move",
    "make_bucket",
    "remove_bucket",
];

/// Longest `s3://bucket/dir/` prefix of a key, for grouping. Falls back to the
/// bucket root, then to the raw target when it isn't an s3 URI at all.
fn prefix_of(target: &str) -> String {
    let Some(rest) = target.strip_prefix("s3://") else {
        return target.to_string();
    };
    match rest.rfind('/') {
        Some(cut) => format!("s3://{}/", &rest[..cut]),
        None => format!("s3://{}", rest),
    }
}

/// Split an `aws s3` progress line into `(verb, first_target)`.
/// Handles both `delete: <t>` and `copy: <src> to <dst>`.
fn progress_line(line: &str) -> Option<(&'static str, String)> {
    let t = line.trim();
    for verb in VERBS {
        if let Some(rest) = t.strip_prefix(verb).and_then(|r| r.strip_prefix(": ")) {
            let target = rest.split(" to ").next().unwrap_or(rest).trim();
            return Some((verb, target.to_string()));
        }
    }
    None
}

impl ParseHandler {
    pub(crate) fn handle_aws(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        // JSON bodies (describe-*/get-*/list-*) have their own compressor.
        if input.trim_start().starts_with('{') || input.trim_start().starts_with('[') {
            return Self::handle_download(file, ctx);
        }

        // verb -> (count, prefix -> count)
        let mut verbs: BTreeMap<&'static str, (usize, BTreeMap<String, usize>)> = BTreeMap::new();
        let mut problems: Vec<&str> = Vec::new();
        let mut other: Vec<&str> = Vec::new();

        for line in input.lines() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            // AWS's canonical failure line has no colon after "error", so the
            // generic markers miss it — and a truncated error is the one thing
            // this parser must never produce.
            let aws_failure = t.starts_with("An error occurred")
                || t.starts_with("fatal error")
                || t.contains("Access Denied")
                || t.contains("warning:");
            if aws_failure
                || super::super::common::is_error_line(t)
                || super::super::common::is_warning_line(t)
            {
                problems.push(t);
                continue;
            }
            match progress_line(t) {
                Some((verb, target)) => {
                    let e = verbs.entry(verb).or_default();
                    e.0 += 1;
                    *e.1.entry(prefix_of(&target)).or_default() += 1;
                }
                // Not a per-object receipt: keep it, it carries real content
                // (Completed …, totals, table rows from `s3 ls`).
                None => other.push(t),
            }
        }

        if verbs.is_empty() && problems.is_empty() {
            // Nothing recognized — don't pretend. Pass through untouched.
            crate::parse_out::emit(&input);
            if ctx.stats {
                CommandStats::new()
                    .with_reducer("aws-passthrough")
                    .with_input_bytes(input_bytes)
                    .with_output_bytes(input_bytes)
                    .print();
            }
            return Ok(());
        }

        let total: usize = verbs.values().map(|(n, _)| n).sum();
        let output = match ctx.format {
            OutputFormat::Json => {
                let ops: Vec<serde_json::Value> = verbs
                    .iter()
                    .map(|(verb, (n, prefixes))| {
                        serde_json::json!({
                            "op": verb,
                            "objects": n,
                            "prefixes": prefixes.iter().map(|(p, c)| serde_json::json!({"prefix": p, "objects": c})).collect::<Vec<_>>(),
                        })
                    })
                    .collect();
                serde_json::json!({
                    "objects_total": total,
                    "operations": ops,
                    "problems": problems,
                })
                .to_string()
            }
            _ => {
                let mut out = String::new();
                for (verb, (n, prefixes)) in &verbs {
                    let mut ranked: Vec<(&String, &usize)> = prefixes.iter().collect();
                    ranked.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
                    let shown = ranked
                        .iter()
                        .take(PREFIXES_SHOWN)
                        .map(|(p, c)| format!("{} ({})", p, c))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let more = if ranked.len() > PREFIXES_SHOWN {
                        format!(" +{} more prefixes", ranked.len() - PREFIXES_SHOWN)
                    } else {
                        String::new()
                    };
                    out.push_str(&format!("{}: {} objects, {}{}\n", verb, n, shown, more));
                }
                if !problems.is_empty() {
                    out.push_str(&format!("problems ({}):\n", problems.len()));
                    for p in problems.iter().take(20) {
                        out.push_str(&format!("  {}\n", p));
                    }
                    if problems.len() > 20 {
                        out.push_str(&format!("  ...+{} more\n", problems.len() - 20));
                    }
                }
                for line in other.iter().take(10) {
                    out.push_str(&format!("{}\n", line));
                }
                if other.len() > 10 {
                    out.push_str(&format!("... +{} more lines\n", other.len() - 10));
                }
                out
            }
        };

        crate::parse_out::emit(&output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("aws")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "aws_tests.rs"]
mod tests;
