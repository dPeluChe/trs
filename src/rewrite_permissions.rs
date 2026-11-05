//! Claude matches its rules after the hook rewrote the command and a hook
//! `allow` overrides them: apply the rules to the original here.
//! Details: docs/development/agent-integrations.md.

use crate::rewrite_gate::Gate;
use crate::rewrite_rules::{is_trusted, read_json, word_prefix, Rules};
use std::path::{Path, PathBuf};

pub(crate) fn gate(cmd: &str, cwd: Option<&str>) -> Gate {
    match load_rules(cwd) {
        Some(rules) => gate_with(&rules, cmd),
        None => Gate::Skip,
    }
}

fn gate_with(rules: &Rules, cmd: &str) -> Gate {
    let segs = split_segments(cmd)
        .into_iter()
        .filter(|s| !(*s == "cd" || s.starts_with("cd ")));
    rules.decide(segs, rule_matches, read_only)
}

/// `None` when a settings file exists but cannot be read or parsed.
/// `allow` from a project file counts only once Claude trusts that folder,
/// as Claude itself ignores it until then; deny/ask always count.
fn load_rules(cwd: Option<&str>) -> Option<Rules> {
    let mut trusted = None;
    let mut rules = Rules::default();
    for (path, project_dir) in settings_files(cwd) {
        let Some(json) = read_json(&path)? else {
            continue;
        };
        let allow_counts =
            project_dir.is_none_or(|d| is_trusted(d, trusted.get_or_insert_with(trusted_dirs)));
        rules.extend_from(&json, allow_counts);
    }
    Some(rules)
}

/// Folders where the user accepted Claude Code's trust dialog.
fn trusted_dirs() -> Vec<PathBuf> {
    if !crate::rewrite_rules::reads_host_config() {
        return Vec::new();
    }
    let file = match std::env::var_os("CLAUDE_CONFIG_DIR") {
        Some(dir) => PathBuf::from(dir).join(".claude.json"),
        None => match crate::init::home_dir() {
            Ok(home) => home.join(".claude.json"),
            Err(_) => return Vec::new(),
        },
    };
    let Some(Some(json)) = read_json(&file) else {
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
    if crate::rewrite_rules::reads_host_config() {
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
pub(crate) fn rule_matches(rule: &str, cmd: &str) -> bool {
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
        return word_prefix(cmd, prefix);
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

/// `*` matches any run of bytes; linear backtracking (one star to retry).
pub(crate) fn glob(pat: &[u8], s: &[u8]) -> bool {
    let (mut p, mut i) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    while i < s.len() {
        if p < pat.len() && pat[p] == b'*' {
            star = Some((p, i));
            p += 1;
        } else if p < pat.len() && pat[p] == s[i] {
            p += 1;
            i += 1;
        } else if let Some((sp, si)) = star {
            p = sp + 1;
            star = Some((sp, si + 1));
            i = si + 1;
        } else {
            return false;
        }
    }
    pat[p..].iter().all(|&c| c == b'*')
}

/// Leading `NAME=value` words removed.
pub(crate) fn strip_env(seg: &str) -> &str {
    let mut rest = seg;
    while let Some(word) = rest.split_whitespace().next() {
        if !crate::rewrite_decide::looks_like_env_assignment(word) {
            break;
        }
        rest = rest[rest.find(word).unwrap_or(0) + word.len()..].trim_start();
    }
    rest
}

/// `uv run --with x rm -rf d` runs `rm`: a deny rule must see past the options.
fn skip_runner_opts<'a>(wrapper: &str, mut body: &'a str) -> &'a str {
    while wrapper.ends_with(" run") && body.starts_with('-') {
        let Some((opt, rest)) = body.split_once(char::is_whitespace) else {
            return body;
        };
        body = rest.trim_start();
        if crate::classifier_args::RUNNER_VALUE_OPTS.contains(&opt) {
            body = body
                .split_once(char::is_whitespace)
                .map_or("", |(_, r)| r.trim_start());
        }
    }
    body
}

/// The command a rule is written against: `VAR=` words and the wrappers trs
/// rewrites through (`time git push`) removed.
pub(crate) fn strip_wrappers(seg: &str) -> &str {
    let mut rest = strip_env(seg);
    while let Some((wrapper, inner)) = crate::rewrite_decide::strip_transparent_prefix(rest) {
        rest = strip_env(skip_runner_opts(wrapper, inner.trim_start()));
    }
    rest
}

/// Commands Claude approves without asking inside the working dir; anything
/// with a redirect or substitution is not, options that write or execute are
/// refused, and so are paths that may leave the project.
pub(crate) fn read_only(seg: &str) -> bool {
    let seg = seg.replace("2>&1", "");
    if seg.contains(['>', '<', '`', '$']) {
        return false;
    }
    let words: Vec<&str> = seg.split_whitespace().collect();
    let Some(&prog) = words.first() else {
        return false;
    };
    let risky = words.iter().any(|w| {
        w.starts_with("--output") || *w == "--pre" || w.starts_with(['/', '~']) || w.contains("..")
    });
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
pub(crate) fn split_segments(cmd: &str) -> Vec<&str> {
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
        assert_eq!(gate_with(&r, "time git push"), Gate::Skip);
        assert_eq!(gate_with(&r, "nohup CI=1 rm -rf build"), Gate::Skip);
    }

    #[test]
    fn glob_handles_stars_without_blowing_up() {
        assert!(glob(b"git * main", b"git push origin main"));
        assert!(glob(b"*", b""));
        assert!(!glob(b"git *", b"gi"));
        let long = "a ".repeat(5000);
        assert!(!glob(b"* b * c * d *", long.as_bytes()));
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
        assert_eq!(gate_with(&r, "ls src"), Gate::Allow);
        assert_eq!(gate_with(&r, "cat ~/.aws/credentials"), Gate::Defer);
        assert_eq!(gate_with(&r, "ls /etc"), Gate::Defer);
        assert_eq!(gate_with(&r, "cat ../x"), Gate::Defer);
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
