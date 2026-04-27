use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
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

            if trimmed.contains('\t') {
                // TSV format: number\ttitle\tauthor:branch\tstate\tdate
                let fields: Vec<&str> = trimmed.split('\t').collect();
                if fields.len() >= 2 {
                    let number = fields[0].trim();
                    let title = fields[1].trim();
                    let author = fields
                        .get(2)
                        .map(|s| {
                            let s = s.trim();
                            if let Some(pos) = s.find(':') {
                                &s[..pos]
                            } else if let Some(pos) = s.find('/') {
                                &s[..pos]
                            } else {
                                s
                            }
                        })
                        .unwrap_or("");
                    prs.push(
                        serde_json::json!({"number": number, "title": title, "author": author}),
                    );
                }
            } else if trimmed.contains('#') {
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
                        prs.push(
                            serde_json::json!({"number": number, "title": title, "author": author}),
                        );
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

    /// Parse `gh pr view N` output.
    /// Keeps title, state, author, url, labels, and first 3 lines of body.
    pub(crate) fn handle_gh_pr_view(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        let mut fields: std::collections::HashMap<&str, String> = std::collections::HashMap::new();
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
            if let Some((key, val)) = line.split_once(':') {
                let k = key.trim().to_lowercase();
                let v = val.trim().to_string();
                match k.as_str() {
                    "title" => {
                        fields.insert("title", v);
                    }
                    "state" => {
                        fields.insert("state", v);
                    }
                    "author" => {
                        fields.insert("author", v);
                    }
                    "url" => {
                        fields.insert("url", v);
                    }
                    "labels" if !v.is_empty() => {
                        fields.insert("labels", v);
                    }
                    "number" => {
                        fields.insert("number", v);
                    }
                    "body" => {
                        in_body = true;
                    }
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

    /// Parse `gh pr checks <pr>` output (TTY emoji or TSV).
    /// Keeps check name, status, and duration; summarises pass/fail counts.
    pub(crate) fn handle_gh_pr_checks(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let raw_input = Self::read_input_raw(file)?;
        let input = super::super::common::strip_emojis(&raw_input);
        let input_bytes = raw_input.len();

        #[derive(serde::Serialize)]
        struct Check {
            name: String,
            status: String,
            duration: String,
        }

        let mut checks: Vec<Check> = Vec::new();

        for (raw_line, clean_line) in raw_input.lines().zip(input.lines()) {
            let trimmed = clean_line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with("All checks")
                || trimmed.starts_with("Some checks")
                || trimmed.starts_with("No checks")
            {
                continue;
            }

            if trimmed.contains('\t') {
                // TSV: name\tstatus\tduration\turl
                let f: Vec<&str> = trimmed.splitn(4, '\t').collect();
                if f.len() >= 2 {
                    checks.push(Check {
                        name: f[0].trim().to_string(),
                        status: f[1].trim().to_string(),
                        duration: f.get(2).map(|s| s.trim()).unwrap_or("").to_string(),
                    });
                }
            } else {
                let status = if raw_line.contains('\u{2705}') || raw_line.contains("pass") {
                    "pass"
                } else if raw_line.contains('\u{274C}')
                    || raw_line.contains("fail")
                    || raw_line.contains("error")
                {
                    "fail"
                } else if raw_line.contains('\u{1F7E1}')
                    || raw_line.contains("pending")
                    || raw_line.contains("queued")
                {
                    "pending"
                } else {
                    continue;
                };

                let name = trimmed
                    .trim_start_matches(|c: char| !c.is_alphanumeric())
                    .split("  ")
                    .next()
                    .unwrap_or(trimmed)
                    .trim()
                    .to_string();

                let duration = trimmed
                    .split_whitespace()
                    .find(|t| t.ends_with('s') && t.chars().any(|c| c.is_ascii_digit()))
                    .unwrap_or("")
                    .to_string();

                if !name.is_empty() {
                    checks.push(Check {
                        name,
                        status: status.to_string(),
                        duration,
                    });
                }
            }
        }

        let pass = checks.iter().filter(|c| c.status == "pass").count();
        let fail = checks.iter().filter(|c| c.status == "fail").count();
        let pending = checks.iter().filter(|c| c.status == "pending").count();

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({
                "checks": checks,
                "summary": {"pass": pass, "fail": fail, "pending": pending},
            })
            .to_string(),
            _ => {
                let mut out = format!("checks: {} pass, {} fail", pass, fail);
                if pending > 0 {
                    out.push_str(&format!(", {} pending", pending));
                }
                out.push('\n');
                for c in checks.iter().filter(|c| c.status != "pass") {
                    let dur = if c.duration.is_empty() {
                        String::new()
                    } else {
                        format!(", {}", c.duration)
                    };
                    out.push_str(&format!(
                        "  {} {} ({}{})\n",
                        if c.status == "fail" { "✗" } else { "~" },
                        Self::truncate_str(&c.name, 60),
                        c.status,
                        dur
                    ));
                }
                out
            }
        };

        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("gh-pr-checks")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(checks.len())
                .print();
        }
        Ok(())
    }
}
