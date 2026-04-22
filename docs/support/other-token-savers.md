# Other token-saving tools

trs is one of several tools in the shell-output-compression space for
AI agents. This page lists the alternatives we're aware of, for folks
evaluating options or migrating between tools.

We don't link to these projects directly — go search for them if you
want to compare. The list is descriptive, not promotional, and we
update it as new tools appear.

## Alternatives we've analyzed

- **rtk** (Rust Token Killer) — another Rust-based CLI proxy for
  shell compression. TOML filter pipeline, SQLite usage tracking,
  dedicated `rtk gain` analytics. Overlaps significantly with trs on
  the core rewrite surface.
- **token-optimizer** — Node-based compressor, installs as a global
  npm package. Hook integration focused on Claude Code.
- **token-saver** — early-stage shell wrapper, smaller scope than
  rtk / trs.
- **ccp** (Claude Code Proxy) — proxy-layer approach that sits
  between the agent and the network, compressing responses
  in-flight. Different positioning from per-command rewriters.
- **repomix** — repo → LLM context tool. Overlaps with `trs ingest`
  but focuses on project-digest output rather than per-command
  compression.
- **claw-compactor** — niche compactor for Claude outputs.
- **QMD** — Markdown-focused compressor.
- **Pi Coding Agent** — agent-side rules/prompt compression, not a
  shell wrapper.

## How trs positions itself

- **Rewrites input and output.** Input hooks compress what the agent
  *sees* (`trs rewrite`). The output-saver block compresses what the
  agent *emits*. Most alternatives cover only one side.
- **Nine agents supported.** See [`agents.md`](./agents.md).
- **30+ dedicated parsers.** Plus generic compression fallback so
  unknown commands still save ~30–40%.
- **Single static binary.** No Node / Python / Ruby runtime. ~12 ms
  startup.

## Installing alongside another tool

`trs init` detects existing competitor hooks in your agent configs
and aborts rather than stacking two compressors on the same command
(that would produce double-compressed garbled output). Resolve the
conflict one of two ways:

```bash
trs init --all --global --replace   # scrub competitor hooks, install trs
trs init --all --global --force     # install alongside (risky — double compression)
```

`--replace` cleanly removes hooks matching the known-competitor list
from the target config before writing the trs hook. The other
compressor's binary stays installed — only its hook wiring is
removed.

See also:
- [`docs/features/init.md`](../features/init.md) — collision
  detection mechanics, `@imports` traversal, per-agent behavior.
