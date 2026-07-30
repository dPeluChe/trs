//! `trs stats --gaps` — where compression is still being left on the table.
//!
//! Split out of `stats_render.rs` to keep that file under the repo's LOC
//! limit. The default stats view ranks by savings achieved; this one ranks by
//! what still reached the agent, which is the question that finds missing
//! parsers.

use crate::tracker::{format_bytes_human, HistoryEntry};

use super::stats_render::CommandAgg;

/// Commands ranked by bytes that passed through with little compression —
/// the inverse of `print_by_command`, which ranks by what already works.
///
/// This exists because the analysis had to be done by hand once: a month of
/// history showed `aws` was the largest source of uncompressed output at 1%
/// savings, and nothing in the tool surfaced it. Two signals matter — volume
/// wasted, and whether the binary is even in the registry.
pub(crate) fn print_gaps(entries: &[HistoryEntry], limit: usize) {
    use std::collections::BTreeMap;

    let mut agg: BTreeMap<String, CommandAgg> = BTreeMap::new();
    for entry in entries {
        // Group by binary (basename), not by full command line: the question
        // is "which tool needs a parser", not "which invocation was big".
        let Some(first) = entry.cmd.split_whitespace().next() else {
            continue;
        };
        let bin = first.rsplit(['/', '\\']).next().unwrap_or(first);
        if bin.is_empty() {
            continue;
        }
        let e = agg.entry(bin.to_string()).or_default();
        e.count += 1;
        e.in_bytes += entry.in_bytes;
        e.out_bytes += entry.out_bytes;
    }

    let mut rows: Vec<(String, CommandAgg)> =
        agg.into_iter().filter(|(_, a)| a.in_bytes > 0).collect();
    // Wasted bytes = what reached the agent. Ranking by this rather than by
    // percentage keeps a 1%-of-300MB command above a 40%-of-2KB one.
    rows.sort_by_key(|(_, a)| std::cmp::Reverse(a.out_bytes));
    rows.truncate(limit);

    println!("trs — compression gaps");
    println!("{}", "=".repeat(56));
    println!("Ranked by bytes that still reached the agent.\n");
    println!(
        "  {:<14}{:>7}{:>11}{:>9}{:>8}",
        "command", "uses", "passed", "saved", "parser"
    );
    println!("  {}", "─".repeat(50));
    for (bin, a) in &rows {
        let pct = 100 - (a.saved() * 100 / a.in_bytes.max(1));
        let known = if crate::command_registry::is_known_binary(bin) {
            "yes"
        } else {
            "NO"
        };
        println!(
            "  {:<14}{:>7}{:>11}{:>8}%{:>8}",
            bin,
            a.count,
            format_bytes_human(a.out_bytes),
            100 - pct,
            known
        );
    }
    println!("\n`parser: NO` means the binary has no entry in the command");
    println!("registry — usually the cheapest win. High `passed` with a");
    println!("registered parser means the parser has headroom.");
}
