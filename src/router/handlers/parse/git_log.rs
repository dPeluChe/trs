use super::super::common::{CommandContext, CommandResult, CommandStats};
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
    pub(crate) fn handle_git_log(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut commits: Vec<(String, String, String, String)> = Vec::new(); // hash, date, author, message
        let mut hash = String::new();
        let mut author = String::new();
        let mut date = String::new();
        let mut msg: Vec<String> = Vec::new();
        let mut in_commit = false;

        // Detect format: --oneline vs full
        let is_oneline = !input.contains("Author: ") && !input.contains("commit ");

        if is_oneline {
            for line in input.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                if let Some(space_pos) = trimmed.find(' ') {
                    let h = &trimmed[..space_pos];
                    let m = &trimmed[space_pos+1..];
                    let m = if m.len() > 60 { format!("{}...", &m[..57]) } else { m.to_string() };
                    commits.push((h.to_string(), String::new(), String::new(), m));
                }
            }
        } else {
            for line in input.lines() {
                if let Some(h) = line.strip_prefix("commit ") {
                    if in_commit {
                        let subject = Self::extract_subject(&msg);
                        commits.push((hash.clone(), date.clone(), author.clone(), subject));
                        msg.clear();
                    }
                    hash = h.chars().take(7).collect();
                    in_commit = true;
                } else if let Some(a) = line.strip_prefix("Author: ") {
                    author = a.split('<').next().unwrap_or(a).trim().to_string();
                } else if let Some(d) = line.strip_prefix("Date:") {
                    date = d.trim().to_string();
                } else if in_commit && !line.trim().is_empty() {
                    msg.push(line.trim().to_string());
                }
            }
            if in_commit {
                let subject = Self::extract_subject(&msg);
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
                // Compact: hash message (no author, no date — minimal)
                let mut out = String::new();
                for (h, _d, _a, m) in &commits {
                    out.push_str(&format!("{} {}\n", h, m));
                }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("git-log").with_output_mode(ctx.format).with_input_bytes(input_bytes).with_output_bytes(output.len()).with_items_processed(commits.len()).print(); }
        Ok(())
    }

    fn extract_subject(msg: &[String]) -> String {
        let full = msg.join(" ");
        let subject = full.lines().next().unwrap_or("").trim().to_string();
        if subject.len() > 60 { format!("{}...", &subject[..57]) } else { subject }
    }
}
