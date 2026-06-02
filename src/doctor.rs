//! `trs doctor` — Validate trs installation health.
//!
//! Runs a series of checks to verify that trs is correctly installed and
//! all runtime dependencies are available. Designed for fresh installs,
//! CI pipelines, and debugging broken setups.

use std::fmt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::init::{check_tool, AiTool};

/// Result of a single health check.
#[derive(Debug, Clone)]
pub(crate) struct Check {
    pub name: &'static str,
    pub status: CheckStatus,
    pub detail: String,
    /// Extra lines shown indented below the check (e.g., version, path).
    pub sub: Vec<String>,
    /// Actionable hint shown on failure/warn (e.g., "→ trs init <tool>").
    pub hint: String,
}

impl Check {
    fn pass(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Pass,
            detail: detail.into(),
            sub: Vec::new(),
            hint: String::new(),
        }
    }

    fn warn(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Warn,
            detail: detail.into(),
            sub: Vec::new(),
            hint: String::new(),
        }
    }

    fn fail(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Fail,
            detail: detail.into(),
            sub: Vec::new(),
            hint: String::new(),
        }
    }

    fn with_sub(mut self, lines: Vec<String>) -> Self {
        self.sub = lines;
        self
    }

    fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

impl fmt::Display for CheckStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pass => write!(f, "PASS"),
            Self::Warn => write!(f, "WARN"),
            Self::Fail => write!(f, "FAIL"),
        }
    }
}

/// Counts of pass/warn/fail from a set of checks.
struct Summary {
    pass: usize,
    warn: usize,
    fail: usize,
    total: usize,
}

impl Summary {
    fn from_checks(checks: &[Check]) -> Self {
        let mut pass = 0;
        let mut warn = 0;
        let mut fail = 0;
        for c in checks {
            match c.status {
                CheckStatus::Pass => pass += 1,
                CheckStatus::Warn => warn += 1,
                CheckStatus::Fail => fail += 1,
            }
        }
        Self {
            pass,
            warn,
            fail,
            total: checks.len(),
        }
    }
}

/// Run all doctor checks and return results.
pub(crate) fn run_checks() -> Vec<Check> {
    vec![
        check_version(),
        check_path_accessible(),
        check_dep("git", "git", true, "install git: https://git-scm.com"),
        check_dep(
            "rg",
            "ripgrep",
            false,
            "brew install ripgrep (trs search/replace need it)",
        ),
        check_config_dir(),
        check_history_writable(),
        check_stdin_pipeline(),
        check_hooks_installed(),
        check_codex_hooks_orphan(),
        check_output_saver_installed(),
        check_agent_docs_health(),
    ]
}

/// Flag legacy `trs rewrite` entries in `~/.codex/hooks.json`. Codex versions
/// vary in `updatedInput` support — orphans from pre-v0.6.x installs cause
/// "PreToolUse hook returned unsupported updatedInput" errors on every tool
/// call. We no longer install Codex hooks; this surfaces the leftover so
/// users know to run `trs uninstall codex`.
fn check_codex_hooks_orphan() -> Check {
    use std::fs;
    let Ok(home) = crate::init::home_dir() else {
        return Check::pass("codex hooks.json", "no HOME — skipped".to_string());
    };
    let path = home.join(".codex").join("hooks.json");
    if !path.exists() {
        return Check::pass("codex hooks.json", "no orphan trs entry".to_string());
    }
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Check::pass("codex hooks.json", "unreadable — skipped".to_string()),
    };
    if !content.contains("trs rewrite") {
        return Check::pass("codex hooks.json", "no orphan trs entry".to_string());
    }
    let ver = crate::codex::detect_version()
        .map(|(a, b, c)| format!(" (detected codex-cli {a}.{b}.{c})"))
        .unwrap_or_default();
    Check::warn(
        "codex hooks.json",
        format!("legacy `trs rewrite` entry in {}{}", path.display(), ver),
    )
    .with_hint(
        "Codex's PreToolUse still rejects `updatedInput` command rewrite \
         (\"unsupported updatedInput\" errors) — documented but not yet \
         implemented in the runtime (openai/codex#18491), so trs stays \
         rules-only. Run `trs uninstall codex` to scrub, or re-run \
         `trs init codex --global` (auto-scrubs).",
    )
}

