//! Permission gate for Gemini CLI: TOML policies and `tools.allowed`; it ignores a hook allow, so the gate only decides Defer (rewrite, no decision) or Skip.
//! Contract: docs/development/agent-integrations.md (permissions).

use crate::rewrite_gate::Gate;
use crate::rewrite_permissions::{read_only, split_segments, strip_env};
use std::path::{Path, PathBuf};

#[path = "rewrite_perm_gemini_policy.rs"]
mod policy;
use policy::{add_allowed, add_toml, parse_settings, Rules};

/// What the rewritten command looks like to Gemini's prefix rules.
const REWRITTEN_PROBE: &str = "TRS_AGENT=gemini trs x";

/// `cmd` is the ORIGINAL command; `cwd` comes from the hook payload.
pub(crate) fn gate(cmd: &str, cwd: Option<&str>) -> Gate {
    let home = std::env::var_os("GEMINI_CLI_HOME")
        .map(PathBuf::from)
        .or_else(|| crate::init::home_dir().ok());
    let Some(home) = home else {
        return Gate::Skip;
    };
    let env = TrustEnv {
        trust: std::env::var("GEMINI_CLI_TRUST_WORKSPACE").ok(),
        restricted: std::env::var("GEMINI_RESTRICTED_MODE").as_deref() == Ok("true"),
    };
    match load(&home.join(".gemini"), cwd.map(Path::new), &env) {
        Some(rules) => gate_with(&rules, cmd),
        None => Gate::Skip,
    }
}

struct TrustEnv {
    trust: Option<String>,
    restricted: bool,
}

fn gate_with(rules: &Rules, cmd: &str) -> Gate {
    if rules.block_all {
        return Gate::Skip;
    }
    let segs: Vec<&str> = split_segments(cmd)
        .into_iter()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "cd" && !s.starts_with("cd "))
        .collect();
    // Gemini checks the whole command and each part; a leading VAR= dodges its prefix rules.
    for text in std::iter::once(cmd.trim()).chain(segs.iter().copied()) {
        for t in [text, strip_env(text)] {
            if rules.deny.iter().chain(&rules.ask).any(|p| p.matches(t)) {
                return Gate::Skip;
            }
        }
    }
    if rules.blanket_allow {
        return Gate::Defer;
    }
    // Narrow allow rules deny whatever they do not match, and `TRS_AGENT=.. trs` matches none of theirs.
    if !rules.allow.is_empty() && !rules.allow.iter().any(|p| p.matches(REWRITTEN_PROBE)) {
        return Gate::Skip;
    }
    // Gemini runs read-only commands without asking; a rewrite would add a prompt.
    if !segs.is_empty() && segs.iter().all(|s| read_only(strip_env(s))) {
        return Gate::Skip;
    }
    Gate::Defer
}

/// `None` when a rule file exists but cannot be read or parsed.
fn load(gem_dir: &Path, cwd: Option<&Path>, env: &TrustEnv) -> Option<Rules> {
    let user_file = gem_dir.join("settings.json");
    let user = read_settings(&user_file)?;
    let mut allowed = user.allowed;
    let mut yolo = user.yolo;
    let project_file = cwd.map(|c| c.join(".gemini/settings.json"));
    if let Some(pf) = project_file.filter(|pf| *pf != user_file && pf.exists()) {
        if trusted(cwd?, user.folder_trust, env, gem_dir) {
            let project = read_settings(&pf)?;
            // tools.allowed is replaced, not merged, by the workspace scope.
            allowed = project.allowed.or(allowed);
            yolo |= project.yolo;
        }
    }
    let mut rules = Rules::default();
    add_allowed(&mut rules, &allowed.unwrap_or_default());
    rules.blanket_allow |= yolo;
    let mut dirs = vec![gem_dir.join("policies")];
    if !cfg!(test) {
        dirs.push(system_dir().join("policies"));
    }
    for dir in dirs {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut files: Vec<PathBuf> = rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "toml"))
            .collect();
        files.sort();
        for f in files {
            add_toml(&mut rules, &std::fs::read_to_string(&f).ok()?)?;
        }
    }
    Some(rules)
}

fn system_dir() -> PathBuf {
    PathBuf::from(if cfg!(target_os = "macos") {
        "/Library/Application Support/GeminiCli"
    } else if cfg!(windows) {
        "C:\\ProgramData\\gemini-cli"
    } else {
        "/etc/gemini-cli"
    })
}

fn read_settings(path: &Path) -> Option<policy::Settings> {
    match std::fs::read_to_string(path) {
        Ok(text) => parse_settings(&text),
        Err(e) if crate::rewrite_rules::is_missing(&e) => Some(policy::Settings {
            folder_trust: true,
            ..Default::default()
        }),
        Err(_) => None,
    }
}

