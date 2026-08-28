//! The write half of `output-saver`: installing a block into an agent
//! config and removing it again. Kept apart from the read half (verify and
//! scan) in `output_saver_core.rs`: the seam is mutating versus read-only,
//! not disk versus no disk. Scanning reads files too.

use std::fs;
use std::path::PathBuf;

use crate::output_saver::{
    sentinel_wrapped, standalone_file, IMPORT_FILENAME, IMPORT_FILENAME_LEGACY, SENTINEL_END,
    SENTINEL_START,
};
use crate::output_saver_core::{replace_between, resolve_target_with_home, Target};

pub(crate) fn install_agent(agent_id: &str) -> Result<String, String> {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    install_agent_with_home(agent_id, home.as_deref())
}

pub(crate) fn install_agent_with_home(
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
            // The `@import` line into the root config is an implementation
            // detail — showing just the trs.md path keeps refresh output
            // scannable (this same message repeats for every import-based
            // agent, several of which share ~/.gemini/trs.md).
            Ok(format!(
                "wrote {}",
                crate::path_display::tilde(&saver_path.display().to_string())
            ))
        }
        Target::RulesDir { path } => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("{}: {}", parent.display(), e))?;
            }
            fs::write(&path, standalone_file())
                .map_err(|e| format!("{}: {}", path.display(), e))?;
            Ok(format!(
                "wrote {}",
                crate::path_display::tilde(&path.display().to_string())
            ))
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
            Ok(format!(
                "updated {}",
                crate::path_display::tilde(&path.display().to_string())
            ))
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

pub(crate) fn remove_agent_with_home(
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
