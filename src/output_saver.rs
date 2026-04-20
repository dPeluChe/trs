//! `trs output-saver` — install a compact set of output-reduction rules
//! into the user's agent configs.
//!
//! trs already compresses what agents SEE (command output → compact form
//! via `trs rewrite`). This module handles the symmetric problem: what
//! agents EMIT. By dropping a short rules block into each agent's global
//! config we steer replies away from preambles ("Sure!"), narration
//! ("Now I will…"), and other low-signal filler.
//!
//! Design choices worth knowing:
//!
//! - **Check-first, install-on-demand.** Plain `trs output-saver` runs a
//!   read-only scan and prints what would change. `--install` is the
//!   explicit opt-in. This mirrors what audit-docs does for bloat — we
//!   never silently rewrite someone's config.
//! - **Imports where possible, inline where not.** Claude Code and
//!   Gemini CLI support `@file.md` imports, so we drop a standalone
//!   file and append one import line. Codex and Windsurf are plain
//!   markdown — we append the block wrapped in HTML-comment sentinels
//!   so re-install is idempotent.
//! - **Skip plugin-based agents.** Droid/OpenCode/Kilo/Antigravity have
//!   no global rules mechanism we can target without writing into user
//!   project files. We say so explicitly rather than pretending.

use std::fs;
use std::path::PathBuf;

/// The rules block itself — exposed as a macro so `init_templates.rs`
/// can splice it into `const &str` templates via `concat!`. Const-valued
/// `&str` constants cannot be composed in `concat!` across modules in
/// Rust, hence the macro form. `BLOCK` below re-materializes the same
/// literal for runtime callers.
#[macro_export]
macro_rules! output_saver_block_literal {
    () => {
        r#"## Output saver — keep replies cheap

These rules reduce tokens on every agent reply:

- No preambles. Don't open with "Sure!", "Great question!", "Absolutely!",
  "I'll help you...", or "You're absolutely right!". Start with the answer.
- No narration. Don't announce what you're about to do or recap what you
  just did — the diff / tool output already shows it.
- Result first; explanation only if non-obvious. State the finding, show
  the fix, stop.
- Structured output when the data is structured: bullets, tables, JSON.
  Prose only when the reader is human and the content is narrative.
- Never invent file paths, function names, or API fields. If unknown,
  say "UNKNOWN" or return null — guessing costs more tokens than asking.
- One pass: don't iterate on passing code, don't refactor / polish unless
  asked.

User instructions always override these rules."#
    };
}

pub(crate) const BLOCK: &str = output_saver_block_literal!();

const SENTINEL_START: &str = "<!-- trs:output-saver:start v1 -->";
const SENTINEL_END: &str = "<!-- trs:output-saver:end -->";

/// Import filename used by Claude Code and Gemini CLI when we install
/// as a standalone file + `@import` line.
const IMPORT_FILENAME: &str = "trs-output-saver.md";

/// Wrap the block with a banner suitable for a standalone file (used
/// when the agent supports `@imports`).
fn standalone_file() -> String {
    format!(
        "# trs — output saver\n\n\
         Installed by `trs output-saver --install`. Remove with\n\
         `trs output-saver --remove` or delete this file plus the\n\
         `@{}` import line in the parent config.\n\n{}\n",
        IMPORT_FILENAME, BLOCK
    )
}

/// Wrap the block in sentinels for idempotent inline installs.
fn sentinel_wrapped() -> String {
    format!("\n{}\n\n{}\n\n{}\n", SENTINEL_START, BLOCK, SENTINEL_END)
}

