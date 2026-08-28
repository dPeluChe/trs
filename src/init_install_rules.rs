//! Installing the rules blocks: the prose trs splices into files it does not
//! own (`AGENTS.md`, `GEMINI.md`, `.windsurfrules`). Kept apart from the hook
//! and plugin writers next door, which merge JSON and own whole files.
//!
//! Every writer here refreshes on drift. A block that installs once and then
//! freezes is worse than no block: the text ships a fix and reaches nobody,
//! and the stale copy can end up contradicting the block beside it.

use std::fs;
use std::path::{Path, PathBuf};

use crate::init::{file_has_any_trs_marker, has_trs_marker, home_dir, InstallOpts};
use crate::init_templates::{
    ANTIGRAVITY_RULES_SECTION, ANTIGRAVITY_RULES_SENTINEL_END, ANTIGRAVITY_RULES_SENTINEL_START,
    CODEX_AGENTS_SECTION, CODEX_AGENTS_SENTINEL_END, CODEX_AGENTS_SENTINEL_START, CODEX_HOOKS,
};

use crate::init_install::{
    ensure_parent, merge_json_hook, remove_hookspec_at, scrub_legacy_codex_hook,
};

/// The heading a pre-sentinel trs rules block opens with. Specific enough to
/// tell an old block apart from a file that merely mentions trs.
const LEGACY_RULES_HEADING: &str = "## Terminal Output Optimization";

/// Append the trs rules block to `AGENTS.md`. `--global` targets
/// `~/.codex/AGENTS.md` (Codex reads it globally); otherwise project root.
/// Idempotent against the trs sentinel + markers.
///
/// Also defensively scrubs legacy `trs rewrite` entries from
/// `~/.codex/hooks.json` — pre-v0.6.x installs wrote a PreToolUse hook
/// there, but Codex versions vary in `updatedInput` support so orphans
/// cause "unsupported updatedInput" errors on every tool call. Scrub
/// runs only on `--global` since that's where the orphan lives.
pub(crate) fn install_codex_agents(opts: InstallOpts) -> Result<String, String> {
    if opts.global {
        if let Ok(home) = home_dir() {
            let codex_dir = home.join(".codex");
            let hooks_path = codex_dir.join("hooks.json");
            if crate::codex::rewrite_hook_available() {
                // codex >= 0.134 honors updatedInput — install the real
                // PreToolUse rewrite hook (merge preserves the user's other
                // hooks; idempotent).
                match merge_json_hook(&codex_dir, &hooks_path, CODEX_HOOKS, opts) {
                    Ok(msg) => println!("  codex hook: {}", msg),
                    Err(e) => eprintln!("  warning: could not install codex hook: {}", e),
                }
            } else if let Err(e) = scrub_legacy_codex_hook(&hooks_path, opts.dry_run) {
                // Older/undetectable codex rejects updatedInput — scrub any
                // orphan trs entry so it doesn't error on every tool call.
                eprintln!("  warning: could not scrub {}: {}", hooks_path.display(), e);
            }
        }
    }

    let path = if opts.global {
        home_dir()?.join(".codex").join("AGENTS.md")
    } else {
        PathBuf::from("AGENTS.md")
    };
    let rules_msg = write_agents_md_block(&path, opts)?;

    // Compose the output-saver reply-brevity rules as their own
    // sentinel-managed block instead of embedding a (sentinel-less) copy in
    // CODEX_AGENTS_SECTION. The installer is idempotent — re-runs replace the
    // block in place — so `trs init codex` + `trs output-saver --install`
    // can run in any order without duplicating it. Global only: the
    // output-saver installer targets ~/.codex/AGENTS.md.
    if opts.global && !opts.dry_run {
        if let Err(e) = crate::output_saver::install_agent("codex") {
            eprintln!("  note: output-saver block install failed: {}", e);
        }
    }
    Ok(rules_msg)
}

