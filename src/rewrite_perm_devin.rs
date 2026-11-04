//! Permission gate for Devin CLI: `Exec(prefix)` rules in its config files.
//! Contract: docs/development/agent-integrations.md (permissions).

use crate::rewrite_gate::Gate;
use crate::rewrite_permissions::split_segments;
use crate::rewrite_rules::{read_json, word_prefix, Rules};
use std::path::{Path, PathBuf};

/// `cmd` is the ORIGINAL command. The Devin payload has no `cwd`, so the
/// project dir comes from `DEVIN_PROJECT_DIR`, then the process cwd.
pub(crate) fn gate(cmd: &str, cwd: Option<&str>) -> Gate {
    let dir = cwd
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("DEVIN_PROJECT_DIR").map(PathBuf::from))
        .or_else(|| std::env::current_dir().ok());
    match load_rules(&config_files(dir.as_deref())) {
        Some(rules) => gate_with(&rules, cmd),
        None => Gate::Skip,
    }
}

fn gate_with(rules: &Rules, cmd: &str) -> Gate {
    rules.decide(split_segments(cmd), rule_matches, |_| false)
}

/// User file first, then each project dir from the root down to `dir`.
/// The bool marks project files: their `allow` is ignored (no trust state to read).
fn config_files(dir: Option<&Path>) -> Vec<(PathBuf, bool)> {
    let mut files = Vec::new();
    if crate::rewrite_rules::reads_host_config() {
        let user = if cfg!(windows) {
            std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("devin/config.json"))
        } else {
            crate::init::home_dir()
                .ok()
                .map(|h| h.join(".config/devin/config.json"))
        };
        files.extend(user.map(|p| (p, false)));
    }
    let mut project: Vec<(PathBuf, bool)> = dir
        .into_iter()
        .flat_map(Path::ancestors)
        .flat_map(|d| {
            [
                (d.join(".devin/config.json"), true),
                (d.join(".devin/config.local.json"), true),
            ]
        })
        .collect();
    project.reverse();
    files.extend(project);
    files
}

/// `None` when a config file exists but cannot be read or parsed.
fn load_rules(files: &[(PathBuf, bool)]) -> Option<Rules> {
    let mut rules = Rules::default();
    for (path, project) in files {
        if let Some(json) = read_json(path)? {
            rules.extend_from(&json, !project);
        }
    }
    Some(rules)
}

/// `Exec` or `Exec(prefix)`; the prefix must match whole words, no wildcards.
fn rule_matches(rule: &str, cmd: &str) -> bool {
    let prefix = if rule.eq_ignore_ascii_case("exec") {
        return true;
    } else if let Some(p) = rule.strip_prefix("Exec(").and_then(|r| r.strip_suffix(')')) {
        p.trim()
    } else {
        return false;
    };
    prefix.is_empty() || word_prefix(cmd, prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(allow: &[&str], ask: &[&str], deny: &[&str]) -> Rules {
        let own = |l: &[&str]| l.iter().map(|s| s.to_string()).collect();
        Rules {
            allow: own(allow),
            ask: own(ask),
            deny: own(deny),
        }
    }

    #[test]
    fn exec_prefix_matches_whole_words_only() {
        assert!(rule_matches("Exec(git)", "git"));
        assert!(rule_matches("Exec(git)", "git status"));
        assert!(rule_matches("Exec(git push)", "git push origin main"));
        assert!(!rule_matches("Exec(git)", "gitk"));
        assert!(!rule_matches("Exec(git push)", "git pushx"));
        assert!(!rule_matches("Exec(git *)", "git status"));
        assert!(!rule_matches("Fetch(domain:*)", "git"));
        assert!(rule_matches("Exec", "anything"));
    }

    #[test]
    fn deny_or_ask_on_any_segment_stops_the_rewrite() {
        let r = rules(&["Exec(git)"], &["Exec(git push)"], &["Exec(rm)"]);
        assert_eq!(gate_with(&r, "git push origin main"), Gate::Skip);
        assert_eq!(gate_with(&r, "git status && rm -rf build"), Gate::Skip);
        assert_eq!(gate_with(&r, "CI=1 rm -rf build"), Gate::Skip);
        assert_eq!(gate_with(&r, "git log | rm x"), Gate::Skip);
    }

    #[test]
    fn only_commands_fully_covered_by_allow_are_approved() {
        let r = rules(&["Exec(cargo test)", "Exec(git status)"], &[], &[]);
        assert_eq!(gate_with(&r, "cargo test --lib"), Gate::Allow);
        assert_eq!(gate_with(&r, "cargo test && git status"), Gate::Allow);
        assert_eq!(gate_with(&r, "cargo test && cargo build"), Gate::Defer);
        assert_eq!(gate_with(&r, "git log"), Gate::Defer);
        assert_eq!(gate_with(&r, "cd x && cargo test"), Gate::Defer);
        assert_eq!(gate_with(&r, ""), Gate::Defer);
    }

    fn write(dir: &Path, rel: &str, text: &str) {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    #[test]
    fn project_files_add_deny_and_ask_but_not_allow() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            ".devin/config.json",
            r#"{"permissions":{"allow":["Exec(ls)"],"deny":["Exec(rm)"]}}"#,
        );
        write(
            dir.path(),
            ".devin/config.local.json",
            r#"{"permissions":{"ask":["Exec(git push)"]}}"#,
        );
        let sub = dir.path().join("a/b");
        std::fs::create_dir_all(&sub).unwrap();
        let cwd = sub.to_str();
        assert_eq!(gate("rm x", cwd), Gate::Skip);
        assert_eq!(gate("git push", cwd), Gate::Skip);
        assert_eq!(gate("ls", cwd), Gate::Defer);
    }

    #[test]
    fn unreadable_config_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), ".devin/config.json", "{ not json");
        assert_eq!(gate("git status", dir.path().to_str()), Gate::Skip);
    }

    #[test]
    fn no_config_defers() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(gate("git status", dir.path().to_str()), Gate::Defer);
    }
}
