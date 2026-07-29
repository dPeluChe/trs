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
fn test_transparent_prefix_venv_runners() {
    // poetry run / uv run / pdm run pass the inner command through
    // their venv; trs strips them, routes the inner, re-prepends.
    assert_eq!(
        maybe_rewrite("poetry run pytest"),
        Some("poetry run trs pytest".into())
    );
    assert_eq!(
        maybe_rewrite("uv run python script.py"),
        Some("uv run trs python script.py".into())
    );
    assert_eq!(
        maybe_rewrite("pdm run pytest -v"),
        Some("pdm run trs pytest -v".into())
    );
}

#[test]
fn test_npm_run_not_transparent_prefix() {
    // `bun run <script>` / `npm run <script>` interpret the next token
    // as a script name in package.json, not a command. Stripping them
    // would break (e.g. `bun run trs typecheck` would look for a script
    // named "trs"). They MUST stay in REWRITE_PREFIXES path (`trs bun
    // run typecheck` runs bun normally; trs sees the full output).
    let bun = maybe_rewrite("bun run typecheck").unwrap();
    assert!(
        bun.starts_with("trs bun run "),
        "bun run must be wrapped, not stripped: {}",
        bun
    );
    let pnpm = maybe_rewrite("pnpm run build").unwrap();
    assert!(pnpm.starts_with("trs pnpm run "));
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
fn test_captured_output_is_left_raw() {
    // Redirecting must stay an escape hatch: rewriting here would put trs's
    // compressed summary in the file instead of the command's real output.
    for cmd in [
        "git diff > out.txt",
        "git log >> history.txt",
        "npm run build 2> err.log",
        "npm run build &> all.log",
        "npm run build | tee out.log",
        "npm run build 2>&1 | tee out.log",
        "git show main:src/App.tsx > out.tsx",
    ] {
        assert_eq!(maybe_rewrite(cmd), None, "must not rewrite: {}", cmd);
    }
    // Command substitution: the caller consumes the value directly. It also
    // must not be split mid-`$( )`, which produced a mangled command.
    assert_eq!(maybe_rewrite("OUT=$(npm run build)"), None);
    assert_eq!(maybe_rewrite("echo `git rev-parse HEAD`"), None);
}

#[test]
fn test_pipes_and_discards_still_rewrite() {
    // These still reach the agent as text — the case trs compresses for.
    assert_eq!(
        maybe_rewrite("npm run build | head -5"),
        Some("trs npm run build | head -5".into())
    );
    // /dev/null is a discard, not a capture — nobody reads it back.
    assert_eq!(
        maybe_rewrite("git status 2>/dev/null"),
        Some("trs git status 2>/dev/null".into())
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

#[test]
fn test_rewrite_inline_scripts_and_generic_clis() {
    // bash -c, node -e, awk, du, jq — no dedicated parser but generic
    // ANSI/whitespace compression should kick in via REWRITE_PREFIXES.
    assert_eq!(
        maybe_rewrite("bash -c \"echo hello\""),
        Some("trs bash -c \"echo hello\"".into())
    );
    assert_eq!(
        maybe_rewrite("node -e 'console.log(1)'"),
        Some("trs node -e 'console.log(1)'".into())
    );
    assert_eq!(
        maybe_rewrite("awk '/foo/ {print}' file.txt"),
        Some("trs awk '/foo/ {print}' file.txt".into())
    );
    assert_eq!(maybe_rewrite("du -h dist/"), Some("trs du -h dist/".into()));
    assert_eq!(
        maybe_rewrite("jq -r .name package.json"),
        Some("trs jq -r .name package.json".into())
    );
}

#[test]
fn test_node_prefix_does_not_match_nodemon() {
    // Whole-word prefix matching — `node ` shouldn't swallow `nodemon`.
    let out = maybe_rewrite("nodemon server.js");
    // Falls to generic unknown-command path, which DOES rewrite, but
    // the wrapper position is what matters: trs goes IN FRONT of the
    // full command, not between "node" and "mon".
    assert_eq!(out, Some("trs nodemon server.js".into()));
}

#[test]
fn test_never_rewrites_inside_heredoc_or_quotes() {
    // Field report: ` && ` was substituted inside a heredoc body, so the
    // injected text landed in the written file — in one case corrupting a
    // security test that then reported a real escape as blocked.
    assert_eq!(maybe_rewrite("cat <<'EOF'\nC: foo && bar\nEOF"), None);
    assert_eq!(maybe_rewrite("python3 - <<EOF\nx = 1\nEOF"), None);

    // Quoted operators are data, not operators: the message and the search
    // pattern must survive byte-for-byte.
    let m = maybe_rewrite(r#"git commit -m "fix a && b""#).unwrap();
    assert!(m.ends_with(r#"git commit -m "fix a && b""#), "got: {}", m);
    let g = maybe_rewrite(r#"grep -r "foo && bar" src/"#).unwrap();
    assert!(g.contains(r#""foo && bar""#), "pattern was altered: {}", g);
}

#[test]
fn test_never_rewrites_multiline_scripts() {
    // A newline separates statements. Collapsing them turned `V=abc` into a
    // command-scoped env prefix, so `$V` expanded before the assignment
    // applied and non-exported values silently vanished.
    assert_eq!(maybe_rewrite("V=abc\nprintf '%s' \"$V\""), None);
    assert_eq!(maybe_rewrite("cargo build\ncargo test"), None);
}

#[test]
fn test_never_wraps_shell_keywords() {
    // `trs for x in …` makes the shell read `for` as a program and `do` as a
    // syntax error, killing the command outright.
    for cmd in [
        "for f in a b; do git add $f; done",
        "while read l; do git add $l; done",
        "if git diff --quiet; then git status; fi",
        "case $x in a) git status;; esac",
    ] {
        assert_eq!(maybe_rewrite(cmd), None, "must not rewrite: {}", cmd);
    }
}

#[test]
fn test_simple_commands_still_compress() {
    // The guards above must not cost the common case.
    assert_eq!(maybe_rewrite("cargo test"), Some("trs cargo test".into()));
    assert_eq!(
        maybe_rewrite("cd /tmp && git status"),
        Some("cd /tmp && trs git status".into())
    );
    assert_eq!(
        maybe_rewrite("cargo test | head -5"),
        Some("trs cargo test | head -5".into())
    );
}
