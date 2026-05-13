//! Rewrite-decision logic for `trs rewrite`. Decides whether a shell
//! command should be wrapped with `trs` and produces the rewritten string.
//! Wire-format envelopes (Claude / Gemini / Cursor) live in `rewrite.rs`.

/// Commands (prefixes) trs knows how to compress. Intentionally broad —
/// keep loosely in sync with classifier.rs. Unknown commands still get
/// generic compression (whitespace + ANSI).
const REWRITE_PREFIXES: &[&str] = &[
    "git ",
    "ls ",
    "ls\n",
    "lsd ",
    "exa ",
    "eza ",
    "tree ",
    "find ",
    "fd ",
    "grep ",
    "rg ",
    "ag ",
    "ack ",
    "tail ",
    "cargo ",
    "npm ",
    "pnpm ",
    "bun ",
    "yarn ",
    "pip ",
    "pip3 ",
    "pytest",
    "jest",
    "vitest",
    "make ",
    "make\n",
    "cmake ",
    "tsc ",
    "gcc ",
    "g++ ",
    "clang ",
    "javac ",
    "docker ",
    "gh ",
    "env\n",
    "env ",
    "printenv",
    "wc ",
    "wget ",
    "curl ",
    "eslint",
    "biome ",
    "ruff ",
    "pylint",
    "golangci-lint",
    "ollama ",
    "kubectl ",
    "swift ",
    "xcodebuild ",
    "ping ",
    "brew ",
    "python ",
    "python3 ",
    "npx ",
    "ps ",
    "uv ",
];

/// Commands that should NEVER be rewritten (internal, cd, pipes, etc.)
const SKIP_PREFIXES: &[&str] = &[
    "trs ", "cd ", "echo ", "cat ", "head ", "tail -f", "export ", "source ", ".", "set ",
    "unset ", "alias ", "which ", "type ", "true", "false", "exit", "return",
];

/// Always-on wrappers: shell builtins the shell consumes before exec, plus
/// 1-token process wrappers with no required args. Arg-taking wrappers
/// (`sudo`, `env`, `nice -n N`) go through user config instead.
const TRANSPARENT_PREFIX_BUILTINS: &[&str] = &[
    "noglob",
    "command",
    "builtin",
    "exec",
    "nocorrect",
    "time",
    "nohup",
    "setsid",
    "unbuffer",
    "stdbuf",
];

/// Decide if a command should be rewritten through trs. Returns
/// `Some(rewritten)` or `None` (leave unchanged).
pub(crate) fn maybe_rewrite(cmd: &str) -> Option<String> {
    let trimmed = cmd.trim();

    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with("trs ") {
        return None;
    }

    // Explicit per-invocation opt-out. `TRS_SKIP=1 <cmd>` survives downstream
    // because the shell strips the env-var assignment before exec.
    if trimmed.starts_with("TRS_SKIP=") {
        return None;
    }

    // Env-var prefix stays attached to the wrapped command so the shell
    // applies it to the real program, not to trs.
    if let Some((env_prefix, body)) = split_env_prefix(trimmed) {
        return maybe_rewrite(body).map(|r| format!("{} {}", env_prefix, r));
    }

    // Transparent wrapper: `time cargo test` → `time trs cargo test`.
    // Strip → recurse → re-prepend keeps the wrapping semantics intact.
    if let Some((wrapper, body)) = strip_transparent_prefix(trimmed) {
        return maybe_rewrite(body).map(|r| format!("{} {}", wrapper, r));
    }

    // Hot path: most commands are plain "git X" with no shell ops. One
    // byte scan rejects them before the more expensive contains() checks.
    let has_shell_op = trimmed
        .as_bytes()
        .iter()
        .any(|b| matches!(b, b'&' | b'|' | b'>' | b'<' | b';'));

    // `&&` chains: rewrite each segment independently. Checked BEFORE
    // SKIP_PREFIXES so `cd X && git Y` doesn't get short-circuited by `cd`.
    if has_shell_op
        && trimmed.contains(" && ")
        && !trimmed.contains(" | ")
        && !trimmed.contains(" ; ")
    {
        let segments: Vec<&str> = trimmed.split(" && ").map(str::trim).collect();
        let mut any_changed = false;
        let mut rewritten: Vec<String> = Vec::with_capacity(segments.len());
        for seg in &segments {
            match maybe_rewrite(seg) {
                Some(r) => {
                    any_changed = true;
                    rewritten.push(r);
                }
                None => rewritten.push(seg.to_string()),
            }
        }
        if any_changed {
            return Some(rewritten.join(" && "));
        }
        return None;
    }

    for skip in SKIP_PREFIXES {
        if trimmed.starts_with(skip) || trimmed == skip.trim() {
            return None;
        }
    }

    // `;` chains are independent commands — too unpredictable to rewrite.
    if has_shell_op && trimmed.contains(" ; ") {
        return None;
    }

    // Pipe/redirect: rewrite ONLY the first segment. Agents routinely
    // append `| head -N`, `| grep X`, or `> file` — rewriting the data
    // producer preserves shell semantics while keeping compression.
    if has_shell_op {
        if let Some((first, rest)) = split_at_shell_op(trimmed) {
            let rewritten = maybe_rewrite(first)?;
            return Some(format!("{}{}", rewritten, rest));
        }
    }

    if trimmed.contains("$(") || trimmed.contains('`') {
        return None;
    }

    // Security: don't let agents skip pre-commit hooks via --no-verify.
    if (trimmed.starts_with("git commit") || trimmed.starts_with("git push"))
        && (trimmed.contains("--no-verify") || trimmed.contains("-n "))
    {
        eprintln!("[trs] blocked: --no-verify is not allowed (protects pre-commit hooks)");
        std::process::exit(2);
    }

    for prefix in REWRITE_PREFIXES {
        let p = prefix.trim();
        if trimmed.starts_with(prefix) || trimmed == p {
            return Some(format!("trs {}", trimmed));
        }
    }

    // Unknown command — generic compression (whitespace + ANSI). Skip
    // bare assignment-like patterns to avoid wrapping `FOO=bar`.
    if trimmed.contains('=') && !trimmed.contains(' ') {
        return None;
    }

    Some(format!("trs {}", trimmed))
}

