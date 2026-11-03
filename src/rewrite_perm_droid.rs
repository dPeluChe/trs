//! Droid matches its command lists against the REWRITTEN command and a hook
//! `allow` skips them, so the hook applies them to the original command.
//! Contract: docs/development/agent-integrations.md (Factory Droid).

#[path = "rewrite_perm_droid_match.rs"]
mod matcher;

use crate::rewrite_gate::Gate;
use crate::rewrite_rules::read_json;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// Droid's built-in lists; settings lists are unioned with them.
const DEFAULT_ALLOW: &[&str] = &[
    "ls",
    "pwd",
    "dir",
    "git status",
    "git diff",
    "git log",
    "git show",
    "git blame",
    "git ls-files",
];
const DEFAULT_DENY: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "rm -rf .",
    "rm -rf ~",
    "rm -rf ~/*",
    "rm -rf $HOME",
    "rm -r /",
    "rm -r /*",
    "rm -r ~",
    "rm -r ~/*",
    "mkfs",
    "mkfs.ext4",
    "mkfs.ext3",
    "mkfs.vfat",
    "mkfs.ntfs",
    "dd if=/dev/zero of=/dev",
    "dd of=/dev",
    "shutdown",
    "reboot",
    "halt",
    "poweroff",
    "init 0",
    "init 6",
    ":(){ :|: & };:",
    ":() { :|:& };:",
    "chmod -R 777 /",
    "chmod -R 000 /",
    "chown -R",
    "Format-Volume",
    "format.com",
    "powershell Remove-Item -Recurse -Force",
];

/// Where the settings live; empty in unit tests so they never read real config.
#[derive(Default)]
struct Roots {
    user_home: Option<PathBuf>,
    managed: Option<PathBuf>,
}

#[derive(Default)]
struct Lists {
    allow: Vec<String>,
    deny: Vec<String>,
    block: Vec<String>,
}

/// `cmd` is the ORIGINAL command; `cwd` and `permission_mode` come from the hook payload.
pub(crate) fn gate(cmd: &str, cwd: Option<&str>, permission_mode: Option<&str>) -> Gate {
    gate_in(&roots(), cmd, cwd, permission_mode)
}

