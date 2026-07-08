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
//!
//! The rules in `BLOCK` aren't arbitrary opinions — each one has either
//! an empirical source (Anthropic's leaked Claude Code system prompt,
//! Anthropic A/B-validated) or a research-backed prompt-engineering
//! principle (positive vs negative instructions, from pink-elephant
//! studies). For the per-rule provenance and what was deliberately
//! NOT included, see `docs/features/output-saver.md` § "Why these
//! rules — research backing".

/// The rules block itself — exposed as a macro so `init_templates.rs`
/// can splice it into `const &str` templates via `concat!`. Const-valued
/// `&str` constants cannot be composed in `concat!` across modules in
/// Rust, hence the macro form. `BLOCK` below re-materializes the same
/// literal for runtime callers.
#[macro_export]
macro_rules! output_saver_block_literal {
    () => {
        r#"## Output saver — keep replies cheap

Keep replies under ~100 words unless the task needs more. Between tool
calls, stay under ~25 words. Match shape to task — a one-line question
gets a one-line answer, no headers.

Open with the answer or the diff. End when the answer ends.

- Result first; explanation only if non-obvious. State the finding, show
  the fix, stop.
- Let tool output speak for itself; don't restate or recap what the diff
  already shows.
- Structured output when the data is structured: bullets, tables, JSON.
  Prose only when the reader is human and the content is narrative.
- Never invent file paths, function names, or API fields. If unknown,
  say "UNKNOWN" or return null — guessing costs more tokens than asking.
- One pass: don't iterate on passing code, don't refactor / polish unless
  asked.
- In code: no comments by default. If one is truly needed, write a terse
  reference note for the WHY (not a walkthrough) — at most 3 lines and
  ~200 characters total. Never paragraph docstrings or restating the code.

User instructions always override these rules."#
    };
}

pub(crate) const BLOCK: &str = output_saver_block_literal!();

pub(crate) const SENTINEL_START: &str = "<!-- trs:output-saver:start v1 -->";
pub(crate) const SENTINEL_END: &str = "<!-- trs:output-saver:end -->";

/// Import filename used by Claude Code and Gemini CLI when we install
/// as a standalone file + `@import` line.
pub(crate) const IMPORT_FILENAME: &str = "trs.md";

/// Previous filename — migrated to IMPORT_FILENAME on install/refresh.
pub(crate) const IMPORT_FILENAME_LEGACY: &str = "trs-output-saver.md";

/// Wrap the block with a banner suitable for a standalone file (used
/// when the agent supports `@imports`).
///
/// Hook-context only. This file lands in agents whose shell output is
/// already routed through trs by a pre-tool hook (Claude, Gemini,
/// Cursor; same shape used for InlineFile agents whose AGENTS.md is
/// auto-loaded). The agent doesn't need to know how to invoke trs —
/// the hook handles every command transparently. Earlier versions of
/// this template documented `trs raw` and `TRS_SKIP=1` as escape
/// hatches; in practice agents reached for them defensively on
/// routine commands (`TRS_SKIP=1 grep …`, `TRS_SKIP=1 npm test`),
/// throwing away 60–99% of the savings the hook just bought. Mere
/// visibility of a bypass option created the temptation, so we no
/// longer mention them. One short defensive line keeps the agent
/// from misreading compact output as garbled — that's the entire
/// trs-awareness budget. Bypass mechanisms still exist for humans
/// (see `trs --help` / public docs); they just aren't promoted to
/// the model. No-hook agents (Codex / Antigravity / Windsurf) get a
/// different template via `init_templates.rs` that does prescribe
/// the `trs <cmd>` prefix, since there's no hook to do it for them.
pub(crate) fn standalone_file() -> String {
    format!(
        "# trs — token-reducing shell\n\n\
         Installed by `trs output-saver --install`. Remove with\n\
         `trs output-saver --remove` or delete this file plus the\n\
         `@{}` import line in the parent config.\n\n\
         ## Shell output\n\n\
         Shell command output is automatically routed through trs (a\n\
         token-reduction hook) and may appear in compact form. The\n\
         compression is purely presentational — repetition and noise\n\
         collapsed; signal preserved. There is no detail in raw output\n\
         that the compressed form hides from you, so treat what arrives\n\
         as authoritative and write normal shell commands.\n\n\
         {}\n",
        IMPORT_FILENAME, BLOCK
    )
}

