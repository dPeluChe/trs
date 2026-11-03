//! The script plugins (OpenCode, Kilo, Pi, OpenClaw, Hermes) must follow the
//! same rewrite rules as the hook agents: a pipe, a redirect or a command
//! substitution is never routed through trs, because the next stage would
//! read compressed text (`ls | wc -l` gave 10 instead of 12).

use assert_cmd::Command;

fn rewrite(cmd: &str) -> String {
    let out = Command::cargo_bin("trs")
        .unwrap()
        .arg("rewrite")
        .write_stdin(cmd)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn trs_rewrite_leaves_pipes_captures_and_trs_commands_alone() {
    for cmd in [
        "ls -la /usr | wc -l",
        "git log | head -5",
        "cat a > b",
        "x=$(git status)",
        "trs git status",
        "TRS_AGENT=opencode trs git status",
        "TRS_SKIP=1 git status",
    ] {
        assert_eq!(rewrite(cmd), "", "{cmd} must not be rewritten");
    }
    assert_eq!(rewrite("git status"), "trs git status");
    assert_eq!(rewrite("cd a && git status"), "cd a && trs git status");
}

#[test]
fn no_template_prefixes_trs_blindly() {
    for (name, text) in [
        ("opencode", &*trs_templates::OPENCODE),
        ("kilo", &*trs_templates::KILO),
        ("pi", &*trs_templates::PI),
        ("openclaw", &*trs_templates::OPENCLAW),
        ("hermes", &*trs_templates::HERMES),
    ] {
        assert!(text.contains("\"rewrite\""), "{name} must ask trs rewrite");
        assert!(
            !text.contains("`trs ${") && !text.contains("f\"trs {"),
            "{name} still prefixes trs to every command"
        );
    }
}

/// What `trs init --global <agent>` writes, read back from a temp HOME.
mod trs_templates {
    use assert_cmd::Command;
    use std::sync::LazyLock;

    static HOME: LazyLock<tempfile::TempDir> = LazyLock::new(|| {
        let home = tempfile::tempdir().unwrap();
        for agent in ["opencode", "kilo", "pi", "openclaw", "hermes"] {
            Command::cargo_bin("trs")
                .unwrap()
                .env("HOME", home.path())
                .env("USERPROFILE", home.path())
                .env("TRS_HOME", home.path().join("trs"))
                .args(["init", "--global", agent])
                .assert()
                .success();
        }
        home
    });

    fn read(rel: &str) -> String {
        std::fs::read_to_string(HOME.path().join(rel)).unwrap_or_default()
    }

    pub static OPENCODE: LazyLock<String> =
        LazyLock::new(|| read(".config/opencode/plugins/trs.ts"));
    pub static KILO: LazyLock<String> = LazyLock::new(|| read(".config/kilo/plugins/trs.ts"));
    pub static PI: LazyLock<String> = LazyLock::new(|| read(".pi/agent/extensions/trs.ts"));
    pub static OPENCLAW: LazyLock<String> =
        LazyLock::new(|| read(".openclaw/plugins/trs/index.js"));
    pub static HERMES: LazyLock<String> =
        LazyLock::new(|| read(".hermes/plugins/trs-rewrite/__init__.py"));
}

/// Runs the installed OpenCode plugin under node, with the real `trs` first
/// on PATH. Skipped where node is missing.
#[cfg(unix)]
#[test]
fn the_opencode_plugin_keeps_pipes_intact_and_routes_the_rest() {
    let Ok(node) = which_node() else {
        eprintln!("skipped: node not found");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let plugin = dir.path().join("trs.mjs");
    std::fs::write(&plugin, trs_templates::OPENCODE.as_str()).unwrap();
    let driver = dir.path().join("run.mjs");
    std::fs::write(
        &driver,
        r#"import plugin from "./trs.mjs";
const hooks = await plugin.server();
const out = [];
for (const c of ["ls -la /usr | wc -l", "git status", "cd a && git status", "cat a > b"]) {
  const o = { args: { command: c } };
  await hooks["tool.execute.before"]({ tool: "bash" }, o);
  out.push(o.args.command);
}
console.log(JSON.stringify(out));
"#,
    )
    .unwrap();
    let trs_dir = assert_cmd::cargo::cargo_bin("trs");
    let path = format!(
        "{}:{}",
        trs_dir.parent().unwrap().display(),
        std::env::var("PATH").unwrap()
    );
    let out = std::process::Command::new(node)
        .arg(&driver)
        .env("PATH", path)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let got: Vec<String> = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        got,
        [
            "ls -la /usr | wc -l",
            "trs git status",
            "cd a && trs git status",
            "cat a > b"
        ]
    );
}

#[cfg(unix)]
fn which_node() -> Result<std::path::PathBuf, ()> {
    std::env::var_os("PATH")
        .into_iter()
        .flat_map(|p| std::env::split_paths(&p).collect::<Vec<_>>())
        .map(|d| d.join("node"))
        .find(|p| p.is_file())
        .ok_or(())
}
