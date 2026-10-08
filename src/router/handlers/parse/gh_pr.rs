use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

/// Titles of conventional commits and PRs run to ~90 characters; cutting them
/// lower hides what a PR or issue is about.
const LIST_TITLE_MAX: usize = 100;

fn str_of<'a>(v: &'a serde_json::Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}

/// The one state every row has (`gh pr list` defaults to OPEN), so it can sit
/// in the header instead of repeating on each line.
fn shared_state<'a>(states: &[&'a str]) -> Option<&'a str> {
    let first = *states.first()?;
    (!first.is_empty() && states.iter().all(|s| *s == first)).then_some(first)
}

fn shared_suffix(state: Option<&str>) -> String {
    state.map_or(String::new(), |s| format!(" ({s})"))
}

/// ` [MERGED]` after the number, unless the header already says it for all rows.
fn state_tag(shared: Option<&str>, state: &str) -> String {
    if shared.is_some() || state.is_empty() {
        String::new()
    } else {
        format!(" [{state}]")
    }
}

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
                // TSV format: number\ttitle\tbranch\tstate\tdate
                let fields: Vec<&str> = trimmed.split('\t').collect();
                if fields.len() >= 2 {
                    let col = |i: usize| fields.get(i).map_or("", |s| s.trim());
                    prs.push(serde_json::json!({
                        "number": col(0), "title": col(1), "branch": col(2), "state": col(3)
                    }));
                }
            } else if trimmed.contains('#') {
                if let Some(hash_pos) = trimmed.find('#') {
                    let rest = &trimmed[hash_pos + 1..];
                    let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                    if parts.len() >= 2 {
                        let number = parts[0].trim();
                        let remainder = parts[1].trim();
                        let (title, branch) = if let Some(paren_start) = remainder.rfind('(') {
                            (
                                remainder[..paren_start].trim(),
                                remainder[paren_start + 1..].trim_end_matches(')').trim(),
                            )
                        } else {
                            (remainder, "")
                        };
                        prs.push(serde_json::json!({
                            "number": number, "title": title, "branch": branch, "state": ""
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
                    let states: Vec<&str> = prs.iter().map(|p| str_of(p, "state")).collect();
                    let shared = shared_state(&states);
                    let mut out =
                        format!("pull requests: {}{}\n", prs.len(), shared_suffix(shared));
                    for pr in &prs {
                        let branch = str_of(pr, "branch");
                        let state = str_of(pr, "state");
                        out.push_str(&format!(
                            "  #{}{} {}{}\n",
                            str_of(pr, "number"),
                            state_tag(shared, state),
                            Self::truncate_str(str_of(pr, "title"), LIST_TITLE_MAX),
                            if branch.is_empty() {
                                String::new()
                            } else {
                                format!(" ({branch})")
                            },
                        ));
                    }
                    out
                }
            }
        };
        crate::parse_out::emit(&output);
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
                // TSV format: number\tSTATE\ttitle\tlabels\tdate. Older gh left the
                // state out: number\ttitle\tlabels\tdate.
                let fields: Vec<&str> = trimmed.split('\t').collect();
                if fields.len() >= 2 {
                    let col = |i: usize| fields.get(i).map_or("", |s| s.trim());
                    let has_state = fields.len() >= 3
                        && matches!(col(1).to_ascii_uppercase().as_str(), "OPEN" | "CLOSED");
                    let (state, title, labels) = if has_state {
                        (col(1), col(2), col(3))
                    } else {
                        ("", col(1), col(2))
                    };
                    issues.push(serde_json::json!({
                        "number": col(0), "title": title, "labels": labels, "state": state
                    }));
                }
            } else if trimmed.contains('#') {
                if let Some(hash_pos) = trimmed.find('#') {
                    let rest = &trimmed[hash_pos + 1..];
                    let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                    if parts.len() >= 2 {
                        let number = parts[0].trim();
                        let title = parts[1].trim();
                        issues.push(
                            serde_json::json!({"number": number, "title": title, "state": ""}),
                        );
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
                    let states: Vec<&str> = issues.iter().map(|i| str_of(i, "state")).collect();
                    let shared = shared_state(&states);
                    let mut out = format!("issues: {}{}\n", issues.len(), shared_suffix(shared));
                    for issue in &issues {
                        let labels = str_of(issue, "labels");
                        let state = str_of(issue, "state");
                        out.push_str(&format!(
                            "  #{}{} {}{}\n",
                            str_of(issue, "number"),
                            state_tag(shared, state),
                            Self::truncate_str(str_of(issue, "title"), LIST_TITLE_MAX),
                            if labels.is_empty() {
                                String::new()
                            } else {
                                format!(" ({labels})")
                            },
                        ));
                    }
                    out
                }
            }
        };
        crate::parse_out::emit(&output);
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
    /// Keeps the non-empty metadata and the whole body: reading the
    /// description is why an agent runs `gh pr view`. Non-TTY gh separates
    /// the two with a `--` line, not a `body:` key.
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
                let blank_run = line.trim().is_empty()
                    && body_lines.last().is_none_or(|l: &String| l.is_empty());
                if !blank_run {
                    body_lines.push(line.trim_end().to_string());
                }
                continue;
            }
            if line.trim() == "--" {
                in_body = true;
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
                    "number" => {
                        fields.insert("number", v);
                    }
                    "body" => {
                        in_body = true;
                    }
                    _ if !v.is_empty() => {
                        const KEPT: [&str; 6] = [
                            "labels",
                            "reviewers",
                            "assignees",
                            "milestone",
                            "additions",
                            "deletions",
                        ];
                        if let Some(key) = KEPT.iter().find(|x| **x == k) {
                            fields.insert(key, v);
                        }
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
                let body = strip_html_comments(&body_lines.join("\n"));
                if !body.trim().is_empty() {
                    obj.insert("body".to_string(), serde_json::Value::String(body));
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
                if let (Some(a), Some(d)) = (fields.get("additions"), fields.get("deletions")) {
                    out.push_str(&format!("changes: +{} -{}\n", a, d));
                }
                for k in ["reviewers", "assignees", "milestone"] {
                    if let Some(v) = fields.get(k) {
                        out.push_str(&format!("{}: {}\n", k, v));
                    }
                }
                if !url.is_empty() {
                    out.push_str(&format!("url: {}\n", url));
                }
                let body = strip_html_comments(&body_lines.join("\n"));
                if !body.trim().is_empty() {
                    out.push_str("--\n");
                    out.push_str(body.trim());
                    out.push('\n');
                }
                out
            }
        };

        crate::parse_out::emit(&output);
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

        crate::parse_out::emit(&output);
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

/// PR templates leave `<!-- ... -->` guidance the author never deleted.
fn strip_html_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        match rest[start..].find("-->") {
            Some(end) => rest = &rest[start + end + 3..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}
