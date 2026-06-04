//! Stats rendering — the human (`print_by_command/agent/summary/history`) and
//! JSON (`print_json`) views for `trs stats`. The dispatcher, aggregation, and
//! time/normalize helpers live in `stats.rs`.

use super::stats::{format_date, format_timestamp, local_offset, normalize_cmd, today_date_label};

use std::collections::HashMap;

use crate::tracker::{format_bytes_human, HistoryEntry};
use time::OffsetDateTime;

#[derive(Debug, Default)]
struct CommandAgg {
    count: usize,
    in_bytes: usize,
    out_bytes: usize,
    /// Subset of `count` recorded as a bypass observation
    /// (`TRS_SKIP=1` prefix). Surfaces as a column in the
    /// `--by-agent` view so the user can tell which agents reach for
    /// the escape hatch and whether prompt-level interventions are
    /// reducing it.
    bypass_count: usize,
}

impl CommandAgg {
    fn saved(&self) -> usize {
        self.in_bytes.saturating_sub(self.out_bytes)
    }

    fn avg_reduction_pct(&self) -> f64 {
        if self.in_bytes == 0 {
            0.0
        } else {
            (self.saved() as f64 / self.in_bytes as f64) * 100.0
        }
    }
}

/// Aggregate by normalised command family and print sorted by tokens saved.
pub(crate) fn print_by_command(entries: &[HistoryEntry], limit: usize) {
    use std::collections::BTreeMap;

    let mut agg: BTreeMap<String, CommandAgg> = BTreeMap::new();
    for entry in entries {
        let key = normalize_cmd(&entry.cmd);
        if key.is_empty() {
            continue;
        }
        let e = agg.entry(key).or_default();
        e.count += 1;
        e.in_bytes += entry.in_bytes;
        e.out_bytes += entry.out_bytes;
    }

    let mut rows: Vec<(String, CommandAgg)> = agg.into_iter().collect();
    rows.sort_by_key(|(_, a)| std::cmp::Reverse(a.saved()));
    rows.truncate(limit);

    println!("trs Token Savings — by command");
    println!("{}", "=".repeat(50));
    println!(
        "  {:<22} {:>5} {:>7}  {:>6}  {:>10}",
        "COMMAND", "CALLS", "SHARE", "AVG -%", "SAVED"
    );
    println!("{}", "\u{2500}".repeat(50));

    let total_saved: usize = rows.iter().map(|(_, a)| a.saved()).sum();
    for (cmd, stats) in &rows {
        let share = if total_saved > 0 {
            100.0 * stats.saved() as f64 / total_saved as f64
        } else {
            0.0
        };
        println!(
            "  {:<22} {:>5} {:>6.1}%  {:>5.0}%  {:>10}",
            truncate_cmd(cmd, 22),
            stats.count,
            share,
            stats.avg_reduction_pct(),
            format_bytes_human(stats.saved() / 4)
        );
    }
    println!();
    println!("More: https://github.com/dPeluChe/trs/blob/main/docs/features/stats.md");
}

/// Per-agent breakdown. Aggregates count / tokens saved / avg
/// compression for each distinct `TRS_AGENT` label in the log.
/// Rules-only agents and direct-shell invocations show up as
/// "(untagged)" since no programmatic signal is available there.
pub(crate) fn print_by_agent(entries: &[HistoryEntry]) {
    use std::collections::BTreeMap;

    let mut agg: BTreeMap<String, CommandAgg> = BTreeMap::new();
    for entry in entries {
        let key = entry
            .agent
            .clone()
            .unwrap_or_else(|| "(untagged)".to_string());
        let e = agg.entry(key).or_default();
        e.count += 1;
        e.in_bytes += entry.in_bytes;
        e.out_bytes += entry.out_bytes;
        if entry.bypass.unwrap_or(false) {
            e.bypass_count += 1;
        }
    }

    let total_cmds: usize = agg.values().map(|a| a.count).sum();
    let any_bypass = agg.values().any(|a| a.bypass_count > 0);
    let mut rows: Vec<(String, CommandAgg)> = agg.into_iter().collect();
    rows.sort_by_key(|(_, a)| std::cmp::Reverse(a.saved()));

    println!("trs Token Savings — by agent");
    println!("{}", "=".repeat(60));
    println!(
        "  {:<14} {:>6} {:>8}  {:>6}  {:>10}  {:>10}",
        "AGENT", "CALLS", "SHARE", "AVG -%", "SAVED", "BYPASS"
    );
    println!("{}", "\u{2500}".repeat(60));
    for (agent, stats) in &rows {
        let share = if total_cmds > 0 {
            100.0 * stats.count as f64 / total_cmds as f64
        } else {
            0.0
        };
        let bypass_cell = format_bypass_cell(stats.bypass_count, stats.count);
        println!(
            "  {:<14} {:>6} {:>7.1}%  {:>5.0}%  {:>10}  {:>10}",
            agent,
            stats.count,
            share,
            stats.avg_reduction_pct(),
            format_bytes_human(stats.saved() / 4),
            bypass_cell,
        );
    }
    println!();
    println!("Labels come from TRS_AGENT env var injected by trs rewrite");
    println!("(Claude / Gemini / Cursor / Droid) and the OpenCode / Kilo plugin");
    println!("templates. Rules-based agents (Codex / Antigravity / Windsurf) and");
    println!("direct shell invocations land under (untagged).");
    if any_bypass {
        println!();
        println!("BYPASS counts commands the agent prefixed with TRS_SKIP=1 — trs stepped");
        println!("aside and didn't compress. High rates suggest the agent is reaching for");
        println!("the escape hatch on routine commands; consider refreshing prompts.");
    }
    println!();
    println!("More: https://github.com/dPeluChe/trs/blob/main/docs/features/stats.md");
}

