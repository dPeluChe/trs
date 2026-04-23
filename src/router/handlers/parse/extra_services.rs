use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
    /// Truncate a string to max_len chars, appending "..." if truncated.
    /// UTF-8 safe.
    fn truncate_str(s: &str, max_len: usize) -> String {
        crate::formatter::helpers::truncate(s, max_len)
    }

    /// Parse `gh pr list` output (TTY emoji format or non-TTY TSV).
    pub(crate) fn handle_gh_pr(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut prs: Vec<serde_json::Value> = Vec::new();

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Detect format: TSV (non-TTY) has tabs
            if trimmed.contains('\t') {
                // TSV format: number\ttitle\tauthor:branch\tstate\tdate
                let fields: Vec<&str> = trimmed.split('\t').collect();
                if fields.len() >= 2 {
                    let number = fields[0].trim();
                    let title = fields[1].trim();
                    // Extract just author name from "user:branch" or "dependabot/..."
                    let author = fields
                        .get(2)
                        .map(|s| {
                            let s = s.trim();
                            // "user:branch" → "user"
                            if let Some(pos) = s.find(':') {
                                &s[..pos]
                            } else if let Some(pos) = s.find('/') {
                                // "dependabot/go_modules/..." → "dependabot"
                                &s[..pos]
                            } else {
                                s
                            }
                        })
                        .unwrap_or("");
                    prs.push(serde_json::json!({
                        "number": number, "title": title, "author": author
                    }));
                }
            } else if trimmed.contains('#') {
                // TTY format: #123 title (author)
                if let Some(hash_pos) = trimmed.find('#') {
                    let rest = &trimmed[hash_pos + 1..];
                    let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                    if parts.len() >= 2 {
                        let number = parts[0].trim();
                        let remainder = parts[1].trim();
                        let (title, author) = if let Some(paren_start) = remainder.rfind('(') {
                            (
                                remainder[..paren_start].trim(),
                                remainder[paren_start + 1..].trim_end_matches(')').trim(),
                            )
                        } else {
                            (remainder, "")
                        };
                        prs.push(serde_json::json!({
                            "number": number, "title": title, "author": author
                        }));
                    }
                }
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => {
                serde_json::json!({"pull_requests": prs, "count": prs.len()}).to_string()
            }
            _ => {
                if prs.is_empty() {
                    "no open pull requests\n".to_string()
                } else {
                    let mut out = format!("pull requests: {}\n", prs.len());
                    for pr in &prs {
                        let title = Self::truncate_str(pr["title"].as_str().unwrap_or(""), 60);
                        let author = Self::truncate_str(pr["author"].as_str().unwrap_or(""), 30);
                        out.push_str(&format!(
                            "  #{} {} ({})\n",
                            pr["number"].as_str().unwrap_or(""),
                            title,
                            author
                        ));
                    }
                    out
                }
            }
        };
        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("gh-pr")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(prs.len())
                .print();
        }
        Ok(())
    }

    /// Parse `gh issue list` output (TTY or TSV).
    pub(crate) fn handle_gh_issue(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut issues: Vec<serde_json::Value> = Vec::new();

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.contains('\t') {
                // TSV format: number\ttitle\tlabels\tdate
                let fields: Vec<&str> = trimmed.split('\t').collect();
                if fields.len() >= 2 {
                    let number = fields[0].trim();
                    let title = fields[1].trim();
                    let labels = fields.get(2).map(|s| s.trim()).unwrap_or("");
                    issues.push(
                        serde_json::json!({"number": number, "title": title, "labels": labels}),
                    );
                }
            } else if trimmed.contains('#') {
                if let Some(hash_pos) = trimmed.find('#') {
                    let rest = &trimmed[hash_pos + 1..];
                    let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                    if parts.len() >= 2 {
                        let number = parts[0].trim();
                        let title = parts[1].trim();
                        issues.push(serde_json::json!({"number": number, "title": title}));
                    }
                }
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => {
                serde_json::json!({"issues": issues, "count": issues.len()}).to_string()
            }
            _ => {
                if issues.is_empty() {
                    "no open issues\n".to_string()
                } else {
                    let mut out = format!("issues: {}\n", issues.len());
                    for issue in &issues {
                        let title = Self::truncate_str(issue["title"].as_str().unwrap_or(""), 60);
                        out.push_str(&format!(
                            "  #{} {}\n",
                            issue["number"].as_str().unwrap_or(""),
                            title
                        ));
                    }
                    out
                }
            }
        };
        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("gh-issue")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(issues.len())
                .print();
        }
        Ok(())
    }

    /// Parse `gh run list` output.
    pub(crate) fn handle_gh_run(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        // Read raw input to detect status emoji markers before stripping
        let raw_input = Self::read_input_raw(file)?;
        let input = super::super::common::strip_emojis(&raw_input);
        let input_bytes = raw_input.len();
        let mut runs: Vec<serde_json::Value> = Vec::new();

        let raw_lines: Vec<&str> = raw_input.lines().collect();
        let clean_lines: Vec<&str> = input.lines().collect();

        for (i, line) in clean_lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let raw_line = raw_lines.get(i).unwrap_or(&"");

            // Detect format: TSV (non-TTY) has tabs
            if trimmed.contains('\t') {
                // TSV format: status\tconclusion\tname\tdisplay_title\tbranch\tevent\tid\telapsed\tdate
                let fields: Vec<&str> = trimmed.split('\t').collect();
                if fields.len() >= 3 {
                    let status_text = fields[0].trim().to_lowercase();
                    let conclusion = fields[1].trim().to_lowercase();
                    let name = fields[2].trim();
                    let event = fields.get(5).map(|s| s.trim()).unwrap_or("");
                    let status = if conclusion == "success" {
                        "success"
                    } else if conclusion == "failure" {
                        "failure"
                    } else if status_text == "in_progress" {
                        "in_progress"
                    } else if conclusion == "cancelled" {
                        "cancelled"
                    } else {
                        &status_text
                    };
                    runs.push(serde_json::json!({"name": name, "event": event, "status": status}));
                }
            } else {
                // TTY format: skip headers
                if trimmed.starts_with("Workflow") || trimmed.starts_with("Showing") {
                    continue;
                }

                // Parse: name [id]
                if let Some(bracket_start) = trimmed.rfind('[') {
                    let name = trimmed[..bracket_start].trim();
                    let id = trimmed[bracket_start + 1..].trim_end_matches(']').trim();

                    let status = if raw_line.contains('\u{2705}')
                        || raw_line.contains("success")
                        || raw_line.contains("completed")
                    {
                        "success"
                    } else if raw_line.contains('\u{274C}')
                        || raw_line.contains("failure")
                        || raw_line.contains("failed")
                    {
                        "failure"
                    } else if raw_line.contains("in_progress")
                        || raw_line.contains("queued")
                        || raw_line.contains('\u{1F7E1}')
                    {
                        "in_progress"
                    } else if raw_line.contains('\u{1F534}') || raw_line.contains("cancelled") {
                        "cancelled"
                    } else {
                        "unknown"
                    };

                    if !name.is_empty() {
                        runs.push(serde_json::json!({"name": name, "id": id, "status": status}));
                    }
                }
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => {
                serde_json::json!({"runs": runs, "count": runs.len()}).to_string()
            }
            _ => {
                if runs.is_empty() {
                    "no workflow runs\n".to_string()
                } else {
                    let mut out = format!("runs: {}\n", runs.len());
                    for run in &runs {
                        let marker = match run["status"].as_str().unwrap_or("") {
                            "success" => "+",
                            "failure" => "-",
                            "in_progress" => "~",
                            _ => "?",
                        };
                        let name = Self::truncate_str(run["name"].as_str().unwrap_or(""), 50);
                        let event = run["event"].as_str().unwrap_or("");
                        if !event.is_empty() {
                            out.push_str(&format!("  {} {} ({})\n", marker, name, event));
                        } else {
                            out.push_str(&format!("  {} {}\n", marker, name));
                        }
                    }
                    out
                }
            }
        };
        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("gh-run")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(runs.len())
                .print();
        }
        Ok(())
    }

    /// Parse `gh pr view N` output.
    /// Keeps title, state, author, url, labels, and first 3 lines of body.
    pub(crate) fn handle_gh_pr_view(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        let mut fields: std::collections::HashMap<&str, String> =
            std::collections::HashMap::new();
        let mut body_lines: Vec<String> = Vec::new();
        let mut in_body = false;

        for line in input.lines() {
            if in_body {
                let trimmed = line.trim();
                if !trimmed.is_empty() && body_lines.len() < 3 {
                    body_lines.push(trimmed.to_string());
                }
                continue;
            }
            // Key:  value lines
            if let Some((key, val)) = line.split_once(':') {
                let k = key.trim().to_lowercase();
                let v = val.trim().to_string();
                match k.as_str() {
                    "title" => { fields.insert("title", v); }
                    "state" => { fields.insert("state", v); }
                    "author" => { fields.insert("author", v); }
                    "url" => { fields.insert("url", v); }
                    "labels" if !v.is_empty() => { fields.insert("labels", v); }
                    "number" => { fields.insert("number", v); }
                    "body" => { in_body = true; }
                    _ => {}
                }
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => {
                let mut obj = serde_json::Map::new();
                for (k, v) in &fields {
                    obj.insert(k.to_string(), serde_json::Value::String(v.clone()));
                }
                if !body_lines.is_empty() {
                    obj.insert(
                        "body_preview".to_string(),
                        serde_json::Value::String(body_lines.join(" ")),
                    );
                }
                serde_json::Value::Object(obj).to_string()
            }
            _ => {
                let title = fields.get("title").map(|s| s.as_str()).unwrap_or("?");
                let state = fields.get("state").map(|s| s.as_str()).unwrap_or("?");
                let author = fields.get("author").map(|s| s.as_str()).unwrap_or("?");
                let number = fields.get("number").map(|s| s.as_str()).unwrap_or("");
                let url = fields.get("url").map(|s| s.as_str()).unwrap_or("");
                let labels = fields.get("labels").map(|s| s.as_str()).unwrap_or("");

                let mut out = format!("PR #{}: {} [{}] by {}\n", number, title, state, author);
                if !labels.is_empty() {
                    out.push_str(&format!("labels: {}\n", labels));
                }
                if !url.is_empty() {
                    out.push_str(&format!("url: {}\n", url));
                }
                if !body_lines.is_empty() {
                    out.push_str(&format!("body: {}\n", body_lines.join(" | ")));
                }
                out
            }
        };

        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("gh-pr-view")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }
}
