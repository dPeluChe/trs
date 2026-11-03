//! Claude Code checks its permission rules against the command *after* the
//! hook rewrote it, and a hook `allow` overrides them, so a `deny`/`ask` rule
//! for `git push` never saw `trs git push`. The hook reads the user's rules
//! and applies them to the original command. Details: docs/development/agent-integrations.md.

use std::path::{Path, PathBuf};

/// What the Claude hook does with a command, given the user's rules.
#[derive(Debug, PartialEq)]
pub(crate) enum Gate {
    /// A deny/ask rule matches, or the rules cannot be read: leave the command alone.
    Skip,
    /// Every part is allowed (rule or read-only): rewrite and approve.
    Allow,
    /// Rewrite, but let Claude's own permission flow decide.
    Defer,
}

#[derive(Default)]
struct Rules {
    allow: Vec<String>,
    ask: Vec<String>,
    deny: Vec<String>,
}

pub(crate) fn gate(cmd: &str, cwd: Option<&str>) -> Gate {
    match Rules::load(cwd) {
        Some(rules) => gate_with(&rules, cmd),
        None => Gate::Skip,
    }
}

fn gate_with(rules: &Rules, cmd: &str) -> Gate {
    let mut all_allowed = true;
    let mut any = false;
    for seg in split_segments(cmd) {
        if seg == "cd" || seg.starts_with("cd ") {
            continue;
        }
        any = true;
        let bare = strip_env(seg);
        if rules
            .deny
            .iter()
            .chain(&rules.ask)
            .any(|r| rule_matches(r, seg) || rule_matches(r, bare))
        {
            return Gate::Skip;
        }
        if !(rules.allow.iter().any(|r| rule_matches(r, seg)) || read_only(seg)) {
            all_allowed = false;
        }
    }
    if any && all_allowed {
        Gate::Allow
    } else {
        Gate::Defer
    }
}

impl Rules {
    /// `None` when a settings file exists but cannot be read or parsed.
    /// `allow` from a project file counts only once Claude trusts that folder,
    /// as Claude itself ignores it until then; deny/ask always count.
    fn load(cwd: Option<&str>) -> Option<Self> {
        let trusted = trusted_dirs();
        let mut rules = Rules::default();
        for (path, project_dir) in settings_files(cwd) {
            let Ok(text) = std::fs::read_to_string(&path) else {
                if path.exists() {
                    return None;
                }
                continue;
            };
            let json: serde_json::Value = serde_json::from_str(&text).ok()?;
            let perms = json.get("permissions");
            let list = |key: &str| -> Vec<String> {
                let arr = perms.and_then(|p| p.get(key)).and_then(|v| v.as_array());
                arr.into_iter()
                    .flatten()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            };
            if project_dir.is_none_or(|d| d.ancestors().any(|a| trusted.iter().any(|t| t == a))) {
                rules.allow.extend(list("allow"));
            }
            rules.ask.extend(list("ask"));
            rules.deny.extend(list("deny"));
        }
        Some(rules)
    }
}

/// Folders where the user accepted Claude Code's trust dialog.
fn trusted_dirs() -> Vec<PathBuf> {
    if cfg!(test) {
        return Vec::new();
    }
    let file = match std::env::var_os("CLAUDE_CONFIG_DIR") {
        Some(dir) => PathBuf::from(dir).join(".claude.json"),
        None => match crate::init::home_dir() {
            Ok(home) => home.join(".claude.json"),
            Err(_) => return Vec::new(),
        },
    };
    let Ok(text) = std::fs::read_to_string(file) else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    json.get("projects")
        .and_then(|p| p.as_object())
        .into_iter()
        .flatten()
        .filter(|(_, v)| v.get("hasTrustDialogAccepted").and_then(|t| t.as_bool()) == Some(true))
        .map(|(k, _)| PathBuf::from(k))
        .collect()
}

