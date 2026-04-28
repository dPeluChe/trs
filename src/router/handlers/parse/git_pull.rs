use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
    /// Parse `git pull` / `git fetch` output.
    /// Strips remote progress noise; keeps branch update lines and the
    /// file-change summary.
    pub(crate) fn handle_git_pull(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        // "Already up to date." — nothing to strip, pass through as-is.
        let trimmed_input = input.trim();
        if trimmed_input.eq_ignore_ascii_case("already up to date.")
            || trimmed_input.eq_ignore_ascii_case("already up-to-date.")
        {
            let output = "already up to date\n".to_string();
            print!("{}", output);
            if ctx.stats {
                CommandStats::new()
                    .with_reducer("git-pull")
                    .with_input_bytes(input_bytes)
                    .with_output_bytes(output.len())
                    .print();
            }
            return Ok(());
        }

        let mut from_url = String::new();
        let mut branch_updates: Vec<String> = Vec::new();
        let mut summary_lines: Vec<String> = Vec::new();
        let mut in_summary = false;

        for line in input.lines() {
            let trimmed = line.trim();

            // Strip remote progress noise
            if trimmed.starts_with("remote:")
                || trimmed.starts_with("Unpacking objects:")
                || trimmed.starts_with("Counting objects:")
                || trimmed.starts_with("Compressing objects:")
                || trimmed.starts_with("Receiving objects:")
                || trimmed.starts_with("Resolving deltas:")
            {
                continue;
            }

            // "From <url>" — keep once
            if trimmed.starts_with("From ") && from_url.is_empty() {
                from_url = trimmed.trim_start_matches("From ").trim().to_string();
                continue;
            }

            // Branch update lines: "   abc..def  branch -> origin/branch"
            // or "   [new branch]  name -> origin/name"
            if (trimmed.contains("->") || trimmed.starts_with('['))
                && !trimmed.starts_with("Updating")
                && !trimmed.starts_with("Fast-forward")
                && !trimmed.starts_with("Merge")
            {
                let clean = trimmed
                    .trim_start_matches(|c: char| c == ' ' || c == '*' || c == '+' || c == '-');
                if !clean.is_empty() {
                    branch_updates.push(clean.to_string());
                }
                continue;
            }

            // Diff-stat section starts after "Fast-forward" or "Merge made by"
            if trimmed.starts_with("Fast-forward") || trimmed.starts_with("Merge made by") {
                in_summary = true;
                continue;
            }

            // "Updating abc..def" — skip, info is in branch updates
            if trimmed.starts_with("Updating ") {
                continue;
            }

            if in_summary && !trimmed.is_empty() {
                summary_lines.push(trimmed.to_string());
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => {
                let summary = summary_lines.last().cloned().unwrap_or_default();
                serde_json::json!({
                    "from": from_url,
                    "branches": branch_updates,
                    "summary": summary,
                })
                .to_string()
                    + "\n"
            }
            _ => {
                let mut out = String::new();
                if !from_url.is_empty() {
                    out.push_str(&format!("from: {}\n", from_url));
                }
                for b in &branch_updates {
                    out.push_str(&format!("  {}\n", b));
                }
                for s in &summary_lines {
                    out.push_str(&format!("  {}\n", s));
                }
                if out.is_empty() {
                    // fetch with nothing new or unrecognised format — compact passthrough
                    for line in input.lines() {
                        let t = line.trim();
                        if !t.is_empty() && !t.starts_with("remote:") {
                            out.push_str(t);
                            out.push('\n');
                        }
                    }
                }
                if out.is_empty() {
                    out.push_str("no changes\n");
                }
                out
            }
        };

        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("git-pull")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }
}
