//! Droid's command-list semantics (allow normalization, deny/block token-gap
//! patterns), ported from its 0.202.0 matcher and checked against it.

use crate::rewrite_decide::looks_like_env_assignment;
use crate::rewrite_permissions::split_segments;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

const WRAPPERS: [&str; 4] = ["sudo", "time", "nohup", "env"];
const SHELLS: [&str; 5] = ["sh", "bash", "zsh", "dash", "ksh"];
const READERS: [&str; 5] = ["head", "tail", "grep", "cat", "echo"];

/// Quote-aware whitespace split; quotes stay in the word.
fn words(seg: &str) -> Vec<&str> {
    let (mut out, mut start, mut quote) = (Vec::new(), None, None);
    for (i, c) in seg.char_indices() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '\'' | '"') => {
                quote = Some(c);
                start.get_or_insert(i);
            }
            (None, c) if c.is_whitespace() => {
                if let Some(s) = start.take() {
                    out.push(&seg[s..i]);
                }
            }
            _ => {
                start.get_or_insert(i);
            }
        }
    }
    if let Some(s) = start {
        out.push(&seg[s..]);
    }
    out
}

fn wrapper_takes_value(wrapper: &str, opt: &str) -> bool {
    let list: &[&str] = match wrapper {
        "sudo" => &[
            "-u",
            "-g",
            "-p",
            "-r",
            "-t",
            "-C",
            "-D",
            "-R",
            "-T",
            "-U",
            "-h",
            "--user",
            "--group",
            "--prompt",
            "--role",
            "--type",
            "--close-from",
            "--chdir",
            "--chroot",
            "--other-user",
            "--host",
        ],
        "env" => &["-u", "--unset", "-C", "--chdir", "-S", "--split-string"],
        "time" => &["-f", "--format", "-o", "--output"],
        _ => &[],
    };
    list.contains(&opt)
}

/// Index of the program word, past env assignments and `sudo|time|nohup|env`.
fn program_index(w: &[&str]) -> Option<usize> {
    let mut i = 0;
    while i < w.len() {
        if looks_like_env_assignment(w[i]) {
            i += 1;
        } else if WRAPPERS.contains(&w[i]) {
            let wrapper = w[i];
            i += 1;
            while i < w.len() && w[i].starts_with('-') {
                i += 1 + usize::from(wrapper_takes_value(wrapper, w[i]));
            }
        } else {
            return Some(i);
        }
    }
    None
}

fn unquote(s: &str) -> &str {
    s.trim_matches(|c| c == '"' || c == '\'')
}

/// Droid's allow-list key for one simple command: `git status`, `ls`, `gh pr`.
pub(super) fn normalize(seg: &str) -> Option<String> {
    let w = words(seg);
    let i = program_index(&w)?;
    let prog = unquote(w[i]);
    if prog == "cd" {
        return None;
    }
    let prog = prog.rsplit('/').next().unwrap_or(prog);
    if prog.is_empty() || prog.starts_with('-') || prog.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if !["git", "npm", "yarn", "pnpm", "bun", "gh", "glab"].contains(&prog) {
        return Some(prog.to_string());
    }
    let want = if prog == "gh" || prog == "glab" { 2 } else { 1 };
    let (mut subs, mut skip) = (Vec::new(), false);
    for &u in &w[i + 1..] {
        if skip {
            skip = false;
            continue;
        }
        let digits_off = u.trim_start_matches(|c: char| c.is_ascii_digit());
        if digits_off.starts_with(['<', '>', '&']) {
            skip = digits_off.chars().all(|c| matches!(c, '<' | '>' | '&'));
            continue;
        }
        if u.starts_with('-') {
            skip = match prog {
                "git" => ["-C", "-c", "--git-dir", "--work-tree"].contains(&u),
                "gh" | "glab" => ["-R", "--repo"].contains(&u),
                _ => ["--prefix", "--workspace", "--dir", "-C"].contains(&u),
            };
            continue;
        }
        if u.starts_with(['"', '\'']) {
            continue;
        }
        subs.push(u);
        if subs.len() >= want {
            break;
        }
    }
    Some(if subs.is_empty() {
        prog.to_string()
    } else {
        format!("{prog} {}", subs.join(" "))
    })
}

/// Normalized keys of every simple command in `text` (allow entries use this too).
pub(super) fn keys(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for k in split_segments(text).into_iter().filter_map(normalize) {
        if !out.contains(&k) {
            out.push(k);
        }
    }
    out
}

static FD_REDIRECT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[0-9]*>&[0-9]+|[0-9]*>>?\s*/dev/null").unwrap());
static FUNC_DEF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*[A-Za-z_][A-Za-z0-9_-]*\s*\(\s*\)\s*\{").unwrap());
static CHAIN_SEP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*(?:&&|\|\|?|;)\s*").unwrap());
static RM_ARGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?:^|\s)(?:sudo\s+)?rm\s+(.*)").unwrap());

