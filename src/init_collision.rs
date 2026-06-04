//! Pre-install collision detection for `trs init`.
//!
//! Scans target config files for hooks or rules installed by other
//! shell-compression tools (rtk, token-optimizer, …). Installing trs
//! alongside another compressor risks double-compression — where our
//! output feeds the other tool's parser, or vice versa, producing
//! corrupted data that looks like success at the hook layer.
//!
//! The flow: `trs init` calls `detect`, prints the report, aborts unless
//! the user passes `--replace` (scrub competitors, install trs) or
//! `--force` (install alongside and eat the risk).
//!
//! Why heuristic-only: we deliberately do NOT flag every unknown hook —
//! users have lint runners, analytics webhooks, notify scripts on their
//! PreToolUse events that have nothing to do with us. We match on a
//! short, curated list of known compressor binaries.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::init::AiTool;

/// Max recursion depth when following `@imports`. Claude Code currently
/// caps imports at 5 levels — we stay below that so a circular chain
/// never eats our stack even if the visited-set guard somehow missed it.
const IMPORT_MAX_DEPTH: usize = 3;

#[derive(Debug, Clone)]
pub(crate) struct Collision {
    pub location: PathBuf,
    pub kind: CollisionKind,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub(crate) enum CollisionKind {
    /// A PreToolUse-style hook already runs a competing shell-rewrite
    /// binary. Double-compression is the risk.
    HookBinary { binary: String },
    /// A rules/instructions file already references another compressor.
    /// Adding trs won't corrupt output, but the agent sees conflicting
    /// guidance.
    RulesCompressor { signature: String },
}

/// Competitor hook-command substrings. Anything matched here inside a
/// JSON config's string values is flagged. Keep this list short and
/// specific — false positives here abort the user's install.
const HOOK_COMPETITORS: &[&str] = &[
    "rtk rewrite",
    "rtk proxy",
    "rtk git",
    "rtk hook", // v0.37.2+ Windows binary hook format
    "token-optimizer",
    "tokopt",
];

/// Competitor signatures in rules/docs files. Broader than the hook list
/// because rules text can describe the tool without invoking it.
const RULES_SIGNATURES: &[&str] = &[
    "rtk (Rust Token Killer)",
    "RTK - Rust Token Killer",
    "rtk rewrite",
    "token-optimizer",
];

/// Scan every target path for `tool` and return collisions.
pub(crate) fn detect(tool: &AiTool, global: bool) -> Vec<Collision> {
    let mut out = Vec::new();
    for path in target_paths(tool, global) {
        if !path.exists() {
            continue;
        }
        let is_json = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("json"));
        if is_json {
            out.extend(scan_json(&path));
        } else {
            let mut visited = HashSet::new();
            out.extend(scan_text(&path, IMPORT_MAX_DEPTH, &mut visited));
        }
    }
    out
}