/// Count how many of the supported agents have the trs output-saver
/// block installed AND whether the content matches the current canonical
/// template. Drift (manual edits, stale content from older installs)
/// surfaces as a warning so users know to run `--refresh`.
fn check_output_saver_installed() -> Check {
    use crate::output_saver::{verify_agent, VerifyStatus, AGENTS};
    let mut installed = 0;
    let mut drifted: Vec<&str> = Vec::new();
    let mut supported = 0;
    for agent in AGENTS {
        match verify_agent(agent.id) {
            VerifyStatus::Ok => {
                installed += 1;
                supported += 1;
            }
            VerifyStatus::Drifted => {
                installed += 1;
                supported += 1;
                drifted.push(agent.id);
            }
            VerifyStatus::NotInstalled | VerifyStatus::NotDetected => {
                supported += 1;
            }
            VerifyStatus::Unsupported => {}
        }
    }
    if installed == 0 {
        return Check::warn("output saver", "output-saver not installed")
            .with_hint("`trs output-saver --install` adds anti-preamble rules to agent configs");
    }
    let label = if drifted.is_empty() {
        format!(
            "output-saver ({}/{} agents configured, content matches canonical)",
            installed, supported
        )
    } else {
        format!(
            "output-saver ({}/{} configured, {} drifted: {})",
            installed,
            supported,
            drifted.len(),
            drifted.join(", ")
        )
    };
    if drifted.is_empty() {
        Check::pass("output saver", label).with_hint("run `trs output-saver` to review or extend")
    } else {
        Check::warn("output saver", label)
            .with_hint("run `trs output-saver --refresh` to restore the canonical block")
    }
}

/// Scan cwd for agent instruction files and surface a token budget summary.
/// Always visible when any file exists — the `trs audit-docs` suggestion
/// belongs on the happy path too, not only when docs are already bloated.
fn check_agent_docs_health() -> Check {
    let Some(root) = std::env::current_dir().ok() else {
        return Check::pass("agent docs", "no cwd");
    };

    // Keep in sync with audit_docs::KNOWN_PATHS for the single-file entries.
    const DOC_PATHS: &[&str] = &["CLAUDE.md", "AGENTS.md", "GEMINI.md", ".windsurfrules"];
    const BLOAT_TOKENS: usize = 5000;

    let mut found: Vec<(String, usize)> = Vec::new();
    for rel in DOC_PATHS {
        let path = root.join(rel);
        if let Ok(content) = std::fs::read_to_string(&path) {
            let tokens = crate::audit_docs::estimate_tokens(&content);
            found.push((rel.to_string(), tokens));
        }
    }

    if found.is_empty() {
        // No agent docs in cwd — don't clutter output.
        return Check::pass("agent docs", "no agent docs in cwd");
    }

    let total_tokens: usize = found.iter().map(|(_, t)| t).sum();
    let bloated: Vec<&(String, usize)> = found.iter().filter(|(_, t)| *t > BLOAT_TOKENS).collect();

    let summary = format!(
        "{} file{}, {} tokens loaded per agent session",
        found.len(),
        if found.len() == 1 { "" } else { "s" },
        human_k(total_tokens)
    );

    if bloated.is_empty() {
        // Healthy — still surface audit-docs so the user knows it exists.
        Check::pass("agent docs", summary)
            .with_hint("run `trs audit-docs` to review duplicates / dead refs / embedded bloat")
    } else {
        let bloat_detail = bloated
            .iter()
            .map(|(name, t)| format!("{} ({})", name, human_k(*t)))
            .collect::<Vec<_>>()
            .join(", ");
        Check::warn(
            "agent docs",
            format!("{} — oversized: {}", summary, bloat_detail),
        )
        .with_hint("run `trs audit-docs` to find duplicates / dead refs / embedded bloat")
    }
}