/// Which global-config location we target per agent. `None` = the agent
/// has no global rules mechanism we can safely write to; we skip it.
#[derive(Debug, Clone)]
enum Target {
    /// Write `trs-output-saver.md` in `dir` and append `@trs-output-saver.md`
    /// to `dir/root_file` (creating the root file if missing).
    Imported { dir: PathBuf, root_file: String },
    /// Drop a standalone file into a rules directory that the agent
    /// auto-loads. No root-config edit needed.
    RulesDir { path: PathBuf },
    /// Append the block inline to a single rules file, wrapped in
    /// sentinels for idempotent re-installs.
    InlineFile { path: PathBuf },
    /// Not supported for this agent. Carries the reason for display.
    NotSupported { reason: &'static str },
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Agent {
    pub id: &'static str,
    pub display: &'static str,
}

pub(crate) const AGENTS: &[Agent] = &[
    Agent {
        id: "claude",
        display: "Claude Code",
    },
    Agent {
        id: "gemini",
        display: "Gemini CLI",
    },
    Agent {
        id: "cursor",
        display: "Cursor",
    },
    Agent {
        id: "codex",
        display: "Codex",
    },
    Agent {
        id: "windsurf",
        display: "Windsurf",
    },
    Agent {
        id: "droid",
        display: "Factory Droid",
    },
    Agent {
        id: "opencode",
        display: "OpenCode",
    },
    Agent {
        id: "kilo",
        display: "Kilo Code",
    },
    Agent {
        id: "antigravity",
        display: "Antigravity",
    },
];

/// Resolve the install target for `agent_id`. `home` is injectable so
/// parallel tests don't race on `std::env::set_var("HOME", …)`;
/// production wrappers pass `env("HOME")` directly.
fn resolve_target_with_home(agent_id: &str, home: Option<&std::path::Path>) -> Target {
    let push_home = |rel: &str| home.map(|h| h.join(rel));

    match agent_id {
        "claude" => push_home(".claude")
            .map(|dir| Target::Imported {
                dir,
                root_file: "CLAUDE.md".into(),
            })
            .unwrap_or(Target::NotSupported {
                reason: "HOME not set",
            }),
        "gemini" => push_home(".gemini")
            .map(|dir| Target::Imported {
                dir,
                root_file: "GEMINI.md".into(),
            })
            .unwrap_or(Target::NotSupported {
                reason: "HOME not set",
            }),
        "cursor" => push_home(".cursor/rules/trs-output-saver.mdc")
            .map(|path| Target::RulesDir { path })
            .unwrap_or(Target::NotSupported {
                reason: "HOME not set",
            }),
        "codex" => push_home(".codex/AGENTS.md")
            .map(|path| Target::InlineFile { path })
            .unwrap_or(Target::NotSupported {
                reason: "HOME not set",
            }),
        "windsurf" => push_home(".codeium/windsurf/memories/global_rules.md")
            .map(|path| Target::InlineFile { path })
            .unwrap_or(Target::NotSupported {
                reason: "HOME not set",
            }),
        // OpenCode's plugin API exposes tool-level hooks only (no
        // prompt-mutation hook) — but it auto-loads
        // `~/.config/opencode/AGENTS.md` globally per the rules docs, so
        // we reach the LLM through that path instead.
        "opencode" => push_home(".config/opencode/AGENTS.md")
            .map(|path| Target::InlineFile { path })
            .unwrap_or(Target::NotSupported {
                reason: "HOME not set",
            }),
        // Kilo Code is a fork of OpenCode sharing the same loader — it
        // auto-loads `~/.config/kilo/AGENTS.md` (confirmed in its
        // session/instruction.ts). Kilo does expose an
        // `experimental.chat.system.transform` plugin hook we could use
        // for dynamic injection later, but the static AGENTS.md path
        // isn't marked experimental so we prefer it.
        "kilo" => push_home(".config/kilo/AGENTS.md")
            .map(|path| Target::InlineFile { path })
            .unwrap_or(Target::NotSupported {
                reason: "HOME not set",
            }),
        // Factory Droid is an official AGENTS.md adopter — the CLI
        // auto-loads `~/.factory/AGENTS.md` as the global fallback
        // (project AGENTS.md files override it). Same pattern as
        // Codex/OpenCode/Kilo. Reference:
        //   https://docs.factory.ai/cli/configuration/agents-md
        "droid" => push_home(".factory/AGENTS.md")
            .map(|path| Target::InlineFile { path })
            .unwrap_or(Target::NotSupported {
                reason: "HOME not set",
            }),
        "antigravity" => Target::NotSupported {
            reason: "rules are per-project only — run `trs init antigravity`",
        },
        _ => Target::NotSupported {
            reason: "unknown agent",
        },
    }
}

/// What the scan found for an agent.
#[derive(Debug, Clone)]
pub(crate) enum Status {
    /// Agent's config dir doesn't exist on this system — probably not
    /// installed. Skip.
    NotDetected,
    /// Install would write new content.
    NotInstalled,
    /// Our sentinels or import line are already present — re-install
    /// will be idempotent.
    AlreadyInstalled,
    /// Couldn't target this agent (plugin-based or unknown).
    Unsupported { reason: &'static str },
}

pub(crate) fn scan_agent(agent_id: &str) -> Status {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    scan_agent_with_home(agent_id, home.as_deref())
}

fn scan_agent_with_home(agent_id: &str, home: Option<&std::path::Path>) -> Status {
    let target = resolve_target_with_home(agent_id, home);
    match target {
        Target::NotSupported { reason } => Status::Unsupported { reason },
        Target::Imported { dir, root_file } => {
            if !dir.exists() {
                return Status::NotDetected;
            }
            let root = dir.join(&root_file);
            let saver = dir.join(IMPORT_FILENAME);
            let has_import = fs::read_to_string(&root)
                .map(|c| c.contains(&format!("@{}", IMPORT_FILENAME)))
                .unwrap_or(false);
            if has_import && saver.exists() {
                Status::AlreadyInstalled
            } else {
                Status::NotInstalled
            }
        }
        Target::RulesDir { path } => {
            let parent = path.parent();
            if parent.is_none_or(|p| !p.exists())
                && !path.ancestors().nth(2).map(|p| p.exists()).unwrap_or(false)
            {
                return Status::NotDetected;
            }
            if path.exists() {
                Status::AlreadyInstalled
            } else {
                Status::NotInstalled
            }
        }
        Target::InlineFile { path } => {
            let parent = path.parent();
            if parent.is_none_or(|p| !p.exists()) {
                return Status::NotDetected;
            }
            let has_sentinels = fs::read_to_string(&path)
                .map(|c| c.contains(SENTINEL_START) && c.contains(SENTINEL_END))
                .unwrap_or(false);
            if has_sentinels {
                Status::AlreadyInstalled
            } else {
                Status::NotInstalled
            }
        }
    }
}

/// Install for `agent_id`. Returns a human-readable one-line action
/// description on success (e.g. "wrote ~/.claude/trs-output-saver.md +
/// @import"); or `Err` with a reason.
pub(crate) fn install_agent(agent_id: &str) -> Result<String, String> {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    install_agent_with_home(agent_id, home.as_deref())
}

fn install_agent_with_home(
    agent_id: &str,
    home: Option<&std::path::Path>,
) -> Result<String, String> {
    let target = resolve_target_with_home(agent_id, home);
    match target {
        Target::NotSupported { reason } => Err(reason.to_string()),
        Target::Imported { dir, root_file } => {
            fs::create_dir_all(&dir).map_err(|e| format!("{}: {}", dir.display(), e))?;
            let saver_path = dir.join(IMPORT_FILENAME);
            fs::write(&saver_path, standalone_file())
                .map_err(|e| format!("{}: {}", saver_path.display(), e))?;

            let root_path = dir.join(&root_file);
            let import_line = format!("@{}", IMPORT_FILENAME);
            let existing = fs::read_to_string(&root_path).unwrap_or_default();
            if !existing.contains(&import_line) {
                let sep = if existing.is_empty() || existing.ends_with('\n') {
                    ""
                } else {
                    "\n"
                };
                let updated = format!("{}{}{}\n", existing, sep, import_line);
                fs::write(&root_path, updated)
                    .map_err(|e| format!("{}: {}", root_path.display(), e))?;
            }
            Ok(format!(
                "wrote {} and ensured {} imports it",
                saver_path.display(),
                root_path.display()
            ))
        }
        Target::RulesDir { path } => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("{}: {}", parent.display(), e))?;
            }
            fs::write(&path, standalone_file())
                .map_err(|e| format!("{}: {}", path.display(), e))?;
            Ok(format!("wrote {}", path.display()))
        }
        Target::InlineFile { path } => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("{}: {}", parent.display(), e))?;
            }
            let existing = fs::read_to_string(&path).unwrap_or_default();
            let updated = if existing.contains(SENTINEL_START) && existing.contains(SENTINEL_END) {
                replace_between(&existing, SENTINEL_START, SENTINEL_END, &sentinel_wrapped())
            } else if existing.is_empty() {
                sentinel_wrapped().trim_start().to_string()
            } else {
                format!("{}{}", existing.trim_end(), sentinel_wrapped())
            };
            fs::write(&path, updated).map_err(|e| format!("{}: {}", path.display(), e))?;
            Ok(format!("updated {}", path.display()))
        }
    }
}

