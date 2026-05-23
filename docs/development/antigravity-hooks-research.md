# Antigravity hooks — research notes

**Status (2026-05-22)**: agy v1.0.1 does NOT expose user-configurable
PreTool hooks for shell (Bash) commands. trs cannot intercept Antigravity
tool calls until Google ships this surface upstream. v0.6.6 reverted the
v0.6.5 jetski PreToolUse integration and reclassified Antigravity (IDE +
CLI) as rules-only — same posture as Codex CLI and Windsurf.

This file records what we tested, what we found, and what would unblock
re-enabling the programmatic hook.

## Context

Google launched **Antigravity 2.0** on 2026-05-19, simultaneously
releasing the desktop IDE and a CLI binary (`agy`). Both products are
built on Google's internal **jetski** agent framework — visible in the
binary as `google3/third_party/jetski/...` symbols.

trs v0.6.4 wrongly aliased both Antigravity variants to the Gemini CLI
hook harness (`BeforeTool` entry in `~/.gemini/settings.json`). agy
silently ignored that entry — the hook never fired.

trs v0.6.5 attempted to fix this by routing the install to
`~/.gemini/antigravity-{cli,ide}/hooks.json` with a `PreToolUse` event
matching Claude/Codex's envelope (per agy binary strings that reference
`PreToolUse`, `pre_tool_hooks`, etc.). The file loaded successfully
(`cli.log: loaded 1 named hooks from 1 hooks.json file(s)`), but the
hook never fired when agy ran a Bash command. v0.6.6 reverted the
integration.

## Investigation summary

### What we tested

Five distinct hook schemas, each in a fresh `agy` restart with a probe
side-channel (`/tmp/agy_*_fired.log` written by the hook command itself,
so success doesn't depend on trs's response parsing):

1. **Claude-style with regex matcher**: `hooks.PreToolUse[{matcher: ".*",
   hooks: [...], name: "trs-rewrite"}]` — loads, never fires.
2. **Claude-style with exact tool name**: `matcher: "Bash"` — loads,
   never fires.
3. **Claude-style without `name`**: `matcher: ".*"` only — loads, never
   fires.
4. **With `enabled: true`**: explicit enablement flag — loads, never
   fires.
5. **Multi-schema test** (flat `pre_tool_use: "cmd"`, named hook +
   `pre_tool_hooks` reference array): produces a parse error:
   > `failed to parse hooks.json: json: cannot unmarshal array into Go value of type jsonhook.JSONHookSpec`
6. **Single JSONHookSpec at top level** (no `hooks.` wrapper): loads as
   one subagent, never fires for Bash.

The cli.log line `Surfacing tool confirmation: "Bash"` followed by
`Step_RunCommand approved=true` appears for every Bash invocation, but
**no hook-execution log line follows** — confirming the user-config
PreToolHook is not on the Bash execution path.

### Binary evidence

Key findings from `strings ~/.local/bin/agy`:

- `JSONHookSpec` has `jsonschema_description` strings that describe it
  as a **subagent**, not a tool-call wrapper:
  ```
  json:"name"        jsonschema_description:"Unique name for the subagent. Used to invoke it via invoke_subagent."
  json:"description" jsonschema_description:"Human-readable description of what this subagent does and when it should be used."
  ```
- `RegisterPreToolHook` exists in `gemini_coder/framework/registry/`
  but is **internal-only** — no JSON-driven registration path found.
- The only user-visible PreToolHooks in the binary are hardcoded MCP
  browser hooks (`NewMcpBrowserInterceptSnapshotHook`,
  `NewMcpBrowserRecordingStartHook`,
  `NewMcpBrowserScaleCoordinatesHook`) — none related to Bash.
- Bash invocations go through `*gemini_coder_go_proto.Step_RunCommand`,
  which is a separate execution path from the jetski PreTool hook
  pipeline.

### Conclusion