fn human_k(n: usize) -> String {
    if n >= 1000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}

/// Print doctor results in spark-style format.
pub(crate) fn print_report(checks: &[Check]) {
    println!();
    println!("  TRS Doctor \u{2014} Installation Health Check");
    println!();

    for check in checks {
        let marker = match check.status {
            CheckStatus::Pass => "\u{2713}", // ✓
            CheckStatus::Warn => "~",
            CheckStatus::Fail => "\u{2717}", // ✗
        };

        if check.hint.is_empty() {
            println!("  {} {}", marker, check.detail);
        } else {
            println!("  {} {}  \u{2192} {}", marker, check.detail, check.hint);
        }

        for line in &check.sub {
            println!("    {}", line);
        }
    }

    let s = Summary::from_checks(checks);

    println!();
    println!("  {}", "\u{2500}".repeat(35));
    println!(
        "  {} passed   {} failed   {} warnings",
        s.pass, s.fail, s.warn
    );

    if s.fail > 0 {
        println!();
        println!("  Run the suggested commands to fix issues.");
    }

    println!();
    println!("  More: https://github.com/dPeluChe/trs/blob/main/docs/features/doctor.md");
}

/// Print doctor results in JSON format.
pub(crate) fn print_report_json(checks: &[Check]) {
    let entries: Vec<serde_json::Value> = checks
        .iter()
        .map(|c| {
            serde_json::json!({
                "name": c.name,
                "status": c.status.to_string().to_lowercase(),
                "detail": c.detail,
                "hint": c.hint,
            })
        })
        .collect();

    let s = Summary::from_checks(checks);

    let report = serde_json::json!({
        "checks": entries,
        "summary": {
            "total": s.total,
            "pass": s.pass,
            "fail": s.fail,
            "warn": s.warn,
            "healthy": s.fail == 0,
        }
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&report).unwrap_or_default()
    );
}

// ============================================================
// Individual checks
// ============================================================

/// Check: trs binary — version and path.
fn check_version() -> Check {
    let version = env!("CARGO_PKG_VERSION");
    let path = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    Check::pass("trs binary", "trs binary").with_sub(vec![
        format!("version: {}", version),
        format!("path:    {}", path),
    ])
}

/// Check: `trs` is findable in PATH (via `which -a` / `where` to catch duplicates).
fn check_path_accessible() -> Check {
    // `which -a` on Unix and `where` on Windows both print every match in PATH.
    let (cmd, args): (&str, &[&str]) = if cfg!(windows) {
        ("where", &["trs"])
    } else {
        ("which", &["-a", "trs"])
    };
    match Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let paths: Vec<String> = stdout
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
            if paths.is_empty() {
                return Check::fail("PATH", "trs not found in PATH")
                    .with_hint("curl -fsSL https://usetrs.dev/install.sh | sh");
            }
            let primary = paths[0].clone();
            if paths.len() == 1 {
                Check::pass("PATH", "trs in PATH").with_sub(vec![format!("path: {}", primary)])
            } else {
                let mut sub = vec![format!("active: {}", primary)];
                for p in paths.iter().skip(1) {
                    sub.push(format!("shadowed: {}", p));
                }
                sub.push(format!(
                    "{} trs binaries in PATH — the first one wins",
                    paths.len()
                ));
                Check::warn("PATH", "multiple trs binaries found")
                    .with_sub(sub)
                    .with_hint("uninstall the duplicates (npm uninstall -g @dpeluche/trs / cargo uninstall trs-cli / brew uninstall trs) or reorder PATH")
            }
        }
        _ => Check::fail("PATH", "trs not found in PATH")
            .with_hint("curl -fsSL https://usetrs.dev/install.sh | sh"),
    }
}