/// Swap the text between two sentinels for the current template, leaving
/// every byte outside them exactly as it was.
///
/// Deliberately not `output_saver_core::replace_between`: that one collapses
/// the newlines after the closing sentinel, which is right for the block it
/// owns and wrong here. This file holds two trs blocks plus the user's own
/// rules, and eating the blank line between them runs three separate sets of
/// instructions together into one wall of prose.
///
/// Returns None when there is no sentinel pair to replace.
fn refresh_sentinel_block(content: &str, start: &str, end: &str, block: &str) -> Option<String> {
    // Exactly one of each, or refuse. `find` would otherwise pair the FIRST
    // start with the FIRST end after it without checking they belong to the
    // same block, and replace everything between. A user who documents trs in
    // their own rules file (a fenced example containing the marker) would have
    // every line between that example and the real block silently deleted.
    //
    // Two well-formed blocks also land here: that is what the backticked-marker
    // bug produced in the field, and refreshing only the first while leaving
    // the second stale is a half-fix that reports success. Declining says so.
    if content.matches(start).count() != 1 || content.matches(end).count() != 1 {
        return None;
    }
    let s = content.find(start)?;
    let e = content[s..].find(end).map(|pos| s + pos + end.len())?;
    // A start after its end means the pair is not a block; leave it alone.
    if e <= s {
        return None;
    }
    Some(format!("{}{}{}", &content[..s], block, &content[e..]))
}

/// Append the trs sentinel block to an `AGENTS.md`. Shared write path for
/// Codex (project + global) and Zed (project only) — one template, no copies.
fn write_agents_md_block(path: &Path, opts: InstallOpts) -> Result<String, String> {
    if path.exists() {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;

        // Sentinel block present: refresh it when the template moved, the way
        // the plugin installer already does. Without this a rules-text fix
        // never reaches anyone who installed before it, and `--force` does not
        // help either. That is not hypothetical: after the em-dash sweep,
        // installed blocks kept using em dashes while the output-saver block
        // sitting 50 lines above them told the agent never to write one.
        if let Some(updated) = refresh_sentinel_block(
            &content,
            CODEX_AGENTS_SENTINEL_START,
            CODEX_AGENTS_SENTINEL_END,
            CODEX_AGENTS_SECTION.trim(),
        ) {
            if updated == content {
                return Ok(format!("{} (already configured)", path.display()));
            }
            if opts.dry_run {
                return Ok(format!(
                    "{} (would refresh trs rules block)",
                    path.display()
                ));
            }
            fs::write(path, updated).map_err(|e| e.to_string())?;
            return Ok(format!("{} (refreshed)", path.display()));
        }

        // A block from before the sentinels existed, recognised by the
        // section's own heading rather than by "is trs mentioned anywhere".
        // The crate-wide marker also matches `trs rewrite`, which any AGENTS.md
        // that merely documents trs contains, including this repo's own: that
        // turned a vague message into a confident wrong diagnosis.
        if content.contains(LEGACY_RULES_HEADING) {
            // Its end cannot be located reliably, so report rather than guess
            // at a delete: a bad cut here eats the user's own instructions.
            return Ok(format!(
                "{} (already configured; a trs rules block from before the sentinels is in there, \
                 look for \"{}\" and delete through the end of that section)",
                path.display(),
                LEGACY_RULES_HEADING
            ));
        }
        if file_has_any_trs_marker(&content) {
            return Ok(format!("{} (already configured)", path.display()));
        }
        if opts.dry_run {
            return Ok(format!("{} (would append trs rules block)", path.display()));
        }
        ensure_parent(path)?;
        let updated = format!("{}\n{}", content, CODEX_AGENTS_SECTION);
        fs::write(path, updated).map_err(|e| e.to_string())?;
        return Ok(path.display().to_string());
    }
    if opts.dry_run {
        return Ok(format!("{} (would create with trs rules)", path.display()));
    }
    ensure_parent(path)?;
    fs::write(path, CODEX_AGENTS_SECTION.trim()).map_err(|e| e.to_string())?;
    Ok(path.display().to_string())
}

/// Zed Agent Panel — rules-only. The native agent reads the project
/// `AGENTS.md` as always-on instructions (no tool hooks: zed#52688), so the
/// sentinel block IS the integration. ACP external agents (Claude Code,
/// Codex, Gemini CLI, OpenCode) are covered by their own trs hooks.
pub(crate) fn install_zed_agents(opts: InstallOpts) -> Result<String, String> {
    write_agents_md_block(Path::new("AGENTS.md"), opts)
}

