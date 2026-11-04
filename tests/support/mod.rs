//! Helpers shared by integration tests: `mod support;`. Each test file uses a
//! subset, hence the `dead_code` allowance.
#![allow(dead_code)]

use std::path::Path;
use std::process::Command;

/// Run `git` in `dir` as a fixed identity, so commits work on a machine with
/// no git config (CI). Panics if git fails; returns its stdout.
pub fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .output()
        .expect("git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// `PATH` with `dir` first, so programs in it shadow the real ones.
pub fn path_with(dir: &Path) -> String {
    format!("{}:{}", dir.display(), std::env::var("PATH").unwrap())
}

/// A stand-in program: an executable `#!/bin/sh` script with this body.
#[cfg(unix)]
pub fn fake_program(path: &Path, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// Write `text` to `root/rel`, creating parent dirs.
pub fn write_file(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// Run `trs rewrite --caller <caller>` with `input` on stdin and `home` as
/// HOME and (under `trs/`) TRS_HOME. `None` means no output: the command was
/// left alone.
pub fn run_hook(
    caller: &str,
    input: &serde_json::Value,
    home: &Path,
    envs: &[(&str, &Path)],
    unset: &[&str],
    cwd: Option<&Path>,
) -> Option<serde_json::Value> {
    run_hook_with(&["--caller", caller], input, home, envs, unset, cwd)
}

/// `run_hook` with explicit `trs rewrite` flags (none = a legacy hook).
pub fn run_hook_with(
    flags: &[&str],
    input: &serde_json::Value,
    home: &Path,
    envs: &[(&str, &Path)],
    unset: &[&str],
    cwd: Option<&Path>,
) -> Option<serde_json::Value> {
    let mut c = assert_cmd::Command::cargo_bin("trs").unwrap();
    c.env("HOME", home)
        .env("USERPROFILE", home)
        .env("TRS_HOME", home.join("trs"))
        .arg("rewrite")
        .args(flags)
        .write_stdin(input.to_string());
    for k in unset {
        c.env_remove(k);
    }
    for (k, v) in envs {
        c.env(k, v);
    }
    if let Some(cwd) = cwd {
        c.current_dir(cwd);
    }
    let out = c.output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    (!text.trim().is_empty()).then(|| serde_json::from_str(&text).unwrap())
}

/// The `permissionDecision` of a Claude-shaped hook answer.
pub fn decision(v: &serde_json::Value) -> Option<&str> {
    v["hookSpecificOutput"]["permissionDecision"].as_str()
}