/// Paths where a competitor hook/rule could realistically live for
/// `tool`. Scans both home and project locations regardless of the
/// install mode — the agent loads both at runtime, so a competitor in
/// either location will still double-compress commands even if we only
/// installed globally.
///
/// `_global` is kept in the signature for future extensibility (e.g.
/// silencing project scans when running from an unrelated cwd).
fn target_paths(tool: &AiTool, _global: bool) -> Vec<PathBuf> {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    let mut v = Vec::new();

    let push_home = |v: &mut Vec<PathBuf>, rel: &str| {
        if let Some(h) = home.as_ref() {
            v.push(h.join(rel));
        }
    };

    match tool {
        AiTool::Claude => {
            push_home(&mut v, ".claude/settings.json");
            push_home(&mut v, ".claude/CLAUDE.md");
            v.push(PathBuf::from("hooks/hooks.json"));
            v.push(PathBuf::from(".claude/settings.json"));
            v.push(PathBuf::from("CLAUDE.md"));
        }
        AiTool::Gemini => {
            push_home(&mut v, ".gemini/settings.json");
            push_home(&mut v, ".gemini/GEMINI.md");
            v.push(PathBuf::from(".gemini/settings.json"));
            v.push(PathBuf::from("GEMINI.md"));
        }
        AiTool::Cursor => {
            push_home(&mut v, ".cursor/hooks.json");
            v.push(PathBuf::from(".cursor/hooks.json"));
        }
        AiTool::Droid => {
            push_home(&mut v, ".factory/settings.json");
            v.push(PathBuf::from(".factory/settings.json"));
        }
        AiTool::OpenCode => {
            push_home(&mut v, ".config/opencode/plugins/trs.ts");
            v.push(PathBuf::from(".opencode/plugins/trs.ts"));
        }
        AiTool::Kilo => {
            push_home(&mut v, ".config/kilo/plugins/trs.ts");
            v.push(PathBuf::from(".kilo/plugins/trs.ts"));
        }
        AiTool::Pi => {
            push_home(&mut v, ".pi/agent/extensions/trs.ts");
            v.push(PathBuf::from(".pi/extensions/trs.ts"));
        }
        AiTool::Codex => {
            push_home(&mut v, ".codex/AGENTS.md");
            v.push(PathBuf::from("AGENTS.md"));
        }
        AiTool::Antigravity => {
            // v0.6.5+: jetski hooks.json under each variant's data dir.
            push_home(&mut v, ".gemini/antigravity-ide/hooks.json");
            v.push(PathBuf::from(".gemini/antigravity-ide/hooks.json"));
        }
        AiTool::AntigravityCLI => {
            push_home(&mut v, ".gemini/antigravity-cli/hooks.json");
            v.push(PathBuf::from(".gemini/antigravity-cli/hooks.json"));
        }
        AiTool::Windsurf => {
            push_home(&mut v, ".codeium/windsurf/memories/global_rules.md");
            v.push(PathBuf::from(".windsurfrules"));
        }
    }
    // Dedup: when cwd coincidentally equals $HOME the same path gets
    // pushed twice — keeps the report clean.
    v.sort();
    v.dedup();
    v
}

fn scan_json(path: &Path) -> Vec<Collision> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) else {
        return Vec::new();
    };
    let mut strings = Vec::new();
    collect_string_values(&val, &mut strings);

    let mut out = Vec::new();
    for s in &strings {
        for comp in HOOK_COMPETITORS {
            // `!contains("trs ")` guards against matching a user's own
            // wrapper script like `./run-via-trs-and-rtk.sh`.
            if s.contains(comp) && !s.contains("trs ") {
                out.push(Collision {
                    location: path.to_path_buf(),
                    kind: CollisionKind::HookBinary {
                        binary: comp.to_string(),
                    },
                    detail: truncate(s, 80),
                });
                break;
            }
        }
    }
    out
}

/// Scan `path` for compressor signatures, then follow any `@file.md`
/// import lines (Claude Code / Gemini CLI syntax) and scan those too.
/// The `visited` set breaks import cycles; `depth` caps recursion.
fn scan_text(path: &Path, depth: usize, visited: &mut HashSet<PathBuf>) -> Vec<Collision> {
    if depth == 0 {
        return Vec::new();
    }
    // Canonicalize where possible so symlinks + relative paths don't
    // masquerade as distinct entries in the visited set.
    let key = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(key) {
        return Vec::new();
    }
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for sig in RULES_SIGNATURES {
        if content.contains(sig) {
            out.push(Collision {
                location: path.to_path_buf(),
                kind: CollisionKind::RulesCompressor {
                    signature: sig.to_string(),
                },
                detail: format!("file references another compressor ({})", sig),
            });
        }
    }

    for import in extract_imports(&content, path) {
        if import.exists() {
            out.extend(scan_text(&import, depth - 1, visited));
        }
    }
    out
}

/// Extract `@file.md` imports from a Claude/Gemini rules file. Syntax:
/// a line whose first non-whitespace character is `@`, followed by a
/// path token terminated by whitespace or end-of-line. We deliberately
/// ignore `@` mid-line (e.g. "see @foo.md for details") because that
/// isn't the import form Claude recognises.
fn extract_imports(content: &str, base_file: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix('@') else {
            continue;
        };
        // Comment / email / literal `@` — if there's no path-like token,
        // skip.
        let target = rest.split_whitespace().next().unwrap_or("");
        if target.is_empty() {
            continue;
        }
        if let Some(p) = resolve_import(target, base_file) {
            out.push(p);
        }
    }
    out
}

fn resolve_import(target: &str, base_file: &Path) -> Option<PathBuf> {
    if let Some(rel) = target.strip_prefix("~/") {
        let home = std::env::var("HOME").ok()?;
        return Some(PathBuf::from(home).join(rel));
    }
    if target.starts_with('/') {
        return Some(PathBuf::from(target));
    }
    // Relative to the file containing the import. Claude's docs state
    // the base is the importing file's directory.
    base_file.parent().map(|p| p.join(target))
}

