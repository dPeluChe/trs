//! Codex matches its `.rules` on the REWRITTEN command and needs a hook `allow`
//! to apply the rewrite: the gate only decides Allow or Skip.
//! Contract: docs/development/agent-integrations.md (permissions).

use crate::rewrite_gate::Gate;
use crate::rewrite_permissions::{split_segments, strip_env};
use std::path::{Path, PathBuf};

/// One `prefix_rule` pattern: each position holds the accepted alternatives.
type Pattern = Vec<Vec<String>>;

/// `cmd` is the ORIGINAL command; `cwd` comes from the hook payload.
pub(crate) fn gate(cmd: &str, cwd: Option<&str>) -> Gate {
    match load(codex_home().as_deref(), cwd.map(Path::new)) {
        Some(patterns) => gate_with(&patterns, cmd),
        None => Gate::Skip,
    }
}

/// Any rule that matches (forbidden, prompt or allow) stops the rewrite: the
/// rewritten text would no longer match it and the user's intent would be lost.
fn gate_with(patterns: &[Pattern], cmd: &str) -> Gate {
    let hit = split_segments(cmd).into_iter().any(|seg| {
        let tokens = tokenize(strip_env(seg));
        patterns.iter().any(|p| matches_prefix(p, &tokens))
    });
    if hit {
        Gate::Skip
    } else {
        Gate::Allow
    }
}

fn matches_prefix(pattern: &Pattern, tokens: &[String]) -> bool {
    if pattern.is_empty() || pattern.len() > tokens.len() {
        return false;
    }
    pattern
        .iter()
        .zip(tokens)
        .enumerate()
        .all(|(i, (alts, tok))| {
            alts.iter()
                .any(|a| a == tok || (i == 0 && crate::text_util::cmd_basename(tok) == a.as_str()))
        })
}

fn tokenize(seg: &str) -> Vec<String> {
    let (mut out, mut cur, mut quote, mut open) = (Vec::new(), String::new(), '\0', false);
    let mut chars = seg.chars();
    while let Some(c) = chars.next() {
        match c {
            _ if quote != '\0' => {
                if c == quote {
                    quote = '\0';
                } else if c == '\\' && quote == '"' {
                    cur.extend(chars.next());
                } else {
                    cur.push(c);
                }
            }
            '\'' | '"' => (quote, open) = (c, true),
            '\\' => {
                cur.extend(chars.next());
                open = true;
            }
            c if c.is_whitespace() => {
                if open || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                    open = false;
                }
            }
            _ => cur.push(c),
        }
    }
    if open || !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// `$CODEX_HOME` or `~/.codex`.
fn codex_home() -> Option<PathBuf> {
    if !crate::rewrite_rules::reads_host_config() {
        return None;
    }
    std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| crate::init::home_dir().ok().map(|h| h.join(".codex")))
}

/// `None` when a rules or config file exists but cannot be read or parsed.
/// Project rules count only for folders marked trusted in `config.toml`.
fn load(home: Option<&Path>, cwd: Option<&Path>) -> Option<Vec<Pattern>> {
    let mut patterns = Vec::new();
    let mut dirs = Vec::new();
    if let Some(home) = home {
        dirs.push(home.join("rules"));
        if let Some(cwd) = cwd {
            let trusted = trusted_projects(&home.join("config.toml"))?;
            if let Some(root) = cwd.ancestors().position(|a| trusted.iter().any(|t| t == a)) {
                dirs.extend(
                    cwd.ancestors()
                        .take(root + 1)
                        .map(|a| a.join(".codex/rules")),
                );
            }
        }
    }
    for dir in dirs {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) if crate::rewrite_rules::is_missing(&e) => continue,
            Err(_) => return None,
        };
        for entry in entries {
            let path = entry.ok()?.path();
            if path.extension().is_some_and(|e| e == "rules") {
                let text = std::fs::read_to_string(&path).ok()?;
                patterns.extend(parse_rules(&text)?);
            }
        }
    }
    Some(patterns)
}

fn trusted_projects(config: &Path) -> Option<Vec<PathBuf>> {
    let text = match std::fs::read_to_string(config) {
        Ok(text) => text,
        Err(e) => return (crate::rewrite_rules::is_missing(&e)).then(Vec::new),
    };
    let doc: toml::Table = text.parse().ok()?;
    let Some(projects) = doc.get("projects") else {
        return Some(Vec::new());
    };
    Some(
        projects
            .as_table()?
            .iter()
            .filter(|(_, v)| v.get("trust_level").and_then(|t| t.as_str()) == Some("trusted"))
            .map(|(k, _)| PathBuf::from(k))
            .collect(),
    )
}