/// Remove our install for `agent_id`. Deletes the standalone file +
/// import line (for Imported / RulesDir) or the sentinel block (for
/// InlineFile). No-op when nothing was installed.
pub(crate) fn remove_agent(agent_id: &str) -> Result<String, String> {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    remove_agent_with_home(agent_id, home.as_deref())
}

fn remove_agent_with_home(
    agent_id: &str,
    home: Option<&std::path::Path>,
) -> Result<String, String> {
    let target = resolve_target_with_home(agent_id, home);
    match target {
        Target::NotSupported { reason } => Err(reason.to_string()),
        Target::Imported { dir, root_file } => {
            let saver_path = dir.join(IMPORT_FILENAME);
            if saver_path.exists() {
                fs::remove_file(&saver_path)
                    .map_err(|e| format!("{}: {}", saver_path.display(), e))?;
            }
            let root_path = dir.join(&root_file);
            if let Ok(existing) = fs::read_to_string(&root_path) {
                let import_line = format!("@{}", IMPORT_FILENAME);
                let stripped: String = existing
                    .lines()
                    .filter(|l| l.trim() != import_line)
                    .collect::<Vec<_>>()
                    .join("\n");
                let final_content = if existing.ends_with('\n') && !stripped.is_empty() {
                    format!("{}\n", stripped)
                } else {
                    stripped
                };
                fs::write(&root_path, final_content)
                    .map_err(|e| format!("{}: {}", root_path.display(), e))?;
            }
            Ok(format!("removed {} and import line", saver_path.display()))
        }
        Target::RulesDir { path } => {
            if path.exists() {
                fs::remove_file(&path).map_err(|e| format!("{}: {}", path.display(), e))?;
                Ok(format!("removed {}", path.display()))
            } else {
                Ok(format!("nothing to remove at {}", path.display()))
            }
        }
        Target::InlineFile { path } => {
            let existing = fs::read_to_string(&path).unwrap_or_default();
            if existing.contains(SENTINEL_START) && existing.contains(SENTINEL_END) {
                let stripped = replace_between(&existing, SENTINEL_START, SENTINEL_END, "");
                fs::write(&path, stripped.trim_end_matches('\n').to_string() + "\n")
                    .map_err(|e| format!("{}: {}", path.display(), e))?;
                Ok(format!(
                    "removed output-saver block from {}",
                    path.display()
                ))
            } else {
                Ok(format!("no output-saver block in {}", path.display()))
            }
        }
    }
}

