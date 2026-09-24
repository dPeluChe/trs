use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

/// Truncate `subject` if `max` is set and leaves room for at least one real
/// char plus the ellipsis (`max >= 4`). `None` / `Some(n<4)` is a no-op.
fn apply_truncate(subject: &str, max: Option<usize>) -> String {
    match max {
        Some(n) if n >= 4 => crate::formatter::helpers::truncate(subject, n),
        _ => subject.to_string(),
    }
}

/// `subject · 3 days ago`: the time rides on the subject line.
fn with_time(block: &str, time: &str) -> String {
    match (time.is_empty(), block.split_once('\n')) {
        (true, _) => block.to_string(),
        (false, Some((subject, rest))) => format!("{} · {}\n{}", subject, time, rest),
        (false, None) => format!("{} · {}", block, time),
    }
}

/// Trailers that record process, not content.
const NOISE_TRAILERS: &[&str] = &[
    "Signed-off-by:",
    "Co-authored-by:",
    "Co-Authored-By:",
    "Claude-Session:",
    "Change-Id:",
    "Reviewed-on:",
];

/// What an agent can use from a commit body without reading all of it: the
/// opening paragraph (the "why", two lines at most), a squash merge's list of
/// squashed subjects, any line naming a
/// breaking change, an issue it closes, or a revert, and how many lines were
/// left out along with the command that shows them.
fn condense_body(msg: &[String], hash: &str) -> Vec<String> {
    let body: Vec<&str> = msg
        .iter()
        .skip_while(|l| l.is_empty())
        .skip(1)
        .map(String::as_str)
        .filter(|l| !NOISE_TRAILERS.iter().any(|t| l.starts_with(t)))
        .skip_while(|l| l.is_empty())
        .collect();
    let opening: Vec<&str> = body
        .iter()
        .copied()
        .take_while(|l| !l.is_empty())
        .take(2)
        .collect();
    let signal = |l: &&&str| {
        let lower = l.to_ascii_lowercase();
        l.contains("BREAKING")
            || lower.starts_with("this reverts commit")
            || [
                "fixes #",
                "fixed #",
                "closes #",
                "closed #",
                "resolves #",
                "refs #",
            ]
            .iter()
            .any(|k| lower.contains(k))
    };
    // A squash merge lists each squashed commit as `* subject`: the index of
    // what went in.
    let squashed = |l: &&&str| l.starts_with("* ") && l.contains(": ");
    let mut kept: Vec<String> = opening
        .iter()
        .chain(body.iter().skip(opening.len()).filter(squashed).take(5))
        .chain(body.iter().skip(opening.len()).filter(signal).take(3))
        .map(|l| crate::formatter::helpers::truncate(l, 160))
        .collect();
    // Files the omitted text names: cheap, and it answers "which commit
    // touched X" without opening each one.
    let shown = kept.join(" ");
    let mut files: Vec<&str> = Vec::new();
    for word in body
        .iter()
        .flat_map(|l| l.split(|c: char| c.is_whitespace() || "`'\"(),".contains(c)))
    {
        let word = word.trim_end_matches([':', '.', ';']);
        let looks_like_file = word.rsplit_once('.').is_some_and(|(stem, ext)| {
            stem.len() >= 2
                && (1..=5).contains(&ext.len())
                && ext.chars().all(|c| c.is_ascii_alphanumeric())
                && ext.chars().any(|c| c.is_ascii_alphabetic())
        }) && !word.contains("://")
            && !word.starts_with("//");
        if looks_like_file && !files.contains(&word) && !shown.contains(word) && files.len() < 8 {
            files.push(word);
        }
    }
    if !files.is_empty() {
        kept.push(format!("files: {}", files.join(", ")));
    }
    let content = body.iter().filter(|l| !l.is_empty()).count();
    if content + 1 > kept.len() {
        let omitted = content.saturating_sub(kept.len() - usize::from(!files.is_empty()));
        kept.push(format!("[+{} lines: git show {}]", omitted, hash));
    }
    kept
}