/// Tolerant reader for `prefix_rule(pattern=[...], decision="...", ...)`.
/// Anything else (other statements, odd literals) returns `None`: fail closed.
fn parse_rules(text: &str) -> Option<Vec<Pattern>> {
    let mut out = Vec::new();
    let mut rest = text;
    loop {
        rest = rest.trim_start();
        if let Some(after) = rest.strip_prefix('#') {
            rest = after.split_once('\n').map_or("", |(_, r)| r);
            continue;
        }
        if rest.is_empty() {
            return Some(out);
        }
        let body = rest.strip_prefix("prefix_rule(")?;
        let close = find_close(body)?;
        out.push(parse_call(&body[..close])?);
        rest = &body[close + 1..];
    }
}

/// Index of the `)` that closes a call whose `(` was just consumed.
fn find_close(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let (mut depth, mut i) = (0usize, 0);
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => i = skip_string(b, i)?,
            b'#' => {
                i += b[i..]
                    .iter()
                    .position(|&c| c == b'\n')
                    .unwrap_or(b.len() - i)
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' if depth == 0 => return Some(i),
            b')' | b']' | b'}' => depth = depth.checked_sub(1)?,
            _ => {}
        }
        i += 1;
    }
    None
}

/// Index of the closing quote of the string opening at `i`.
fn skip_string(b: &[u8], i: usize) -> Option<usize> {
    let q = b[i];
    let mut j = i + 1;
    while j < b.len() {
        match b[j] {
            b'\\' => j += 1,
            c if c == q => return Some(j),
            _ => {}
        }
        j += 1;
    }
    None
}

