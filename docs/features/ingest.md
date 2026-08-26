# `trs ingest`: project digest for AI agents

`trs ingest` walks a repo and produces a compact, token-budget-aware
Markdown digest of the codebase (structure + key files + signatures),
ready to paste into an AI agent's context.

## Quick reference

```bash
trs ingest                      # write digest, print path to stdout
trs ingest --budget 128k        # fit to token budget (signatures first)
trs ingest --deps               # dependency graph only, no content
trs ingest --symbols            # add a flat symbol → file index
trs ingest --changed            # only files with uncommitted changes
trs ingest --since-last         # only files changed since last ingest
trs ingest --fresh              # reuse cached digest if HEAD unchanged
trs ingest -o ~/ctx.md          # custom output path (no shadow save)
trs ingest --print              # emit content to stdout instead of path
trs ingest --agent              # agent format ⇒ implicit --print (content to stdout)
trs ingest --warn-at 40k        # stderr warning if digest exceeds N tokens
trs ingest --list               # list saved digests + HEAD sha + stale markers
trs ingest --read myproject     # read a saved digest by name
trs ingest --html               # self-contained visual report (see below)
trs ingest --html --max-loc 400 # tune the oversized-file threshold
```

**stdout contract:** by default `trs ingest` writes the digest to the store
and prints only the **saved path** (cheap for callers). The agent output
format implies `--print`, so `trs ingest --agent` emits the **digest content**
to stdout directly, no second read step. Passing `-o <file>` always wins:
the content goes to the file and stdout stays the path.

**In-band budget warning (agent mode):** the oversized-digest warning normally
goes to stderr, but agents run with `2>/dev/null`, so it never reaches them.
When `--agent` pulls a large digest with **no `--budget`** set (over the
`--warn-at` threshold), the warning is also written **inside the digest header**
(the one thing the agent always reads), nudging a re-run with `--budget`. Set
a budget, or `--warn-at 0` to silence it.

## What the digest contains

```
# <project name>

> <one-line purpose — manifest description or README first paragraph>

## Structure
  <file tree, gitignore-aware>

## Dependencies
  <from Cargo.toml / package.json / pyproject.toml / go.mod>

## Architecture (module roles)
  <top modules grouped by import-graph role — see below>

## Files (highlights)
  <content of priority files — README, main.rs, etc.>

## Files (signatures)
  <function signatures for files that didn't fit in the budget>
```

Priority is roughly: manifest / README / entry points → business-logic
files → everything else. When the budget is tight, later files drop
to **signatures-only** (function/class/struct names, no bodies) so
the agent still sees the shape of the codebase.

### About line + module roles (the "purpose layer")

Right after the title, the digest states the project's **intent**, the
manifest `description`, or the README's first prose paragraph, so the
reading agent gets *what this is*, not just *what files exist*.

The **Architecture** section classifies the most-connected modules by
pure import-graph topology (fan-in ↓ / fan-out ↑), no AST required:

- **entry**: roots nothing imports (main, CLI).
- **core**: high fan-in, everything routes through them.
- **leaf**: imported by many, import nothing (utils, types).
- **internal**: mid-graph plumbing.

This is the same signal the `--html` report draws as a colored graph,
rendered as a compact list so it costs almost nothing in tokens.

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
`<N>` tokens, useful in CI where you want a hard ceiling.

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

- `--changed`: only uncommitted files (working tree vs HEAD). Useful
  for "here's what I'm touching, help me finish it."
- `--since-last`: only files changed since the last ingest of this
  project. Useful for incremental context rollouts on long-running
  sessions.
- `--deps`: dependency graph only, no file contents. Useful as a
  lightweight primer when the full digest would exceed budget.

## Visual HTML report (`--html`)

`trs ingest --html` swaps the Markdown digest for a **self-contained
HTML report**, a single file with all CSS/JS inlined (no CDN, survives
a strict CSP), light/dark theme aware. It's for humans skimming a repo,
where the Markdown digest is for agents reading one.

```bash
trs ingest --html                    # writes <project>-report.html
trs ingest --html -o report.html     # custom path
trs ingest --html --max-loc 400      # flag files over 400 LOC (default 500)
```

The report renders, from the same ingest data:

- **KPIs**: lines, files, symbols indexed, oversized-file count.
- **File mix**: distribution by extension.
- **Where the code lives**: LOC-by-module bars; click a bar to expand
  its files.
- **How it connects**: an interactive force-directed module graph.
  Circle size = LOC; **color = role** (entry / core / leaf / internal,
  the same purpose layer as the Markdown digest). Hover to preview,
  click a node to pin its links, drag to rearrange.
- **Oversized files**: anything over `--max-loc`, split candidates.
- **Isolated modules**: code folders with **no import edge in or
  out**, unreachable via imports, so likely dead or standalone. This
  is a *module-level* heuristic; it self-suppresses when import
  resolution looks incomplete for the language (>40% flagged), and
  symbol/function-level dead code is deferred to the language's own
  tool (`cargo`, `knip`, `vulture`).
- **Duplicate functions**: near-duplicate function pairs found with
  MinHash + LSH over token shingles (copy-paste candidates to unify).
- **Assets & binaries**: images, media, fonts skipped by the digest
  but real weight in the repo, heaviest first.

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

The post-processing is off by default: enable only if you're sure
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
the cwd basename). Delete files here to clear cache. trs will
regenerate on the next `trs ingest`.

## Reading saved digests

```bash
trs ingest --read myproject       # print content to stdout
trs ingest --read myproject | pbcopy   # copy to clipboard
```

Useful when you want to paste the digest into a chat UI that doesn't
have file-upload, or to verify what `trs ingest --fresh` is
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

## Live example: trs ingesting itself

[`docs/development/codebase-digest.md`](../development/codebase-digest.md)
is the output of `trs ingest --budget 128k` run against this repo.
~100 KB / ~26 k tokens covering 187 files, a realistic example of
what a digest looks like, and a drop-in context primer for agents
working on the trs codebase itself.

## See also

- [`docs/support/commands.md`](../support/commands.md), `trs ingest`
  in context with other built-in tools.
- [`docs/features/stats.md`](./stats.md): how ingest runs show up in
  the savings dashboard.