/// Wrap the block in sentinels for idempotent inline installs.
pub(crate) fn sentinel_wrapped() -> String {
    format!("\n{}\n\n{}\n\n{}\n", SENTINEL_START, BLOCK, SENTINEL_END)
}

/// Which global-config location we target per agent. `None` = the agent
/// has no global rules mechanism we can safely write to; we skip it.
// Targets + scan/verify/install/remove live in output_saver_core; re-exported
// so callers keep using `crate::output_saver::*`.
pub(crate) use crate::output_saver_core::{
    install_agent, remove_agent, replace_between, scan_agent, verify_agent, Status, VerifyStatus,
    AGENTS,
};

/// Entry point called from main.rs. Modes are mutually exclusive —
/// `--print` wins, then `--remove`, then `--refresh`, then
/// `--install`; default is a read-only scan.
pub(crate) fn run(agent: Option<&str>, install: bool, remove: bool, print: bool, refresh: bool) {
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

    if refresh {
        run_refresh(&targets);
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
                println!("  + {:<18}  already installed", display);
                already += 1;
            }
            Status::NotInstalled => {
                println!("  . {:<18}  not yet installed", display);
                installable.push(*id);
            }
            Status::NotDetected => {
                println!("  - {:<18}  not detected", display);
                not_detected += 1;
            }
            Status::Unsupported { reason } => {
                println!("  ~ {:<18}  skipped ({})", display, reason);
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
    println!("More: https://github.com/dPeluChe/trs/blob/main/docs/features/output-saver.md");
}

fn run_install(targets: &[&str]) {
    println!("trs output-saver — install\n");
    let mut wrote = 0;
    let mut skipped = 0;
    for id in targets {
        let display = agent_display(id);
        match scan_agent(id) {
            Status::NotDetected => {
                println!("  - {:<18}  skipped (not detected)", display);
                skipped += 1;
                continue;
            }
            Status::Unsupported { reason } => {
                println!("  ~ {:<18}  skipped ({})", display, reason);
                skipped += 1;
                continue;
            }
            _ => {}
        }
        match install_agent(id) {
            Ok(msg) => {
                println!("  + {:<18}  {}", display, msg);
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

/// Re-install the block only where `scan_agent` already reports
/// `AlreadyInstalled`. Agents that don't have it yet are skipped —
/// this is the path `trs upgrade` calls to pick up template changes
/// without adding the block to agents the user never opted in for.
fn run_refresh(targets: &[&str]) {
    println!("trs output-saver — refresh\n");
    let mut refreshed = 0;
    let mut skipped_not_present = 0;
    let mut skipped_unsupported = 0;

    for id in targets {
        let display = agent_display(id);
        match scan_agent(id) {
            Status::AlreadyInstalled => match install_agent(id) {
                Ok(msg) => {
                    println!("  + {:<18}  {}", display, msg);
                    refreshed += 1;
                }
                Err(e) => {
                    eprintln!("  ! {}  refresh failed: {}", display, e);
                }
            },
            Status::Unsupported { .. } => {
                skipped_unsupported += 1;
            }
            Status::NotInstalled | Status::NotDetected => {
                // Don't touch: user hasn't opted in for this agent.
                skipped_not_present += 1;
            }
        }
    }
    println!();
    println!(
        "  {} refreshed, {} skipped (not installed), {} skipped (unsupported)",
        refreshed, skipped_not_present, skipped_unsupported
    );
    if refreshed == 0 && skipped_not_present > 0 {
        println!();
        println!(
            "No output-saver blocks were installed to refresh. Run \
             `trs output-saver --install` to add the block."
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
            println!("  ~ {:<18}  skipped ({})", display, reason);
            skipped += 1;
            continue;
        }
        match remove_agent(id) {
            Ok(msg) => {
                println!("  + {:<18}  {}", display, msg);
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