/// Check if a dependency command exists and return its version.
fn check_dep(cmd: &str, label: &str, required: bool, hint: &str) -> Check {
    let name: &'static str = match cmd {
        "git" => "dep:git",
        "rg" => "dep:rg",
        _ => "dep:other",
    };
    match Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        Ok(out) if out.status.success() => {
            let ver = String::from_utf8_lossy(&out.stdout);
            let first_line = ver.lines().next().unwrap_or("").trim().to_string();
            Check::pass(name, format!("{} available", label))
                .with_sub(vec![format!("version: {}", first_line)])
        }
        _ => {
            let mut c = if required {
                Check::fail(name, format!("{} not found", label))
            } else {
                Check::warn(name, format!("{} not found", label))
            };
            c.hint = hint.to_string();
            c
        }
    }
}

/// Check: ~/.trs/ directory exists or can be created.
fn check_config_dir() -> Check {
    let Some(home) = crate::tracker::home_dir() else {
        return Check::warn("config dir", "HOME not set")
            .with_hint("set HOME environment variable");
    };

    let dir = home.join(".trs");
    if dir.exists() && dir.is_dir() {
        Check::pass("config dir", "config directory").with_sub(vec![dir.display().to_string()])
    } else if !dir.exists() {
        match std::fs::create_dir_all(&dir) {
            Ok(_) => Check::pass("config dir", "config directory (created)")
                .with_sub(vec![dir.display().to_string()]),
            Err(e) => Check::fail("config dir", format!("cannot create: {}", e))
                .with_hint(format!("mkdir -p {}", dir.display())),
        }
    } else {
        Check::fail("config dir", "path exists but is not a directory").with_hint(format!(
            "rm {} && mkdir -p {}",
            dir.display(),
            dir.display()
        ))
    }
}

/// Check: history.jsonl is writable.
fn check_history_writable() -> Check {
    let Some(home) = crate::tracker::home_dir() else {
        return Check::warn("history", "HOME not set");
    };

    let probe = home.join(".trs").join(".doctor_probe");
    match std::fs::write(&probe, "ok") {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            let history = home.join(".trs").join("history.jsonl");
            if history.exists() {
                let size = std::fs::metadata(&history).map(|m| m.len()).unwrap_or(0);
                let human = crate::tracker::format_bytes_human(size as usize);
                Check::pass("history", "history writable")
                    .with_sub(vec![format!("size: {} tracked", human)])
            } else {
                Check::pass("history", "history writable (no history yet)")
            }
        }
        Err(e) => Check::fail("history", format!("~/.trs/ not writable: {}", e))
            .with_hint("check permissions on ~/.trs/"),
    }
}

/// Check: stdin pipeline works (pipe "hello" through `trs clean`).
fn check_stdin_pipeline() -> Check {
    let trs = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => PathBuf::from("trs"),
    };

    match Command::new(&trs)
        .args(["clean"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                let _ = stdin.write_all(b"doctor probe\n");
            }
            match child.wait_with_output() {
                Ok(out) if out.status.success() => {
                    let output = String::from_utf8_lossy(&out.stdout);
                    if output.contains("doctor") || output.contains("probe") {
                        Check::pass("stdin pipe", "stdin pipeline functional")
                    } else {
                        Check::warn(
                            "stdin pipe",
                            format!("unexpected output: {}", output.trim()),
                        )
                    }
                }
                Ok(out) => Check::fail(
                    "stdin pipe",
                    format!("exit code {}", out.status.code().unwrap_or(-1)),
                ),
                Err(e) => Check::fail("stdin pipe", format!("failed: {}", e)),
            }
        }
        Err(e) => Check::fail("stdin pipe", format!("cannot spawn: {}", e))
            .with_hint("verify trs binary is executable"),
    }
}

/// Check: are any AI tool hooks installed? Delegates to init.rs.
fn check_hooks_installed() -> Check {
    let tools = AiTool::all_tools();
    let total = tools.len();
    let hooks_found = tools.iter().filter(|t| check_tool(t)).count();
    if hooks_found > 0 {
        Check::pass(
            "hooks",
            format!("AI tool hooks ({}/{} configured)", hooks_found, total),
        )
    } else {
        Check::warn("hooks", "no AI tool hooks installed")
            .with_hint("trs init --all  (or trs init <tool>)")
    }
}

#[cfg(test)]
#[path = "doctor_tests.rs"]
mod tests;
