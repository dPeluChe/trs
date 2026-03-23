//! `trs rewrite` — Hook command rewriter for AI coding tools.
//!
//! Called by PreToolUse/BeforeTool hooks. Reads the command from stdin,
//! decides if it should go through trs, and outputs the rewritten command.
//!
//! Protocol (Claude Code / Gemini CLI):
//!   stdin:  {"tool_name":"Bash","tool_input":{"command":"git status"}}
//!   stdout: (empty = no change, or modified JSON)
//!   exit 0 = allow
//!
//! For commands trs knows how to compress, rewrites "git status" → "trs git status".
//! For commands trs doesn't handle, passes through unchanged.

use std::io::Read;

/// Commands (prefixes) that trs should rewrite.
/// Keep in sync with classifier.rs — but intentionally broad.
/// Unknown commands still get generic compression.
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
];

/// Commands that should NEVER be rewritten (internal, cd, pipes, etc.)
const SKIP_PREFIXES: &[&str] = &[
    "trs ", "cd ", "echo ", "cat ", "head ", "tail -f", "export ", "source ", ".", "set ",
    "unset ", "alias ", "which ", "type ", "true", "false", "exit", "return",
];

/// Run the rewrite logic. Called from main.rs.
pub(crate) fn run_rewrite() {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        // No input or read error — allow unchanged
        return;
    }

    let input = input.trim();
    if input.is_empty() {
        return;
    }

    // Try to parse as JSON (Claude Code / Gemini protocol)
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(input) {
        handle_json_protocol(&json);
        return;
    }

    // Plain text mode (simple command string)
    if let Some(rewritten) = maybe_rewrite(input) {
        println!("{}", rewritten);
    }
}

/// Handle JSON hook protocol (Claude Code PreToolUse / Gemini BeforeTool).
fn handle_json_protocol(json: &serde_json::Value) {
    // Extract the command from tool_input.command
    let command = json
        .get("tool_input")
        .and_then(|ti| ti.get("command"))
        .and_then(|c| c.as_str());

    let Some(cmd) = command else {
        // Not a bash command or unknown format — allow unchanged
        return;
    };

    if let Some(rewritten) = maybe_rewrite(cmd) {
        // Output modified tool_input
        let output = serde_json::json!({
            "tool_input": {
                "command": rewritten
            }
        });
        println!("{}", output);
    }
    // Empty stdout = no change (allow as-is)
}

/// Decide if a command should be rewritten to go through trs.
/// Returns Some(rewritten) or None (leave unchanged).
fn maybe_rewrite(cmd: &str) -> Option<String> {
    let trimmed = cmd.trim();

    // Never rewrite empty commands
    if trimmed.is_empty() {
        return None;
    }

    // Never rewrite commands that are already using trs
    if trimmed.starts_with("trs ") {
        return None;
    }

    // Never rewrite internal/shell commands
    for skip in SKIP_PREFIXES {
        if trimmed.starts_with(skip) || trimmed == skip.trim() {
            return None;
        }
    }

    // Never rewrite pipes or chains (let the shell handle them)
    if trimmed.contains(" | ") || trimmed.contains(" && ") || trimmed.contains(" ; ") {
        return None;
    }

    // Never rewrite commands with redirections
    if trimmed.contains(" > ") || trimmed.contains(" >> ") || trimmed.contains(" < ") {
        return None;
    }

    // Never rewrite subshells or command substitution
    if trimmed.contains("$(") || trimmed.contains('`') {
        return None;
    }

    // Block --no-verify on git commands (security: don't let agents skip hooks)
    if (trimmed.starts_with("git commit") || trimmed.starts_with("git push"))
        && (trimmed.contains("--no-verify") || trimmed.contains("-n "))
    {
        eprintln!("[trs] blocked: --no-verify is not allowed (protects pre-commit hooks)");
        std::process::exit(2); // Exit 2 = block in Claude Code hook protocol
    }

    // Check if this is a command trs handles
    for prefix in REWRITE_PREFIXES {
        let p = prefix.trim();
        if trimmed.starts_with(prefix) || trimmed == p {
            return Some(format!("trs {}", trimmed));
        }
    }

    // Unknown command — still rewrite for generic compression
    // (whitespace collapse, ANSI stripping)
    // But skip very short commands or assignment-like patterns
    if trimmed.contains('=') && !trimmed.contains(' ') {
        return None; // VAR=value
    }

    Some(format!("trs {}", trimmed))
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
    fn test_skip_pipes() {
        assert_eq!(maybe_rewrite("git log | grep fix"), None);
        assert_eq!(maybe_rewrite("find . -name '*.rs' | xargs wc"), None);
    }

    #[test]
    fn test_skip_redirections() {
        assert_eq!(maybe_rewrite("git diff > out.txt"), None);
        assert_eq!(maybe_rewrite("cat < input.txt"), None);
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
        // Unknown commands still get generic compression
        assert_eq!(
            maybe_rewrite("terraform plan"),
            Some("trs terraform plan".into())
        );
    }

    #[test]
    fn test_json_protocol() {
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
    fn test_skip_echo() {
        assert_eq!(maybe_rewrite("echo hello"), None);
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
}