`~/.gemini/antigravity-{cli,ide}/hooks.json` is the loading point for
**subagents** (invocable via `invoke_subagent`), not for tool-call
wrappers. The jetski PreTool hook system exists in code and is wired up
for MCP tools, but there is no user-config surface exposing it for
Bash. As of agy v1.0.1, trs cannot intercept Antigravity shell commands
without an upstream change.

## What unblocks re-enabling the integration

Any of the following would let trs re-introduce a programmatic
Antigravity hook:

1. **Google ships user-configurable PreToolHook for Bash** with a
   documented JSON schema in `hooks.json` (or a separate config file).
   Watch:
   - https://discuss.ai.google.dev/c/antigravity
   - https://antigravity.google/docs (when more pages exist)
   - https://github.com/google-antigravity/antigravity-cli (currently
     no docs in the public repo)
2. **Public documentation of the existing internal hook system** — even
   without new code, a reference for how to register against
   `Step_RunCommand` from a config file would let trs re-target.
3. **An upstream PR adding a `--hook-config` flag or a documented JSON
   schema field that wires custom hooks to Bash dispatch.**

When any of these lands, the revert to remove is mechanical:
- Add an `ANTIGRAVITY_HOOKS` template back to `init_templates.rs`
- Restore `Self::Antigravity` / `Self::AntigravityCLI` arms in
  `init.rs::spec()`
- Add the hook target to `init_install.rs`'s dispatch
- Update `uninstall.rs::candidate_paths` (already covers both the
  v0.6.5 jetski path and v0.6.4 settings.json paths for cleanup)

## What still works in v0.6.6

Even though the programmatic hook is off, **output-saver still works**:

- trs writes `~/.gemini/trs.md` and an `@trs.md` import line in
  `~/.gemini/GEMINI.md`. Both the Antigravity IDE and `agy` honor
  Gemini's `@import` resolution at session start.
- The new `ANTIGRAVITY_RULES_SECTION` block (also in `GEMINI.md`)
  recommends manual `trs <cmd>` prefixing for shell commands.

In practice: agy and the IDE see the trs anti-preamble + numeric-budget
rules at session start, and the user (or the agent voluntarily) can
prefix `trs` to capture token savings on individual commands.

## Why we didn't just leave v0.6.5 installed

The v0.6.5 hooks.json file loaded cleanly but never fired. That's the
worst kind of bug — silent. Users assume compression is active and see
no errors, but their token savings stay at zero for Antigravity
sessions. The revert removes the file outright on `trs init` (and
sweeps the v0.6.4 BeforeTool orphan and the pre-v0.6.4 rules file too)
so the install surface accurately reflects what's running.

## Reproducing

```bash
# Plant a side-channel probe in agy's hooks.json
cat > ~/.gemini/antigravity-cli/hooks.json <<'EOF'
{
  "hooks": {
    "PreToolUse": [{
      "name": "probe",
      "matcher": ".*",
      "hooks": [{
        "type": "command",
        "command": "date +%s >> /tmp/agy_probe.log; trs rewrite"
      }]
    }]
  }
}
EOF
rm -f /tmp/agy_probe.log

# Restart agy fresh, ask it to "ejecuta ls en bash y muéstrame la salida"
# Then check:
ls /tmp/agy_probe.log    # never created → hook didn't fire
grep -iE 'hook|customization' ~/.gemini/antigravity-cli/cli.log
# Expected: only `loaded N named hooks` at startup, no further hook activity.
```

## Related links

- [Hooks in Antigravity — Google AI Developers Forum](https://discuss.ai.google.dev/t/hooks-in-antigravity/120458)
- [Antigravity 2.0 launch announcement](https://antigravityide.org/blog/introducing-google-antigravity-2-0/)
- [Antigravity CLI deep dive (third-party, 2026-05)](https://agentpedia.codes/blog/antigravity-cli-deep-dive)
- [trs PR #47](https://github.com/dPeluChe/trs/pull/47) — v0.6.5 jetski integration that's reverted in v0.6.6
- This investigation: branch `fix/antigravity-rules-only-revert`
