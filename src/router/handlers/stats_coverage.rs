//! `trs stats --coverage` — analyze `~/.trs/history.jsonl` and surface
//! commands with poor compression. Used to find gaps in parser coverage
//! and to give users a one-shot report they can paste into a GitHub issue.

use crate::tracker::HistoryEntry;

/// Compression threshold below which an entry counts as "low compression".
const LOW_PCT: u8 = 10;

/// Minimum entry count to surface a command — drops one-off noise.
const MIN_COUNT: usize = 5;

/// "Low compression" filter: an aggregated row qualifies if at least
/// this percentage of entries are below `LOW_PCT`.
const LOW_RATIO_PCT: f64 = 40.0;

/// Average input bytes below which we ignore an entry — short outputs
/// don't have enough material to compress and shouldn't be flagged.
const MIN_AVG_IN_BYTES: f64 = 256.0;

#[derive(Default)]
struct Agg {
    count: usize,
    in_bytes: u64,
    low_count: usize,
    sample: String,
}

impl Agg {
    fn avg_in(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.in_bytes as f64 / self.count as f64
        }
    }
    fn low_pct(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.low_count as f64 / self.count as f64 * 100.0
        }
    }
}

/// Entry point — called from `handle_stats` when `--coverage` is set.
/// Coverage analysis over a time window.
///
/// The window is not cosmetic. Run over the full history this view keeps
/// naming problems that were already solved: after the `aws` parser shipped,
/// `aws` still ranked first at 0%, because the bytes it counted were spent
/// before the fix existed. A gap list that never forgets stops being a to-do
/// list. Callers pass `days`; the CLI defaults it to 30.
/// The `--json` payload for a window, without printing. `trs report coverage`
/// uses this so the report and the flag can never describe different data.
/// None when the window holds no history.
pub(crate) fn coverage_json_for(
    entries: &[HistoryEntry],
    limit: usize,
    days: u64,
) -> Option<String> {
    let cutoff = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        .saturating_sub(days * 86_400);
    let windowed: Vec<HistoryEntry> = entries.iter().filter(|e| e.ts >= cutoff).cloned().collect();
    if windowed.is_empty() {
        return None;
    }
    let (binaries, sub_cmds) = aggregate(&windowed);
    Some(coverage_json(&windowed, &binaries, &sub_cmds, limit))
}

pub(crate) fn print_coverage(entries: &[HistoryEntry], limit: usize, json: bool, days: u64) {
    let cutoff = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        .saturating_sub(days * 86_400);
    let windowed: Vec<HistoryEntry> = entries.iter().filter(|e| e.ts >= cutoff).cloned().collect();
    let entries: &[HistoryEntry] = &windowed;
    if entries.is_empty() {
        println!("No history yet. Run some commands through trs to start tracking.");
        return;
    }
    let (binaries, sub_cmds) = aggregate(entries);

    if json {
        print_json(entries, &binaries, &sub_cmds, limit);
        return;
    }
    print_human(entries, &binaries, &sub_cmds, limit);
}

/// Parse `(binary, subcmd)` from a command string, skipping leading
/// `NAME=value` env tokens and resolving absolute paths to their basename.
fn parse_cmd(cmd: &str) -> Option<(String, String)> {
    let mut s = cmd.trim_start();
    loop {
        let first = s.split_whitespace().next()?;
        if !looks_like_env_token(first) {
            break;
        }
        s = s[first.len()..].trim_start();
    }
    let mut parts = s.split_whitespace();
    let binary = parts.next()?.to_string();
    let binary = if let Some(name) = binary.rsplit('/').next() {
        if binary.starts_with('/') {
            name.to_string()
        } else {
            binary
        }
    } else {
        binary
    };
    let sub = parts.next().unwrap_or("").to_string();
    Some((binary, sub))
}