fn roots() -> Roots {
    if cfg!(test) {
        return Roots::default();
    }
    let home = std::env::var_os("FACTORY_HOME_OVERRIDE")
        .map(PathBuf::from)
        .or_else(|| crate::init::home_dir().ok());
    let system = if cfg!(target_os = "macos") {
        "/Library/Application Support/Factory"
    } else if cfg!(windows) {
        "C:\\Program Files\\Factory"
    } else {
        "/etc/factory"
    };
    let managed = std::env::var_os("FACTORY_ORG_MANAGED_SETTINGS_LOCAL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(system).join("settings.json"));
    Roots {
        user_home: home,
        managed: Some(managed),
    }
}

fn gate_in(roots: &Roots, cmd: &str, cwd: Option<&str>, mode: Option<&str>) -> Gate {
    let Some(lists) = load(roots, cwd) else {
        return Gate::Skip;
    };
    if matcher::denied(cmd, &lists.deny) || matcher::blocked(cmd, &lists.block) {
        return Gate::Skip;
    }
    let mut allow: HashSet<String> = lists.allow.iter().flat_map(|a| matcher::keys(a)).collect();
    for d in &lists.deny {
        for k in matcher::keys(d) {
            allow.remove(&k);
        }
    }
    if matcher::all_allowed(cmd, &allow) {
        return Gate::Allow;
    }
    // Droid drops riskLevel from a rewritten command (it counts as high), so
    // without an allow only auto-high runs it unprompted. The hook payload does
    // not give us the original riskLevel: any other or unknown mode skips.
    if mode == Some("auto-high") {
        Gate::Defer
    } else {
        Gate::Skip
    }
}

/// `None` when a settings file exists but cannot be read or parsed.
fn load(roots: &Roots, cwd: Option<&str>) -> Option<Lists> {
    let mut lists = Lists {
        allow: DEFAULT_ALLOW.iter().map(|s| s.to_string()).collect(),
        deny: DEFAULT_DENY.iter().map(|s| s.to_string()).collect(),
        block: Vec::new(),
    };
    if let Some(home) = &roots.user_home {
        for name in ["settings.json", "settings.local.json"] {
            merge(&mut lists, &read(&home.join(".factory").join(name))?)?;
        }
    }
    if let Some(path) = &roots.managed {
        merge(&mut lists, &read(path)?)?;
    }
    if let Some(cwd) = cwd {
        for dir in project_dirs(Path::new(cwd), roots.user_home.as_deref()) {
            for name in ["settings.json", "settings.local.json"] {
                merge(&mut lists, &read(&dir.join(".factory").join(name))?)?;
            }
        }
    }
    Some(lists)
}

/// Every dir from cwd up to the git root (or cwd); the home dir is the user level.
fn project_dirs(cwd: &Path, home: Option<&Path>) -> Vec<PathBuf> {
    let root = cwd
        .ancestors()
        .find(|d| d.join(".git").exists())
        .unwrap_or(cwd);
    let mut dirs = Vec::new();
    for d in cwd.ancestors() {
        if Some(d) != home {
            dirs.push(d.to_path_buf());
        }
        if d == root {
            break;
        }
    }
    dirs
}

/// `Some(json)`, or `None` if the file exists but is unusable. A missing file is `{}`.
fn read(path: &Path) -> Option<serde_json::Value> {
    match read_json(path)? {
        None => Some(serde_json::json!({})),
        Some(json) => json.is_object().then_some(json),
    }
}

/// A list key that is present but not an array of strings makes the file unusable,
/// and so does `permissionRules`, a newer rule format this gate does not read.
/// Droid honors the legacy lists of an untrusted project too (verified on 1.2.0).
fn merge(lists: &mut Lists, json: &serde_json::Value) -> Option<()> {
    let rules = json
        .pointer("/general/permissionRules")
        .or_else(|| json.get("permissionRules"));
    if rules.is_some_and(|r| !r.is_null()) {
        return None;
    }
    let take = |key: &str, into: &mut Vec<String>| -> Option<()> {
        let Some(v) = json.get(key) else {
            return Some(());
        };
        for item in v.as_array()? {
            into.push(item.as_str()?.trim().to_string());
        }
        Some(())
    };
    take("commandAllowlist", &mut lists.allow)?;
    take("commandDenylist", &mut lists.deny)?;
    take("commandBlocklist", &mut lists.block)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, rel: &str, text: &str) {
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    fn run(home: &Path, cwd: &Path, cmd: &str, mode: Option<&str>) -> Gate {
        let roots = Roots {
            user_home: Some(home.to_path_buf()),
            managed: None,
        };
        gate_in(&roots, cmd, cwd.to_str(), mode)
    }
    const USER: &str = r#"{"commandAllowlist":["cargo test"],"commandDenylist":["git push"]}"#;

    #[test]
    fn deny_hits_skip_and_allowed_commands_are_approved() {
        let (home, cwd) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        write(home.path(), ".factory/settings.json", USER);
        let g = |c, m| run(home.path(), cwd.path(), c, m);
        assert_eq!(g("git push origin main", Some("auto-high")), Gate::Skip);
        assert_eq!(g("git status && git push", Some("auto-high")), Gate::Skip);
        assert_eq!(g("rm -rf /home/u", Some("auto-high")), Gate::Skip);
        assert_eq!(g("cargo test --lib", Some("off")), Gate::Allow);
        assert_eq!(g("git status", None), Gate::Allow);
        assert_eq!(g("ls -la && cargo test", None), Gate::Allow);
    }

    #[test]
    fn unlisted_commands_defer_only_in_auto_high() {
        let (home, cwd) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        write(home.path(), ".factory/settings.json", USER);
        let g = |m| run(home.path(), cwd.path(), "make build", m);
        assert_eq!(g(Some("auto-high")), Gate::Defer);
        for m in [
            Some("off"),
            Some("auto-low"),
            Some("auto-medium"),
            Some("spec"),
            None,
        ] {
            assert_eq!(g(m), Gate::Skip, "{m:?}");
        }
    }

    #[test]
    fn a_project_allow_counts_even_in_an_untrusted_folder() {
        let (home, proj) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        std::fs::create_dir_all(proj.path().join(".git")).unwrap();
        write(
            proj.path(),
            ".factory/settings.local.json",
            r#"{"commandAllowlist":["make"],"commandBlocklist":["deploy"]}"#,
        );
        let g = |c| run(home.path(), proj.path(), c, Some("off"));
        assert_eq!(g("make all"), Gate::Allow);
        assert_eq!(g("deploy now"), Gate::Skip);
    }

    #[test]
    fn unusable_settings_fail_closed() {
        let (home, cwd) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        for bad in [
            "{ nope",
            r#"{"commandDenylist":"git push"}"#,
            "[]",
            r#"{"general":{"permissionRules":[{"tool":"Execute"}]}}"#,
        ] {
            write(home.path(), ".factory/settings.json", bad);
            assert_eq!(
                run(home.path(), cwd.path(), "git status", None),
                Gate::Skip,
                "{bad}"
            );
        }
    }
}