impl ParseHandler {
    pub(crate) fn handle_git_log(
        file: &Option<std::path::PathBuf>,
        truncate: Option<usize>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        // hash, date, author, subject, condensed body
        let mut commits: Vec<(String, String, String, String, Vec<String>)> = Vec::new();
        let mut hash = String::new();
        let mut author = String::new();
        let mut date = String::new();
        let mut msg: Vec<String> = Vec::new();
        let mut in_commit = false;

        let is_oneline = !input.contains("Author: ") && !input.contains("commit ");

        if is_oneline {
            for line in input.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Some(space_pos) = trimmed.find(' ') {
                    let h = &trimmed[..space_pos];
                    let m = apply_truncate(&trimmed[space_pos + 1..], truncate);
                    commits.push((h.to_string(), String::new(), String::new(), m, Vec::new()));
                }
            }
        } else {
            for line in input.lines() {
                if let Some(h) = line.strip_prefix("commit ") {
                    if in_commit {
                        let subject = apply_truncate(&Self::extract_subject(&msg), truncate);
                        let body = condense_body(&msg, &hash);
                        commits.push((hash.clone(), date.clone(), author.clone(), subject, body));
                        msg.clear();
                    }
                    hash = h.chars().take(7).collect();
                    in_commit = true;
                } else if let Some(a) = line.strip_prefix("Author: ") {
                    // Extract first name only (before space or <)
                    let full_name = a.split('<').next().unwrap_or(a).trim();
                    author = full_name
                        .split_whitespace()
                        .next()
                        .unwrap_or(full_name)
                        .to_string();
                } else if let Some(d) = line.strip_prefix("Date:") {
                    date = d.trim().to_string();
                } else if in_commit && (!line.trim().is_empty() || !msg.is_empty()) {
                    msg.push(line.trim().to_string());
                }
            }
            if in_commit {
                let subject = apply_truncate(&Self::extract_subject(&msg), truncate);
                let body = condense_body(&msg, &hash);
                commits.push((hash, date, author, subject, body));
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => {
                let jc: Vec<serde_json::Value> = commits.iter().map(|(h,d,a,m,b)| {
                    serde_json::json!({"hash": h, "date": d, "author": a, "message": m, "body": b})
                }).collect();
                serde_json::json!({"commits": jc, "count": commits.len()}).to_string()
            }
            _ => {
                // Group commits by author (preserving order of first appearance)
                let mut out = String::new();
                let mut groups: Vec<(String, Vec<(String, String, String)>)> = Vec::new(); // (author, [(hash, time, msg)])

                for (h, d, _a, m, body) in &commits {
                    let time = if d.is_empty() {
                        String::new()
                    } else {
                        Self::relative_time(d)
                    };
                    let author = _a.clone();
                    let m = std::iter::once(m.clone())
                        .chain(body.iter().map(|l| format!("  {}", l)))
                        .collect::<Vec<_>>()
                        .join("\n");
                    // Find or create group
                    if let Some(group) = groups.iter_mut().find(|(a, _)| a == &author) {
                        group.1.push((h.clone(), time, m.clone()));
                    } else {
                        groups.push((author, vec![(h.clone(), time, m.clone())]));
                    }
                }

                // Single author: flat list (no header)
                // Multiple authors: grouped with author label
                if groups.len() == 1 {
                    let (_author, items) = &groups[0];
                    for (h, time, m) in items {
                        out.push_str(&format!("{} {}\n", h, with_time(m, time)));
                    }
                } else {
                    for (author, items) in &groups {
                        let label = if author.is_empty() { "?" } else { author };
                        out.push_str(&format!("[{}]\n", label));
                        for (h, time, m) in items {
                            out.push_str(&format!("{} {}\n", h, with_time(m, time)));
                        }
                    }
                }
                out
            }
        };
        crate::parse_out::emit(&output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("git-log")
                .with_output_mode(ctx.format)
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(commits.len())
                .print();
        }
        Ok(())
    }

    /// The first line. Joining the message with spaces first left no line
    /// break to stop at, so the "subject" was the whole body on one line.
    fn extract_subject(msg: &[String]) -> String {
        msg.iter()
            .find(|l| !l.is_empty())
            .cloned()
            .unwrap_or_default()
    }

    /// Convert a git date string to relative time like "5 min ago", "2 hrs ago", "3 days ago"
    fn relative_time(date_str: &str) -> String {
        // Git date format: "Tue Mar 17 12:09:30 2026 -0600"
        let parts: Vec<&str> = date_str.split_whitespace().collect();
        if parts.len() < 5 {
            return date_str.to_string();
        }

        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let month = months
            .iter()
            .position(|&m| m == parts[1])
            .map(|i| i as u32 + 1);
        let day: Option<u32> = parts[2].parse().ok();
        let time_parts: Vec<&str> = parts[3].split(':').collect();
        let hour: Option<u32> = time_parts.first().and_then(|h| h.parse().ok());
        let minute: Option<u32> = time_parts.get(1).and_then(|m| m.parse().ok());
        let second: Option<u32> = time_parts.get(2).and_then(|s| s.parse().ok());
        let year: Option<i32> = parts[4].parse().ok();

        if let (Some(m), Some(d), Some(h), Some(min), Some(sec), Some(y)) =
            (month, day, hour, minute, second, year)
        {
            // Proper epoch calculation with cumulative days per month
            let month_days: [u64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
            let mut days: u64 = 0;
            for yr in 1970..y as u64 {
                days += if yr % 4 == 0 && (yr % 100 != 0 || yr % 400 == 0) {
                    366
                } else {
                    365
                };
            }
            days += month_days.get((m - 1) as usize).copied().unwrap_or(0);
            // Leap day adjustment for current year
            if m > 2 {
                let yr = y as u64;
                if yr % 4 == 0 && (yr % 100 != 0 || yr % 400 == 0) {
                    days += 1;
                }
            }
            days += d as u64 - 1;
            let commit_epoch = days * 86400 + h as u64 * 3600 + min as u64 * 60 + sec as u64;

            // Adjust for timezone offset if present (e.g., "-0600")
            let tz_offset_secs: i64 = if let Some(tz) = parts.get(5) {
                let sign = if tz.starts_with('-') { -1i64 } else { 1i64 };
                let tz_num = tz.trim_start_matches(|c: char| c == '+' || c == '-');
                if let Ok(n) = tz_num.parse::<i64>() {
                    sign * ((n / 100) * 3600 + (n % 100) * 60)
                } else {
                    0
                }
            } else {
                0
            };
            let commit_utc = (commit_epoch as i64 - tz_offset_secs) as u64;

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            if now >= commit_utc {
                let diff = now - commit_utc;
                if diff < 60 {
                    return "just now".to_string();
                }
                if diff < 3600 {
                    return format!("{} min ago", diff / 60);
                }
                if diff < 86400 {
                    return format!("{} hrs ago", diff / 3600);
                }
                if diff < 604800 {
                    return format!("{} days ago", diff / 86400);
                }
                if diff < 2592000 {
                    return format!("{} weeks ago", diff / 604800);
                }
                return format!("{} months ago", diff / 2592000);
            }
        }
        // Fallback
        if parts.len() >= 3 {
            format!("{} {}", parts[1], parts[2])
        } else {
            date_str.to_string()
        }
    }
}