/// Split leading `NAME=value` tokens off a command. Heuristic for what
/// counts as `NAME=value`: starts with ASCII letter or underscore, then
/// alphanumeric or underscore, contains `=`. Excludes `--flag=val` and
/// URL-ish strings. Multi-token prefixes (`A=1 B=2 cargo build`) are
/// collected left-to-right.
fn split_env_prefix(cmd: &str) -> Option<(String, &str)> {
    let mut rest = cmd;
    let mut collected: Vec<&str> = Vec::new();
    loop {
        let trimmed = rest.trim_start();
        if trimmed.is_empty() {
            break;
        }
        let Some(space_at) = trimmed.find(char::is_whitespace) else {
            break;
        };
        let token = &trimmed[..space_at];
        if !looks_like_env_assignment(token) {
            break;
        }
        collected.push(token);
        rest = &trimmed[space_at..];
    }
    if collected.is_empty() {
        None
    } else {
        Some((collected.join(" "), rest.trim_start()))
    }
}

/// True when `token` matches the shell env-var assignment shape
/// `NAME=value`. Used by both `split_env_prefix` (this module) and
/// `cmd_bypasses_trs` (in `rewrite.rs`).
pub(crate) fn looks_like_env_assignment(token: &str) -> bool {
    let Some(eq_at) = token.find('=') else {
        return false;
    };
    if eq_at == 0 {
        return false;
    }
    let name = &token[..eq_at];
    let first = name.chars().next().unwrap();
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn strip_transparent_prefix(cmd: &str) -> Option<(&str, &str)> {
    for prefix in TRANSPARENT_PREFIX_BUILTINS {
        if let Some(rest) = strip_word_prefix(cmd, prefix) {
            return Some((prefix, rest));
        }
    }
    for prefix in &crate::config::config().hooks.transparent_prefixes {
        if let Some(rest) = strip_word_prefix(cmd, prefix.as_str()) {
            return Some((prefix.as_str(), rest));
        }
    }
    None
}

/// Whole-word prefix match. `prefix="time"` matches `"time x"` but not
/// `"timeout"`. A bare wrapper with no trailing whitespace is rejected.
fn strip_word_prefix<'a>(cmd: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = cmd.strip_prefix(prefix)?;
    let next = rest.as_bytes().first()?;
    if next.is_ascii_whitespace() {
        Some(rest.trim_start())
    } else {
        None
    }
}

