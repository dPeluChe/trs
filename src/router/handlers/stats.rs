//! Stats command handler.
//!
//! Displays token savings statistics from the execution history.

use std::collections::HashMap;

use crate::tracker::{self, format_bytes_human, HistoryEntry};

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

    // Convert bytes to estimated tokens (bytes / 4)
    let in_tokens = total_in / 4;
    let out_tokens = total_out / 4;
    let saved_tokens = total_saved / 4;

    println!("trs Token Savings");
    println!("{}", "=".repeat(35));
    println!("Total commands:    {}", total_cmds);
    println!("Input tokens:      {}", format_bytes_human(in_tokens));
    println!("Output tokens:     {}", format_bytes_human(out_tokens));
    println!(
        "Tokens saved:      {} ({:.1}%)",
        format_bytes_human(saved_tokens),
        avg_pct
    );

    // Efficiency meter
    let filled = (avg_pct / 5.0).round() as usize;
    let filled = filled.min(20);
    let empty = 20 - filled;
    println!(
        "Efficiency: {}{} {:.0}%",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty),
        avg_pct
    );

    // Top commands by tokens saved
    let mut agg: HashMap<String, CommandAgg> = HashMap::new();
    for entry in entries {
        let e = agg.entry(entry.cmd.clone()).or_default();
        e.count += 1;
        e.in_bytes += entry.in_bytes;
        e.out_bytes += entry.out_bytes;
    }

    let mut sorted: Vec<(String, CommandAgg)> = agg.into_iter().collect();
    sorted.sort_by(|a, b| b.1.saved().cmp(&a.1.saved()));
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
}

/// Print recent command history (last 20 entries).
fn print_history(entries: &[HistoryEntry]) {
    let start = if entries.len() > 20 {
        entries.len() - 20
    } else {
        0
    };
    let recent = &entries[start..];

    println!("Recent Commands");
    println!("{}", "\u{2500}".repeat(50));
    for entry in recent {
        let saved = entry.in_bytes.saturating_sub(entry.out_bytes);
        let pct = if entry.in_bytes == 0 {
            0
        } else {
            ((saved as f64 / entry.in_bytes as f64) * 100.0) as u8
        };
        println!(
            "  {:<25} {:>5} -> {:>5}  -{:>2}%  {}ms",
            truncate_cmd(&entry.cmd, 25),
            format_bytes_human(entry.in_bytes),
            format_bytes_human(entry.out_bytes),
            pct,
            entry.ms
        );
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

    let mut json = serde_json::json!({
        "total_commands": entries.len(),
        "input_bytes": total_in,
        "output_bytes": total_out,
        "saved_bytes": total_saved,
        "input_tokens": total_in / 4,
        "output_tokens": total_out / 4,
        "saved_tokens": total_saved / 4,
        "avg_reduction_pct": (avg_pct * 10.0).round() / 10.0,
    });

    if include_history {
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
    sorted.sort_by(|a, b| b.1.saved().cmp(&a.1.saved()));
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

    println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
}

/// Truncate a command string to fit within a given width.
fn truncate_cmd(cmd: &str, max_len: usize) -> String {
    if cmd.len() <= max_len {
        cmd.to_string()
    } else {
        format!("{}...", &cmd[..max_len.saturating_sub(3)])
    }
}
