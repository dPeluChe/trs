# AGENTS.md: trs (Token-Reducing Shell)

## What is trs

A Rust CLI that transforms noisy terminal output into compact, structured signal.
Reduces token consumption by 68-99% for developers, AI agents, and automation pipelines.

## Pre-generated codebase digest

[`docs/development/codebase-digest.md`](./docs/development/codebase-digest.md)
is a snapshot of the entire trs codebase produced by `trs ingest`.
Drop it into any agent's context for an instant map of the project
without having to run `trs ingest` yourself.

The digest can drift from HEAD between releases. Regenerate before
tagging a release, or whenever `src/` has moved meaningfully.

Run `./scripts/sync-codebase-digest.sh`. It uses `trs` from `PATH` and
falls back to `./target/release/trs` if one isn't installed.

## Related commands worth knowing

- `trs stats`: cumulative savings across commands (shows date range, today
  vs. average, last command, top-reducers).
- `trs doctor`: installation health check. Warns when AGENTS.md / CLAUDE.md
  exceed ~5k tokens and points at `trs audit-docs`.
- `trs audit-docs`: static analysis of agent instruction files
  (CLAUDE.md, AGENTS.md, rules files). Surfaces cross-file duplicates,
  embedded code/SQL/JSON blocks that belong elsewhere, dead `@imports`,
  and (for code fences) cross-references declared symbols against
  the actual source tree (so you can REMOVE ones already defined in src/
  and EXTRACT ones that don't live anywhere yet).
- `trs output-saver`: install a short output-reduction rules block
  into each agent's global config (AGENTS.md / CLAUDE.md / Cursor rules).
  Closes the symmetric gap: trs compresses what agents SEE via
  `trs rewrite`; output-saver compresses what they EMIT via
  anti-preamble / anti-narration / structured-output directives.
  Check-first by default (`--install` to commit, `--remove` to undo).
- `trs ingest`: project digest for LLM consumption. Use symbol index,
  compression levels, or `owner/repo` URL shorthand.
- `trs init --show`: see which AI agents have trs hooks installed.
- `trs upgrade`: detects the install channel (npm / curl) and re-runs
  it for the latest release. `--check` dry-runs, `-y` skips the
  confirmation prompt. See [`docs/features/upgrade.md`](./docs/features/upgrade.md).
- `trs init <tool>`: now runs a collision pre-check: detects hooks
  from other token-compression tools (via `@imports` too) and aborts
  by default. `--replace` removes competitor hooks cleanly, `--force`
  installs alongside (risky, double-compression can corrupt output).
  See [`docs/support/other-token-savers.md`](./docs/support/other-token-savers.md)
  for the list of tools we coexist with.
- `trs stats --by-agent`: breakdown by which AI agent fired each
  rewrite. Reads the `TRS_AGENT` env var that `trs rewrite` and
  plugin templates inject into the command line. Rules-only agents
  and direct shell runs show as `(untagged)`.
- **TRS_SKIP=1 prefix**: per-invocation bypass. Agents (or users)
  that need raw output for a specific command can prefix
  `TRS_SKIP=1 <cmd>`; `trs rewrite` passes it through unchanged.

## Architecture

`src/` is ~216 files. Rather than mirror the tree here (it drifts the
moment anything moves), generate it on demand. trs does this itself:

    trs ingest --print          # structure + module roles + symbols
    trs ingest --html           # same, as a visual report

The load-bearing entry points:

| Path | Role |
|---|---|
| `main.rs` / `cli.rs` / `commands.rs` | entry, flag precedence, command enums |
| `classifier*.rs` | command -> parser routing, subprocess execution |
| `rewrite*.rs` | the hook: decides what gets wrapped with `trs` |
| `router/handlers/` | one parser per command family |
| `formatter/` | output shapes (json / compact / raw) |
| `ingest/` | project digest + `--html` report |
| `output_saver*.rs` | the rules block installed into agent configs |
| `schema/` | shared output types |

## Key Design Decisions

- **Auto-detect**: `trs git status` detects "git" + "status" and routes to git-status parser
- **Flags anywhere**: `trs git status --json` and `trs --json git status` both work
- **Pipe support**: `git status | trs parse git-status` also works
- **No runtime deps**: Single binary, ~7MB, works on macOS/Linux/Windows
- **Modular by design**: 210+ .rs files. Most stay well under 500 LOC; a handful of feature-complete modules (audit_docs, output_saver, init) are larger because splitting them would fragment a single feature across files for no benefit.
- **Token tracking**: Every execution logged to ~/.trs/history.jsonl
- **3-tier fallback**: parser OK → degraded → truncated passthrough with `[trs:passthrough]`
- **Generic fallback**: commands without parser get whitespace/ANSI compression (20-40%)
- **Config system**: `~/.trs/config.toml` for tunable limits
- **Agent integrations**: 9 agents supported across 3 integration types
  (hook / plugin / rules). Wire-format differs per hook agent
  (Claude/Gemini/Cursor), see [`docs/development/agent-integrations.md`](./docs/development/agent-integrations.md)
  for per-agent mechanism, quirks, and test prompts.

## Development

- `cargo build`: build; `cargo install --path .`: install globally.
- `cargo test --no-fail-fast`: full suite (one failing suite shouldn't
  mask the rest).
- `./docs/development/benchmarks/benchmark.sh`: compare against other
  token-savers; see
  [`docs/development/benchmarks/README.md`](./docs/development/benchmarks/README.md).

The exact lint/build/test commands the ship gate uses live in the
`## ship config` block of `CLAUDE.md`.

## Testing

- Unit tests live beside the code in `src/`; integration tests in `tests/`.
- 540+ CLI integration tests (tests/cli_*.rs, 26 files)
- 800+ additional integration tests (70+ test files)
- Run `cargo test --no-fail-fast` for the current totals. CI gates on ubuntu, macOS and Windows with zero warnings.
