//! Rewrite-decision logic for `trs rewrite`. Decides whether a shell
//! command should be wrapped with `trs` and produces the rewritten string.
//! Wire-format envelopes (Claude / Gemini / Cursor) live in `rewrite.rs`.

/// Commands that should NEVER be rewritten (internal, cd, pipes, etc.)
// `ssh `: 537 wraps in 30 days of real use saved 1.9%. The remote command is
// opaque to trs, and wrapping holds a long session's output until it exits.
const SKIP_PREFIXES: &[&str] = &[
    "trs ", "cd ", "echo ", "cat ", "head ", "tail -f", "export ", "source ", ".", "set ",
    "unset ", "alias ", "which ", "type ", "true", "false", "exit", "return", "ssh ",
];

/// Project-local build launchers: `"."` above skips local scripts, which may
/// be interactive, but these run a known build tool with a parser.
const BUILD_WRAPPERS: &[&str] = &["./mvnw", "./gradlew"];

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

    // Explicit per-invocation opt-out. `TRS_SKIP=1 <cmd>` and the
    // historical `TRS_DISABLE=1 <cmd>` alias both bypass rewriting.
    // The shell strips the env-var assignment before exec, so the
    // downstream program never sees these markers.
    if trimmed.starts_with("TRS_SKIP=") || trimmed.starts_with("TRS_DISABLE=") {
        return None;
    }

    // Output is being captured somewhere the user will read raw (file
    // redirect, `| tee`, command substitution). Rewriting here would put the
    // COMPRESSED text in the file instead of the command's real output, so
    // redirection would stop being an escape hatch. Checked before the
    // env-prefix split because `OUT=$(cmd)` otherwise splits mid-substitution
    // and produces a mangled command.
    if captures_output(trimmed) {
        return None;
    }

    // Text-level rewriting is only sound on a flat single command; anything
    // else gets corrupted silently. Compression is never worth that.
    if !is_simple_command(trimmed) {
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
    // The split is quote-aware: a ` && ` inside `-m "fix a && b"` is text the
    // user is passing along, not an operator to slice on.
    if has_shell_op && find_unquoted_str(trimmed, " && ").is_some() {
        let segments: Vec<&str> = split_and_chain(trimmed);
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

    let build_wrapper = BUILD_WRAPPERS
        .iter()
        .any(|w| trimmed == *w || trimmed.strip_prefix(w).is_some_and(|r| r.starts_with(' ')));
    for skip in SKIP_PREFIXES {
        if !build_wrapper && (trimmed.starts_with(skip) || trimmed == skip.trim()) {
            return None;
        }
    }

    // `;` chains are independent commands — too unpredictable to rewrite.
    if has_shell_op && trimmed.contains(" ; ") {
        return None;
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

    let first = trimmed.split_whitespace().next().unwrap_or("");

    // Verbatim commands re-layout bytes; the generic catch-all below would
    // collapse the spacing that IS their output. Leave them unwrapped, and
    // see through `bash -c` too: wrapping there costs the child its tty, so
    // `column` falls back to 80 columns before anything downstream can help.
    let rest = &trimmed[first.len()..];
    if crate::command_registry::is_verbatim_invocation(first, rest) {
        return None;
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

/// Keywords opening a compound construct: `trs for x in …` makes the shell
/// read `for` as a program and `do` as a syntax error.
const SHELL_KEYWORDS: &[&str] = &[
    "for", "while", "until", "if", "case", "select", "function", "do", "then", "else", "elif",
    "fi", "done", "esac", "time{", "{", "(",
];

/// True when the command is one flat command, the only shape where editing the
/// text is equivalent to wrapping the command. Every rejected shape below was
/// corrupted in the field — per-shape evidence in
/// `docs/development/agent-integrations.md` § "What the hook refuses to rewrite".
fn is_simple_command(cmd: &str) -> bool {
    if cmd.contains('\n') || cmd.contains('\r') {
        return false;
    }
    // Heredoc / herestring: everything after it is data.
    if cmd.contains("<<") {
        return false;
    }
    if contains_unquoted(cmd, ';') {
        return false;
    }
    // Array literal: the env-prefix split lands inside the parens and adds a
    // phantom element (`[uno][trs][dos]`) — corrupt data, no exit code.
    if find_unquoted_str(cmd, "=(").is_some() {
        return false;
    }
    // Subshell / brace group: wrapping makes the shell die parsing.
    if cmd.starts_with('(') || cmd.starts_with('{') {
        return false;
    }
    let first = cmd.split_whitespace().next().unwrap_or("");
    // Function definition: `f() { … }`.
    if first.ends_with("()") {
        return false;
    }
    let first = first.trim_end_matches(|c: char| c == '{' || c == '(');
    if SHELL_KEYWORDS.contains(&first) {
        return false;
    }
    true
}

/// True when `needle` appears outside quotes — quoting is what separates an
/// operator from a literal the caller is passing along.
fn contains_unquoted(cmd: &str, needle: char) -> bool {
    find_unquoted(cmd, needle).is_some()
}

/// Byte offset of the first unquoted occurrence of `needle`.
fn find_unquoted(cmd: &str, needle: char) -> Option<usize> {
    let mut single = false;
    let mut double = false;
    let mut escaped = false;
    for (i, c) in cmd.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' if !single => escaped = true,
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            _ if c == needle && !single && !double => return Some(i),
            _ => {}
        }
    }
    None
}

/// Split on ` && ` only where it is a real operator (outside quotes).
fn split_and_chain(cmd: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = cmd;
    loop {
        let Some(pos) = find_unquoted_str(rest, " && ") else {
            out.push(rest.trim());
            return out;
        };
        out.push(rest[..pos].trim());
        rest = &rest[pos + 4..];
    }
}

/// Byte offsets of characters outside any quote and not escaped.
fn unquoted_indices(cmd: &str) -> impl Iterator<Item = usize> + '_ {
    let (mut single, mut double, mut escaped) = (false, false, false);
    cmd.char_indices().filter_map(move |(i, c)| {
        if escaped {
            escaped = false;
            return None;
        }
        match c {
            '\\' if !single => escaped = true,
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            _ if !single && !double => return Some(i),
            _ => {}
        }
        None
    })
}

/// Byte offset of the first unquoted occurrence of `needle` (multi-char).
fn find_unquoted_str(cmd: &str, needle: &str) -> Option<usize> {
    unquoted_indices(cmd).find(|&i| cmd[i..].starts_with(needle))
}

/// An unquoted `|` or `|&` that is not half of `||`. `grep "a|b"` is an
/// argument, not a pipe.
fn pipes_output(cmd: &str) -> bool {
    let b = cmd.as_bytes();
    unquoted_indices(cmd)
        .any(|i| b[i] == b'|' && b.get(i + 1) != Some(&b'|') && (i == 0 || b[i - 1] != b'|'))
}

/// True when something other than the agent consumes the output: a file
/// redirect, a pipe, or command substitution. Each expects the command's real
/// bytes, so handing it trs's summary returns a wrong answer, not a shorter one.
/// Not flagged: fd duplication (`2>&1`) and discards (`2>/dev/null`).
pub(super) fn captures_output(cmd: &str) -> bool {
    // Command substitution — the caller consumes the value directly.
    if cmd.contains("$(") || cmd.contains('`') {
        return true;
    }
    // A pipe hands stdout to a program that parses the real bytes: `| grep`
    // searched trs's summary, `| wc -l` counted it, `| jq` failed on it.
    if pipes_output(cmd) {
        return true;
    }
    // A `>` / `>>` whose target is a real path. Two things don't count: an fd
    // duplication (`2>&1`) and a discard (`2>/dev/null`) — neither leaves a
    // copy anyone reads, and discarding stderr is a very common agent idiom.
    // Byte scanning, not `chars().collect()`: this runs on every hook call,
    // and collecting the whole command into a Vec<char> to compare four ASCII
    // characters is the one real allocation on that path. Every byte compared
    // here is ASCII, and an ASCII byte never occurs inside a multi-byte UTF-8
    // sequence, so an index found this way is always a char boundary.
    let b = cmd.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'>' {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < b.len() && b[j] == b'>' {
            j += 1;
        }
        while j < b.len() && b[j] == b' ' {
            j += 1;
        }
        if j >= b.len() {
            return true;
        }
        if b[j] == b'&' {
            i = j;
            continue;
        }
        let end = cmd[j..]
            .find(char::is_whitespace)
            .map_or(cmd.len(), |k| j + k);
        if &cmd[j..end] != "/dev/null" {
            return true;
        }
        i = j;
    }
    false
}

#[cfg(test)]
#[path = "rewrite_decide_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "rewrite_decide_capture_tests.rs"]
mod capture_tests;