/// Render the BYPASS column for one row. Zero is shown as plain "0"
/// (no parens) so the eye skips over it; non-zero shows the count and
/// rate, which is what the user is actually looking for.
fn format_bypass_cell(bypass_count: usize, total_count: usize) -> String {
    if bypass_count == 0 {
        "0".to_string()
    } else if total_count == 0 {
        format!("{}", bypass_count)
    } else {
        let pct = 100.0 * bypass_count as f64 / total_count as f64;
        format!("{} ({:.1}%)", bypass_count, pct)
    }
}

/// Print the full summary view with efficiency meter and top commands.
pub(crate) fn print_summary(entries: &[HistoryEntry], top_limit: usize) {
    let total_cmds = entries.len();
    let total_in: usize = entries.iter().map(|e| e.in_bytes).sum();
    let total_out: usize = entries.iter().map(|e| e.out_bytes).sum();
    let total_saved = total_in.saturating_sub(total_out);
    let avg_pct = if total_in == 0 {
        0.0
    } else {
        (total_saved as f64 / total_in as f64) * 100.0
    };

    let in_tokens = total_in / 4;
    let out_tokens = total_out / 4;
    let saved_tokens = total_saved / 4;

    let first_ts = entries.iter().map(|e| e.ts).min().unwrap_or(0);
    let last_ts = entries.iter().map(|e| e.ts).max().unwrap_or(0);
    let span_secs = last_ts.saturating_sub(first_ts);
    let days = ((span_secs as f64 / 86400.0).ceil() as u64).max(1);
    let tokens_per_day = saved_tokens as u64 / days;

    // Today window: entries whose ts falls on the same local date as `now`.
    let offset = local_offset();
    let today_entries = today_entries(entries, offset);
    let today_saved_tokens: usize = today_entries
        .iter()
        .map(|e| e.in_bytes.saturating_sub(e.out_bytes))
        .sum::<usize>()
        / 4;

    println!("trs Token Savings");
    println!("{}", "=".repeat(35));
    println!(
        "Period:            {} → {} ({} day{})",
        format_timestamp(first_ts, offset),
        format_timestamp(last_ts, offset),
        days,
        if days == 1 { "" } else { "s" }
    );
    println!("Total commands:    {}", total_cmds);
    println!("Input tokens:      {}", format_bytes_human(in_tokens));
    println!("Output tokens:     {}", format_bytes_human(out_tokens));
    println!(
        "Tokens saved:      {} ({:.1}%)",
        format_bytes_human(saved_tokens),
        avg_pct
    );
    println!(
        "Tokens per day:    {} (avg)",
        format_bytes_human(tokens_per_day as usize)
    );
    println!(
        "Today:             {} saved across {} command{}",
        format_bytes_human(today_saved_tokens),
        today_entries.len(),
        if today_entries.len() == 1 { "" } else { "s" }
    );

    let filled = (avg_pct / 5.0).round() as usize;
    let filled = filled.min(20);
    let empty = 20 - filled;
    println!(
        "Efficiency: {}{} {:.0}%",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty),
        avg_pct
    );

    // Last command footer — confirms tracking is live and points at the
    // detail view for anyone who wants more than the top-N summary.
    if let Some(last) = entries.last() {
        println!();
        println!(
            "Last: {} ({})",
            truncate_cmd(&last.cmd, 40),
            format_timestamp(last.ts, offset)
        );
    }

    // Top commands by tokens saved
    let mut agg: HashMap<String, CommandAgg> = HashMap::new();
    for entry in entries {
        let e = agg.entry(entry.cmd.clone()).or_default();
        e.count += 1;
        e.in_bytes += entry.in_bytes;
        e.out_bytes += entry.out_bytes;
    }

    let mut sorted: Vec<(String, CommandAgg)> = agg.into_iter().collect();
    sorted.sort_by_key(|b| std::cmp::Reverse(b.1.saved()));
    sorted.truncate(top_limit);

    if !sorted.is_empty() {
        println!();
        println!("Top Commands");
        println!("{}", "\u{2500}".repeat(35));
        for (cmd, stats) in &sorted {
            println!(
                "  {:<20} {:>3}x  -{:.0}%  {} saved",
                truncate_cmd(cmd, 20),
                stats.count,
                stats.avg_reduction_pct(),
                format_bytes_human(stats.saved() / 4)
            );
        }
    }

    println!();
    println!("For full history: trs stats --history");
    println!();
    println!("More: https://github.com/dPeluChe/trs/blob/main/docs/features/stats.md");
}

