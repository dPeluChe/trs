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
    /// Break down totals by AI agent (from `TRS_AGENT` attribution).
    pub by_agent: bool,
    /// Aggregate by normalised command family (strips paths/flags).
    pub by_command: bool,
    /// Coverage analysis: which commands are passing through with poor
    /// compression vs which have effective parsers.
    pub coverage: bool,
    /// Row cap. Overrides the default for either `--history` (20) or
    /// the summary's Top Commands table (15).
    pub limit: Option<usize>,
}

/// Default row cap for `--history`.
const DEFAULT_HISTORY_LIMIT: usize = 20;
/// Default row cap for the summary's Top Commands table.
const DEFAULT_TOP_LIMIT: usize = 15;

/// Aggregated statistics for a single command name.
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

/// Execute the stats command and print results to stdout.
pub fn handle_stats(input: &StatsInput) {
    let entries = if input.project {
        tracker::read_project_history()
    } else {
        tracker::read_history()
    };

    if input.coverage {
        let limit = input.limit.unwrap_or(15);
        super::stats_coverage::print_coverage(&entries, limit, input.json);
        return;
    }

    if input.json {
        let history_limit = input.limit.unwrap_or(DEFAULT_HISTORY_LIMIT);
        let top_limit = input.limit.unwrap_or(DEFAULT_TOP_LIMIT);
        print_json(&entries, input.history, history_limit, top_limit);
        return;
    }

    if entries.is_empty() {
        println!("No history yet. Run some commands through trs to start tracking.");
        return;
    }

    if input.by_agent {
        print_by_agent(&entries);
        return;
    }

    if input.by_command {
        let limit = input.limit.unwrap_or(DEFAULT_TOP_LIMIT);
        print_by_command(&entries, limit);
        return;
    }

    if input.history {
        let limit = input.limit.unwrap_or(DEFAULT_HISTORY_LIMIT);
        print_history(&entries, limit);
    } else {
        let top_limit = input.limit.unwrap_or(DEFAULT_TOP_LIMIT);
        print_summary(&entries, top_limit);
    }
}

/// Normalise a full command string to its "family" key:
/// strips file paths, numeric IDs, and flags — keeps only the
/// binary name plus up to two meaningful subcommand tokens.
///
/// Examples:
///   "grep -rn foo /src"          → "grep"
///   "git diff --stat origin/main" → "git diff"
///   "npm run format:check"        → "npm run format:check"
///   "gh pr view 123"              → "gh pr view"
///   "cargo test --lib"            → "cargo test"
fn normalize_cmd(cmd: &str) -> String {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let base = match parts.first() {
        Some(b) => *b,
        None => return String::new(),
    };

    // Helper: is a token a "meaningful" subcommand (not a flag, path, or number)?
    let is_subcmd = |t: &str| -> bool {
        !t.starts_with('-')
            && !t.starts_with('/')
            && !t.starts_with('~')
            && !t.starts_with('.')
            && !t
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
    };

    match base {
        // Commands where binary + subcommand matters
        "git" | "cargo" | "docker" | "kubectl" | "helm" | "terraform" | "aws" => {
            match parts.get(1).copied().filter(|t| is_subcmd(t)) {
                Some(sub) => format!("{} {}", base, sub),
                None => base.to_string(),
            }
        }
        // gh has 3-token commands: gh pr view, gh run list
        "gh" => {
            let sub1 = parts.get(1).copied().filter(|t| is_subcmd(t));
            let sub2 = parts.get(2).copied().filter(|t| is_subcmd(t));
            match (sub1, sub2) {
                (Some(s1), Some(s2)) => format!("{} {} {}", base, s1, s2),
                (Some(s1), None) => format!("{} {}", base, s1),
                _ => base.to_string(),
            }
        }
        // npm/pnpm/bun/yarn run <script> — include script name
        "npm" | "pnpm" | "bun" | "yarn" => {
            let sub = parts.get(1).copied().unwrap_or("");
            if sub == "run" {
                let script = parts.get(2).copied().filter(|t| is_subcmd(t));
                match script {
                    Some(s) => format!("{} run {}", base, s),
                    None => format!("{} run", base),
                }
            } else if is_subcmd(sub) {
                format!("{} {}", base, sub)
            } else {
                base.to_string()
            }
        }
        // Everything else: binary name only
        _ => base.to_string(),
    }
}

/// Aggregate by normalised command family and print sorted by tokens saved.
fn print_by_command(entries: &[HistoryEntry], limit: usize) {
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
fn print_by_agent(entries: &[HistoryEntry]) {
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
fn print_summary(entries: &[HistoryEntry], top_limit: usize) {
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
fn print_history(entries: &[HistoryEntry], limit: usize) {
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
fn print_json(
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
mod tests {
    use super::*;

    #[test]
    fn format_bypass_cell_zero_is_plain() {
        // Eye-skip rendering for the common case.
        assert_eq!(format_bypass_cell(0, 100), "0");
        assert_eq!(format_bypass_cell(0, 0), "0");
    }

    #[test]
    fn format_bypass_cell_includes_rate_when_nonzero() {
        // Non-zero shows count and rate so the user can tell at a
        // glance whether bypass is rare or chronic.
        assert_eq!(format_bypass_cell(3, 142), "3 (2.1%)");
        assert_eq!(format_bypass_cell(50, 100), "50 (50.0%)");
    }

    #[test]
    fn format_bypass_cell_zero_total_omits_rate() {
        // Defensive: avoid div-by-zero when total is 0 but bypass > 0
        // (shouldn't happen in practice — bypass entries also count
        // as commands — but the guard keeps the function total).
        assert_eq!(format_bypass_cell(2, 0), "2");
    }
}