fn collect_string_values(val: &serde_json::Value, out: &mut Vec<String>) {
    match val {
        serde_json::Value::String(s) => out.push(s.clone()),
        serde_json::Value::Object(o) => {
            for v in o.values() {
                collect_string_values(v, out);
            }
        }
        serde_json::Value::Array(a) => {
            for v in a {
                collect_string_values(v, out);
            }
        }
        _ => {}
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{}…", truncated)
    }
}

/// Render the collision report a user sees before the install aborts.
pub(crate) fn format_report(tool: &AiTool, collisions: &[Collision]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "\nFound {} potential collision(s) while preparing {} install:\n\n",
        collisions.len(),
        tool.name()
    ));
    for c in collisions {
        match &c.kind {
            CollisionKind::HookBinary { binary } => {
                out.push_str(&format!(
                    "  ! {} — existing hook runs '{}'\n    {}\n",
                    c.location.display(),
                    binary,
                    c.detail
                ));
            }
            CollisionKind::RulesCompressor { signature } => {
                out.push_str(&format!(
                    "  ! {} — references '{}'\n    {}\n",
                    c.location.display(),
                    signature,
                    c.detail
                ));
            }
        }
    }
    out.push('\n');
    out.push_str("Risk: running two shell-compression tools on the same\n");
    out.push_str("command can double-compress and corrupt output.\n\n");
    out.push_str("Recommended: migrate to trs.\n");
    out.push_str(
        "  trs init <tool> --replace   remove competitor hooks, install trs   [recommended]\n",
    );
    out.push_str("  trs init <tool> --force     install alongside                      [risky]\n");
    out.push_str("  abort                       (default) fix the collisions manually\n");
    out
}

/// True if any collision in the set is a JSON hook collision that
/// `--replace` can scrub automatically. Rules-file collisions need
/// manual edits — we don't rewrite someone's markdown for them.
pub(crate) fn any_hook_collisions(collisions: &[Collision]) -> bool {
    collisions
        .iter()
        .any(|c| matches!(c.kind, CollisionKind::HookBinary { .. }))
}

/// Read `path` as JSON and drop any hook entries whose strings match a
/// known competitor. Writes the file back when a change was made. Safe
/// to call when the file isn't the one trs writes to — the scrub is
/// scoped to arrays under a top-level `hooks` object, which is the
/// convention every supported agent uses.
///
/// Returns `Ok(true)` when the file was modified, `Ok(false)` when it
/// was already clean, `Err` on I/O or parse failures.
pub(crate) fn scrub_file(path: &Path) -> Result<bool, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let mut val: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("{}: {}", path.display(), e))?;

    let mut changed = false;
    if let Some(hooks) = val.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        for (_event, event_val) in hooks.iter_mut() {
            if let Some(arr) = event_val.as_array_mut() {
                let before = arr.len();
                arr.retain(|e| !is_competitor_hook(e));
                if arr.len() != before {
                    changed = true;
                }
            }
        }
    }

    if changed {
        let pretty =
            serde_json::to_string_pretty(&val).map_err(|e| format!("{}: {}", path.display(), e))?;
        fs::write(path, format!("{}\n", pretty))
            .map_err(|e| format!("{}: {}", path.display(), e))?;
    }
    Ok(changed)
}

/// True when a collision lives in a JSON file we can auto-scrub.
pub(crate) fn is_json_location(c: &Collision) -> bool {
    c.location
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("json"))
        && matches!(c.kind, CollisionKind::HookBinary { .. })
}

/// Scrub competitor entries from a parsed JSON hooks tree. Called from
/// `init::merge_json_hook` when `--replace` is active.
pub(crate) fn is_competitor_hook(val: &serde_json::Value) -> bool {
    match val {
        serde_json::Value::String(s) => {
            HOOK_COMPETITORS.iter().any(|c| s.contains(c)) && !s.contains("trs ")
        }
        serde_json::Value::Object(o) => o.values().any(is_competitor_hook),
        serde_json::Value::Array(a) => a.iter().any(is_competitor_hook),
        _ => false,
    }
}

#[cfg(test)]
#[path = "init_collision_tests.rs"]
mod tests;