/// Entries whose timestamp falls on the same local-date as "now".
fn today_entries(entries: &[HistoryEntry], offset: time::UtcOffset) -> Vec<&HistoryEntry> {
    let now = OffsetDateTime::now_utc().to_offset(offset);
    let (today_y, today_m, today_d) = (now.year(), now.month(), now.day());
    entries
        .iter()
        .filter(|e| {
            OffsetDateTime::from_unix_timestamp(e.ts as i64)
                .map(|dt| dt.to_offset(offset))
                .map(|dt| (dt.year(), dt.month(), dt.day()) == (today_y, today_m, today_d))
                .unwrap_or(false)
        })
        .collect()
}

/// Print the last `limit` history entries, **newest first**. Matches
/// the convention of `git log`, `journalctl`, `history`, and every other
/// "recent activity" view a user expects to scroll from top.
pub(crate) fn print_history(entries: &[HistoryEntry], limit: usize) {
    let start = entries.len().saturating_sub(limit);
    let recent = &entries[start..];

    let offset = local_offset();
    let today_count = today_entries(entries, offset).len();
    let today_label = today_date_label(offset);
    println!(
        "Recent Commands ({}, {} command{} today)",
        today_label,
        today_count,
        if today_count == 1 { "" } else { "s" }
    );
    println!("{}", "\u{2500}".repeat(64));
    for entry in recent.iter().rev() {
        let saved = entry.in_bytes.saturating_sub(entry.out_bytes);
        let pct = if entry.in_bytes == 0 {
            0
        } else {
            ((saved as f64 / entry.in_bytes as f64) * 100.0) as u8
        };
        println!(
            "  {}  {:<25} {:>5} -> {:>5}  -{:>2}%  {}ms",
            format_timestamp(entry.ts, offset),
            truncate_cmd(&display_cmd(&entry.cmd), 25),
            format_bytes_human(entry.in_bytes),
            format_bytes_human(entry.out_bytes),
            pct,
            entry.ms
        );
    }
    println!();
    println!("More: https://github.com/dPeluChe/trs/blob/main/docs/features/stats.md");
}

/// When a logged command's first token is an absolute path, show the
/// basename instead so users see "trs rewrite" rather than
/// "/Users/you/.local/bin/trs rewrite" eating the entire column width.
/// Also folds embedded newlines to spaces — `python3 -c "..."` and
/// `bash -c "..."` often log a multi-line script that would break the
/// table layout of `--history` otherwise.
/// Used only for display — the history file still stores the full
/// command verbatim.
fn display_cmd(cmd: &str) -> String {
    // Collapse newlines and tabs to single spaces so multi-line
    // scripts stay on one row. Carriage returns are dropped outright
    // (some loggers embed them mid-line).
    let single_line: String = cmd
        .chars()
        .map(|c| match c {
            '\n' | '\t' => ' ',
            '\r' => ' ',
            other => other,
        })
        .collect();
    // Squash runs of whitespace to a single space for readability.
    let mut squashed = String::with_capacity(single_line.len());
    let mut prev_space = false;
    for ch in single_line.chars() {
        if ch == ' ' {
            if !prev_space {
                squashed.push(' ');
            }
            prev_space = true;
        } else {
            squashed.push(ch);
            prev_space = false;
        }
    }
    let trimmed = squashed.trim_start();
    let Some(first_end) = trimmed.find(char::is_whitespace) else {
        if trimmed.starts_with('/') {
            let base = trimmed.rsplit('/').next().unwrap_or(trimmed);
            return base.to_string();
        }
        return trimmed.to_string();
    };
    let (first, rest) = trimmed.split_at(first_end);
    if first.starts_with('/') {
        let base = first.rsplit('/').next().unwrap_or(first);
        format!("{}{}", base, rest)
    } else {
        trimmed.to_string()
    }
}

