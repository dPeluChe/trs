use super::super::common::{CommandContext, CommandResult, CommandStats};
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
    pub(crate) fn handle_git_branch(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut current = String::new();
        let mut local: Vec<String> = Vec::new();
        let mut remote: Vec<String> = Vec::new();

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.contains("->") { continue; }
            let is_current = trimmed.starts_with('*');
            let name = trimmed.trim_start_matches("* ").trim().to_string();
            if is_current { current = name.clone(); }
            if name.starts_with("remotes/") || name.starts_with("origin/") {
                remote.push(name.trim_start_matches("remotes/").to_string());
            } else { local.push(name); }
        }

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({"current": current, "local": local, "remote": remote, "local_count": local.len(), "remote_count": remote.len()}).to_string(),
            _ => {
                let mut out = String::new();
                // Filter out remote branches that duplicate local ones
                let unique_remote: Vec<&String> = remote.iter().filter(|r| {
                    let short = r.split('/').last().unwrap_or(r);
                    !local.iter().any(|l| l == short)
                }).collect();
                // Minimal: if only one local branch and no unique remotes, just show current
                let other_local: Vec<&String> = local.iter().filter(|b| *b != &current).collect();
                out.push_str(&format!("* {}\n", current));
                if !other_local.is_empty() {
                    for b in &other_local { out.push_str(&format!("  {}\n", b)); }
                }
                if !unique_remote.is_empty() {
                    for b in &unique_remote { out.push_str(&format!("  {}\n", b)); }
                }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("git-branch").with_input_bytes(input_bytes).with_output_bytes(output.len()).print(); }
        Ok(())
    }
}
