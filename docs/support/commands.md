# Supported commands

Every command supported by trs falls into one of four levels.

1. **Dedicated parser.** trs spawns the tool, parses its native output,
   and emits a structured compact form. Typical reduction **68–99%**.
2. **Dispatched alias.** A different binary with the same semantics
   (e.g. `rg` for `grep`, `eza` for `ls`) gets routed to the same
   parser. No configuration, the dispatcher recognizes the binary
   name.
3. **Generic compression.** Commands without a parser still get ANSI
   stripping, whitespace collapse, and repeated-line deduplication.
   Typical reduction **30–40%** "free."
4. **Passthrough.** Commands where trs detects a flag that already
   produces structured output (`--json`, `--porcelain`) are passed
   through untouched, the agent gets the raw structured form.

## Commands with dedicated parsers

### VCS: git

| Command | Subcommands parsed |
|---|---|
| `git` | `status`, `diff`, `log`, `branch`, `push`, `pull`, `fetch`, `show`, `stash show -p`, `stash pop`, `stash apply`, `grep` |

Notes: `--no-verify` is blocked on `git commit` / `git push` to
protect pre-commit hooks from AI agents that default to bypassing
them. `git status --porcelain` passes through untouched.
`git show` and `git stash show -p` are routed to the diff parser
(~90% reduction on commits with modifications).
`git push / pull / fetch` strips `remote:` progress lines (Counting,
Compressing, Total), ~85% reduction vs ~35% without this filter.

### Build: Rust

| Command | Subcommands parsed |
|---|---|
| `cargo` | `build`, `check`, `clippy`, `test`, `fmt`, `install`, `add` |

Notes: env-var prefix preserved, `RUSTFLAGS=xyz cargo build` is
rewritten to `RUSTFLAGS=xyz trs cargo build` so the flag still reaches
cargo.

### Build: JavaScript / TypeScript

| Command | Subcommands parsed |
|---|---|
| `npm` | `install` (+`i`, `ci`), `test`, `ls` / `list`, `audit`, `outdated`, `run` |
| `pnpm` | `install` (+`i`), `test`, `ls`, `audit`, `outdated`, `why`, `add`, `update`, `up`, `run`, `dlx`, `exec` |
| `yarn` | `install`, `test` |
| `bun` | `install`, `test`, `run` |
| `npx` / `bunx` / `pnpm dlx` | routed to whichever inner command is invoked |

`run` subcommand routing: `build*` → Build parser, `test*` → Test parser, `lint` / `type-check` / `typecheck` / `check` / `format` / `format:check` → Lint parser. Other script names fall through to generic compression.

### Build: Go

| Command | Subcommands parsed |
|---|---|
| `go` | `test`, `build`, `mod` |

### Build: Python

| Command | Subcommands parsed |
|---|---|
| `pip` / `pip3` | `install`, `list`, `freeze`, `show` |
| `uv` | `pip`, `sync`, `add`, `remove`, `run`, `tree` |
| `python3 -m <module>` | routed: `pytest` → test, `mypy` / `ruff` / `pylint` / `flake8` → lint, `unittest` → test |

### Tests

| Command | What gets parsed |
|---|---|
| `pytest` | full runner output, pass/fail counts, failure locations, tracebacks |
| `jest` | full runner output, suite summary, failed assertions |
| `vitest` | full runner output |
| `cargo test` | see "Build, Rust" |
| `go test` | see "Build, Go" |
| `npm test` / `pnpm test` / `bun test` / `yarn test` | dispatched to the inner runner |

### Linters

| Command | What gets parsed |
|---|---|
| `cargo clippy` | grouped by file + rule |
| `eslint` | issues grouped by file |
| `biome` | issues grouped by file |
| `ruff` | issues grouped by file |
| `pylint` | issues grouped by file |
| `golangci-lint` | issues grouped by file |
| `tsc` | `file(line,col): error TS6133: …`, grouped by file, ~80% reduction |

`tsc` is also reached via `npx tsc`, `bunx tsc`, `pnpm dlx tsc`, and `uv run tsc`.
`biome` is also reached via `npx @biomejs/biome` (full package-name form).
Dispatched the same as any other linter (compact by default, `--json` for structured output).

### Files & search

| Command | Aliases routed | Notes |
|---|---|---|
| `ls` | `lsd`, `exa`, `eza` | long format parsed; `--json` passthrough |
| `find` | `fd` | result list compacted |
| `grep` | `rg`, `ag`, `ack` | line/match format; also `git grep` |
| `tree` | — | directory tree compressed |
| `tail` | `journalctl` | log-tail with error filter |

### File reading intercepts

These commands are intercepted before subprocess spawn: trs reads the
file directly and applies `filter_minimal` (strip comment-only lines,
normalize blank lines). Falls back to raw content when filtering would
return empty output (e.g. an all-comment slice).

| Command | Pattern | Typical reduction |
|---|---|---|
| `cat` | `cat FILE [FILE2…]`, no flags | 10–35% on code files |
| `head` | `head [-n N \| -N] FILE` | 5–20% |
| `sed` | `sed -n X,Yp FILE` (line-range only) | 10–25% vs 0% passthrough |

Any other `sed` form (substitutions, in-place `-i`, multiple files)
falls through to the subprocess path unchanged.

### Containers & GitHub CLI