/// Split a command at the first shell operator (pipe or redirect) so the
/// left side can be rewritten independently. Longer delimiters (`" >> "`)
/// are tried before their prefixes (`" > "`) so we don't misclassify them.
fn split_at_shell_op(s: &str) -> Option<(&str, &str)> {
    [" | ", " >> ", " > ", " < "]
        .iter()
        .filter_map(|op| s.find(op).map(|pos| (pos, *op)))
        .min_by_key(|(pos, _)| *pos)
        .map(|(pos, _)| (&s[..pos], &s[pos..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewrite_git() {
        assert_eq!(maybe_rewrite("git status"), Some("trs git status".into()));
        assert_eq!(maybe_rewrite("git log -5"), Some("trs git log -5".into()));
    }

    #[test]
    fn test_rewrite_cargo() {
        assert_eq!(maybe_rewrite("cargo test"), Some("trs cargo test".into()));
        assert_eq!(
            maybe_rewrite("cargo clippy"),
            Some("trs cargo clippy".into())
        );
    }

    #[test]
    fn test_skip_already_trs() {
        assert_eq!(maybe_rewrite("trs git status"), None);
    }

    #[test]
    fn test_skip_cd() {
        assert_eq!(maybe_rewrite("cd /tmp"), None);
    }

    #[test]
    fn test_skip_trs_skip_env_var() {
        assert_eq!(maybe_rewrite("TRS_SKIP=1 git status"), None);
        assert_eq!(maybe_rewrite("TRS_SKIP=true cargo test"), None);
        assert_eq!(maybe_rewrite("TRS_SKIP= git log"), None);
    }

    #[test]
    fn test_transparent_prefix_builtins() {
        assert_eq!(
            maybe_rewrite("time cargo test"),
            Some("time trs cargo test".into())
        );
        assert_eq!(
            maybe_rewrite("nohup cargo build"),
            Some("nohup trs cargo build".into())
        );
        assert_eq!(
            maybe_rewrite("noglob git status"),
            Some("noglob trs git status".into())
        );
        assert_eq!(
            maybe_rewrite("command git log -5"),
            Some("command trs git log -5".into())
        );
    }

    #[test]
    fn test_transparent_prefix_partial_match_not_stripped() {
        // `timeout` must not match the `time` prefix (whole-word only).
        let r = maybe_rewrite("timeout 5 cargo test").unwrap();
        assert!(
            !r.starts_with("time trs"),
            "expected `time` not to match `timeout`, got: {}",
            r
        );
    }

    #[test]
    fn test_transparent_prefix_alone_not_stripped() {
        // Bare wrapper falls through to generic rewrite; don't loop.
        let out = maybe_rewrite("time");
        assert!(out != Some("time".into()), "shouldn't strip bare wrapper");
    }

    #[test]
    fn test_transparent_prefix_composes_with_env() {
        assert_eq!(
            maybe_rewrite("RUSTFLAGS=-C time cargo test"),
            Some("RUSTFLAGS=-C time trs cargo test".into())
        );
    }

    #[test]
    fn test_transparent_prefix_nested() {
        assert_eq!(
            maybe_rewrite("nohup time cargo build"),
            Some("nohup time trs cargo build".into())
        );
    }

    #[test]
    fn test_transparent_prefix_skips_when_inner_skipped() {
        // `time cd /tmp` — inner is in SKIP_PREFIXES, whole expr returns None.
        assert_eq!(maybe_rewrite("time cd /tmp"), None);
    }

    #[test]
    fn test_strip_word_prefix_strict() {
        assert_eq!(
            strip_word_prefix("time cargo test", "time"),
            Some("cargo test")
        );
        assert_eq!(strip_word_prefix("timeout 5 x", "time"), None);
        assert_eq!(strip_word_prefix("time", "time"), None);
        assert_eq!(strip_word_prefix("foo bar", ""), None);
    }

    #[test]
    fn test_trs_skip_does_not_match_other_env_vars() {
        assert!(maybe_rewrite("RUSTFLAGS=-C git status").is_some());
    }

    #[test]
    fn test_env_prefix_stays_in_front() {
        assert_eq!(
            maybe_rewrite("RUSTFLAGS=-C cargo build"),
            Some("RUSTFLAGS=-C trs cargo build".into())
        );
    }

    #[test]
    fn test_multi_env_prefix() {
        assert_eq!(
            maybe_rewrite("A=1 B=2 cargo build"),
            Some("A=1 B=2 trs cargo build".into())
        );
    }

    #[test]
    fn test_env_prefix_before_unknown_command_still_rewrites() {
        let out = maybe_rewrite("FOO=bar something-weird arg").unwrap();
        assert!(out.starts_with("FOO=bar trs "));
    }

    #[test]
    fn test_flag_looks_like_assignment_not_matched() {
        // --crate-name=foo is NOT an env-var assignment.
        let out = maybe_rewrite("--crate-name=foo cargo build").unwrap();
        assert!(!out.starts_with("--crate-name=foo trs"));
    }

    #[test]
    fn test_split_env_prefix_empty_when_none() {
        assert!(split_env_prefix("cargo build").is_none());
        assert!(split_env_prefix("git status").is_none());
    }

    #[test]
    fn test_stderr_redirects_survive_rewrite() {
        // Shell handles `2>&1` etc. before trs sees the command — we just
        // keep them attached to the wrapped command.
        assert_eq!(
            maybe_rewrite("cargo test 2>&1"),
            Some("trs cargo test 2>&1".into())
        );
        assert_eq!(
            maybe_rewrite("git status 2>/dev/null"),
            Some("trs git status 2>/dev/null".into())
        );
        assert_eq!(
            maybe_rewrite("cargo test 2>&1 | head -5"),
            Some("trs cargo test 2>&1 | head -5".into())
        );
    }

    #[test]
    fn test_rewrite_pipe_first_segment() {
        assert_eq!(
            maybe_rewrite("git log | grep fix"),
            Some("trs git log | grep fix".into())
        );
        assert_eq!(
            maybe_rewrite("find . -name '*.rs' | xargs wc"),
            Some("trs find . -name '*.rs' | xargs wc".into())
        );
        assert_eq!(
            maybe_rewrite("git status | head -3"),
            Some("trs git status | head -3".into())
        );
    }

    #[test]
    fn test_rewrite_multi_pipe_first_segment_only() {
        assert_eq!(
            maybe_rewrite("git log | head -5 | tail -1"),
            Some("trs git log | head -5 | tail -1".into())
        );
    }

    #[test]
    fn test_rewrite_redirect_first_segment() {
        assert_eq!(
            maybe_rewrite("git diff > out.txt"),
            Some("trs git diff > out.txt".into())
        );
        assert_eq!(
            maybe_rewrite("git log >> history.txt"),
            Some("trs git log >> history.txt".into())
        );
    }

    #[test]
    fn test_skip_pipe_when_first_segment_is_skipped() {
        assert_eq!(maybe_rewrite("cat < input.txt"), None);
        assert_eq!(maybe_rewrite("echo hello | xxd"), None);
        assert_eq!(maybe_rewrite("trs git status | head -3"), None);
    }

    #[test]
    fn test_skip_subshells() {
        assert_eq!(maybe_rewrite("echo $(git status)"), None);
    }

    #[test]
    fn test_skip_assignments() {
        assert_eq!(maybe_rewrite("FOO=bar"), None);
    }

    #[test]
    fn test_skip_empty() {
        assert_eq!(maybe_rewrite(""), None);
    }

    #[test]
    fn test_rewrite_unknown_command() {
        assert_eq!(
            maybe_rewrite("terraform plan"),
            Some("trs terraform plan".into())
        );
    }

    #[test]
    fn test_skip_echo() {
        assert_eq!(maybe_rewrite("echo hello"), None);
    }

    #[test]
    fn test_json_protocol() {
        // Pulls the command out of the JSON envelope and verifies the
        // rewrite logic on it — the entry path's smoke test.
        let input = r#"{"tool_name":"Bash","tool_input":{"command":"git status"}}"#;
        let json: serde_json::Value = serde_json::from_str(input).unwrap();
        let cmd = json
            .get("tool_input")
            .and_then(|ti| ti.get("command"))
            .and_then(|c| c.as_str())
            .unwrap();
        assert_eq!(maybe_rewrite(cmd), Some("trs git status".into()));
    }

    #[test]
    fn test_rewrite_env() {
        assert_eq!(maybe_rewrite("env"), Some("trs env".into()));
    }

    #[test]
    fn test_skip_shell_builtins() {
        assert_eq!(maybe_rewrite("export PATH=/usr/bin"), None);
        assert_eq!(maybe_rewrite("source .env"), None);
    }

    #[test]
    fn test_rewrite_cd_chain() {
        assert_eq!(
            maybe_rewrite("cd /tmp && git status"),
            Some("cd /tmp && trs git status".into())
        );
        assert_eq!(
            maybe_rewrite("cd /path/to/project && cargo test"),
            Some("cd /path/to/project && trs cargo test".into())
        );
    }

    #[test]
    fn test_rewrite_multi_chain() {
        assert_eq!(
            maybe_rewrite("cd /tmp && git status && git log"),
            Some("cd /tmp && trs git status && trs git log".into())
        );
        assert_eq!(
            maybe_rewrite("cargo fmt && cargo clippy"),
            Some("trs cargo fmt && trs cargo clippy".into())
        );
    }

    #[test]
    fn test_skip_cd_chain_with_pipe() {
        assert_eq!(maybe_rewrite("cd /tmp && git status | less"), None);
    }

    #[test]
    fn test_skip_cd_chain_all_skips() {
        assert_eq!(maybe_rewrite("cd /tmp && echo hello"), None);
    }
}
