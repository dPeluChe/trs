//! `trs doctor` — Validate trs installation health.
//!
//! Runs a series of checks to verify that trs is correctly installed and
//! all runtime dependencies are available. Designed for fresh installs,
//! CI pipelines, and debugging broken setups.

use std::fmt;

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
    pub(crate) fn pass(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Pass,
            detail: detail.into(),
            sub: Vec::new(),
            hint: String::new(),
        }
    }

    pub(crate) fn warn(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Warn,
            detail: detail.into(),
            sub: Vec::new(),
            hint: String::new(),
        }
    }

    pub(crate) fn fail(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Fail,
            detail: detail.into(),
            sub: Vec::new(),
            hint: String::new(),
        }
    }

    pub(crate) fn with_sub(mut self, lines: Vec<String>) -> Self {
        self.sub = lines;
        self
    }

    pub(crate) fn with_hint(mut self, hint: impl Into<String>) -> Self {
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

use crate::doctor_checks::*;

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

#[cfg(test)]
#[path = "doctor_tests.rs"]
mod tests;
