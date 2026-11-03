//! Gemini rule sources: `tools.allowed` in settings.json and `[[rule]]` TOML
//! policies, reduced to what the gate needs for `run_shell_command`.

use regex::Regex;
use serde_json::Value;

pub(crate) enum Pat {
    Prefix(String),
    Regex(Regex),
    Always,
    Never,
}

impl Pat {
    /// Gemini anchors a prefix at the start of the (sub)command and wants a word break after it.
    pub(crate) fn matches(&self, text: &str) -> bool {
        match self {
            Pat::Prefix(p) => text.strip_prefix(p.as_str()).is_some_and(|r| {
                r.is_empty() || r.starts_with(char::is_whitespace) || r.starts_with('"')
            }),
            Pat::Regex(re) => {
                let escaped = serde_json::to_string(text).unwrap_or_default();
                re.is_match(text) || re.is_match(escaped.trim_matches('"'))
            }
            Pat::Always => true,
            Pat::Never => false,
        }
    }
}

#[derive(Default)]
pub(crate) struct Rules {
    pub allow: Vec<Pat>,
    pub deny: Vec<Pat>,
    pub ask: Vec<Pat>,
    pub blanket_allow: bool,
    pub block_all: bool,
}

#[derive(Default)]
pub(crate) struct Settings {
    pub allowed: Option<Vec<String>>,
    pub yolo: bool,
    pub folder_trust: bool,
}

/// Settings are JSON with comments; `None` when the text does not parse.
pub(crate) fn parse_settings(text: &str) -> Option<Settings> {
    let v: Value = serde_json::from_str(&strip_comments(text)).ok()?;
    v.as_object()?;
    let allowed = v
        .pointer("/tools/allowed")
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        });
    Some(Settings {
        allowed,
        yolo: v
            .pointer("/general/defaultApprovalMode")
            .and_then(|m| m.as_str())
            == Some("yolo"),
        folder_trust: v
            .pointer("/security/folderTrust/enabled")
            .and_then(|b| b.as_bool())
            .unwrap_or(true),
    })
}

fn strip_comments(s: &str) -> String {
    let (mut out, mut it) = (String::new(), s.chars().peekable());
    let mut in_str = false;
    while let Some(c) = it.next() {
        if in_str {
            out.push(c);
            if c == '\\' {
                out.extend(it.next());
            } else if c == '"' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
            out.push(c);
        } else if c == '/' && it.peek() == Some(&'/') {
            while it.next_if(|&n| n != '\n').is_some() {}
        } else if c == '/' && it.peek() == Some(&'*') {
            it.next();
            while let Some(n) = it.next() {
                if n == '*' && it.next_if_eq(&'/').is_some() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

const SHELL: [&str; 3] = ["run_shell_command", "ShellTool", "*"];

/// `run_shell_command` or `run_shell_command(prefix)` entries of `tools.allowed`.
pub(crate) fn add_allowed(rules: &mut Rules, entries: &[String]) {
    for e in entries {
        let inner = SHELL.iter().find_map(|n| {
            if e == n {
                Some("")
            } else {
                e.strip_prefix(n)?.strip_prefix('(')?.strip_suffix(')')
            }
        });
        match inner.map(str::trim) {
            Some("") => rules.blanket_allow = true,
            Some(p) => rules.allow.push(Pat::Prefix(p.into())),
            None => {}
        }
    }
}

/// `None` when the TOML does not parse or holds a regex we cannot compile.
pub(crate) fn add_toml(rules: &mut Rules, text: &str) -> Option<()> {
    let doc: toml::Table = text.parse().ok()?;
    for rule in doc
        .get("rule")
        .and_then(|r| r.as_array())
        .into_iter()
        .flatten()
    {
        let t = rule.as_table()?;
        if !names(t.get("toolName"))
            .iter()
            .any(|n| SHELL.contains(&n.as_str()))
        {
            continue;
        }
        let decision = t.get("decision").and_then(|d| d.as_str())?;
        let opaque = if decision == "allow" {
            Pat::Never
        } else {
            Pat::Always
        };
        let mut pats: Vec<Pat> = names(t.get("commandPrefix"))
            .into_iter()
            .map(|p| Pat::Prefix(p.trim().into()))
            .collect();
        if let Some(re) = t.get("commandRegex").and_then(|r| r.as_str()) {
            pats.push(Pat::Regex(Regex::new(&format!("^(?:{re})")).ok()?));
        }
        if t.contains_key("argsPattern") {
            pats.push(opaque);
        }
        let blanket = pats.is_empty();
        match decision {
            "allow" if blanket => rules.blanket_allow = true,
            "allow" => rules.allow.extend(pats),
            "deny" if blanket => rules.block_all = true,
            "deny" => rules.deny.extend(pats),
            "ask_user" if !blanket => rules.ask.extend(pats),
            "ask_user" => {}
            _ => return None,
        }
    }
    Some(())
}

fn names(v: Option<&toml::Value>) -> Vec<String> {
    match v {
        Some(toml::Value::String(s)) => vec![s.clone()],
        Some(toml::Value::Array(a)) => a
            .iter()
            .filter_map(|s| s.as_str().map(String::from))
            .collect(),
        _ => Vec::new(),
    }
}
