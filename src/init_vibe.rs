//! `trs init vibe`: appends a sentinel-wrapped `[[hooks]]` entry to
//! `~/.vibe/hooks.toml`. Appending keeps the user's comments and layout, which
//! a TOML round trip would lose.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::init::{home_dir, InstallOpts};
use crate::init_install::ensure_parent;
use crate::init_install_rules::refresh_sentinel_block;
use crate::init_templates::{VIBE_HOOK, VIBE_HOOKS_FILE, VIBE_SENTINEL_END, VIBE_SENTINEL_START};

pub(crate) fn install_vibe_hook(opts: InstallOpts) -> Result<String, String> {
    if !opts.global {
        eprintln!("note: Mistral Vibe hooks are global, installing to ~/.vibe");
    }
    install_at(&home_dir()?.join(VIBE_HOOKS_FILE), opts)
}

fn install_at(path: &Path, opts: InstallOpts) -> Result<String, String> {
    let existing = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) if e.kind() == ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("Cannot read {}: {}", path.display(), e)),
    };
    let updated = if existing.contains(VIBE_SENTINEL_START) {
        refresh_sentinel_block(
            &existing,
            VIBE_SENTINEL_START,
            VIBE_SENTINEL_END,
            VIBE_HOOK.trim(),
        )
        .ok_or_else(|| format!("{} has a damaged trs block", path.display()))?
    } else {
        let body = existing.trim_end();
        let gap = if body.is_empty() { "" } else { "\n\n" };
        format!("{body}{gap}{VIBE_HOOK}")
    };
    if updated == existing {
        return Ok(format!("{} (already configured)", path.display()));
    }
    if opts.dry_run {
        return Ok(format!("{} (would add trs hook)", path.display()));
    }
    ensure_parent(path)?;
    crate::fs_atomic::write_if_unchanged(path, &existing, &updated)
        .map_err(|e| format!("Cannot write {}: {}", path.display(), e))?;
    Ok(path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uninstall_scrub::remove_between_sentinels;

    fn opts() -> InstallOpts {
        InstallOpts {
            global: true,
            replace: false,
            force: false,
            dry_run: false,
        }
    }

    #[test]
    fn the_hook_is_valid_toml_with_the_fields_vibe_reads() {
        let t: toml::Value = toml::from_str(VIBE_HOOK).unwrap();
        let hook = &t["hooks"][0];
        assert_eq!(hook["type"].as_str(), Some("pre_tool"));
        assert_eq!(hook["match"].as_str(), Some("bash"));
        assert_eq!(hook["command"].as_str(), Some("trs rewrite --caller vibe"));
    }

    #[test]
    fn install_keeps_user_hooks_and_remove_restores_them() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".vibe/hooks.toml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mine = "# my hooks\n[[hooks]]\nname = \"mine\"\n";
        fs::write(&path, mine).unwrap();

        install_at(&path, opts()).unwrap();
        install_at(&path, opts()).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert_eq!(after.matches(VIBE_SENTINEL_START).count(), 1);
        assert_eq!(
            toml::from_str::<toml::Value>(&after).unwrap()["hooks"]
                .as_array()
                .unwrap()
                .len(),
            2
        );

        remove_between_sentinels(&path, VIBE_SENTINEL_START, VIBE_SENTINEL_END, false).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap().trim(), mine.trim());
    }

    #[test]
    fn a_stale_block_is_refreshed_in_place() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hooks.toml");
        let old = format!("{VIBE_SENTINEL_START}\nold\n{VIBE_SENTINEL_END}\n# after\n");
        fs::write(&path, old).unwrap();
        install_at(&path, opts()).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("trs rewrite --caller vibe"), "{after}");
        assert!(after.ends_with("# after\n"), "{after}");
        assert!(!after.contains("old"), "{after}");
    }

    #[test]
    fn dry_run_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hooks.toml");
        let opts = InstallOpts {
            dry_run: true,
            ..opts()
        };
        install_at(&path, opts).unwrap();
        assert!(!path.exists());
    }
}
