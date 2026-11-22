use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

#[derive(Clone, Copy)]
pub(crate) enum ViewKind {
    Pr,
    Issue,
}

impl ViewKind {
    fn label(self) -> &'static str {
        match self {
            Self::Pr => "PR",
            Self::Issue => "Issue",
        }
    }

    fn reducer(self) -> &'static str {
        match self {
            Self::Pr => "gh-pr-view",
            Self::Issue => "gh-issue-view",
        }
    }
}

/// Printed in this order when non-empty; `gh` prints the rest of its
/// header (empty sub-issues, projects, ...) on every view.
const KEPT: [&str; 14] = [
    "labels",
    "additions",
    "deletions",
    "reviewers",
    "assignees",
    "milestone",
    "projects",
    "comments",
    "issue-type",
    "parent",
    "sub-issues",
    "sub-issues-completed",
    "blocked-by",
    "blocking",
];

struct Comment {
    meta: Vec<(String, String)>,
    body: String,
}

impl Comment {
    fn get(&self, key: &str) -> &str {
        self.meta
            .iter()
            .find(|(k, _)| k == key)
            .map_or("", |(_, v)| v.as_str())
    }

    fn heading(&self) -> String {
        let mut tags: Vec<&str> = Vec::new();
        for key in ["association", "status"] {
            let v = self.get(key);
            if !v.is_empty() && v != "none" {
                tags.push(v);
            }
        }
        if self.get("edited") == "true" {
            tags.push("edited");
        }
        let author = self.get("author");
        match tags.is_empty() {
            true => format!("@{author}"),
            false => format!("@{author} ({})", tags.join(", ")),
        }
    }
}

fn header_field(line: &str) -> Option<(String, String)> {
    let (key, val) = line.split_once(':')?;
    Some((key.trim().to_lowercase(), val.trim().to_string()))
}

/// `--comments` without a TTY prints only comment blocks: a header with
/// `association:`, `--`, the body, `--`. A body may hold its own `--` line,
/// so a block ends only where the next header starts or the input does.
fn parse_comments(input: &str) -> Option<Vec<Comment>> {
    let lines: Vec<&str> = input.lines().collect();
    let first = lines.iter().position(|l| !l.trim().is_empty())?;
    if !lines[first].starts_with("author:") {
        return None;
    }
    let mut comments = Vec::new();
    let mut i = first;
    while i < lines.len() {
        let mut meta = Vec::new();
        while i < lines.len() && lines[i] != "--" {
            meta.extend(header_field(lines[i]));
            i += 1;
        }
        if !meta.iter().any(|(k, _)| k == "association") {
            return None;
        }
        i += 1;
        let start = i;
        while i < lines.len() {
            let next = lines.get(i + 1).copied().unwrap_or("");
            let ends = lines[i] == "--"
                && (next.starts_with("author:")
                    || lines[i + 1..].iter().all(|l| l.trim().is_empty()));
            if ends {
                break;
            }
            i += 1;
        }
        let body = strip_html_comments(&collapse_blank_runs(&lines[start..i.min(lines.len())]));
        comments.push(Comment {
            meta,
            body: body.trim().to_string(),
        });
        i += 1;
        while i < lines.len() && lines[i].trim().is_empty() {
            i += 1;
        }
    }
    Some(comments)
}

fn collapse_blank_runs(lines: &[&str]) -> String {
    let mut out: Vec<&str> = Vec::new();
    for line in lines {
        let line = line.trim_end();
        if !(line.is_empty() && out.last().is_none_or(|l| l.is_empty())) {
            out.push(line);
        }
    }
    out.join("\n")
}

fn render_comments(comments: &[Comment], format: OutputFormat) -> String {
    if format == OutputFormat::Json {
        let list: Vec<serde_json::Value> = comments
            .iter()
            .map(|c| {
                let mut obj: serde_json::Map<String, serde_json::Value> = c
                    .meta
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect();
                obj.insert("body".into(), serde_json::Value::String(c.body.clone()));
                serde_json::Value::Object(obj)
            })
            .collect();
        return serde_json::json!({ "comments": list }).to_string();
    }
    let n = comments.len();
    let mut out = format!("{n} comment{}\n", if n == 1 { "" } else { "s" });
    for c in comments {
        out.push_str(&format!("--\n{}\n", c.heading()));
        if !c.body.is_empty() {
            out.push_str(&c.body);
            out.push('\n');
        }
    }
    out
}

impl ParseHandler {
    /// `gh pr view` / `gh issue view` without a TTY: keeps the fields an
    /// agent acts on and the body, drops empty headers and template comments.
    pub(crate) fn handle_gh_view(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
        kind: ViewKind,
    ) -> CommandResult {
        let input = Self::read_input(file)?;

        let output = match parse_comments(&input) {
            Some(comments) if !comments.is_empty() => render_comments(&comments, ctx.format),
            _ => {
                let Some(output) = render_view(&input, ctx.format, kind) else {
                    return Self::emit_unparsed(&input, kind.reducer(), ctx);
                };
                output
            }
        };

        crate::parse_out::emit(&output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer(kind.reducer())
                .with_input_bytes(input.len())
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }
}

fn render_view(input: &str, format: OutputFormat, kind: ViewKind) -> Option<String> {
    let mut fields: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut body_lines: Vec<&str> = Vec::new();
    let mut in_body = false;

    for line in input.lines() {
        if in_body {
            body_lines.push(line);
            continue;
        }
        if line.trim() == "--" {
            in_body = true;
            continue;
        }
        let Some((k, v)) = header_field(line) else {
            continue;
        };
        match k.as_str() {
            "body" => in_body = true,
            "title" | "state" | "author" | "url" | "number" => {
                fields.insert(k, v);
            }
            _ if !v.is_empty() && KEPT.contains(&k.as_str()) => {
                fields.insert(k, v);
            }
            _ => {}
        }
    }
    if fields.is_empty() && !input.trim().is_empty() {
        return None;
    }
    let body = strip_html_comments(&collapse_blank_runs(&body_lines));

    if format == OutputFormat::Json {
        let mut obj: serde_json::Map<String, serde_json::Value> = fields
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();
        if !body.trim().is_empty() {
            obj.insert("body".to_string(), serde_json::Value::String(body));
        }
        return Some(serde_json::Value::Object(obj).to_string());
    }

    let get = |k: &str| fields.get(k).map_or("", String::as_str);
    let or_q = |v: &'static str| match get(v) {
        "" => "?".to_string(),
        s => s.to_string(),
    };
    let mut out = format!(
        "{} #{}: {} [{}] by {}\n",
        kind.label(),
        get("number"),
        or_q("title"),
        or_q("state"),
        or_q("author")
    );
    for k in KEPT {
        match (k, get(k)) {
            (_, "") | ("deletions", _) => {}
            ("additions", a) => out.push_str(&format!("changes: +{} -{}\n", a, get("deletions"))),
            (k, v) => out.push_str(&format!("{k}: {v}\n")),
        }
    }
    if !get("url").is_empty() {
        out.push_str(&format!("url: {}\n", get("url")));
    }
    if !body.trim().is_empty() {
        out.push_str("--\n");
        out.push_str(body.trim());
        out.push('\n');
    }
    Some(out)
}

/// PR templates leave `<!-- ... -->` guidance the author never deleted.
pub(crate) fn strip_html_comments(text: &str) -> String {
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