| Command | Subcommands parsed |
|---|---|
| `docker` | `ps`, `logs`, `build` |
| `gh` | `pr list`, `pr view`, `pr diff`, `pr checks`, `issue list`, `run list`, `run view`, plus `gh api <endpoint>` passthrough tracked in stats |

`gh pr view` extracts title, state, author, url, labels, and a
3-line body preview. `gh pr diff` routes to the git-diff parser
(~90% reduction). `gh pr checks` summarises pass/fail/pending
counts and lists only non-passing checks with duration. `gh run view`
extracts title, conclusion, job counts, up to 3 annotations, and
the run URL.

### Cloud CLIs

| Command | Subcommands parsed |
|---|---|
| `aws` | `s3` recursive operations (`rm`, `rb`, `cp`, `sync`, `mv`); `describe-*` / `get-*` / `list-*` JSON bodies |

Recursive `s3` calls print one receipt line per object
(`delete: s3://bucket/key`), which is the most repetitive output trs
sees, a real 3.3 MB deletion compressed to 275 bytes (99.99%). The
summary keeps the verb, the object count and the busiest key prefixes:

```
delete: 50000 objects, s3://bucket/logs/2026/01/ (596), … +81 more prefixes
problems (1):
  An error occurred (AccessDenied) when calling the DeleteObject operation
```

Failures are pulled out and never truncated: AWS's canonical
`An error occurred (…)` line carries no colon after "error", so the
generic error markers miss it and it is matched explicitly. JSON bodies
are forwarded to the same compressor `gh api` uses; output that matches
neither shape passes through untouched rather than being guessed at.

### System & network

| Command | What gets parsed |
|---|---|
| `ps` | column-aware (robust against right-aligned columns) |
| `du` | sorted by size descending, largest 15 kept, tail summarized |
| `lsof` | one row per process instead of one per descriptor, address kept |
| `pgrep` | identical command lines merged, `argv[0]` shortened to basename |
| `env` | sorted, secrets masked |
| `wc` | line/word/byte totals |
| `brew` | `list`, `outdated`, `services` |
| `curl` | headers + body compression; `curl -I` parses headers only |
| `wget` | progress output stripped |

## Built-in trs tools (not wrappers)

These are native trs commands: no external binary behind them. They
ship inside the single-binary install.

| Command | Purpose |
|---|---|
| `trs json` | jq-lite query engine (`-q '.users[].name'`) |
| `trs read` | file reader with `-l minimal` (strip comments) / `-l aggressive` (signatures only) |
| `trs search` | ripgrep-powered content search |
| `trs replace` | ripgrep-powered replace with `--dry-run` |
| `trs err` | error filter, only errors/warnings from a wrapped command |
| `trs tail` | log tail with `--errors` filter |
| `trs clean` | `--no-ansi`, `--collapse-blanks`, `--dedup-lines` |
| `trs html2md` | HTML → Markdown |
| `trs find` | gitignore-aware walker |
| `trs is-clean` | repo clean check (exit 0 if clean, 1 otherwise) |
| `trs raw` | passthrough that still tracks stats |
| `trs stats` | token-savings dashboard |
| `trs ingest` | whole-repo digest for agent context |

## Dispatch mechanisms

### Chain-aware rewrite

When a command chain uses `&&` or `;`, each rewritable segment gets
wrapped independently.

```bash
cd src && cargo test              → cd src && trs cargo test
cargo fmt && cargo clippy         → trs cargo fmt && trs cargo clippy
```

Pipes pass through untouched: trs never splits a pipeline because
the compressed form would lose the byte-stream contract.

### Pipe syntax (stdin)

Any parser is also reachable over stdin:

```bash
git status | trs parse git-status
cargo build 2>&1 | trs parse cargo-build
```

### Env-var prefix

Leading `VAR=value` assignments are preserved on the rewrite:

```bash
RUSTFLAGS='-C target-cpu=native' cargo build
→ RUSTFLAGS='-C target-cpu=native' trs cargo build
```

### Bypass: `TRS_SKIP=1`

To run a single command without any trs wrapping (for debugging or
when you specifically need raw output):

```bash
TRS_SKIP=1 git status             # runs raw git, no parser, no tracking
```

### Redirections

`2>&1` and other stdout/stderr redirections are stripped at the
rewrite layer and reapplied to the wrapped command, so trs sees the
same combined stream the shell would have produced.

## Generic compression (the fallback)

Any command not listed above still flows through trs's generic
reducer: ANSI strips, whitespace collapsed, repeated lines deduped.
Typical reduction 30–40% for free with no format-specific knowledge.

One class is exempt. Commands whose output is a re-layout of their
input carry meaning in exactly the runs of spaces and blank lines the
reducer collapses, so trs hands their output back untouched: `awk`,
`base64`, `basenc`, `column`, `comm`, `cut`, `expand`, `fold`,
`hexdump`, `iconv`, `join`, `jq`, `nl`, `od`, `paste`, `printf`,
`rev`, `sort`, `strings`, `tac`, `tr`, `unexpand`, `uniq`, `xxd`,
`yq`. See [`docs/support/safety.md`](./safety.md).

If a dedicated parser exists but errors out mid-parse, trs falls back
to **truncated passthrough** rather than silent failure, the full
raw output is also saved to `~/.trs/tee/` so you can recover it.

See also:
- [`docs/features/formats.md`](../features/formats.md): the six
  output formats every command supports.
- [`docs/features/stats.md`](../features/stats.md): how command
  routing is logged and visualized.
