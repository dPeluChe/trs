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
    // Inline scripts and generic CLIs — no dedicated parser, but
    // generic ANSI/whitespace compression alone yields ~30-40% on the
    // larger outputs (`bash -c "..."` with concatenated diagnostics,
    // `du -h` trees, `awk` data dumps).
    "bash ",
    "node ",
    "awk ",
    "du ",
    "jq ",
];

/// Commands that should NEVER be rewritten (internal, cd, pipes, etc.)
const SKIP_PREFIXES: &[&str] = &[
    "trs ", "cd ", "echo ", "cat ", "head ", "tail -f", "export ", "source ", ".", "set ",
    "unset ", "alias ", "which ", "type ", "true", "false", "exit", "return",
];

/// Always-on wrappers stripped before routing and re-prepended on the
/// rewrite. Two groups:
///
/// - Shell builtins + 1-token process wrappers: the shell or the wrapper
///   consume them before exec without changing what command runs.
/// - Venv runners (`poetry run`, `uv run`, `pdm run`): the wrapper
///   passes the inner command through uninterpreted, so trs can route
///   the inner cmd to its parser. Notably NOT included: `bun run`,
///   `pnpm run`, `npm run`, `yarn`, `npx` — those interpret the
///   following token as a script name or package, not a command.
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
    "poetry run",
    "uv run",
    "pdm run",
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

/// Split leading `NAME=value` tokens off a command. Heuristic: starts with
/// ASCII letter or underscore, then alphanumeric or underscore, contains
/// `=`. Excludes `--flag=val` and URL-ish strings. Multi-token prefixes
/// (`A=1 B=2 cargo build`) are collected left-to-right.
pub(super) fn split_env_prefix(cmd: &str) -> Option<(String, &str)> {
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
/// `NAME=value`. Used by both `split_env_prefix` here and `cmd_bypasses_trs`
/// in `rewrite.rs`.
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
pub(super) fn strip_word_prefix<'a>(cmd: &'a str, prefix: &str) -> Option<&'a str> {
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
#[path = "rewrite_decide_tests.rs"]
mod tests;
