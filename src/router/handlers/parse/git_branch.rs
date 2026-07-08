use std::collections::BTreeMap;

use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

/// Branches-per-prefix threshold above which we collapse to `prefix/* (+N more)`.
/// 3 is the smallest count where listing is clearly worse than a summary.
const COLLAPSE_THRESHOLD: usize = 3;

/// Always show at most this many individual names inside a collapsed group
/// before switching to "+N more" — keeps output bounded on repos with huge
/// branch lists.
const NAMES_SHOWN_PER_GROUP: usize = 2;

impl ParseHandler {
    pub(crate) fn handle_git_branch(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut current = String::new();
        let mut local: Vec<String> = Vec::new();
        let mut remote: Vec<String> = Vec::new();

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.contains("->") {
                continue;
            }
            let is_current = trimmed.starts_with('*');
            let name = trimmed.trim_start_matches("* ").trim().to_string();
            if is_current {
                current = name.clone();
            }
            if name.starts_with("remotes/") || name.starts_with("origin/") {
                remote.push(name.trim_start_matches("remotes/").to_string());
            } else {
                local.push(name);
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({
                "current": current,
                "local": local,
                "remote": remote,
                "local_count": local.len(),
                "remote_count": remote.len(),
            })
            .to_string(),
            _ => render_compact(&current, &local, &remote),
        };
        crate::parse_out::emit(&output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("git-branch")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }
}

/// Render a compact text view by grouping branches under their common prefix
/// and collapsing groups larger than `COLLAPSE_THRESHOLD`. Remote branches
/// that only shadow a local are dropped entirely.
fn render_compact(current: &str, local: &[String], remote: &[String]) -> String {
    let mut out = String::new();

    // Drop remote branches whose tail name matches any local branch — the
    // local ref is already tracked, showing both is pure noise.
    let remote_unique: Vec<&String> = remote
        .iter()
        .filter(|r| {
            let short = r.split('/').next_back().unwrap_or(r);
            !local.iter().any(|l| l == short)
        })
        .collect();

    out.push_str(&format!("* {}\n", current));

    let other_local: Vec<&String> = local.iter().filter(|b| *b != current).collect();
    if !other_local.is_empty() {
        render_group(&mut out, &other_local, "");
    }
    if !remote_unique.is_empty() {
        render_group(&mut out, &remote_unique, "");
    }
    out
}

/// Group a flat branch list by its first path segment and emit either the
/// full names (for small groups) or a `prefix/* (k/N)` summary (for large ones).
fn render_group(out: &mut String, branches: &[&String], indent: &str) {
    let mut groups: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    let mut bare: Vec<&str> = Vec::new(); // no slash → can't group
    for b in branches {
        match b.split_once('/') {
            Some((prefix, rest)) => groups.entry(prefix.to_string()).or_default().push(rest),
            None => bare.push(b.as_str()),
        }
    }

    // Emit bare names first — they're usually main branches worth surfacing.
    for name in &bare {
        out.push_str(&format!("{}  {}\n", indent, name));
    }

    for (prefix, children) in &groups {
        if children.len() < COLLAPSE_THRESHOLD {
            for c in children {
                out.push_str(&format!("{}  {}/{}\n", indent, prefix, c));
            }
            continue;
        }
        // Collapse: show first NAMES_SHOWN_PER_GROUP tails + count of the rest.
        let shown: Vec<&&str> = children.iter().take(NAMES_SHOWN_PER_GROUP).collect();
        let hidden = children.len() - shown.len();
        let tail_list: Vec<String> = shown.iter().map(|s| s.to_string()).collect();
        out.push_str(&format!(
            "{}  {}/{{{}}} (+{} more)\n",
            indent,
            prefix,
            tail_list.join(", "),
            hidden,
        ));
    }
}