/// Gemini's folder trust: env wins, then the setting, then `trustedFolders.json`.
fn trusted(cwd: &Path, enabled: bool, env: &TrustEnv, gem_dir: &Path) -> bool {
    if env.restricted || env.trust.as_deref() == Some("false") {
        return false;
    }
    if env.trust.as_deref() == Some("true") || !enabled {
        return true;
    }
    let Ok(text) = std::fs::read_to_string(gem_dir.join("trustedFolders.json")) else {
        return false;
    };
    let Ok(serde_json::Value::Object(map)) = serde_json::from_str(&text) else {
        return false;
    };
    let mut trusted = false;
    for (path, kind) in &map {
        let p = Path::new(path);
        let root = match kind.as_str() {
            Some("TRUST_FOLDER") => Some(p),
            Some("TRUST_PARENT") => p.parent(),
            Some("DO_NOT_TRUST") if cwd.starts_with(p) => return false,
            _ => None,
        };
        trusted |= root.is_some_and(|r| cwd.starts_with(r));
    }
    trusted
}

#[cfg(test)]
mod tests {
    use super::*;

    const OFF: TrustEnv = TrustEnv {
        trust: None,
        restricted: false,
    };

    /// A throwaway `~/.gemini` with the given settings and policy files.
    fn gem(settings: Option<&str>, policies: &[(&str, &str)]) -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        if let Some(s) = settings {
            std::fs::write(d.path().join("settings.json"), s).unwrap();
        }
        std::fs::create_dir_all(d.path().join("policies")).unwrap();
        for (n, t) in policies {
            std::fs::write(d.path().join("policies").join(n), t).unwrap();
        }
        d
    }

    fn run(d: &tempfile::TempDir, cwd: Option<&Path>, env: &TrustEnv, cmd: &str) -> Gate {
        load(d.path(), cwd, env).map_or(Gate::Skip, |r| gate_with(&r, cmd))
    }

    const DENY: &str = "[[rule]]\ntoolName = \"run_shell_command\"\ncommandPrefix = [\"git push\", \"rm -rf\"]\ndecision = \"deny\"\npriority = 900\n";

    #[test]
    fn toml_deny_and_ask_stop_the_rewrite_even_behind_env_or_chains() {
        let ask = "[[rule]]\ntoolName = \"run_shell_command\"\ncommandPrefix = \"npm publish\"\ndecision = \"ask_user\"\npriority = 900\n";
        let d = gem(None, &[("a.toml", DENY), ("b.toml", ask)]);
        for cmd in [
            "git push origin main",
            "FOO=1 git push",
            "cargo build && rm -rf build",
            "npm publish",
        ] {
            assert_eq!(run(&d, None, &OFF, cmd), Gate::Skip, "{cmd}");
        }
        assert_eq!(run(&d, None, &OFF, "cargo build"), Gate::Defer);
        assert_eq!(run(&d, None, &OFF, "git pushy"), Gate::Defer, "word break");
    }

    #[test]
    fn command_regex_and_opaque_rules() {
        let re = "[[rule]]\ntoolName = \"run_shell_command\"\ncommandRegex = \"git (push|reset)\"\ndecision = \"deny\"\n";
        let d = gem(None, &[("a.toml", re)]);
        assert_eq!(run(&d, None, &OFF, "git reset --hard"), Gate::Skip);
        assert_eq!(run(&d, None, &OFF, "cargo build"), Gate::Defer);
        let args = "[[rule]]\ntoolName = \"run_shell_command\"\nargsPattern = \"x\"\ndecision = \"ask_user\"\n";
        let d = gem(None, &[("a.toml", args)]);
        assert_eq!(run(&d, None, &OFF, "cargo build"), Gate::Skip);
        let bad = "[[rule]]\ntoolName = \"run_shell_command\"\ncommandRegex = \"(\"\ndecision = \"deny\"\n";
        assert_eq!(
            run(&gem(None, &[("a.toml", bad)]), None, &OFF, "ls"),
            Gate::Skip
        );
    }

    #[test]
    fn blanket_deny_skips_and_other_tools_are_ignored() {
        let all = "[[rule]]\ntoolName = \"run_shell_command\"\ndecision = \"deny\"\n";
        assert_eq!(
            run(&gem(None, &[("a.toml", all)]), None, &OFF, "cargo build"),
            Gate::Skip
        );
        let other = "[[rule]]\ntoolName = \"write_file\"\ndecision = \"deny\"\n";
        assert_eq!(
            run(&gem(None, &[("a.toml", other)]), None, &OFF, "cargo build"),
            Gate::Defer
        );
    }

    #[test]
    fn narrow_allow_skips_unless_it_covers_the_rewrite_or_is_blanket() {
        let narrow = r#"{ // comment
          "tools": {"allowed": ["run_shell_command(cargo test)"]} /* x */ }"#;
        let d = gem(Some(narrow), &[]);
        assert_eq!(run(&d, None, &OFF, "cargo test"), Gate::Skip);
        assert_eq!(run(&d, None, &OFF, "cargo build"), Gate::Skip);
        let covers = r#"{"tools":{"allowed":["run_shell_command(TRS_AGENT=gemini trs)"]}}"#;
        assert_eq!(
            run(&gem(Some(covers), &[]), None, &OFF, "cargo build"),
            Gate::Defer
        );
        let blanket = r#"{"tools":{"allowed":["run_shell_command"]}}"#;
        assert_eq!(
            run(&gem(Some(blanket), &[]), None, &OFF, "git status"),
            Gate::Defer
        );
        let yolo = r#"{"general":{"defaultApprovalMode":"yolo"}}"#;
        assert_eq!(
            run(&gem(Some(yolo), &[]), None, &OFF, "git status"),
            Gate::Defer
        );
    }

    #[test]
    fn blanket_allow_does_not_override_a_deny() {
        let blanket = r#"{"tools":{"allowed":["run_shell_command"]}}"#;
        let d = gem(Some(blanket), &[("a.toml", DENY)]);
        assert_eq!(run(&d, None, &OFF, "git push"), Gate::Skip);
    }

    #[test]
    fn read_only_commands_are_left_alone_unless_allowed_wholesale() {
        let d = gem(None, &[]);
        assert_eq!(run(&d, None, &OFF, "git status"), Gate::Skip);
        assert_eq!(run(&d, None, &OFF, "ls -la && git log -3"), Gate::Skip);
        assert_eq!(run(&d, None, &OFF, "cargo build"), Gate::Defer);
        assert_eq!(
            run(&d, None, &OFF, "git status && cargo build"),
            Gate::Defer
        );
    }

    #[test]
    fn unreadable_files_fail_closed() {
        assert_eq!(
            run(&gem(Some("{ nope"), &[]), None, &OFF, "cargo build"),
            Gate::Skip
        );
        assert_eq!(
            run(
                &gem(None, &[("a.toml", "[[rule")]),
                None,
                &OFF,
                "cargo build"
            ),
            Gate::Skip
        );
    }

    #[test]
    fn project_allow_counts_only_in_a_trusted_folder_and_replaces_the_user_list() {
        let user = r#"{"tools":{"allowed":["run_shell_command(cargo test)"]}}"#;
        let d = gem(Some(user), &[]);
        let proj = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(proj.path().join(".gemini")).unwrap();
        std::fs::write(
            proj.path().join(".gemini/settings.json"),
            r#"{"tools":{"allowed":["run_shell_command"]}}"#,
        )
        .unwrap();
        let cwd = Some(proj.path());
        assert_eq!(run(&d, cwd, &OFF, "cargo build"), Gate::Skip, "untrusted");
        let yes = TrustEnv {
            trust: Some("true".into()),
            restricted: false,
        };
        assert_eq!(run(&d, cwd, &yes, "cargo build"), Gate::Defer, "trusted");
        let no = TrustEnv {
            trust: Some("false".into()),
            restricted: false,
        };
        assert_eq!(run(&d, cwd, &no, "cargo build"), Gate::Skip);
        std::fs::write(
            d.path().join("trustedFolders.json"),
            format!(r#"{{"{}": "TRUST_FOLDER"}}"#, proj.path().display()),
        )
        .unwrap();
        assert_eq!(
            run(&d, cwd, &OFF, "cargo build"),
            Gate::Defer,
            "trustedFolders.json"
        );
    }

    #[test]
    fn trust_file_kinds() {
        let d = gem(None, &[]);
        let f = |json: &str| {
            std::fs::write(d.path().join("trustedFolders.json"), json).unwrap();
            trusted(Path::new("/w/a/b"), true, &OFF, d.path())
        };
        assert!(f(r#"{"/w/a":"TRUST_FOLDER"}"#));
        assert!(f(r#"{"/w/a/b":"TRUST_PARENT"}"#));
        assert!(!f(r#"{"/w/a":"TRUST_FOLDER","/w/a/b":"DO_NOT_TRUST"}"#));
        assert!(!f("not json"));
        assert!(
            trusted(Path::new("/w"), false, &OFF, d.path()),
            "folderTrust off"
        );
    }
}
