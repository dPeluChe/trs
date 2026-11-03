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

/// Runs `driver` (an ES module importing `./trs.mjs`) under node with the
/// installed plugin text and the real `trs` first on PATH; returns its JSON
/// output. `None` where node is missing.
#[cfg(unix)]
fn run_node(plugin_text: &str, driver: &str) -> Option<serde_json::Value> {
    let Ok(node) = which_node() else {
        eprintln!("skipped: node not found");
        return None;
    };
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("trs.mjs"), plugin_text).unwrap();
    let driver_path = dir.path().join("run.mjs");
    std::fs::write(&driver_path, driver).unwrap();
    let trs_dir = assert_cmd::cargo::cargo_bin("trs");
    let path = format!(
        "{}:{}",
        trs_dir.parent().unwrap().display(),
        std::env::var("PATH").unwrap()
    );
    let out = std::process::Command::new(node)
        .arg(&driver_path)
        .env("PATH", path)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    Some(serde_json::from_slice(&out.stdout).unwrap())
}

/// OpenCode 1.x path: rewrites through `trs rewrite` when the config has no
/// ask/deny rule, and keeps pipes intact.
#[cfg(unix)]
#[test]
fn the_opencode_plugin_keeps_pipes_intact_and_routes_the_rest() {
    let driver = r#"import plugin from "./trs.mjs";
const client = { config: { get: async () => ({ data: {} }) } };
const hooks = await plugin.server({ client, directory: "/x" });
const out = [];
for (const c of ["ls -la /usr | wc -l", "git status", "cd a && git status", "cat a > b"]) {
  const o = { args: { command: c } };
  await hooks["tool.execute.before"]({ tool: "bash" }, o);
  out.push(o.args.command);
}
console.log(JSON.stringify(out));
"#;
    let Some(got) = run_node(&trs_templates::OPENCODE, driver) else {
        return;
    };
    assert_eq!(
        got,
        serde_json::json!([
            "ls -la /usr | wc -l",
            "trs git status",
            "cd a && trs git status",
            "cat a > b"
        ])
    );
}

/// OpenCode 2.x: the plugin asks `permission.assert` about the original
/// segments before rewriting, and a denial stops the command unrewritten.
#[cfg(unix)]
#[test]
fn the_opencode_2x_plugin_asks_permission_about_the_original_command() {
    let driver = r#"import plugin from "./trs.mjs";
async function run(ctxExtra, cmds) {
  const asked = [];
  const hooks = {};
  const ctx = {
    shell: { hook: async () => {} },
    tool: { hook: async (n, f) => { hooks[n] = f; } },
    permission: {
      hook: async (n, f) => { hooks["perm." + n] = f; },
      assert: async (req) => {
        asked.push(req.resources);
        if (req.resources.some((r) => r.startsWith("git push"))) throw new Error("Permission denied: shell");
        if (req.resources.some((r) => r.startsWith("odd"))) {
          const e = new Error("Expected Permission.Source"); e.name = "SchemaError"; throw e;
        }
      },
    },
    ...ctxExtra,
  };
  await plugin.setup(ctx);
  const out = [];
  for (const [i, c] of cmds.entries()) {
    const input = { tool: "shell", id: "id" + i, sessionID: "s", agent: "build", messageID: "m", input: { command: c } };
    let threw = false;
    try { await hooks["execute.before"](input); } catch { threw = true; }
    out.push({ cmd: input.input.command, threw });
  }
  const e1 = { effect: "ask", source: { id: "id1" } };
  const e9 = { effect: "ask", source: { id: "other" } };
  hooks["perm.evaluate"]?.(e1); hooks["perm.evaluate"]?.(e9);
  return { out, asked, evaluate: [e1.effect, e9.effect] };
}
const cmds = ["git push origin", "git status", "cd a && git status", "ls | wc -l", "odd status"];
const res = await run({}, cmds);
const noAssert = await run({ permission: {} }, ["git status"]);
console.log(JSON.stringify({ res, noAssert }));
"#;
    let Some(got) = run_node(&trs_templates::OPENCODE, driver) else {
        return;
    };
    let cmds: Vec<(&str, bool)> = got["res"]["out"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| (o["cmd"].as_str().unwrap(), o["threw"].as_bool().unwrap()))
        .collect();
    assert_eq!(
        cmds,
        [
            ("git push origin", true),
            ("trs git status", false),
            ("cd a && trs git status", false),
            ("ls | wc -l", false),
            ("odd status", false),
        ],
        "denied or unknown commands stay unrewritten, pipes are never asked"
    );
    assert_eq!(
        got["res"]["asked"],
        serde_json::json!([
            ["git push origin"],
            ["git status"],
            ["cd a", "git status"],
            ["odd status"]
        ])
    );
    assert_eq!(got["res"]["evaluate"], serde_json::json!(["allow", "ask"]));
    assert_eq!(got["noAssert"]["out"][0]["cmd"], "git status");
}

/// Kilo (1.x API) cannot ask about the original: it rewrites only when the
/// effective config has no ask/deny rule, and not at all when it cannot read it.
#[cfg(unix)]
#[test]
fn the_kilo_plugin_skips_the_rewrite_when_rules_could_apply() {
    let driver = r#"import { TrsPlugin } from "./trs.mjs";
async function rewritten(client) {
  const hooks = await TrsPlugin({ client, directory: "/x" });
  const o = { args: { command: "git push" } };
  await hooks["tool.execute.before"]({ tool: "bash" }, o);
  return o.args.command;
}
const cfg = (data) => ({ config: { get: async () => ({ data }) } });
console.log(JSON.stringify({
  none: await rewritten(cfg({})),
  allow: await rewritten(cfg({ permission: { bash: "allow" } })),
  allowMap: await rewritten(cfg({ permission: { bash: { "*": "allow" } } })),
  deny: await rewritten(cfg({ permission: { bash: { "*": "allow", "git push *": "deny" } } })),
  ask: await rewritten(cfg({ permission: { bash: { "git push *": "ask" } } })),
  catchAllAsk: await rewritten(cfg({ permission: { "*": "ask" } })),
  agentAsk: await rewritten(cfg({ agent: { build: { permission: { bash: "ask" } } } })),
  unreadable: await rewritten({ config: { get: async () => { throw new Error("x"); } } }),
  noClient: await rewritten(undefined),
}));
"#;
    let Some(got) = run_node(&trs_templates::KILO, driver) else {
        return;
    };
    for key in ["none", "allow", "allowMap"] {
        assert_eq!(got[key], "trs git push", "{key}");
    }
    for key in [
        "deny",
        "ask",
        "catchAllAsk",
        "agentAsk",
        "unreadable",
        "noClient",
    ] {
        assert_eq!(got[key], "git push", "{key} must stay unrewritten");
    }
}

#[test]
fn trs_rewrite_segments_prints_one_simple_command_per_line() {
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["rewrite", "--segments"])
        .write_stdin("cd a && git push; echo 'x && y' | cat")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "cd a\ngit push\necho 'x && y'\ncat\n"
    );
}

/// Pi has no permission engine; a user's `tool_call` gate must keep seeing the
/// original command, so the rewrite lives only in the bash `spawnHook`.
#[test]
fn the_pi_extension_rewrites_only_in_spawn_hook() {
    let pi = trs_templates::PI.as_str();
    assert!(pi.contains("spawnHook"));
    assert!(!pi.contains("tool_call"), "must not hook tool_call");
    assert!(!pi.contains("event.input"), "must not mutate tool input");
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
