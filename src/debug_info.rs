//! `trs debug-info` — bundle everything a bug report needs.
//!
//! When a user hits an issue, pointing them at five different commands
//! and two log paths is annoying. This command dumps one paste-ready
//! report: version, platform, doctor results, recent history, and the
//! most recent tee logs (output captured when trs itself failed).
//!
//! Output is plain text and goes to stdout by default; `--output PATH`
//! redirects to a file for users on slow terminals or when the report
//! is long.
//!
//! Privacy note: the report includes your working directories from
//! history.jsonl and the raw contents of any recent tee logs. Both can
//! contain file paths, command arguments, or — in the tee case — the
//! first N KB of whatever the failing command was processing. Users
//! should skim the output before pasting it publicly.

use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

use crate::doctor;
use crate::tracker;

/// Cap on history lines included so the report doesn't balloon on
/// long-lived installs.
const MAX_HISTORY_LINES: usize = 20;

/// Cap on tee files inspected; each is truncated separately.
const MAX_TEE_FILES: usize = 3;
const MAX_TEE_BYTES_PER_FILE: usize = 4000;

pub(crate) fn run(output: Option<&str>) {
    let report = build_report();
    match output {
        Some(path) => match fs::write(path, &report) {
            Ok(_) => {
                println!("wrote debug report to {}", path);
                println!();
                println!("Attach this file (or paste its contents) when opening an issue at");
                println!("https://github.com/dPeluChe/trs/issues");
            }
            Err(e) => {
                eprintln!("failed to write {}: {}", path, e);
                // Fall through and print to stdout as a backup.
                print!("{}", report);
            }
        },
        None => {
            print!("{}", report);
        }
    }
}

pub(crate) fn build_report() -> String {
    let mut out = String::new();
    out.push_str("=== trs debug-info ===\n");
    out.push_str(
        "Paste this whole block into a GitHub issue.\n\
         Review it first for any paths or data you don't want public.\n\n",
    );

    section(&mut out, "version", version_section());
    section(&mut out, "platform", platform_section());
    section(&mut out, "doctor", doctor_section());
    section(&mut out, "recent history", history_section());
    section(&mut out, "recent tee logs", tee_section());

    out.push_str("=== end debug-info ===\n");
    out
}

fn section(out: &mut String, title: &str, body: String) {
    out.push_str(&format!("--- {} ---\n", title));
    out.push_str(&body);
    if !body.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
}

fn version_section() -> String {
    let mut s = String::new();
    let _ = writeln!(s, "trs {}", env!("CARGO_PKG_VERSION"));
    if let Ok(path) = std::env::current_exe() {
        let _ = writeln!(s, "path: {}", path.display());
    }
    s
}

fn platform_section() -> String {
    let mut s = String::new();
    let _ = writeln!(s, "os: {}", std::env::consts::OS);
    let _ = writeln!(s, "arch: {}", std::env::consts::ARCH);
    // HOME is referenced by nearly every subsystem; flag if missing.
    let _ = writeln!(
        s,
        "HOME: {}",
        std::env::var("HOME").unwrap_or_else(|_| "(unset)".into())
    );
    let _ = writeln!(
        s,
        "PATH entries: {}",
        std::env::var("PATH")
            .map(|p| p.split(':').count())
            .unwrap_or(0)
    );
    // Shell — helps reproduce environment-specific bugs.
    let _ = writeln!(
        s,
        "SHELL: {}",
        std::env::var("SHELL").unwrap_or_else(|_| "(unset)".into())
    );
    s
}

fn doctor_section() -> String {
    // Reuse the doctor check runner and serialize each check line.
    // We don't call print_report_json because it writes straight to
    // stdout; we need the string for our combined report.
    let checks = doctor::run_checks();
    let mut s = String::new();
    for c in &checks {
        let marker = match c.status {
            doctor::CheckStatus::Pass => "PASS",
            doctor::CheckStatus::Warn => "WARN",
            doctor::CheckStatus::Fail => "FAIL",
        };
        let _ = writeln!(s, "[{}] {}", marker, c.detail);
        for sub in &c.sub {
            let _ = writeln!(s, "      {}", sub);
        }
        if !c.hint.is_empty() {
            let _ = writeln!(s, "      hint: {}", c.hint);
        }
    }
    s
}

