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
    ]
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

/// Check: `trs` is findable in PATH (via `which`/`where`).
fn check_path_accessible() -> Check {
    let cmd = if cfg!(windows) { "where" } else { "which" };
    match Command::new(cmd)
        .arg("trs")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Check::pass("PATH", "trs in PATH").with_sub(vec![format!("path: {}", path)])
        }
        _ => Check::fail("PATH", "trs not found in PATH")
            .with_hint("cargo install --path . or npm i -g tars-cli"),
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
    let hooks_found = tools.iter().filter(|t| check_tool(t)).count();
    if hooks_found > 0 {
        Check::pass(
            "hooks",
            format!("AI tool hooks ({}/6 configured)", hooks_found),
        )
    } else {
        Check::warn("hooks", "no AI tool hooks installed")
            .with_hint("trs init <tool>  (claude, gemini, cursor, codex, opencode, kilo)")
    }
}

#[cfg(test)]
#[path = "doctor_tests.rs"]
mod tests;
