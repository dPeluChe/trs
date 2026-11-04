//! Plugin/extension templates: OpenCode, Kilo, Pi (TypeScript) and
//! OpenClaw (JS) / Hermes (Python). Split from init_templates.rs to keep
//! both under the 500-LOC rule; re-exported there so callers are unchanged.

// Shared by the JS plugins. Pipes, redirects, chains and bypass prefixes are
// decided by `trs rewrite` (rewrite_decide.rs), so every agent follows one
// rule set. Fails open: a missing trs or any error keeps the command as is.
macro_rules! js_via_trs {
    () => {
        r#"import { spawnSync } from "node:child_process";

const runTrs = (args, cmd) => {
  try {
    const r = spawnSync("trs", args, {
      input: cmd,
      encoding: "utf8",
      timeout: 3000,
      shell: process.platform === "win32",
    });
    return r.status === 0 && typeof r.stdout === "string" ? r.stdout : "";
  } catch {
    return "";
  }
};

const viaTrs = (cmd) => runTrs(["rewrite"], cmd).replace(/\r?\n$/, "") || cmd;
"#
    };
}

// 1.x hosts match their permission rules against the rewritten command and
// have no way to ask about the original, so the rewrite happens only when the
// effective config has no ask/deny rule at all (unknown config: no rewrite).
macro_rules! js_rules_gate {
    () => {
        r#"
const allowOnly = (v) =>
  v === undefined || v === "allow" ||
  (typeof v === "object" && v !== null && Object.values(v).every((x) => x === "allow"));

const bashUnrestricted = async (client, directory) => {
  try {
    const c = (await client.config.get({ query: { directory } }))?.data ?? {};
    const sets = [c.permission, ...Object.values(c.agent ?? {}).map((a) => a?.permission)];
    return sets.every((p) =>
      !p || (typeof p === "string" ? p === "allow" : allowOnly(p.bash !== undefined ? p.bash : p["*"])));
  } catch {
    return false;
  }
};
"#
    };
}

// OpenCode plugin: one default export serves 1.x (`server`) and 2.x (`setup`).
// 2.x rejects named exports and calls the shell tool `shell`; see
// docs/development/agent-integrations.md (OpenCode).
pub(crate) const OPENCODE_PLUGIN: &str = concat!(
    "// trs plugin: route commands through trs for token-optimized output\n\n",
    js_via_trs!(),
    js_rules_gate!(),
    r#"
// 2.x checks permission rules on the rewritten text: ask about the original.
const segmentsOf = (cmd) => {
  try {
    const segs = JSON.parse(runTrs(["rewrite", "--segments"], cmd));
    return Array.isArray(segs) && segs.length ? segs : null;
  } catch {
    return null;
  }
};

const isPermissionError = (e) => {
  const tag = String(e?._tag ?? e?.name ?? "");
  return !/Schema/.test(tag) && /permission|reject|declin|denied/i.test(`${tag} ${e?.message ?? ""}`);
};

// OpenCode 1.x
const server = async ({ client, directory } = {}) => ({
  // Cross-platform attribution: set the env var, never a shell prefix.
  "shell.env": async (_input, output) => {
    output.env.TRS_AGENT = "opencode";
  },
  "tool.execute.before": async (input, output) => {
    if (input.tool !== "bash") return;
    const cmd = output.args?.command;
    if (typeof cmd !== "string") return;
    const rewritten = viaTrs(cmd);
    if (rewritten === cmd || !client || !(await bashUnrestricted(client, directory))) return;
    output.args.command = rewritten;
  },
});

export default {
  id: "trs",
  server,
  // OpenCode 2.x
  setup: async (ctx) => {
    const approved = new Set();
    const canAsk = typeof ctx.permission?.assert === "function" && typeof ctx.permission?.hook === "function";
    await ctx.shell.hook("create.before", (input) => {
      input.env.TRS_AGENT = "opencode";
    });
    if (canAsk) {
      // The original was approved by assert: do not prompt again on the rewrite.
      await ctx.permission.hook("evaluate", (input) => {
        if (input.effect === "ask" && approved.delete(input.source?.id)) input.effect = "allow";
      });
    }
    await ctx.tool.hook("execute.before", async (input) => {
      if (input.tool !== "shell") return;
      const cmd = input.input?.command;
      if (typeof cmd !== "string") return;
      const rewritten = viaTrs(cmd);
      const resources = rewritten === cmd || !canAsk ? null : segmentsOf(cmd);
      if (!resources) return;
      try {
        await ctx.permission.assert({
          sessionID: input.sessionID,
          agent: input.agent,
          action: "shell",
          resources,
          source: { type: "tool", messageID: input.messageID, id: input.id },
        });
      } catch (e) {
        if (isPermissionError(e)) throw e;
        return;
      }
      approved.add(input.id);
      input.input = { ...input.input, command: rewritten };
    });
  },
};
"#
);

