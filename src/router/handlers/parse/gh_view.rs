use super::super::common::{CommandContext, CommandResult};
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
        let mut tags: Vec<&str> = ["association", "status"]
            .into_iter()
            .map(|k| self.get(k))
            .filter(|v| !v.is_empty() && *v != "none")
            .collect();
        if self.get("edited") == "true" {
            tags.push("edited");
        }
        let author = self.get("author");
        if tags.is_empty() {
            format!("@{author}")
        } else {
            format!("@{author} ({})", tags.join(", "))
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
    let last = lines.iter().rposition(|l| !l.trim().is_empty())?;
    let mut comments = Vec::new();
    let mut i = first;
    while i <= last {
        let mut meta = Vec::new();
        while i < lines.len() && lines[i] != "--" {
            meta.extend(header_field(lines[i]));
            i += 1;
        }
        if !meta.iter().any(|(k, _)| k == "association") {
            return None;
        }
        let start = i + 1;
        i = start;
        while i < last
            && !(lines[i] == "--" && lines.get(i + 1).is_some_and(|n| n.starts_with("author:")))
        {
            i += 1;
        }
        let end = if i == last && lines[last] != "--" {
            last + 1
        } else {
            i
        };
        comments.push(Comment {
            meta,
            body: clean_body(&lines[start.min(end)..end]),
        });
        i += 1;
    }
    Some(comments)
}

/// Blank runs collapsed, template comments stripped, edges trimmed.
fn clean_body(lines: &[&str]) -> String {
    let mut out: Vec<&str> = Vec::new();
    for line in lines {
        let line = line.trim_end();
        if !(line.is_empty() && out.last().is_none_or(|l| l.is_empty())) {
            out.push(line);
        }
    }
    strip_html_comments(&out.join("\n")).trim().to_string()
}

fn json_object(
    pairs: impl IntoIterator<Item = (String, String)>,
    body: String,
) -> serde_json::Value {
    let mut obj: serde_json::Map<String, serde_json::Value> = pairs
        .into_iter()
        .map(|(k, v)| (k, serde_json::Value::String(v)))
        .collect();
    if !body.is_empty() {
        obj.insert("body".into(), serde_json::Value::String(body));
    }
    serde_json::Value::Object(obj)
}

fn render_comments(comments: Vec<Comment>, format: OutputFormat) -> String {
    use std::fmt::Write;
    let n = comments.len();
    if format == OutputFormat::Json {
        let list: Vec<_> = comments
            .into_iter()
            .map(|c| json_object(c.meta, c.body))
            .collect();
        return serde_json::json!({ "comments": list }).to_string();
    }
    let mut out = format!("{n} comment{}\n", if n == 1 { "" } else { "s" });
    for c in &comments {
        let _ = writeln!(out, "--\n{}", c.heading());
        if !c.body.is_empty() {
            let _ = writeln!(out, "{}", c.body);
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
            Some(comments) => Some(render_comments(comments, ctx.format)),
            None => render_view(&input, ctx.format, kind),
        };
        match output {
            Some(out) => Self::finish(&input, &out, kind.reducer().to_string(), ctx),
            None => Self::emit_unparsed(&input, kind.reducer(), ctx),
        }
    }
}

fn render_view(input: &str, format: OutputFormat, kind: ViewKind) -> Option<String> {
    use std::fmt::Write;
    let mut fields: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut lines = input.lines();
    for line in lines.by_ref() {
        if line.trim() == "--" {
            break;
        }
        let Some((k, v)) = header_field(line) else {
            continue;
        };
        if k == "body" {
            break;
        }
        let always = ["title", "state", "author", "url", "number"].contains(&k.as_str());
        if always || (!v.is_empty() && KEPT.contains(&k.as_str())) {
            fields.insert(k, v);
        }
    }
    if ParseHandler::unread(fields.is_empty(), input) {
        return None;
    }
    let body = clean_body(&lines.collect::<Vec<_>>());

    if format == OutputFormat::Json {
        return Some(json_object(fields, body).to_string());
    }

    let get = |k: &str| fields.get(k).map_or("", String::as_str);
    let or_q = |k: &str| Some(get(k)).filter(|v| !v.is_empty()).unwrap_or("?");
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
            ("additions", a) => {
                let _ = writeln!(out, "changes: +{a} -{}", get("deletions"));
            }
            (k, v) => {
                let _ = writeln!(out, "{k}: {v}");
            }
        }
    }
    if !get("url").is_empty() {
        let _ = writeln!(out, "url: {}", get("url"));
    }
    if !body.is_empty() {
        let _ = writeln!(out, "--\n{body}");
    }
    Some(out)
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
