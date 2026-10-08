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

/// A stand-in program: an executable `#!/bin/sh` script with this body.
#[cfg(unix)]
pub fn fake_program(path: &Path, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}
