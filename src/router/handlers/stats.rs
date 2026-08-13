//! Stats command handler.
//!
//! Displays token savings statistics from the execution history.

use time::OffsetDateTime;

use crate::tracker;

/// Month abbreviations for timestamp formatting.
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Resolve the local timezone offset (cached for the process lifetime).
pub(crate) fn local_offset() -> time::UtcOffset {
    time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC)
}

/// Format a Unix timestamp as YYYY-MM-DD in local time.
/// "Mon Apr 20" style label for today's date in the user's local
/// timezone. Used in the `--history` header so the agent can see the
/// current day of week and calendar date without shelling out to
/// `date`.
pub(crate) fn today_date_label(offset: time::UtcOffset) -> String {
    let now = OffsetDateTime::now_utc().to_offset(offset);
    format!("{:?} {:?} {}", now.weekday(), now.month(), now.day())
}

pub(crate) fn format_date(ts: u64) -> String {
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
pub(crate) fn format_timestamp(ts: u64, offset: time::UtcOffset) -> String {
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
    pub gaps: bool,
    pub days: Option<u64>,
    /// Row cap. Overrides the default for either `--history` (20) or
    /// the summary's Top Commands table (15).
    pub limit: Option<usize>,
}

/// Default row cap for `--history`.
const DEFAULT_HISTORY_LIMIT: usize = 20;
/// Default row cap for the summary's Top Commands table.
const DEFAULT_TOP_LIMIT: usize = 15;

/// Execute the stats command and print results to stdout.
pub fn handle_stats(input: &StatsInput) {
    let entries = if input.project {
        tracker::read_project_history()
    } else {
        tracker::read_history()
    };

    // `--days` scopes every view. The summary keeps lifetime by default on
    // purpose: "tokens saved" is a cumulative total and the honest answer to
    // "was trs worth installing" — unlike the efficiency percentage, it does
    // not get distorted by an old outlier, it just grows.
    let entries: Vec<tracker::HistoryEntry> = match input.days {
        Some(d) => {
            let cutoff = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|x| x.as_secs())
                .unwrap_or(0)
                .saturating_sub(d * 86_400);
            entries.into_iter().filter(|e| e.ts >= cutoff).collect()
        }
        None => entries,
    };

    if input.gaps {
        let limit = input.limit.unwrap_or(15);
        // 30 days by default. Over the full history this view keeps pointing at
        // problems already solved — after the aws parser shipped, `aws` still
        // ranked first at 0%, because the bytes it names were spent before the
        // fix existed. A gap list has to age out or it stops being a to-do.
        let days = input.days.unwrap_or(30);
        super::stats_gaps::print_gaps(&entries, limit, days);
        return;
    }

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
        print_summary(&entries, top_limit, input.days);
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
pub(crate) fn normalize_cmd(cmd: &str) -> String {
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

use super::stats_render::{
    print_by_agent, print_by_command, print_history, print_json, print_summary,
};
