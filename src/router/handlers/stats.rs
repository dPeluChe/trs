//! Stats command handler.
//!
//! Displays token savings statistics from the execution history.

use std::collections::HashMap;

use time::OffsetDateTime;

use crate::tracker::{self, format_bytes_human, HistoryEntry};

/// Month abbreviations for timestamp formatting.
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Resolve the local timezone offset (cached for the process lifetime).
fn local_offset() -> time::UtcOffset {
    time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC)
}

/// Format a Unix timestamp as YYYY-MM-DD in local time.
/// "Mon Apr 20" style label for today's date in the user's local
/// timezone. Used in the `--history` header so the agent can see the
/// current day of week and calendar date without shelling out to
/// `date`.
fn today_date_label(offset: time::UtcOffset) -> String {
    let now = OffsetDateTime::now_utc().to_offset(offset);
    format!("{:?} {:?} {}", now.weekday(), now.month(), now.day())
}

fn format_date(ts: u64) -> String {
    let offset = local_offset();
    match OffsetDateTime::from_unix_timestamp(ts as i64) {
        Ok(dt) => {
            let local = dt.to_offset(offset);
            format!(
                "{:04}-{:02}-{:02}",
                local.year(),
                local.month() as u8,
                local.day()
            )
        }
        Err(_) => "—".to_string(),
    }
}

/// Format a Unix timestamp (seconds) into "Mar 27 14:32" local-time string.
fn format_timestamp(ts: u64, offset: time::UtcOffset) -> String {
    let dt = OffsetDateTime::from_unix_timestamp(ts as i64).unwrap_or(OffsetDateTime::UNIX_EPOCH);
    let local = dt.to_offset(offset);
    let month = MONTHS[local.month() as usize - 1];
    format!(
        "{} {:>2} {:02}:{:02}",
        month,
        local.day(),
        local.hour(),
        local.minute(),
    )
}

/// Input for the stats command.
#[derive(Debug, Clone)]
pub struct StatsInput {
    /// Show recent command history.
    pub history: bool,
    /// Filter to current project only.
    pub project: bool,
    /// Output as JSON.
    pub json: bool,
}

/// Aggregated statistics for a single command name.
#[derive(Debug, Default)]
struct CommandAgg {
    count: usize,
    in_bytes: usize,
    out_bytes: usize,
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

/// Execute the stats command and print results to stdout.
pub fn handle_stats(input: &StatsInput) {
    let entries = if input.project {
        tracker::read_project_history()
    } else {
        tracker::read_history()
    };

    if input.json {
        print_json(&entries, input.history);
        return;
    }

    if entries.is_empty() {
        println!("No history yet. Run some commands through trs to start tracking.");
        return;
    }

    if input.history {
        print_history(&entries);
    } else {
        print_summary(&entries);
    }
}

/// Print the full summary view with efficiency meter and top commands.
fn print_summary(entries: &[HistoryEntry]) {
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
    sorted.truncate(10);

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
    println!("More: https://github.com/dPeluChe/trs/blob/main/docs/commands/stats.md");
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

/// Print recent command history (last 20 entries).
fn print_history(entries: &[HistoryEntry]) {
    let start = if entries.len() > 20 {
        entries.len() - 20
    } else {
        0
    };
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
    for entry in recent {
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
    println!("More: https://github.com/dPeluChe/trs/blob/main/docs/commands/stats.md");
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
fn print_json(entries: &[HistoryEntry], include_history: bool) {
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
    });

    if include_history {
        let offset = local_offset();
        let start = if entries.len() > 20 {
            entries.len() - 20
        } else {
            0
        };
        let recent: Vec<serde_json::Value> = entries[start..]
            .iter()
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
    sorted.truncate(10);

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