/// Settings files in Claude's precedence; the folder is set for project files.
fn settings_files(cwd: Option<&str>) -> Vec<(PathBuf, Option<&Path>)> {
    let mut files = Vec::new();
    // Unit tests must not read the developer's own settings.
    if !cfg!(test) {
        let user_dir = std::env::var_os("CLAUDE_CONFIG_DIR")
            .map(PathBuf::from)
            .or_else(|| crate::init::home_dir().ok().map(|h| h.join(".claude")));
        if let Some(dir) = user_dir {
            files.push((dir.join("settings.json"), None));
            files.push((dir.join("settings.local.json"), None));
        }
        files.push((
            PathBuf::from(if cfg!(target_os = "macos") {
                "/Library/Application Support/ClaudeCode/managed-settings.json"
            } else if cfg!(windows) {
                "C:\\Program Files\\ClaudeCode\\managed-settings.json"
            } else {
                "/etc/claude-code/managed-settings.json"
            }),
            None,
        ));
    }
    for dir in cwd.map(Path::new).into_iter().flat_map(Path::ancestors) {
        files.push((dir.join(".claude/settings.json"), Some(dir)));
        files.push((dir.join(".claude/settings.local.json"), Some(dir)));
    }
    files
}

/// `Bash`, `Bash(*)`, `Bash(git push)`, `Bash(git push:*)`, `Bash(git push *)`.
fn rule_matches(rule: &str, cmd: &str) -> bool {
    let inner = if rule == "Bash" {
        ""
    } else if let Some(i) = rule.strip_prefix("Bash(").and_then(|r| r.strip_suffix(')')) {
        i
    } else {
        return false;
    };
    if inner.is_empty() || inner == "*" {
        return true;
    }
    if let Some(prefix) = inner.strip_suffix(":*") {
        let prefix = prefix.trim_end();
        return cmd == prefix || cmd.strip_prefix(prefix).is_some_and(|r| r.starts_with(' '));
    }
    if let Some(bare) = inner.strip_suffix(" *") {
        if cmd == bare {
            return true;
        }
    }
    if inner.contains('*') {
        return glob(inner.as_bytes(), cmd.as_bytes());
    }
    cmd == inner
}

fn glob(pat: &[u8], s: &[u8]) -> bool {
    match pat.split_first() {
        None => s.is_empty(),
        Some((b'*', rest)) => (0..=s.len()).any(|i| glob(rest, &s[i..])),
        Some((c, rest)) => s.first() == Some(c) && glob(rest, &s[1..]),
    }
}

/// Leading `NAME=value` words removed.
fn strip_env(seg: &str) -> &str {
    let mut rest = seg;
    while let Some(word) = rest.split_whitespace().next() {
        if !crate::rewrite_decide::looks_like_env_assignment(word) {
            break;
        }
        rest = rest[rest.find(word).unwrap_or(0) + word.len()..].trim_start();
    }
    rest
}

/// Commands Claude approves without asking; anything with a redirect or
/// substitution is not, and options that write or execute are refused.
fn read_only(seg: &str) -> bool {
    let seg = seg.replace("2>&1", "");
    if seg.contains(['>', '<', '`', '$']) {
        return false;
    }
    let words: Vec<&str> = seg.split_whitespace().collect();
    let Some(&prog) = words.first() else {
        return false;
    };
    let risky = words
        .iter()
        .any(|w| w.starts_with("--output") || *w == "--pre");
    if risky {
        return false;
    }
    match prog {
        "ls" | "cat" | "head" | "tail" | "wc" | "pwd" | "which" | "grep" => true,
        "git" => matches!(
            words.get(1).copied(),
            Some(
                "status"
                    | "log"
                    | "diff"
                    | "show"
                    | "blame"
                    | "ls-files"
                    | "rev-parse"
                    | "describe"
            )
        ),
        _ => false,
    }
}