fn looks_like_env_token(tok: &str) -> bool {
    let Some(eq) = tok.find('=') else {
        return false;
    };
    if eq == 0 {
        return false;
    }
    let name = &tok[..eq];
    let Some(first) = name.chars().next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

type AggMap = std::collections::HashMap<String, Agg>;
type SubMap = std::collections::HashMap<(String, String), Agg>;

fn aggregate(entries: &[HistoryEntry]) -> (AggMap, SubMap) {
    let mut binaries: AggMap = AggMap::new();
    let mut sub_cmds: SubMap = SubMap::new();
    for e in entries {
        let Some((binary, sub)) = parse_cmd(&e.cmd) else {
            continue;
        };
        let b = binaries.entry(binary.clone()).or_default();
        b.count += 1;
        b.in_bytes += e.in_bytes as u64;
        if e.saved_pct < LOW_PCT {
            b.low_count += 1;
        }
        if b.sample.is_empty() && e.in_bytes > 500 && e.saved_pct < LOW_PCT {
            b.sample = sample_of(&e.cmd);
        }

        if !sub.is_empty() && !sub.starts_with('-') {
            let s = sub_cmds.entry((binary, sub)).or_default();
            s.count += 1;
            s.in_bytes += e.in_bytes as u64;
            if e.saved_pct < LOW_PCT {
                s.low_count += 1;
            }
            if s.sample.is_empty() && e.in_bytes > 500 && e.saved_pct < LOW_PCT {
                s.sample = sample_of(&e.cmd);
            }
        }
    }
    (binaries, sub_cmds)
}

/// Trim a command string to a short sample line — 70 chars max.
fn sample_of(cmd: &str) -> String {
    let s = cmd.replace(['\n', '\r'], " ");
    if s.len() <= 70 {
        s
    } else {
        format!("{}…", &s[..69])
    }
}

/// Classify into three tiers:
///   tier_a (red):    high-volume subcommands with poor compression
///   tier_b (yellow): unrecognized binaries, no parser
///   tier_c (green):  well-covered, top by volume
fn classify<'a>(
    binaries: &'a AggMap,
    sub_cmds: &'a SubMap,
    limit: usize,
) -> (
    Vec<((String, String), &'a Agg)>,
    Vec<(String, &'a Agg)>,
    Vec<((String, String), &'a Agg)>,
) {
    let mut tier_a: Vec<_> = sub_cmds
        .iter()
        .filter(|(_, a)| {
            a.count >= MIN_COUNT && a.avg_in() >= MIN_AVG_IN_BYTES && a.low_pct() >= LOW_RATIO_PCT
        })
        .map(|(k, v)| (k.clone(), v))
        .collect();
    tier_a.sort_by_key(|(_, a)| std::cmp::Reverse(a.in_bytes));
    tier_a.truncate(limit);

    let mut tier_b: Vec<_> = binaries
        .iter()
        .filter(|(name, a)| {
            !is_known_binary(name)
                && a.count >= MIN_COUNT
                && a.avg_in() >= MIN_AVG_IN_BYTES
                && a.low_pct() >= LOW_RATIO_PCT
        })
        .map(|(k, v)| (k.clone(), v))
        .collect();
    tier_b.sort_by_key(|(_, a)| std::cmp::Reverse(a.in_bytes));
    tier_b.truncate(limit);

    let mut tier_c: Vec<_> = sub_cmds
        .iter()
        .filter(|(_, a)| a.count >= MIN_COUNT && a.low_pct() < 20.0 && a.in_bytes > 100_000)
        .map(|(k, v)| (k.clone(), v))
        .collect();
    tier_c.sort_by_key(|(_, a)| std::cmp::Reverse(a.in_bytes));
    tier_c.truncate(limit);

    (tier_a, tier_b, tier_c)
}

/// Binaries trs knows how to handle (compresses output beyond ANSI
/// stripping). Sourced from the unified command registry
/// (`command_registry.rs`) — the single source of truth.
fn is_known_binary(name: &str) -> bool {
    crate::command_registry::is_known_binary(name)
}

fn print_human(entries: &[HistoryEntry], binaries: &AggMap, sub_cmds: &SubMap, limit: usize) {
    let (tier_a, tier_b, tier_c) = classify(binaries, sub_cmds, limit);
    let oldest = entries.iter().map(|e| e.ts).min().unwrap_or(0);
    let newest = entries.iter().map(|e| e.ts).max().unwrap_or(0);

    println!("trs stats: coverage analysis\n");
    println!(
        "{} entries  ·  {} → {}\n",
        entries.len(),
        fmt_date(oldest),
        fmt_date(newest)
    );

    if !tier_a.is_empty() {
        println!("Gaps: high-volume subcommands with poor compression");
        print_sub_table(&tier_a);
        println!();
    }
    if !tier_b.is_empty() {
        println!("Unrecognized binaries: no dedicated parser");
        print_bin_table(&tier_b);
        println!();
    }
    if !tier_c.is_empty() {
        println!("Well-covered (top by volume)");
        print_sub_table(&tier_c);
        println!();
    }

    println!("Share this with a parser request:");
    println!("  trs stats --coverage --json > coverage.json");
    println!(
        "  gh issue create --repo dPeluChe/trs --title 'parser-coverage report' --body-file coverage.json"
    );
}

fn print_sub_table(rows: &[((String, String), &Agg)]) {
    println!(
        "  {:<14} {:<14} {:>6} {:>9} {:>6}  sample",
        "binary", "sub", "count", "avg_in", "%low"
    );
    for ((bin, sub), a) in rows {
        println!(
            "  {:<14} {:<14} {:>6} {:>9.0} {:>5.0}%  {}",
            truncate(bin, 14),
            truncate(sub, 14),
            a.count,
            a.avg_in(),
            a.low_pct(),
            a.sample
        );
    }
}

fn print_bin_table(rows: &[(String, &Agg)]) {
    println!(
        "  {:<22} {:>6} {:>9} {:>6}  sample",
        "binary", "count", "avg_in", "%low"
    );
    for (bin, a) in rows {
        println!(
            "  {:<22} {:>6} {:>9.0} {:>5.0}%  {}",
            truncate(bin, 22),
            a.count,
            a.avg_in(),
            a.low_pct(),
            a.sample
        );
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

fn fmt_date(ts: u64) -> String {
    // Minimal YYYY-MM-DD without pulling chrono. Good enough for the
    // report header — readers don't need second-precision here.
    let secs_per_day = 86_400u64;
    let days = ts / secs_per_day;
    // 1970-01-01 + days → naive calendar conversion.
    let (y, m, d) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn days_to_ymd(mut days: u64) -> (i32, u32, u32) {
    let mut y: i32 = 1970;
    loop {
        let yd = if is_leap(y) { 366 } else { 365 };
        if days < yd {
            break;
        }
        days -= yd;
        y += 1;
    }
    let mdays = [
        31,
        if is_leap(y) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m: u32 = 1;
    for (i, &md) in mdays.iter().enumerate() {
        if days < md {
            m = i as u32 + 1;
            break;
        }
        days -= md;
    }
    (y, m, days as u32 + 1)
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Build the `--json` payload. Split from the printer so
/// `trs report coverage` can take the same string without capturing stdout.
fn coverage_json(
    entries: &[HistoryEntry],
    binaries: &AggMap,
    sub_cmds: &SubMap,
    limit: usize,
) -> String {
    let (tier_a, tier_b, tier_c) = classify(binaries, sub_cmds, limit);
    let oldest = entries.iter().map(|e| e.ts).min().unwrap_or(0);
    let newest = entries.iter().map(|e| e.ts).max().unwrap_or(0);

    let to_sub_json = |rows: &[((String, String), &Agg)]| -> Vec<serde_json::Value> {
        rows.iter()
            .map(|((bin, sub), a)| {
                serde_json::json!({
                    "binary": bin,
                    "sub": sub,
                    "count": a.count,
                    "in_bytes": a.in_bytes,
                    "avg_in": a.avg_in() as u64,
                    "low_pct": a.low_pct() as u64,
                    "sample": a.sample,
                })
            })
            .collect()
    };
    let to_bin_json = |rows: &[(String, &Agg)]| -> Vec<serde_json::Value> {
        rows.iter()
            .map(|(bin, a)| {
                serde_json::json!({
                    "binary": bin,
                    "count": a.count,
                    "in_bytes": a.in_bytes,
                    "avg_in": a.avg_in() as u64,
                    "low_pct": a.low_pct() as u64,
                    "sample": a.sample,
                })
            })
            .collect()
    };

    let out = serde_json::json!({
        "trs_version": env!("CARGO_PKG_VERSION"),
        "entries": entries.len(),
        "range": { "from_ts": oldest, "to_ts": newest },
        "gaps": to_sub_json(&tier_a),
        "unrecognized": to_bin_json(&tier_b),
        "well_covered": to_sub_json(&tier_c),
    });
    serde_json::to_string_pretty(&out).unwrap_or_default()
}

fn print_json(entries: &[HistoryEntry], binaries: &AggMap, sub_cmds: &SubMap, limit: usize) {
    println!("{}", coverage_json(entries, binaries, sub_cmds, limit));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cmd_basic() {
        assert_eq!(
            parse_cmd("git status"),
            Some(("git".into(), "status".into()))
        );
        assert_eq!(
            parse_cmd("cargo test"),
            Some(("cargo".into(), "test".into()))
        );
    }

    #[test]
    fn parse_cmd_strips_env_prefix() {
        assert_eq!(
            parse_cmd("RUSTFLAGS=-C cargo build"),
            Some(("cargo".into(), "build".into()))
        );
        assert_eq!(
            parse_cmd("TRS_AGENT=claude git status"),
            Some(("git".into(), "status".into()))
        );
    }

    #[test]
    fn parse_cmd_resolves_absolute_path() {
        assert_eq!(
            parse_cmd("/usr/bin/grep -rn foo src/"),
            Some(("grep".into(), "-rn".into()))
        );
    }

    #[test]
    fn parse_cmd_no_subcommand() {
        let r = parse_cmd("pwd");
        assert!(r.is_some());
        assert_eq!(r.unwrap().1, "");
    }

    #[test]
    fn looks_like_env_token_basic() {
        assert!(looks_like_env_token("FOO=bar"));
        assert!(looks_like_env_token("TRS_AGENT=claude"));
        assert!(!looks_like_env_token("--flag=val"));
        assert!(!looks_like_env_token("=empty"));
        assert!(!looks_like_env_token("plain"));
    }

    #[test]
    fn days_to_ymd_known_dates() {
        // 2020-01-01 = day 18262
        assert_eq!(days_to_ymd(18262), (2020, 1, 1));
        // 1970-01-01 = day 0
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
    }
}