/// Append the Antigravity rules block to `~/.gemini/GEMINI.md`. Shared by
/// both Antigravity IDE and Antigravity CLI (`agy`) — same target file.
///
/// Idempotent against the antigravity rules sentinel + general trs marker.
/// Also defensively scrubs orphaned v0.6.5 jetski hook entries from
/// `~/.gemini/antigravity-{cli,ide}/hooks.json` and the v0.6.4 BeforeTool
/// entry from `~/.gemini/settings.json` — both wrote installs that never
/// actually fired (see docs/development/antigravity-hooks-research.md).
pub(crate) fn install_antigravity_rules(opts: InstallOpts) -> Result<String, String> {
    if let Ok(home) = home_dir() {
        // Best-effort: don't fail the rules install if a scrub hits an FS
        // edge case; log and continue. These files might not exist at all
        // (user is on a fresh machine), which is also a no-op.
        let _ = remove_hookspec_at(
            &home.join(".gemini/antigravity-cli/hooks.json"),
            opts.dry_run,
        );
        let _ = remove_hookspec_at(
            &home.join(".gemini/antigravity-ide/hooks.json"),
            opts.dry_run,
        );
        let _ = scrub_legacy_codex_hook(&home.join(".gemini/settings.json"), opts.dry_run);
    }

    let path = home_dir()?.join(".gemini").join("GEMINI.md");

    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;

        // Same refresh-on-drift the codex block gets. The fuzzy marker used to
        // be checked FIRST here, so a file with a perfectly good sentinel pair
        // never reached the comparison and its block stayed frozen forever.
        if let Some(updated) = refresh_sentinel_block(
            &content,
            ANTIGRAVITY_RULES_SENTINEL_START,
            ANTIGRAVITY_RULES_SENTINEL_END,
            ANTIGRAVITY_RULES_SECTION.trim(),
        ) {
            if updated == content {
                return Ok(format!("{} (already configured)", path.display()));
            }
            if opts.dry_run {
                return Ok(format!(
                    "{} (would refresh trs rules block)",
                    path.display()
                ));
            }
            fs::write(&path, updated).map_err(|e| e.to_string())?;
            return Ok(format!("{} (refreshed)", path.display()));
        }

        if file_has_any_trs_marker(&content) {
            return Ok(format!("{} (already configured)", path.display()));
        }
        if opts.dry_run {
            return Ok(format!(
                "{} (would append Antigravity trs rules block)",
                path.display()
            ));
        }
        ensure_parent(&path)?;
        let updated = format!("{}\n{}", content, ANTIGRAVITY_RULES_SECTION);
        fs::write(&path, updated).map_err(|e| e.to_string())?;
    } else {
        if opts.dry_run {
            return Ok(format!(
                "{} (would create with Antigravity trs rules)",
                path.display()
            ));
        }
        ensure_parent(&path)?;
        fs::write(&path, ANTIGRAVITY_RULES_SECTION.trim()).map_err(|e| e.to_string())?;
    }
    Ok(path.display().to_string())
}

/// Install a rules/instructions file (Antigravity, Windsurf). Project-local
/// only — rules tools lack a global equivalent. Appends if the file exists,
/// preserving any user-written prose.
pub(crate) fn install_rules(
    path_rel: &str,
    content: &str,
    opts: InstallOpts,
) -> Result<String, String> {
    let path = PathBuf::from(path_rel);
    if path.exists() {
        let existing = fs::read_to_string(&path).unwrap_or_default();
        if has_trs_marker(&existing) {
            return Ok(format!("{} (already configured)", path.display()));
        }
        if opts.dry_run {
            return Ok(format!("{} (would append trs rules)", path.display()));
        }
        let updated = format!("{}\n\n{}", existing.trim_end(), content);
        fs::write(&path, updated).map_err(|e| e.to_string())?;
    } else {
        if opts.dry_run {
            return Ok(format!("{} (would create with trs rules)", path.display()));
        }
        ensure_parent(&path)?;
        fs::write(&path, content.trim_start()).map_err(|e| e.to_string())?;
    }
    Ok(path.display().to_string())
}

#[cfg(test)]
#[path = "init_install_rules_tests.rs"]
mod tests;