fn has_file_redirect(seg: &str) -> bool {
    FD_REDIRECT.replace_all(seg, " ").contains('>')
}

/// Droid runs the command unprompted only if every part is a listed key.
pub(super) fn all_allowed(cmd: &str, allow: &HashSet<String>) -> bool {
    if ["$(", "`", "<(", ">("].iter().any(|t| cmd.contains(t)) {
        return false;
    }
    let parts: Vec<(&str, String)> = split_segments(cmd)
        .into_iter()
        .filter_map(|s| normalize(s).map(|k| (s, k)))
        .collect();
    if parts.is_empty() || split_segments(cmd).iter().any(|s| FUNC_DEF.is_match(s)) {
        return false;
    }
    let needed: Vec<&String> = parts
        .iter()
        .filter(|(raw, k)| !(READERS.contains(&k.as_str()) && !has_file_redirect(raw)))
        .map(|(_, k)| k)
        .collect();
    if needed.is_empty() {
        return parts.iter().all(|(_, k)| allow.contains(k));
    }
    needed.iter().all(|k| allow.contains(*k))
}

fn pattern_regex(pat: &str, anchored: bool) -> Option<Regex> {
    let t: Vec<&str> = pat.split_whitespace().collect();
    let esc: Vec<String> = t.iter().map(|x| regex::escape(x)).collect();
    let start = if anchored { "^" } else { r"(^|[\s;&|(`]+)" };
    let end = |w: &str| {
        if w.contains('/') {
            r"([\s;&|)`/]+|$)"
        } else {
            r"([\s;&|)`]+|$)"
        }
    };
    let word_edge = if t
        .first()?
        .ends_with(|c: char| c.is_alphanumeric() || c == '_')
    {
        r"\b"
    } else {
        ""
    };
    let n = t.len();
    let src = if n == 1 {
        format!("{start}{}{word_edge}{}", esc[0], end(t[0]))
    } else {
        let mut s = format!("{start}{}{word_edge}", esc[0]);
        for u in 1..n - 1 {
            let slashy = t[u + 1].starts_with('/');
            let gap = if slashy {
                r"[^;&|.]{0,100}"
            } else {
                r"[^;&|]{0,100}"
            };
            s += &format!("{gap}{}", esc[u]);
        }
        let (f, last) = (&esc[n - 1], t[n - 1]);
        s += &match last {
            "/" => format!(r"[^;&|.]{{0,100}}{f}"),
            "~" => format!(r"[^;&|]{{0,100}}{f}"),
            "." => format!(r"[^;&|]{{0,100}}{f}([\s;&|)`]+|$)"),
            l if l.starts_with('/') => format!(r"[^;&|.]{{0,100}}{f}{}", end(l)),
            l => format!(r"[^;&|]{{0,100}}{f}{}", end(l)),
        };
        s
    };
    Regex::new(&format!("(?i){src}")).ok()
}

/// `$(..)` and backtick bodies of `text`, and `text` with them blanked.
fn substitutions(text: &str) -> (String, Vec<String>) {
    let (mut outer, mut bodies, b) = (String::new(), Vec::new(), text.as_bytes());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'$' && b.get(i + 1) == Some(&b'(') {
            let (mut depth, mut j) = (1, i + 2);
            while j < b.len() && depth > 0 {
                depth += usize::from(b[j] == b'(');
                depth -= usize::from(b[j] == b')');
                j += 1;
            }
            bodies.push(text[i + 2..j.saturating_sub(1).max(i + 2)].to_string());
            outer.push(' ');
            i = j;
        } else if b[i] == b'`' {
            let j = text[i + 1..].find('`').map_or(b.len(), |k| i + 1 + k);
            bodies.push(text[i + 1..j].to_string());
            outer.push(' ');
            i = j + 1;
        } else {
            let ch = text[i..].chars().next().unwrap();
            outer.push(ch);
            i += ch.len_utf8();
        }
    }
    (outer, bodies)
}

/// Texts a pattern is tried on: the command, each part with env/sudo stripped,
/// and nested `$(..)`, backticks and `bash -c` bodies.
fn candidates(text: &str, depth: usize, out: &mut Vec<String>) {
    if depth > 8 || out.len() > 64 {
        return;
    }
    out.push(text.to_string());
    let (outer, bodies) = substitutions(text);
    for body in bodies {
        candidates(&body, depth + 1, out);
    }
    for seg in split_segments(&outer) {
        let w = words(seg);
        let Some(i) = program_index(&w) else { continue };
        out.push(w[i..].join(" "));
        if SHELLS.contains(&unquote(w[i])) {
            if let Some(c) = w[i + 1..]
                .iter()
                .position(|x| x.starts_with('-') && x.ends_with('c'))
            {
                if let Some(body) = w.get(i + 2 + c) {
                    candidates(unquote(body), depth + 1, out);
                }
            }
        }
    }
}

