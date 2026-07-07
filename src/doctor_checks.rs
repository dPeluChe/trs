//! Individual `trs doctor` checks. The `Check` type, dispatcher (`run_checks`),
//! and report rendering live in `doctor.rs`.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::doctor::Check;

use crate::init::{check_tool, AiTool};

/// Validate the `~/.codex/hooks.json` trs entry against the codex version.
/// On codex-cli >= 0.134 the `trs rewrite` PreToolUse hook is the real,
/// working integration (passes). On older builds `updatedInput` is rejected
/// ("unsupported updatedInput" on every tool call), so a trs entry there is
/// an orphan — warn and point at `trs uninstall codex`.
pub(crate) fn check_codex_hooks_orphan() -> Check {
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
        return Check::pass("codex hooks.json", "no trs entry".to_string());
    }
    let version = crate::codex::detect_version();
    let ver_label = version
        .map(|(a, b, c)| format!("codex-cli {a}.{b}.{c}"))
        .unwrap_or_else(|| "version unknown".to_string());
    // On 0.134+ the trs PreToolUse hook is the real, working integration —
    // not an orphan. Only flag it on builds that reject `updatedInput`.
    if version.is_some_and(crate::codex::rewrite_hook_supported) {
        return Check::pass(
            "codex hooks.json",
            format!("trs rewrite hook active ({ver_label})"),
        );
    }
    Check::warn(
        "codex hooks.json",
        format!(
            "`trs rewrite` entry in {} but {}",
            path.display(),
            ver_label
        ),
    )
    .with_hint(
        "This codex build doesn't apply PreToolUse `updatedInput` rewrites \
         (needs >= 0.134), so the hook errors with \"unsupported \
         updatedInput\". Update codex, or run `trs uninstall codex` to scrub \
         and fall back to AGENTS.md rules.",
    )
}

/// Report the Devin CLI trs hook. Unlike Codex there's no version→feature
/// map to gate on, so this can't confirm the runtime honors
/// `updatedInput` — it only surfaces that the hook is wired and flags that
/// live validation is still pending (2026-07 docs research). Passes when
/// the `trs rewrite` entry is present in `~/.config/devin/config.json` or a
/// project `.devin/config.json`; silent-pass when Devin CLI isn't detected.
pub(crate) fn check_devin_cli_hook() -> Check {
    use std::fs;
    if !AiTool::DevinCLI.detect_installed() {
        return Check::pass(
            "devin-cli hook",
            "Devin CLI not detected — skipped".to_string(),
        );
    }
    let mut paths: Vec<PathBuf> = vec![PathBuf::from(".devin/config.json")];
    if let Ok(home) = crate::init::home_dir() {
        paths.insert(0, home.join(".config/devin/config.json"));
    }
    let wired = paths.iter().any(|p| {
        fs::read_to_string(p)
            .map(|c| c.contains("trs rewrite"))
            .unwrap_or(false)
    });
    if !wired {
        return Check::warn(
            "devin-cli hook",
            "Devin CLI detected but trs hook not installed",
        )
        .with_hint("trs init devin-cli --global");
    }
    Check::pass(
        "devin-cli hook",
        "trs rewrite hook wired (exec matcher)".to_string(),
    )
    .with_hint(
        "Validated live 2026-07-07. If runs show up as `claude` in `trs stats`, \
         set `read_config_from.claude: false` in ~/.config/devin/config.json so \
         the devin-cli hook wins over the transitive Claude hook.",
    )
}

/// Count how many of the supported agents have the trs output-saver
/// block installed AND whether the content matches the current canonical
/// template. Drift (manual edits, stale content from older installs)
/// surfaces as a warning so users know to run `--refresh`.
pub(crate) fn check_output_saver_installed() -> Check {
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
pub(crate) fn check_agent_docs_health() -> Check {
    let Some(root) = std::env::current_dir().ok() else {
        return Check::pass("agent docs", "no cwd");
    };

    // Keep in sync with audit_docs::KNOWN_PATHS for the single-file entries.
    const DOC_PATHS: &[&str] = &[
        "CLAUDE.md",
        "AGENTS.md",
        "GEMINI.md",
        ".windsurfrules",
        ".devin/rules/trs.md",
    ];
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

/// Check: trs binary — version and path.
pub(crate) fn check_version() -> Check {
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
pub(crate) fn check_path_accessible() -> Check {
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
            // Dedupe: `which -a` lists one hit per PATH entry, so a directory
            // that appears N times in $PATH yields N identical lines. That's a
            // duplicate-PATH-entry smell, not duplicate binaries.
            let mut unique = paths.clone();
            unique.sort();
            unique.dedup();
            if unique.len() == 1 {
                if paths.len() == 1 {
                    Check::pass("PATH", "trs in PATH").with_sub(vec![format!("path: {}", primary)])
                } else {
                    let dir = std::path::Path::new(&primary)
                        .parent()
                        .map(|d| d.display().to_string())
                        .unwrap_or_else(|| primary.clone());
                    Check::warn("PATH", "duplicate PATH entries")
                        .with_sub(vec![
                            format!("path: {}", primary),
                            format!("{} is listed {} times in $PATH", dir, paths.len()),
                        ])
                        .with_hint(
                            "one trs binary, but its directory repeats in your shell PATH \
                             config — remove the redundant `export PATH=` lines (commonly in \
                             ~/.zshrc / ~/.zprofile when ~/.zshenv already adds it).",
                        )
                }
            } else {
                let mut sub = vec![format!("active: {}", primary)];
                for p in unique.iter().filter(|p| **p != primary) {
                    sub.push(format!("shadowed: {}", p));
                }
                sub.push(format!(
                    "{} distinct trs binaries in PATH — the first one wins",
                    unique.len()
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
pub(crate) fn check_dep(cmd: &str, label: &str, required: bool, hint: &str) -> Check {
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
pub(crate) fn check_config_dir() -> Check {
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
pub(crate) fn check_history_writable() -> Check {
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
pub(crate) fn check_stdin_pipeline() -> Check {
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
pub(crate) fn check_hooks_installed() -> Check {
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