/// Replace the segment between `start` and `end` sentinels (inclusive)
/// with `new_block`. Assumes both sentinels are present — caller must
/// verify. Normalizes whitespace around the splice so repeated calls
/// don't accumulate trailing newlines.
fn replace_between(content: &str, start: &str, end: &str, new_block: &str) -> String {
    let Some(s) = content.find(start) else {
        return content.to_string();
    };
    let Some(e) = content[s..].find(end).map(|pos| s + pos + end.len()) else {
        return content.to_string();
    };
    let before = &content[..s];
    // Strip any newlines sitting between the end sentinel and the rest
    // of the file — otherwise each install/replace cycle appends
    // another \n.
    let after = content[e..].trim_start_matches('\n');
    let combined = format!("{}{}{}", before, new_block.trim_start(), after);
    // Exactly one trailing newline, no more.
    format!("{}\n", combined.trim_end_matches('\n'))
}

/// Entry point called from main.rs. The four modes are mutually
/// exclusive — `--print` wins, then `--remove`, then `--install`,
/// default is a read-only scan.
pub(crate) fn run(agent: Option<&str>, install: bool, remove: bool, print: bool) {
    if print {
        println!("{}", BLOCK);
        return;
    }

    let targets: Vec<&str> = match agent {
        Some(a) => vec![a],
        None => AGENTS.iter().map(|a| a.id).collect(),
    };

    if remove {
        run_remove(&targets);
        return;
    }

    if install {
        run_install(&targets);
        return;
    }

    run_scan(&targets);
}

