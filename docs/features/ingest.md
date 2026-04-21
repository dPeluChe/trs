# `trs ingest` — project digest for AI agents

`trs ingest` walks a repo and produces a compact, token-budget-aware
Markdown digest of the codebase — structure + key files + signatures
— ready to paste into an AI agent's context.

## Quick reference

```bash
trs ingest                      # write digest, print path to stdout
trs ingest --budget 128k        # fit to token budget (signatures first)
trs ingest --deps               # dependency graph only, no content
trs ingest --changed            # only files with uncommitted changes
trs ingest --since-last         # only files changed since last ingest
trs ingest --fresh              # reuse cached digest if HEAD unchanged
trs ingest -o ~/ctx.md          # custom output path (no shadow save)
trs ingest --print              # emit content to stdout instead of path
trs ingest --warn-at 40k        # stderr warning if digest exceeds N tokens
trs ingest --list               # list saved digests + HEAD sha + stale markers
trs ingest --read myproject     # read a saved digest by name
```

## What the digest contains

```
# <project name>

## Structure
  <file tree, gitignore-aware>

## Dependencies
  <from Cargo.toml / package.json / pyproject.toml / go.mod>

## Files (highlights)
  <content of priority files — README, main.rs, etc.>

## Files (signatures)
  <function signatures for files that didn't fit in the budget>
```

Priority is roughly: manifest / README / entry points → business-logic
files → everything else. When the budget is tight, later files drop
to **signatures-only** (function/class/struct names, no bodies) so
the agent still sees the shape of the codebase.

## Budget-aware truncation

`--budget 128k` sets a ~128 000-token target. trs uses a greedy
packing strategy:

1. Manifest + README always fit in full.
2. Entry points (main.rs, index.ts, etc.) fit in full if possible.
3. Remaining files fit in full **or** as signatures, whichever the
   budget allows.
4. If nothing fits at full content, everything degrades to signatures
   so the *map* still exists even at very tight budgets.

Token counts are approximations (~4 bytes per token). Set
`--warn-at <N>` to emit a stderr warning when the digest exceeds
`<N>` tokens — useful in CI where you want a hard ceiling.

## Staleness detection

Digests are saved by default under `~/.trs/ingest/<project-name>.md`
with a sidecar `.meta` file recording the HEAD SHA + mtime at ingest
time. Subsequent `trs ingest --fresh` invocations check:

- **HEAD unchanged** → reuse cached digest (instant).
- **HEAD changed** → regenerate.
- **Uncommitted changes** → always regenerate (the cache is HEAD-
  based, not working-tree-based).

`trs ingest --list` surfaces staleness explicitly:

```
myproject        v0.5.9     2026-04-21 15:30    fresh
old-client       v0.5.6     2026-03-10 09:12    stale (HEAD moved)
```

## Scoped ingests

- `--changed` — only uncommitted files (working tree vs HEAD). Useful
  for "here's what I'm touching, help me finish it."
- `--since-last` — only files changed since the last ingest of this
  project. Useful for incremental context rollouts on long-running
  sessions.
- `--deps` — dependency graph only, no file contents. Useful as a
  lightweight primer when the full digest would exceed budget.

## Ollama post-processing

If you have `ollama` running locally, trs can pipe the digest through
a local model for additional compression / summarization. Configure
via `~/.trs/config.toml`:

```toml
[ingest.ollama]
enabled = true
model = "llama3.2"
prompt = "Summarize this codebase digest while preserving all symbol names and file paths."
```

The post-processing is off by default — enable only if you're sure
about the trade-off (adds latency, changes content that may lose
fidelity).

## Storage layout

```
~/.trs/ingest/
├── myproject.md        # digest content
├── myproject.meta      # { "head_sha": "...", "generated_at": "..." }
├── old-client.md
└── old-client.meta
```

Each digest is keyed by project name (derived from the manifest or
the cwd basename). Delete files here to clear cache — trs will
regenerate on the next `trs ingest`.

## Reading saved digests

```bash
trs ingest --read myproject       # print content to stdout
trs ingest --read myproject | pbcopy   # copy to clipboard
```

Useful when you want to paste the digest into a chat UI that doesn't
have file-upload — or to verify what `trs ingest --fresh` is
actually returning.

## Typical workflows

### One-shot context for a new chat session

```bash
trs ingest --budget 64k | pbcopy
# paste into Claude / GPT / Gemini
```

### Incremental context during a long session

```bash
trs ingest                   # full digest, once at the start
# ... work happens ...
trs ingest --since-last      # only what changed, paste when relevant
```

### CI digest for PR review agents

```bash
trs ingest --changed --budget 32k -o /tmp/pr-context.md --warn-at 20k
# attach /tmp/pr-context.md to the PR review agent invocation
```

## Live example — trs ingesting itself

[`docs/development/codebase-digest.md`](../development/codebase-digest.md)
is the output of `trs ingest --budget 128k` run against this repo.
~100 KB / ~26 k tokens covering 187 files — a realistic example of
what a digest looks like, and a drop-in context primer for agents
working on the trs codebase itself.

## See also

- [`docs/support/commands.md`](../support/commands.md) — `trs ingest`
  in context with other built-in tools.
- [`docs/features/stats.md`](./stats.md) — how ingest runs show up in
  the savings dashboard.