fn history_section() -> String {
    let entries = tracker::read_history();
    if entries.is_empty() {
        return "(no history yet)\n".into();
    }
    let tail_start = entries.len().saturating_sub(MAX_HISTORY_LINES);
    let tail = &entries[tail_start..];
    let mut s = String::new();
    let _ = writeln!(s, "last {} of {} entries:", tail.len(), entries.len());
    for e in tail {
        let agent = e
            .agent
            .as_deref()
            .map(|a| format!(" agent={}", a))
            .unwrap_or_default();
        let cmd = truncate_for_report(&e.cmd, 90);
        let _ = writeln!(
            s,
            "  {} | {}B -> {}B ({}%){} | {}",
            e.ts, e.in_bytes, e.out_bytes, e.saved_pct, agent, cmd
        );
    }
    s
}

fn tee_section() -> String {
    let Some(tee_dir) = tracker::home_dir().map(|h| h.join(".trs/tee")) else {
        return "(HOME unavailable)\n".into();
    };
    if !tee_dir.exists() {
        return "(no tee directory yet, no failures recorded)\n".into();
    }

    let mut files: Vec<(PathBuf, std::time::SystemTime)> = fs::read_dir(&tee_dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter_map(|e| {
                    let path = e.path();
                    let mtime = e.metadata().ok()?.modified().ok()?;
                    if path.extension().and_then(|s| s.to_str()) == Some("log") {
                        Some((path, mtime))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    files.sort_by_key(|(_, mtime)| std::cmp::Reverse(*mtime));
    files.truncate(MAX_TEE_FILES);

    if files.is_empty() {
        return "(no .log files in ~/.trs/tee/)\n".into();
    }

    let mut s = String::new();
    for (path, _) in files {
        let _ = writeln!(s, "# {}", path.display());
        match fs::read_to_string(&path) {
            Ok(content) => {
                let truncated = if content.len() > MAX_TEE_BYTES_PER_FILE {
                    format!(
                        "{}\n... [truncated, original {} bytes] ...\n",
                        &content[..MAX_TEE_BYTES_PER_FILE],
                        content.len()
                    )
                } else {
                    content
                };
                s.push_str(&truncated);
                if !s.ends_with('\n') {
                    s.push('\n');
                }
            }
            Err(e) => {
                let _ = writeln!(s, "(could not read: {})", e);
            }
        }
        s.push('\n');
    }
    s
}

/// Collapse newlines and tabs to single spaces so each history row
/// stays on one line, then truncate for readability. Matches the
/// display policy used in `trs stats --history`.
fn truncate_for_report(cmd: &str, max_chars: usize) -> String {
    let single_line: String = cmd
        .chars()
        .map(|c| match c {
            '\n' | '\t' | '\r' => ' ',
            other => other,
        })
        .collect();
    if single_line.chars().count() <= max_chars {
        return single_line;
    }
    let truncated: String = single_line.chars().take(max_chars).collect();
    format!("{}…", truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_has_expected_sections() {
        let report = build_report();
        assert!(report.contains("--- version ---"));
        assert!(report.contains("--- platform ---"));
        assert!(report.contains("--- doctor ---"));
        assert!(report.contains("--- recent history ---"));
        assert!(report.contains("--- recent tee logs ---"));
        assert!(report.starts_with("=== trs debug-info ==="));
        assert!(report.trim_end().ends_with("=== end debug-info ==="));
    }

    #[test]
    fn truncate_folds_newlines() {
        let cmd = "python3 -c \"import json\nprint('hi')\"";
        let out = truncate_for_report(cmd, 200);
        assert!(!out.contains('\n'));
        assert!(out.contains("import json print"));
    }

    #[test]
    fn truncate_enforces_max() {
        let long = "a".repeat(200);
        let out = truncate_for_report(&long, 50);
        // 50 chars + the ellipsis char = 51 chars visually.
        assert!(out.chars().count() <= 51);
        assert!(out.ends_with('…'));
    }
}
