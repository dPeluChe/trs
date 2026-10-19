//! Plugin/extension templates: OpenCode, Kilo, Pi (TypeScript) and
//! OpenClaw (JS) / Hermes (Python). Split from init_templates.rs to keep
//! both under the 500-LOC rule; re-exported there so callers are unchanged.

// OpenCode/Kilo plugin: prefix `trs`, let trs decide whether to compress or
// passthrough. One default export serves both plugin APIs:
//   - OpenCode 1.x (and Kilo): `server` returns a hooks map. `"shell.env"`
//     injects TRS_AGENT into every shell's environment so attribution works on
//     ALL platforms (the old `TRS_AGENT=opencode trs …` command prefix is
//     POSIX-only, see issue #53); `"tool.execute.before"` prepends `trs ` to
//     the bash command.
//   - OpenCode 2.x: requires a default export with `id` and `setup`; the
//     named-export form is rejected ("Plugin must export a default
//     definition"). `setup` registers the same two hooks on `ctx.shell` and
//     `ctx.tool`; the shell tool is named `shell` there, not `bash`.
//   - the guard skips anything already routed through trs, including the
//     legacy `TRS_AGENT=…` prefix, so a retried command can't snowball
// Reference: https://opencode.ai/docs/plugins/
pub(crate) const OPENCODE_PLUGIN: &str = r#"// trs plugin: route commands through trs for token-optimized output

const skip = (cmd) =>
  cmd.startsWith("trs ") || cmd.startsWith("cd ") || cmd.startsWith("TRS_AGENT=");

// OpenCode 1.x
const server = async () => ({
  // Cross-platform attribution: set the env var, never a shell prefix.
  "shell.env": async (_input, output) => {
    output.env.TRS_AGENT = "opencode";
  },
  "tool.execute.before": async (input, output) => {
    if (input.tool !== "bash") return;
    const cmd = output.args?.command;
    if (typeof cmd !== "string" || skip(cmd)) return;
    output.args.command = `trs ${cmd}`;
  },
});

export default {
  id: "trs",
  server,
  // OpenCode 2.x
  setup: async (ctx) => {
    await ctx.shell.hook("create.before", (input) => {
      input.env.TRS_AGENT = "opencode";
    });
    await ctx.tool.hook("execute.before", (input) => {
      if (input.tool !== "shell") return;
      const cmd = input.input?.command;
      if (typeof cmd !== "string" || skip(cmd)) return;
      input.input = { ...input.input, command: `trs ${cmd}` };
    });
  },
};
"#;

/// Kilo plugin — identical mechanism to OpenCode 1.x but tags the downstream
/// run as `kilo` so history attribution is accurate across forks.
pub(crate) const KILO_PLUGIN: &str = r#"// trs plugin: route commands through trs for token-optimized output

export const TrsPlugin = async () => {
  return {
    "shell.env": async (_input, output) => {
      output.env.TRS_AGENT = "kilo";
    },
    "tool.execute.before": async (input, output) => {
      if (input.tool !== "bash") return;
      const cmd = output.args?.command;
      if (typeof cmd !== "string") return;
      if (cmd.startsWith("trs ") || cmd.startsWith("cd ") || cmd.startsWith("TRS_AGENT=")) return;
      output.args.command = `trs ${cmd}`;
    },
  };
};
"#;

// Pi (pi.dev) extension: overrides the bash tool with a `spawnHook` that
// prepends `trs` and tags the run via `TRS_AGENT=pi` (attribution lives in the
// env, not a shell prefix — works on every platform). Auto-discovered from
// ~/.pi/agent/extensions/ (global) or .pi/extensions/ (project); `/reload` to
// pick up changes. Reference: https://pi.dev (earendil-works/pi).
pub(crate) const PI_EXTENSION: &str = r#"// trs plugin: route commands through trs for token-optimized output
import { createBashTool } from "@earendil-works/pi-coding-agent";

export default function (pi) {
  const bash = createBashTool(process.cwd(), {
    spawnHook: ({ command, cwd, env }) => {
      // Idempotent: skip anything already routed through trs (or a cd).
      const skip =
        typeof command !== "string" ||
        command.startsWith("trs ") ||
        command.startsWith("cd ") ||
        command.startsWith("TRS_AGENT=");
      return {
        command: skip ? command : `trs ${command}`,
        cwd,
        env: { ...env, TRS_AGENT: "pi" },
      };
    },
  });
  pi.registerTool({ ...bash });
}
"#;

// OpenClaw / Hermes plugin templates — installed by init_install_plugins.rs.
pub(crate) const OPENCLAW_PLUGIN_MANIFEST: &str = r#"{
  "id": "trs",
  "name": "trs",
  "description": "Route exec commands through trs for token-optimized output",
  "entry": "index.js"
}
"#;

pub(crate) const OPENCLAW_PLUGIN_INDEX: &str = r#"// trs plugin: route exec commands through trs for token-optimized output
// Validate live: hook payload field names + manifest schema (docs verified
// 2026-06-11: before_tool_call rewrites params; resolve_exec_env merges env).
export default {
  id: "trs",
  register(api) {
    api.on("before_tool_call", (event) => {
      if (event.toolName !== "exec") return;
      const cmd = event.params?.command;
      if (typeof cmd !== "string") return;
      // Idempotent: skip anything already routed through trs (or a cd).
      if (cmd.startsWith("trs ") || cmd.startsWith("cd ") || cmd.startsWith("TRS_AGENT=")) return;
      return { params: { ...event.params, command: `trs ${cmd}` } };
    });
    // Cross-platform attribution: env var, never a shell prefix.
    api.on("resolve_exec_env", () => ({ TRS_AGENT: "openclaw" }));
  },
};
"#;

// Validate live: hook signature + plugin.yaml manifest keys (docs verified
// 2026-06-11: register(ctx) → ctx.register_hook("pre_tool_call", fn)).
pub(crate) const HERMES_PLUGIN_INIT: &str = r#""""trs plugin: route Hermes terminal commands through trs.

Prepends `trs ` to terminal tool commands (idempotent) and tags child
processes via TRS_AGENT for attribution. Fails open on any error.
"""

import os


def register(ctx):
    # Cross-platform attribution: children inherit the env var.
    os.environ.setdefault("TRS_AGENT", "hermes")
    ctx.register_hook("pre_tool_call", _pre_tool_call)


def _pre_tool_call(tool_name=None, args=None, **_kwargs):
    try:
        if tool_name != "terminal" or not isinstance(args, dict):
            return
        command = args.get("command")
        if not isinstance(command, str):
            return
        stripped = command.strip()
        if not stripped or stripped.startswith(("trs ", "cd ", "TRS_AGENT=")):
            return
        args["command"] = f"trs {stripped}"
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