/// Split on commas outside strings and brackets; a trailing comma is fine.
fn split_top(s: &str) -> Option<Vec<&str>> {
    let b = s.as_bytes();
    let (mut parts, mut start, mut depth, mut i) = (Vec::new(), 0, 0usize, 0);
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => i = skip_string(b, i)?,
            b'#' => {
                i += b[i..]
                    .iter()
                    .position(|&c| c == b'\n')
                    .unwrap_or(b.len() - i)
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.checked_sub(1)?,
            b',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    parts.push(&s[start..]);
    parts.retain(|p| !p.trim().is_empty());
    (depth == 0).then_some(parts)
}

fn parse_call(args: &str) -> Option<Pattern> {
    let (mut pattern, mut decision) = (None, None);
    for arg in split_top(args)? {
        let (name, value) = arg.split_once('=')?;
        match name.trim() {
            "pattern" => pattern = Some(parse_pattern(value.trim())?),
            "decision" => decision = Some(parse_string(value.trim())?),
            _ => {}
        }
    }
    let pattern = pattern.filter(|p| !p.is_empty())?;
    matches!(decision?.as_str(), "allow" | "prompt" | "forbidden").then_some(pattern)
}

fn parse_pattern(s: &str) -> Option<Pattern> {
    let inner = s.strip_prefix('[')?.strip_suffix(']')?;
    split_top(inner)?
        .into_iter()
        .map(|el| {
            let el = el.trim();
            match el.strip_prefix('[') {
                Some(alts) => split_top(alts.strip_suffix(']')?)?
                    .into_iter()
                    .map(|a| parse_string(a.trim()))
                    .collect(),
                None => Some(vec![parse_string(el)?]),
            }
        })
        .collect()
}

fn parse_string(s: &str) -> Option<String> {
    let q = *s.as_bytes().first()?;
    if (q != b'"' && q != b'\'') || skip_string(s.as_bytes(), 0)? != s.len() - 1 || s.len() < 2 {
        return None;
    }
    let mut out = String::new();
    let mut chars = s[1..s.len() - 1].chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                other => out.push(other),
            }
        } else {
            out.push(c);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pats(text: &str) -> Vec<Pattern> {
        parse_rules(text).expect("parses")
    }

    #[test]
    fn parses_decisions_alternatives_and_comments() {
        let p = pats(
            "# note\nprefix_rule(pattern=[\"git\", [\"push\", \"pull\"]], decision=\"prompt\", justification=\"x, y\")\n\
             prefix_rule(\n  pattern = ['rm', '-rf'],\n  decision = \"forbidden\",\n)\n",
        );
        assert_eq!(p.len(), 2);
        assert_eq!(p[0][1], ["push", "pull"]);
        assert_eq!(p[1][0], ["rm"]);
    }

    #[test]
    fn unknown_or_broken_forms_fail_closed() {
        for bad in [
            "host_executable(name=\"git\", paths=[\"/usr/bin/git\"])",
            "prefix_rule(pattern=[\"git\"])",
            "prefix_rule(decision=\"allow\")",
            "prefix_rule(pattern=[], decision=\"allow\")",
            "prefix_rule(pattern=[\"git\"], decision=\"maybe\")",
            "prefix_rule(pattern=[x], decision=\"allow\")",
            "prefix_rule(pattern=[\"git\"], decision=\"allow\"",
            "prefix_rule(pattern=[",
            "x = 1",
        ] {
            assert!(parse_rules(bad).is_none(), "{bad}");
        }
        assert!(pats("").is_empty());
        assert!(pats("# only a comment\n").is_empty());
    }

    #[test]
    fn a_matching_rule_of_any_decision_skips() {
        for decision in ["forbidden", "prompt", "allow"] {
            let p = pats(&format!(
                "prefix_rule(pattern=[\"git\",\"push\"], decision=\"{decision}\")"
            ));
            assert_eq!(
                gate_with(&p, "git push origin main"),
                Gate::Skip,
                "{decision}"
            );
        }
    }

    #[test]
    fn matching_is_a_token_prefix_per_segment_after_env() {
        let p = pats("prefix_rule(pattern=[\"git\",\"push\"], decision=\"prompt\")");
        for cmd in [
            "FOO=1 git push",
            "git status && git push",
            "cd x; git push -f",
            "echo hi | /usr/bin/git push",
            "git 'push'",
        ] {
            assert_eq!(gate_with(&p, cmd), Gate::Skip, "{cmd}");
        }
        for cmd in [
            "git status",
            "git pushx",
            "git",
            "echo git push",
            "cargo test",
        ] {
            assert_eq!(gate_with(&p, cmd), Gate::Allow, "{cmd}");
        }
    }

    #[test]
    fn single_token_and_alternative_patterns() {
        let p = pats("prefix_rule(pattern=[[\"rm\",\"rmdir\"]], decision=\"forbidden\")");
        assert_eq!(gate_with(&p, "rmdir build"), Gate::Skip);
        assert_eq!(gate_with(&p, "ls"), Gate::Allow);
        assert_eq!(gate_with(&[], "anything"), Gate::Allow);
    }

    fn write(dir: &Path, rel: &str, text: &str) {
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    const DENY_PUSH: &str = "prefix_rule(pattern=[\"git\",\"push\"], decision=\"forbidden\")";

    #[test]
    fn user_rules_load_and_a_bad_file_fails_closed() {
        let home = tempfile::tempdir().unwrap();
        assert_eq!(load(Some(home.path()), None).unwrap().len(), 0);
        write(home.path(), "rules/default.rules", DENY_PUSH);
        write(home.path(), "rules/notes.txt", "not a rules file");
        assert_eq!(load(Some(home.path()), None).unwrap().len(), 1);
        write(home.path(), "rules/broken.rules", "prefix_rule(pattern=[");
        assert!(load(Some(home.path()), None).is_none());
    }

    #[test]
    fn project_rules_need_a_trusted_project() {
        let home = tempfile::tempdir().unwrap();
        let proj = tempfile::tempdir().unwrap();
        write(proj.path(), ".codex/rules/p.rules", DENY_PUSH);
        let sub = proj.path().join("src");
        std::fs::create_dir_all(&sub).unwrap();
        assert_eq!(load(Some(home.path()), Some(&sub)).unwrap().len(), 0);
        let key = proj.path().display().to_string().replace('\\', "\\\\");
        write(
            home.path(),
            "config.toml",
            &format!("[projects.\"{key}\"]\ntrust_level = \"trusted\"\n"),
        );
        assert_eq!(load(Some(home.path()), Some(&sub)).unwrap().len(), 1);
        write(
            home.path(),
            "config.toml",
            &format!("[projects.\"{key}\"]\ntrust_level = \"untrusted\"\n"),
        );
        assert_eq!(load(Some(home.path()), Some(&sub)).unwrap().len(), 0);
    }

    #[test]
    fn unreadable_config_fails_closed() {
        let home = tempfile::tempdir().unwrap();
        write(home.path(), "config.toml", "this is = = not toml");
        let cwd = home.path().join("p");
        assert!(load(Some(home.path()), Some(&cwd)).is_none());
    }

    #[test]
    fn tokenizer_handles_quotes_and_escapes() {
        assert_eq!(
            tokenize("git  commit -m \"a b\" 'c d'"),
            ["git", "commit", "-m", "a b", "c d"]
        );
        assert_eq!(tokenize("echo a\\ b ''"), ["echo", "a b", ""]);
    }
}
