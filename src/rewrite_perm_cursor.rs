//! Cursor: `~/.cursor/cli-config.json` holds `permissions.deny` entries like
//! `Shell(git push)`; a project's `.cursor/cli.json` adds more. Cursor matches
//! them against the REWRITTEN command (verified on 2026.10.22), so a deny on the
//! original stops the rewrite. Only denies are read: `ask` rules had no effect there.

use crate::rewrite_gate::Gate;
use crate::rewrite_permissions::split_segments;
use crate::rewrite_rules::{read_json, string_list, word_prefix, Rules};
use std::path::{Path, PathBuf};

pub(crate) fn gate(cmd: &str, cwd: Option<&str>) -> Gate {
    let config = if cfg!(test) { None } else { config_dir() };
    match deny_rules(config.as_deref(), cwd) {
        Some(rules) => gate_with(&rules, cmd),
        None => Gate::Skip,
    }
}

fn gate_with(deny: &[String], cmd: &str) -> Gate {
    let rules = Rules {
        deny: deny.to_vec(),
        ..Rules::default()
    };
    rules.decide(split_segments(cmd), shell_rule_matches, |_| false)
}

/// `Shell`, `Shell(*)`, `Shell(git push)`: whole-word prefix of the segment.
fn shell_rule_matches(rule: &str, seg: &str) -> bool {
    let inner = if rule == "Shell" {
        ""
    } else if let Some(i) = rule
        .strip_prefix("Shell(")
        .and_then(|r| r.strip_suffix(')'))
    {
        i
    } else {
        return false;
    };
    let prefix = inner
        .strip_suffix(":*")
        .unwrap_or(inner)
        .trim_end_matches('*')
        .trim_end();
    prefix.is_empty() || word_prefix(seg, prefix)
}

fn config_dir() -> Option<PathBuf> {
    match std::env::var_os("CURSOR_CONFIG_DIR") {
        Some(dir) => Some(PathBuf::from(dir)),
        None => crate::init::home_dir().ok().map(|h| h.join(".cursor")),
    }
}

/// User and project deny rules; `None` when a file exists but cannot be read or parsed.
fn deny_rules(config: Option<&Path>, cwd: Option<&str>) -> Option<Vec<String>> {
    let mut files: Vec<PathBuf> = config
        .map(|d| d.join("cli-config.json"))
        .into_iter()
        .collect();
    if let Some(cwd) = cwd {
        let cwd = Path::new(cwd);
        let root = cwd
            .ancestors()
            .find(|d| d.join(".git").exists())
            .unwrap_or(cwd);
        for dir in cwd.ancestors() {
            files.push(dir.join(".cursor/cli.json"));
            if dir == root {
                break;
            }
        }
    }
    let mut deny = Vec::new();
    for file in files {
        if let Some(json) = read_json(&file)? {
            deny.extend(string_list(&json, "/permissions/deny"));
        }
    }
    Some(deny)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_deny_rule_on_the_original_stops_the_rewrite() {
        let deny = vec!["Shell(git push)".to_string(), "Shell(rm:*)".to_string()];
        assert_eq!(gate_with(&deny, "git push origin main"), Gate::Skip);
        assert_eq!(gate_with(&deny, "cd x && rm -rf y"), Gate::Skip);
        assert_eq!(gate_with(&deny, "FOO=1 git push"), Gate::Skip);
        assert_eq!(gate_with(&deny, "git status"), Gate::Defer);
        assert_eq!(gate_with(&["Shell".to_string()], "ls"), Gate::Skip);
    }

    #[test]
    fn project_cli_json_adds_deny_rules_and_a_bad_one_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        std::fs::create_dir_all(dir.path().join(".cursor")).unwrap();
        let cli = dir.path().join(".cursor/cli.json");
        std::fs::write(&cli, r#"{"permissions":{"deny":["Shell(make deploy)"]}}"#).unwrap();
        let cwd = dir.path().to_str();
        assert_eq!(deny_rules(None, cwd).unwrap(), ["Shell(make deploy)"]);
        std::fs::write(&cli, "{ nope").unwrap();
        assert!(deny_rules(None, cwd).is_none());
    }
}