fn first_match<'a>(cmd: &str, patterns: &'a [String]) -> Option<&'a str> {
    let mut cands = Vec::new();
    candidates(cmd, 0, &mut cands);
    let lower: Vec<String> = cands.iter().map(|c| c.to_lowercase()).collect();
    patterns.iter().map(String::as_str).find(|p| {
        // The pattern's first word is a literal in its regex: skip the compile when absent.
        let first = p
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_lowercase();
        if !lower.iter().any(|c| c.contains(&first)) {
            return false;
        }
        let multi = split_segments(p).len() > 1;
        pattern_regex(p, !multi).is_some_and(|re| cands.iter().any(|c| re.is_match(c)))
    })
}

pub(super) fn blocked(cmd: &str, block: &[String]) -> bool {
    first_match(cmd, block).is_some()
}

/// A denylist hit, except `rm` aimed only at `/tmp` (Droid exempts it).
pub(super) fn denied(cmd: &str, deny: &[String]) -> bool {
    if first_match(cmd, deny).is_none() {
        return false;
    }
    let mut only_tmp = false;
    for part in CHAIN_SEP.split(cmd).map(str::trim) {
        let hit = first_match(part, deny).is_some();
        let Some(m) = RM_ARGS.captures(part) else {
            if hit {
                return true;
            }
            continue;
        };
        if !hit {
            continue;
        }
        only_tmp = true;
        let targets: Vec<&str> = m[1]
            .split_whitespace()
            .filter(|t| !t.starts_with('-'))
            .collect();
        let tmp = |t: &&str| {
            !t.contains("..") && !t.contains(['$', '`']) && (*t == "/tmp" || t.starts_with("/tmp/"))
        };
        if targets.is_empty() || !targets.iter().all(tmp) {
            return true;
        }
    }
    !only_tmp
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(items: &[&str]) -> HashSet<String> {
        items.iter().flat_map(|i| keys(i)).collect()
    }
    fn pats(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn normalizes_like_droid() {
        let n = |s: &str| normalize(s);
        assert_eq!(n("git status"), Some("git status".into()));
        assert_eq!(n("git -C /x push origin"), Some("git push".into()));
        assert_eq!(n("FOO=1 git push"), Some("git push".into()));
        assert_eq!(n("sudo -u bob rm -rf x"), Some("rm".into()));
        assert_eq!(n("gh pr list --json x"), Some("gh pr list".into()));
        assert_eq!(n("npm test -- --x"), Some("npm test".into()));
        assert_eq!(n("GIT push"), Some("GIT".into()));
        assert_eq!(n("TRS_AGENT=droid trs git status"), Some("trs".into()));
        assert_eq!(n("cd /x"), None);
        assert_eq!(keys("echo hi; git push"), ["echo", "git push"]);
    }

    #[test]
    fn deny_patterns_see_the_original_but_not_the_rewrite() {
        let deny = pats(&["git push", "rm -rf /", "sudo"]);
        for hit in [
            "git push origin main",
            "git status && git push -f",
            "FOO=1 git push",
            "git -C /x push origin",
            "sudo git push",
            "bash -c \"git push\"",
            "echo $(git push)",
            "GIT push",
            "rm -rf /home/u",
        ] {
            assert!(denied(hit, &deny), "{hit}");
        }
        for miss in [
            "git pushx",
            "git status",
            "rm -rf ./build",
            "rm -rf /tmp/a",
            "TRS_AGENT=droid trs git push origin main",
        ] {
            assert!(!denied(miss, &deny), "{miss}");
        }
        assert!(blocked("rm -rf /tmp/a", &pats(&["rm -rf /"])));
    }

    #[test]
    fn allow_needs_every_part_listed() {
        let allow = set(&["ls", "git status", "npm test"]);
        assert!(all_allowed("git status | cat", &allow));
        assert!(all_allowed("git status > /tmp/o", &allow));
        assert!(all_allowed("npm test -- --x", &allow));
        assert!(all_allowed("cd x && ls -la", &allow));
        assert!(!all_allowed("git push", &allow));
        assert!(!all_allowed("ls && pwd", &allow));
        assert!(!all_allowed("echo $(ls)", &allow));
        assert!(!all_allowed("cat a > b", &allow));
        assert!(!all_allowed("TRS_AGENT=droid trs git status", &allow));
        assert!(!all_allowed("cd x", &allow));
    }
}