/// Kilo plugin: identical mechanism to OpenCode 1.x but tags the downstream
/// run as `kilo` so history attribution is accurate across forks.
pub(crate) const KILO_PLUGIN: &str = concat!(
    "// trs plugin: route commands through trs for token-optimized output\n\n",
    js_via_trs!(),
    js_rules_gate!(),
    r#"
export const TrsPlugin = async ({ client, directory } = {}) => {
  return {
    "shell.env": async (_input, output) => {
      output.env.TRS_AGENT = "kilo";
    },
    "tool.execute.before": async (input, output) => {
      if (input.tool !== "bash") return;
      const cmd = output.args?.command;
      if (typeof cmd !== "string") return;
      const rewritten = viaTrs(cmd);
      if (rewritten === cmd || !client || !(await bashUnrestricted(client, directory))) return;
      output.args.command = rewritten;
    },
  };
};
"#
);

// Pi (pi.dev) extension: a bash `spawnHook` asks `trs rewrite` and tags the run
// via the `TRS_AGENT=pi` env. `/reload` picks up changes (https://pi.dev).
pub(crate) const PI_EXTENSION: &str = concat!(
    "// trs plugin: route commands through trs for token-optimized output\n",
    r#"import { createBashTool } from "@earendil-works/pi-coding-agent";
"#,
    js_via_trs!(),
    r#"
export default function (pi) {
  const bash = createBashTool(process.cwd(), {
    spawnHook: ({ command, cwd, env }) => ({
      command: typeof command === "string" ? viaTrs(command) : command,
      cwd,
      env: { ...env, TRS_AGENT: "pi" },
    }),
  });
  pi.registerTool({ ...bash });
}
"#
);

// OpenClaw / Hermes plugin templates — installed by init_install_plugins.rs.
pub(crate) const OPENCLAW_PLUGIN_MANIFEST: &str = r#"{
  "id": "trs",
  "name": "trs",
  "description": "Route exec commands through trs for token-optimized output",
  "entry": "index.js"
}
"#;

pub(crate) const OPENCLAW_PLUGIN_INDEX: &str = concat!(
    "// trs plugin: route exec commands through trs for token-optimized output\n",
    "// Validate live: hook payload field names + manifest schema (docs verified\n",
    "// 2026-06-11: before_tool_call rewrites params; resolve_exec_env merges env).\n",
    js_via_trs!(),
    r#"
export default {
  id: "trs",
  register(api) {
    api.on("before_tool_call", (event) => {
      if (event.toolName !== "exec") return;
      const cmd = event.params?.command;
      if (typeof cmd !== "string") return;
      return { params: { ...event.params, command: viaTrs(cmd) } };
    });
    // Cross-platform attribution: env var, never a shell prefix.
    api.on("resolve_exec_env", () => ({ TRS_AGENT: "openclaw" }));
  },
};
"#
);

// Validate live: hook signature + plugin.yaml manifest keys (docs verified
// 2026-06-11: register(ctx) → ctx.register_hook("pre_tool_call", fn)).
pub(crate) const HERMES_PLUGIN_INIT: &str = r#""""trs plugin: route Hermes terminal commands through trs.

Asks `trs rewrite` how to run each terminal command (it leaves pipes,
redirects and bypass prefixes alone) and tags child processes via
TRS_AGENT for attribution. Fails open on any error.
"""

import os
import subprocess


def register(ctx):
    # Cross-platform attribution: children inherit the env var.
    os.environ.setdefault("TRS_AGENT", "hermes")
    ctx.register_hook("pre_tool_call", _pre_tool_call)


def _via_trs(command):
    try:
        r = subprocess.run(
            ["trs", "rewrite"],
            input=command,
            capture_output=True,
            text=True,
            timeout=3,
            shell=(os.name == "nt"),
        )
    except Exception:
        return command
    out = r.stdout.rstrip("\r\n") if r.returncode == 0 else ""
    return out or command


def _pre_tool_call(tool_name=None, args=None, **_kwargs):
    try:
        if tool_name != "terminal" or not isinstance(args, dict):
            return
        command = args.get("command")
        if not isinstance(command, str) or not command.strip():
            return
        args["command"] = _via_trs(command)
    except Exception:
        return
"#;

pub(crate) const HERMES_PLUGIN_YAML: &str = r#"name: trs-rewrite
version: "0.1.0"
description: Rewrite Hermes terminal commands through trs before execution.
author: trs
hooks:
  - pre_tool_call
provides_hooks:
  - pre_tool_call
"#;
