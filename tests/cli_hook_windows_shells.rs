//! The installed hook commands (`trs rewrite --caller x`) must run under both
//! shells an agent can launch them with on Windows. They carry no shell syntax;
//! this proves it by running them under cmd.exe and PowerShell with the JSON on
//! stdin. Windows only: the other platforms have no such shell.
#![cfg(windows)]

use std::io::Write;
use std::process::{Command, Stdio};

fn run(program: &str, args: &[&str]) -> (bool, String) {
    let real = assert_cmd::cargo::cargo_bin("trs");
    let path = format!(
        "{};{}",
        real.parent().unwrap().display(),
        std::env::var("PATH").unwrap()
    );
    let home = tempfile::tempdir().unwrap();
    let mut child = Command::new(program)
        .args(args)
        .env("PATH", path)
        .env("USERPROFILE", home.path())
        .env("HOME", home.path())
        .env("TRS_HOME", home.path().join("trs"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let input = r#"{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"git status"}}"#;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn hook_commands_run_under_cmd_and_powershell() {
    for caller in ["codex", "droid", "vscode", "devin-cli"] {
        let hook = format!("trs rewrite --caller {caller}");
        let shells: [(&str, Vec<&str>); 2] = [
            ("cmd", vec!["/C", &hook]),
            (
                "powershell",
                vec!["-NoProfile", "-NonInteractive", "-Command", &hook],
            ),
        ];
        for (program, args) in shells {
            let (ok, stdout) = run(program, &args);
            assert!(ok, "{program}: `{hook}` exited non-zero");
            assert!(
                stdout.contains("trs git status"),
                "{program}: `{hook}` printed {stdout:?}"
            );
        }
    }
}
