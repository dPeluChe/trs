# Supported commands

Every command supported by trs falls into one of four levels.

1. **Dedicated parser.** trs spawns the tool, parses its native output,
   and emits a structured compact form. Typical reduction **68–99%**.
2. **Dispatched alias.** A different binary with the same semantics
   (e.g. `rg` for `grep`, `eza` for `ls`) gets routed to the same
   parser. No configuration — the dispatcher recognizes the binary
   name.
3. **Generic compression.** Commands without a parser still get ANSI
   stripping, whitespace collapse, and repeated-line deduplication.
   Typical reduction **30–40%** "free."
4. **Passthrough.** Commands where trs detects a flag that already
   produces structured output (`--json`, `--porcelain`) are passed
   through untouched — the agent gets the raw structured form.

## Commands with dedicated parsers

### VCS — git

| Command | Subcommands parsed |
|---|---|
| `git` | `status`, `diff`, `log`, `branch`, `push`, `pull`, `fetch`, `show`, `stash show -p`, `stash pop`, `stash apply` |

Notes: `--no-verify` is blocked on `git commit` / `git push` to
protect pre-commit hooks from AI agents that default to bypassing
them. `git status --porcelain` passes through untouched.
`git show` and `git stash show -p` are routed to the diff parser
(~90% reduction on commits with modifications).

### Build — Rust

| Command | Subcommands parsed |
|---|---|
| `cargo` | `build`, `check`, `clippy`, `test`, `fmt`, `install`, `add` |

Notes: env-var prefix preserved — `RUSTFLAGS=xyz cargo build` is
rewritten to `RUSTFLAGS=xyz trs cargo build` so the flag still reaches
cargo.

### Build — JavaScript / TypeScript

| Command | Subcommands parsed |
|---|---|
| `npm` | `install` (+`i`, `ci`), `test`, `ls` / `list`, `audit`, `outdated`, `run` |
| `pnpm` | `install` (+`i`), `test`, `ls`, `audit`, `outdated`, `why`, `add`, `update`, `up`, `dlx`, `exec` |
| `yarn` | `install`, `test` |
| `bun` | `install`, `test`, `run` |
| `npx` / `pnpm dlx` | routed to whichever inner command is invoked |

### Build — Go

| Command | Subcommands parsed |
|---|---|
| `go` | `test`, `build`, `mod` |

### Build — Python

| Command | Subcommands parsed |
|---|---|
| `pip` / `pip3` | `install`, `list`, `freeze`, `show` |
| `uv` | `pip`, `sync`, `add`, `remove`, `run`, `tree` |
| `python3 -m <module>` | routed: `pytest` → test, `mypy` / `ruff` / `pylint` / `flake8` → lint, `unittest` → test |

### Tests

| Command | What gets parsed |
|---|---|
| `pytest` | full runner output — pass/fail counts, failure locations, tracebacks |
| `jest` | full runner output — suite summary, failed assertions |
| `vitest` | full runner output |
| `cargo test` | see "Build — Rust" |
| `go test` | see "Build — Go" |
| `npm test` / `pnpm test` / `bun test` / `yarn test` | dispatched to the inner runner |

### Linters

| Command | What gets parsed |
|---|---|
| `cargo clippy` | grouped by file + rule |
| `eslint` | issues grouped |
| `biome` | issues grouped |
| `ruff` | issues grouped |
| `pylint` | issues grouped |
| `golangci-lint` | issues grouped |

### Files & search

| Command | Aliases routed | Notes |
|---|---|---|
| `ls` | `lsd`, `exa`, `eza` | long format parsed; `--json` passthrough |
| `find` | `fd` | result list compacted |
| `grep` | `rg`, `ag`, `ack` | line/match format |
| `tree` | — | directory tree compressed |
| `tail` | `journalctl` | log-tail with error filter |

### File reading intercepts

These commands are intercepted before subprocess spawn: trs reads the
file directly and applies `filter_minimal` (strip comment-only lines,
normalize blank lines). Falls back to raw content when filtering would
return empty output (e.g. an all-comment slice).

| Command | Pattern | Typical reduction |
|---|---|---|
| `cat` | `cat FILE [FILE2…]` — no flags | 10–35% on code files |
| `head` | `head [-n N \| -N] FILE` | 5–20% |
| `sed` | `sed -n X,Yp FILE` (line-range only) | 10–25% vs 0% passthrough |

Any other `sed` form (substitutions, in-place `-i`, multiple files)
falls through to the subprocess path unchanged.

### Containers & GitHub CLI

| Command | Subcommands parsed |
|---|---|
| `docker` | `ps`, `logs`, `build` |
| `gh` | `pr list`, `pr view`, `issue list`, `run list`, plus `gh api <endpoint>` passthrough tracked in stats |

`gh pr view` extracts title, state, author, url, labels, and a
3-line body preview — reducing typical PR view output by ~45%.

### System & network

| Command | What gets parsed |
|---|---|
| `ps` | column-aware (robust against right-aligned columns) |
| `env` | sorted, secrets masked |
| `wc` | line/word/byte totals |
| `brew` | `list`, `outdated`, `services` |
| `curl` | headers + body compression; `curl -I` parses headers only |
| `wget` | progress output stripped |

## Built-in trs tools (not wrappers)

These are native trs commands — no external binary behind them. They
ship inside the single-binary install.

| Command | Purpose |
|---|---|
| `trs json` | jq-lite query engine (`-q '.users[].name'`) |
| `trs read` | file reader with `-l minimal` (strip comments) / `-l aggressive` (signatures only) |
| `trs search` | ripgrep-powered content search |
| `trs replace` | ripgrep-powered replace with `--dry-run` |
| `trs err` | error filter — only errors/warnings from a wrapped command |
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

Pipes pass through untouched — trs never splits a pipeline because
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

### Bypass — `TRS_SKIP=1`

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

If a dedicated parser exists but errors out mid-parse, trs falls back
to **truncated passthrough** rather than silent failure — the full
raw output is also saved to `~/.trs/tee/` so you can recover it.

See also:
- [`docs/features/formats.md`](../features/formats.md) — the six
  output formats every command supports.
- [`docs/features/stats.md`](../features/stats.md) — how command
  routing is logged and visualized.
