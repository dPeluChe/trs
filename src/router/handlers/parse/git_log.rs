use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

/// Truncate `subject` if `max` is set and the subject exceeds it. Appends "..."
/// after `max - 3` chars so the total stays within the budget. Returns the
/// subject unchanged when `max` is `None` or `Some(0)` (no-op).
fn apply_truncate(subject: &str, max: Option<usize>) -> String {
    match max {
        Some(n) if n >= 4 && subject.len() > n => {
            let cut = n - 3;
            // Walk back to a char boundary so multi-byte chars aren't split.
            let mut end = cut;
            while end > 0 && !subject.is_char_boundary(end) {
                end -= 1;
            }
            format!("{}...", &subject[..end])
        }
        _ => subject.to_string(),
    }
}

impl ParseHandler {
    pub(crate) fn handle_git_log(
        file: &Option<std::path::PathBuf>,
        truncate: Option<usize>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut commits: Vec<(String, String, String, String)> = Vec::new(); // hash, date, author, message
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
                    commits.push((h.to_string(), String::new(), String::new(), m));
                }
            }
        } else {
            for line in input.lines() {
                if let Some(h) = line.strip_prefix("commit ") {
                    if in_commit {
                        let subject = apply_truncate(&Self::extract_subject(&msg), truncate);
                        commits.push((hash.clone(), date.clone(), author.clone(), subject));
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
                } else if in_commit && !line.trim().is_empty() {
                    msg.push(line.trim().to_string());
                }
            }
            if in_commit {
                let subject = apply_truncate(&Self::extract_subject(&msg), truncate);
                commits.push((hash, date, author, subject));
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => {
                let jc: Vec<serde_json::Value> = commits.iter().map(|(h,d,a,m)| {
                    serde_json::json!({"hash": h, "date": d, "author": a, "message": m})
                }).collect();
                serde_json::json!({"commits": jc, "count": commits.len()}).to_string()
            }
            _ => {
                // Group commits by author (preserving order of first appearance)
                let mut out = String::new();
                let mut groups: Vec<(String, Vec<(String, String, String)>)> = Vec::new(); // (author, [(hash, time, msg)])

                for (h, d, _a, m) in &commits {
                    let time = if d.is_empty() {
                        String::new()
                    } else {
                        Self::relative_time(d)
                    };
                    let author = _a.clone();
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
                    for (h, _time, m) in items {
                        out.push_str(&format!("{} {}\n", h, m));
                    }
                } else {
                    for (author, items) in &groups {
                        let label = if author.is_empty() { "?" } else { author };
                        out.push_str(&format!("[{}]\n", label));
                        for (h, _time, m) in items {
                            out.push_str(&format!("{} {}\n", h, m));
                        }
                    }
                }
                out
            }
        };
        print!("{}", output);
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

    fn extract_subject(msg: &[String]) -> String {
        let full = msg.join(" ");
        full.lines().next().unwrap_or("").trim().to_string()
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