fn agent_display(id: &str) -> &'static str {
    AGENTS
        .iter()
        .find(|a| a.id == id)
        .map(|a| a.display)
        .unwrap_or("<unknown>")
}

fn run_scan(targets: &[&str]) {
    println!("trs output-saver — scan\n");
    let mut installable = Vec::new();
    let mut already = 0;
    let mut unsupported = 0;
    let mut not_detected = 0;

    for id in targets {
        let display = agent_display(id);
        match scan_agent(id) {
            Status::AlreadyInstalled => {
                println!("  + {}  already installed", display);
                already += 1;
            }
            Status::NotInstalled => {
                println!("  . {}  not yet installed", display);
                installable.push(*id);
            }
            Status::NotDetected => {
                println!("  - {}  not detected on this system", display);
                not_detected += 1;
            }
            Status::Unsupported { reason } => {
                println!("  ~ {}  skipped ({})", display, reason);
                unsupported += 1;
            }
        }
    }

    println!();
    println!(
        "  {} installable, {} already installed, {} not detected, {} unsupported",
        installable.len(),
        already,
        not_detected,
        unsupported
    );

    if !installable.is_empty() {
        println!();
        println!("To install, re-run with --install:");
        if installable.len() == targets.len() || targets.len() == AGENTS.len() {
            println!("  trs output-saver --install");
        } else {
            for id in &installable {
                println!("  trs output-saver {} --install", id);
            }
        }
    }

    println!();
    println!("To see the block before installing:  trs output-saver --print");
    println!("To remove a previous install:         trs output-saver --remove");
    println!();
    println!("More: https://github.com/dPeluChe/trs/blob/main/docs/commands/output-saver.md");
}

fn run_install(targets: &[&str]) {
    println!("trs output-saver — install\n");
    let mut wrote = 0;
    let mut skipped = 0;
    for id in targets {
        let display = agent_display(id);
        match scan_agent(id) {
            Status::NotDetected => {
                println!("  - {}  skipped (not detected)", display);
                skipped += 1;
                continue;
            }
            Status::Unsupported { reason } => {
                println!("  ~ {}  skipped ({})", display, reason);
                skipped += 1;
                continue;
            }
            _ => {}
        }
        match install_agent(id) {
            Ok(msg) => {
                println!("  + {}  {}", display, msg);
                wrote += 1;
            }
            Err(e) => {
                eprintln!("  ! {}  install failed: {}", display, e);
            }
        }
    }
    println!();
    println!("  {} installed, {} skipped", wrote, skipped);
    if wrote > 0 {
        eprintln!(
            "note: restart any open agent sessions so the updated rules are \
             re-read from disk."
        );
    }
}

