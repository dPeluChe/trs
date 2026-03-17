use super::super::common::{CommandContext, CommandResult, CommandStats};
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
    pub(crate) fn handle_git_log(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut commits: Vec<(String, String, String, String)> = Vec::new();
        let mut hash = String::new();
        let mut author = String::new();
        let mut date = String::new();
        let mut msg: Vec<String> = Vec::new();
        let mut in_commit = false;

        // Detect format: --oneline (hash message) vs full (commit hash\nAuthor:\nDate:\n\nmessage)
        let is_oneline = !input.contains("Author: ") && !input.contains("commit ");

        if is_oneline {
            // Parse --oneline format: "abc1234 commit message here"
            for line in input.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                if let Some(space_pos) = trimmed.find(' ') {
                    let h = &trimmed[..space_pos];
                    let m = &trimmed[space_pos+1..];
                    commits.push((h.to_string(), String::new(), String::new(), m.to_string()));
                }
            }
        } else {
            for line in input.lines() {
                if let Some(h) = line.strip_prefix("commit ") {
                    if in_commit {
                        let full_msg = msg.join(" ").trim().to_string();
                        // Only take the first line of the message (subject)
                        let subject = full_msg.lines().next().unwrap_or("").to_string();
                        let subject = if subject.len() > 72 { format!("{}...", &subject[..69]) } else { subject };
                        commits.push((hash.clone(), date.clone(), author.clone(), subject));
                        msg.clear();
                    }
                    hash = h.chars().take(7).collect();
                    in_commit = true;
                } else if let Some(a) = line.strip_prefix("Author: ") {
                    author = a.split('<').next().unwrap_or(a).trim().to_string();
                } else if let Some(d) = line.strip_prefix("Date:") {
                    date = d.trim().chars().take(16).collect();
                } else if in_commit && !line.trim().is_empty() {
                    msg.push(line.trim().to_string());
                }
            }
            if in_commit {
                let full_msg = msg.join(" ").trim().to_string();
                let subject = full_msg.lines().next().unwrap_or("").to_string();
                let subject = if subject.len() > 72 { format!("{}...", &subject[..69]) } else { subject };
                commits.push((hash, date, author, subject));
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => {
                let jc: Vec<serde_json::Value> = commits.iter().map(|(h,d,a,m)| serde_json::json!({"hash":h,"date":d,"author":a,"message":m})).collect();
                serde_json::json!({"commits": jc, "count": commits.len()}).to_string()
            }
            _ => {
                let mut out = format!("commits: {}\n", commits.len());
                for (h, d, a, m) in &commits { out.push_str(&format!("  {} {} ({}) {}\n", h, d, a, m)); }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("git-log").with_output_mode(ctx.format).with_input_bytes(input_bytes).with_output_bytes(output.len()).with_items_processed(commits.len()).print(); }
        Ok(())
    }
}
