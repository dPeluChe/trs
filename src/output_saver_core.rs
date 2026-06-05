//! Output-saver core: per-agent targets, scan/verify/install/remove, and the
//! sentinel splice. CLI (`run`) + rules templates live in `output_saver.rs`.
use std::fs;
use std::path::PathBuf;

use crate::output_saver::{
    sentinel_wrapped, standalone_file, BLOCK, IMPORT_FILENAME, IMPORT_FILENAME_LEGACY,
    SENTINEL_END, SENTINEL_START,
};

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
        id: "devin",
        display: "Devin Desktop",
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
    // Antigravity 2.0 (IDE + CLI) shares the Gemini CLI harness — both
    // read `~/.gemini/GEMINI.md` and honor `@imports`. Listed as two
    // entries so `--show` calls each out. The Imported target points to
    // the same dir; install_agent's idempotent merge writes the
    // `@trs.md` import line once.
    Agent {
        id: "antigravity",
        display: "Antigravity IDE",
    },
    Agent {
        id: "antigravity-cli",
        display: "Antigravity CLI",
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
        // Devin Desktop (ex-Windsurf): the legacy `~/.codeium` global memory
        // is still read by Devin, so it remains the global output-saver target.
        "devin" => push_home(".codeium/windsurf/memories/global_rules.md")
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
        // Antigravity 2.0 IDE + CLI both honor `@imports` from
        // `~/.gemini/GEMINI.md` (shared with Gemini CLI). install_agent
        // is idempotent — if Gemini already installed, the import line
        // is detected and not re-added.
        "antigravity" | "antigravity-cli" => push_home(".gemini")
            .map(|dir| Target::Imported {
                dir,
                root_file: "GEMINI.md".into(),
            })
            .unwrap_or(Target::NotSupported {
                reason: "HOME not set",
            }),
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

/// Deeper check than `Status::AlreadyInstalled` — verifies the file
/// content on disk matches the canonical template. Used by `trs doctor`
/// to report drift (manual edits, partial updates from older versions).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum VerifyStatus {
    /// File present and content matches the current canonical template.
    Ok,
    /// File present but contents differ — either a manual edit, a stale
    /// template from a prior release, or a partial write. The user
    /// should run `trs output-saver --refresh` to restore.
    Drifted,
    /// Same enum as `scan_agent` — pass through when nothing to verify.
    NotInstalled,
    NotDetected,
    Unsupported,
}

pub(crate) fn verify_agent(agent_id: &str) -> VerifyStatus {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    verify_agent_with_home(agent_id, home.as_deref())
}

fn verify_agent_with_home(agent_id: &str, home: Option<&std::path::Path>) -> VerifyStatus {
    let target = resolve_target_with_home(agent_id, home);
    match target {
        Target::NotSupported { .. } => VerifyStatus::Unsupported,
        Target::Imported { dir, root_file } => {
            if !dir.exists() {
                return VerifyStatus::NotDetected;
            }
            let saver = dir.join(IMPORT_FILENAME);
            let import_line = format!("@{}", IMPORT_FILENAME);
            let root = dir.join(&root_file);
            let has_import = fs::read_to_string(&root)
                .map(|c| c.contains(&import_line))
                .unwrap_or(false);
            if !has_import || !saver.exists() {
                return VerifyStatus::NotInstalled;
            }
            match fs::read_to_string(&saver) {
                Ok(content) if content == standalone_file() => VerifyStatus::Ok,
                Ok(_) => VerifyStatus::Drifted,
                Err(_) => VerifyStatus::Drifted,
            }
        }
        Target::RulesDir { path } => {
            if !path.exists() {
                return VerifyStatus::NotInstalled;
            }
            // Rules-dir agents (Cursor) write the wrapped block to a
            // dedicated file; compare against the canonical wrap.
            match fs::read_to_string(&path) {
                Ok(content) if content.contains(BLOCK) => VerifyStatus::Ok,
                Ok(_) => VerifyStatus::Drifted,
                Err(_) => VerifyStatus::Drifted,
            }
        }
        Target::InlineFile { path } => {
            if !path.exists() {
                return VerifyStatus::NotInstalled;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                return VerifyStatus::Drifted;
            };
            if !content.contains(SENTINEL_START) || !content.contains(SENTINEL_END) {
                return VerifyStatus::NotInstalled;
            }
            // Pull out the sentinel-delimited slice and compare its
            // inner body with BLOCK. Whitespace between sentinel and
            // BLOCK is ignored — the install path uses `sentinel_wrapped`
            // which adds a leading newline + blank line.
            let s = content.find(SENTINEL_START).unwrap() + SENTINEL_START.len();
            let e = content[s..].find(SENTINEL_END).map(|p| s + p).unwrap_or(s);
            let inner = content[s..e].trim();
            if inner == BLOCK.trim() {
                VerifyStatus::Ok
            } else {
                VerifyStatus::Drifted
            }
        }
    }
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
            // Check current filename.
            let saver = dir.join(IMPORT_FILENAME);
            let has_import = fs::read_to_string(&root)
                .map(|c| c.contains(&format!("@{}", IMPORT_FILENAME)))
                .unwrap_or(false);
            if has_import && saver.exists() {
                return Status::AlreadyInstalled;
            }
            // Check legacy filename — still counts as installed (migration
            // happens on the next `install_agent` call).
            let legacy_saver = dir.join(IMPORT_FILENAME_LEGACY);
            let has_legacy_import = fs::read_to_string(&root)
                .map(|c| c.contains(&format!("@{}", IMPORT_FILENAME_LEGACY)))
                .unwrap_or(false);
            if has_legacy_import && legacy_saver.exists() {
                return Status::AlreadyInstalled;
            }
            Status::NotInstalled
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

            // Migrate legacy file: delete it and strip the old @import line.
            let legacy_path = dir.join(IMPORT_FILENAME_LEGACY);
            if legacy_path.exists() {
                let _ = fs::remove_file(&legacy_path);
                let root_path_tmp = dir.join(&root_file);
                if let Ok(existing) = fs::read_to_string(&root_path_tmp) {
                    let legacy_import = format!("@{}", IMPORT_FILENAME_LEGACY);
                    if existing.contains(&legacy_import) {
                        let stripped: String = existing
                            .lines()
                            .filter(|l| l.trim() != legacy_import.as_str())
                            .collect::<Vec<_>>()
                            .join("\n");
                        let final_content = if existing.ends_with('\n') && !stripped.is_empty() {
                            format!("{}\n", stripped)
                        } else {
                            stripped
                        };
                        let _ = fs::write(&root_path_tmp, final_content);
                    }
                }
            }

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
pub(crate) fn replace_between(content: &str, start: &str, end: &str, new_block: &str) -> String {
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

#[cfg(test)]
#[path = "output_saver_core_tests.rs"]
mod tests;