/// Print stats as JSON.
pub(crate) fn print_json(
    entries: &[HistoryEntry],
    include_history: bool,
    history_limit: usize,
    top_limit: usize,
) {
    let total_in: usize = entries.iter().map(|e| e.in_bytes).sum();
    let total_out: usize = entries.iter().map(|e| e.out_bytes).sum();
    let total_saved = total_in.saturating_sub(total_out);
    let avg_pct = if total_in == 0 {
        0.0
    } else {
        (total_saved as f64 / total_in as f64) * 100.0
    };

    let first_ts = entries.iter().map(|e| e.ts).min().unwrap_or(0);
    let last_ts = entries.iter().map(|e| e.ts).max().unwrap_or(0);
    let span_secs = last_ts.saturating_sub(first_ts);
    let days = ((span_secs as f64 / 86400.0).ceil() as u64).max(1);
    let saved_tokens = total_saved / 4;

    let bypass_count = entries.iter().filter(|e| e.bypass.unwrap_or(false)).count();

    let mut json = serde_json::json!({
        "total_commands": entries.len(),
        "period_start": format_date(first_ts),
        "period_end": format_date(last_ts),
        "period_days": days,
        "tokens_per_day": saved_tokens as u64 / days,
        "input_bytes": total_in,
        "output_bytes": total_out,
        "saved_bytes": total_saved,
        "input_tokens": total_in / 4,
        "output_tokens": total_out / 4,
        "saved_tokens": total_saved / 4,
        "avg_reduction_pct": (avg_pct * 10.0).round() / 10.0,
        "bypass_count": bypass_count,
    });

    if include_history {
        let offset = local_offset();
        let start = entries.len().saturating_sub(history_limit);
        // Newest first — same ordering as the text-mode `print_history`
        // output, so downstream consumers of `--json` see the same shape
        // they'd see scrolling the human view.
        let recent: Vec<serde_json::Value> = entries[start..]
            .iter()
            .rev()
            .map(|e| {
                serde_json::json!({
                    "ts": e.ts,
                    "time": format_timestamp(e.ts, offset),
                    "cmd": e.cmd,
                    "in_bytes": e.in_bytes,
                    "out_bytes": e.out_bytes,
                    "saved_pct": e.saved_pct,
                    "ms": e.ms,
                    "cwd": e.cwd,
                })
            })
            .collect();
        json["history"] = serde_json::Value::Array(recent);
    }

    // Aggregate top commands
    let mut agg: HashMap<String, CommandAgg> = HashMap::new();
    for entry in entries {
        let e = agg.entry(entry.cmd.clone()).or_default();
        e.count += 1;
        e.in_bytes += entry.in_bytes;
        e.out_bytes += entry.out_bytes;
    }
    let mut sorted: Vec<(String, CommandAgg)> = agg.into_iter().collect();
    sorted.sort_by_key(|b| std::cmp::Reverse(b.1.saved()));
    sorted.truncate(top_limit);

    let top: Vec<serde_json::Value> = sorted
        .iter()
        .map(|(cmd, s)| {
            serde_json::json!({
                "cmd": cmd,
                "count": s.count,
                "saved_bytes": s.saved(),
                "saved_tokens": s.saved() / 4,
                "avg_reduction_pct": (s.avg_reduction_pct() * 10.0).round() / 10.0,
            })
        })
        .collect();
    json["top_commands"] = serde_json::Value::Array(top);

    println!(
        "{}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
}

/// Truncate a command string to fit within a given width. UTF-8 safe.
fn truncate_cmd(cmd: &str, max_len: usize) -> String {
    crate::formatter::helpers::truncate(cmd, max_len)
}

#[cfg(test)]
#[path = "stats_render_tests.rs"]
mod tests;