fn run_remove(targets: &[&str]) {
    println!("trs output-saver — remove\n");
    let mut removed = 0;
    let mut skipped = 0;
    for id in targets {
        let display = agent_display(id);
        // Skip unsupported agents quietly — nothing to remove means no
        // failure. Only report real I/O errors via the Err arm.
        if let Status::Unsupported { reason } = scan_agent(id) {
            println!("  ~ {}  skipped ({})", display, reason);
            skipped += 1;
            continue;
        }
        match remove_agent(id) {
            Ok(msg) => {
                println!("  + {}  {}", display, msg);
                removed += 1;
            }
            Err(e) => {
                eprintln!("  ! {}  remove failed: {}", display, e);
            }
        }
    }
    println!();
    println!("  {} removed, {} skipped", removed, skipped);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_between_swaps_segment() {
        let before =
            "pre\n<!-- trs:output-saver:start v1 -->\nold\n<!-- trs:output-saver:end -->\npost";
        let out = replace_between(before, SENTINEL_START, SENTINEL_END, "NEW_BLOCK");
        assert!(out.contains("NEW_BLOCK"));
        assert!(out.contains("pre"));
        assert!(out.contains("post"));
        assert!(!out.contains("old"));
    }

    #[test]
    fn standalone_file_contains_block() {
        let s = standalone_file();
        assert!(s.contains("Output saver"));
        assert!(s.contains("No preambles"));
    }

    #[test]
    fn sentinel_wrapped_is_idempotent_on_replace() {
        // Running replace_between with freshly-wrapped content should
        // preserve the block unchanged.
        let wrapped = sentinel_wrapped();
        let out = replace_between(&wrapped, SENTINEL_START, SENTINEL_END, &sentinel_wrapped());
        // Both retain the sentinels and the block.
        assert!(out.contains(SENTINEL_START));
        assert!(out.contains(SENTINEL_END));
        assert!(out.contains("No preambles"));
    }

    #[test]
    fn scan_unknown_agent_returns_unsupported() {
        let s = scan_agent("bogus");
        matches!(s, Status::Unsupported { .. });
    }

    #[test]
    fn install_and_remove_imported_agent_roundtrip() {
        let dir = std::env::temp_dir().join("trs_os_roundtrip_imported");
        let _ = fs::remove_dir_all(&dir);
        let home = dir.join("home");
        fs::create_dir_all(&home).unwrap();

        let res = install_agent_with_home("claude", Some(&home)).unwrap();
        assert!(res.contains("CLAUDE.md"));
        let claude_md = home.join(".claude/CLAUDE.md");
        let saver = home.join(".claude/trs-output-saver.md");
        assert!(claude_md.exists());
        assert!(saver.exists());
        assert!(fs::read_to_string(&claude_md)
            .unwrap()
            .contains("@trs-output-saver.md"));

        remove_agent_with_home("claude", Some(&home)).unwrap();
        assert!(!saver.exists());
        assert!(!fs::read_to_string(&claude_md)
            .unwrap()
            .contains("@trs-output-saver.md"));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn install_inline_file_is_idempotent() {
        let dir = std::env::temp_dir().join("trs_os_inline_idem");
        let _ = fs::remove_dir_all(&dir);
        let home = dir.join("home");
        fs::create_dir_all(home.join(".codex")).unwrap();
        let agents_path = home.join(".codex/AGENTS.md");
        fs::write(&agents_path, "# User agents\n\nCustom rules.\n").unwrap();

        install_agent_with_home("codex", Some(&home)).unwrap();
        let after1 = fs::read_to_string(&agents_path).unwrap();
        install_agent_with_home("codex", Some(&home)).unwrap();
        let after2 = fs::read_to_string(&agents_path).unwrap();
        assert_eq!(after1, after2, "second install mutated the file");
        assert!(after1.contains("Custom rules."));
        assert!(after1.contains("No preambles"));
        assert_eq!(
            after1.matches(SENTINEL_START).count(),
            1,
            "sentinel duplicated on re-install"
        );
        fs::remove_dir_all(&dir).ok();
    }
}
