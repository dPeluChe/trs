# `trs audit-docs` — find bloat in CLAUDE.md / AGENTS.md / rules files

Agent instruction files (`CLAUDE.md`, `AGENTS.md`, `.cursor/rules/*.mdc`,
`.windsurfrules`) get loaded into **every** agent session — every turn,
every project open, every conversation start. Bloat in these files is
the single most expensive kind of bloat because it multiplies across
every interaction.

`trs audit-docs` is a static analyzer that finds:

- Cross-file duplicate sections (SimHash over 3-word shingles; flags
  blocks with Hamming distance ≤ 6, i.e. ≥ 90% similar).
- Dead `@imports` — references to files that don't exist.
- Embedded code / SQL / JSON / YAML / tables that belong in their own
  files rather than inline in rules.
- Code fences whose declared symbols already exist in the project's
  source tree (so you can replace the snippet with a `src/…:NN` link)
  or don't exist yet (so you can extract them into new files).

## Quick reference

```bash
trs audit-docs                  # audit current directory
trs audit-docs path/to/docs     # audit a specific folder
```

The output groups findings by file with line numbers and a short
one-line description. Nothing is modified — this is a read-only
report.

## What it scans

By default, these paths in the current directory (if they exist):

- `CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `CURSOR.md`
- `.windsurfrules`
- `.cursor/rules/*.mdc`
- `.agent/rules/*.md`
- `.agents/rules/*.md`
- `.codex/rules/*.md`

File walker respects `.gitignore`. Max 2000 files walked, depth cap of
8, so running it on a mega-repo is bounded.

## Duplicate detection (SimHash)

Text blocks are split on double-newlines, tokenized into 3-word
shingles, FNV-1a hashed into 64-bit SimHashes. Two blocks with Hamming
distance ≤ 6 are considered near-duplicates (≥ 90% overlap).

Blocks must be:

- **At least 60 characters**, and
- **At least 2 lines**

Otherwise the detector would flag every similar heading (`## Install`
vs `## Installation`) as duplicates and drown the report in noise.

## Dead `@imports`

Scans `@path/to/file.md` references inside Claude Code / Gemini CLI
rules files. An import is flagged as dead if:

- The path is relative and doesn't resolve to an existing file.
- The path looks like an `@import` (has `./`, `../`, or a known file
  extension) — we explicitly skip npm-package-style `@scope/package`
  mentions to avoid false positives.

## Embedded bloat

Fenced code blocks, JSON blobs, SQL queries, and HTML tables inside
rules files. These inflate the per-session load without adding
instructional value — they're content that should live in source
files, docs, or test fixtures.

For code fences, we cross-reference declared symbols against the
project's actual source tree:

- If the symbol (e.g. `fn handle_request`, `class UserService`)
  **already exists** in `src/`, the fence is flagged as `REMOVE` —
  replace with a source-file link.
- If the symbol **doesn't exist anywhere**, the fence is flagged as
  `EXTRACT` — copy to a real file, then link to it.

Symbol matching uses a blocklist of generic names (`data`, `result`,
`page`, `value`, `json`, …) to avoid matching on project-agnostic
identifiers.

## Language support for symbol extraction

TypeScript, JavaScript, Python, Rust, Go, Swift. Other languages get
duplicate / dead-ref analysis only, no symbol cross-reference.

## Integration with `trs doctor`

`trs doctor` reports the total token budget of agent-docs in the
current directory and hints at `audit-docs` when any file exceeds
~5k tokens. That hint is why you're probably reading this page.

## Philosophy

`audit-docs` is a user tool, not an agent tool. The agent maintainer
runs it periodically. Typical cadence: after adding a new rules file,
after several rounds of edits, before cutting a release of a product
that ships rules. Not every session, not every commit.

## Output format example

```
docs/CLAUDE.md (4 findings):

  L12-L45  ~  near-duplicate with docs/AGENTS.md:L8-L41
  L78-L92  ~  embedded SQL query (25 lines, 680 tokens)
              → move to docs/queries/user-lookup.sql and link
  L103-L120 ~  embedded code fence — symbol `handleLogin` already
              defined in src/auth/handlers.rs:44
              → REMOVE, replace with link
  L138     ~  dead @import: @./removed-rules.md
              → file does not exist; remove the line

total: 4 findings
```

## See also

- [`trs doctor`](doctor.md) — surfaces the warning that points here.
- [`trs output-saver`](output-saver.md) — writes a small rules block
  that `audit-docs` will recognize via its sentinels and skip from
  duplicate-detection noise.
- [`docs/development/agent-integrations.md`](../development/agent-integrations.md) — per-agent
  reference for which paths get auto-loaded.