/// Split on unquoted `&&`, `||`, `;`, `|`, `&` and newlines (`2>&1` and `&>` stay whole).
fn split_segments(cmd: &str) -> Vec<&str> {
    let b = cmd.as_bytes();
    let (mut out, mut start, mut i, mut quote) = (Vec::new(), 0, 0, 0u8);
    while i < b.len() {
        let c = b[i];
        if quote != 0 {
            if c == b'\\' && quote == b'"' {
                i += 1;
            } else if c == quote {
                quote = 0;
            }
        } else {
            match c {
                b'\'' | b'"' => quote = c,
                b'\\' => i += 1,
                b'&' if i > 0 && b[i - 1] == b'>' || b.get(i + 1) == Some(&b'>') => {}
                b'&' | b'|' | b';' | b'\n' => {
                    out.push(&cmd[start..i]);
                    let double = matches!(c, b'&' | b'|') && b.get(i + 1) == Some(&c);
                    i += usize::from(double);
                    start = i + 1;
                }
                _ => {}
            }
        }
        i += 1;
    }
    out.push(&cmd[start.min(cmd.len())..]);
    out.into_iter()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(allow: &[&str], ask: &[&str], deny: &[&str]) -> Rules {
        let own = |l: &[&str]| l.iter().map(|s| s.to_string()).collect();
        Rules {
            allow: own(allow),
            ask: own(ask),
            deny: own(deny),
        }
    }

    #[test]
    fn rule_forms_match_like_claude_code() {
        assert!(rule_matches("Bash", "anything"));
        assert!(rule_matches("Bash(*)", "anything"));
        assert!(rule_matches("Bash(git push:*)", "git push origin main"));
        assert!(rule_matches("Bash(git push:*)", "git push"));
        assert!(!rule_matches("Bash(git push:*)", "git pushx"));
        assert!(rule_matches("Bash(npm run test)", "npm run test"));
        assert!(!rule_matches("Bash(npm run test)", "npm run test --x"));
        assert!(rule_matches("Bash(ls *)", "ls -la"));
        assert!(rule_matches("Bash(ls *)", "ls"));
        assert!(!rule_matches("Bash(ls *)", "lsof"));
        assert!(rule_matches("Bash(git * main)", "git push origin main"));
        assert!(!rule_matches("Read(.env)", "cat .env"));
    }

    #[test]
    fn splits_chains_but_not_quotes_or_redirects() {
        assert_eq!(split_segments("a && b | c; d"), ["a", "b", "c", "d"]);
        assert_eq!(split_segments("echo 'a && b' && c"), ["echo 'a && b'", "c"]);
        assert_eq!(split_segments("git log 2>&1"), ["git log 2>&1"]);
        assert_eq!(split_segments("a &\nb"), ["a", "b"]);
    }

    #[test]
    fn a_deny_or_ask_rule_on_the_original_command_stops_the_rewrite() {
        let r = rules(&["Bash(git:*)"], &["Bash(git push:*)"], &["Bash(rm -rf:*)"]);
        assert_eq!(gate_with(&r, "git push origin main"), Gate::Skip);
        assert_eq!(gate_with(&r, "cd x && git push"), Gate::Skip);
        assert_eq!(gate_with(&r, "rm -rf build"), Gate::Skip);
        assert_eq!(gate_with(&r, "CI=1 rm -rf build"), Gate::Skip);
        assert_eq!(gate_with(&r, "git status && rm -rf build"), Gate::Skip);
    }

    #[test]
    fn only_explicit_allow_or_read_only_commands_are_approved() {
        let r = rules(&["Bash(cargo test:*)"], &[], &[]);
        assert_eq!(gate_with(&r, "cargo test --lib"), Gate::Allow);
        assert_eq!(gate_with(&r, "cd x && cargo test"), Gate::Allow);
        assert_eq!(gate_with(&r, "git status"), Gate::Allow);
        assert_eq!(gate_with(&r, "git log 2>&1"), Gate::Allow);
        assert_eq!(gate_with(&r, "cargo build"), Gate::Defer);
        assert_eq!(gate_with(&r, "cargo test && cargo build"), Gate::Defer);
        assert_eq!(gate_with(&r, "git push"), Gate::Defer);
        assert_eq!(gate_with(&r, "git branch -D x"), Gate::Defer);
        assert_eq!(gate_with(&r, "git diff --output=out.txt"), Gate::Defer);
        assert_eq!(gate_with(&r, "cat a > b"), Gate::Defer);
        assert_eq!(gate_with(&r, ""), Gate::Defer);
    }

    #[test]
    fn unreadable_settings_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        std::fs::write(dir.path().join(".claude/settings.json"), "{ not json").unwrap();
        assert_eq!(gate("git status", dir.path().to_str()), Gate::Skip);
    }
}
