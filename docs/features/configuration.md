# Configuration

trs works without any configuration — every default is tuned for
sensible output on typical workloads. Create a `config.toml` only
when you need to tighten or loosen specific limits.

## Files

Two lookup paths, merged (project overrides home):

1. **`~/.trs/config.toml`** — per-user defaults, applied globally.
2. **`.trs/config.toml`** — project override, committed to the repo.

Later keys win. Unrecognized keys are ignored with a warning on
`trs doctor`.

## Tunable limits

```toml
[limits]
grep_max_results = 200           # max rows surfaced by trs grep / search
status_max_files = 15            # cap on files shown in trs git status
passthrough_max_chars = 2000     # truncation ceiling on raw-fallback output
json_max_depth = 10              # max nesting for trs json tree walks
```

### `grep_max_results`

`trs search` / `trs grep` cap the number of matching lines at this
value. Default 200 — usually enough to eyeball the hit density
without flooding the agent's context. Bump when searching for rare
keywords in large monorepos; lower when the agent is getting
overwhelmed.

### `status_max_files`

`trs git status` caps file listings per section (staged, unstaged,
untracked) at this number. Default 15 — keeps the output bounded
even when you're in the middle of a big refactor. Lower values force
agents to work in smaller committed chunks.

### `passthrough_max_chars`

When a parser errors out mid-parse, trs falls back to truncated
passthrough rather than silent failure. This is the truncation
ceiling. Default 2000 chars. The full raw output is always saved to
`~/.trs/tee/` regardless — this limit only affects what flows into
the agent's context.

### `json_max_depth`

`trs json` recursion cap for tree walks, schema extraction, and
pretty-printing. Default 10. Deeper nesting is truncated with an
ellipsis marker so the agent sees "shape" without the full depth.

## Hook-time wrappers

Some workflows put a wrapper in front of every command — `direnv exec
.`, `docker exec myapp`, `shadowenv exec --`. Without help, the
rewrite layer can't see past the wrapper and the inner command stays
uncompressed. Register the wrapper as a transparent prefix and trs
will strip → route → re-prepend:

```toml
[hooks]
transparent_prefixes = [
  "docker exec myapp",
  "direnv exec .",
  "shadowenv exec --",
]
```

With that, `docker exec myapp git status` rewrites to `docker exec
myapp trs git status` — the wrapper still runs, but the inner
command flows through trs.

Matching is literal — no patterns. Built-in wrappers (no config
needed):

- **Shell builtins** — `noglob`, `command`, `builtin`, `exec`,
  `nocorrect`.
- **Process wrappers** — `time`, `nohup`, `setsid`, `unbuffer`,
  `stdbuf`.
- **Venv runners** — `poetry run`, `uv run`, `pdm run`. The inner
  command is passed through uninterpreted, so `poetry run pytest -v`
  routes the pytest parser correctly.

Notably NOT recognized (and why): `bun run <script>`, `pnpm run
<script>`, `npm run <script>`, `yarn <script>`, and `npx <pkg>` —
the token after the wrapper is a script name or package name, not a
command. Stripping would break (e.g. `bun run trs typecheck` would
look for a script literally named "trs"). These are routed at the
classifier layer instead.

## Bypassing trs for a single command

Two equivalent env-var prefixes disable trs for one invocation:

```bash
TRS_SKIP=1 git log --pretty=format:'%H %s'
TRS_DISABLE=1 npx tsc --noEmit
env TRS_DISABLE=1 cargo test     # env-wrapped form also works
```

Both are recognized at the hook layer (the bypass is logged for
attribution — see `stats --by-agent` BYPASS column) and at the plain-
text rewrite path. The shell strips the env-var assignment before
executing the downstream program, so the bypass is transparent to
git / cargo / etc.

## Inspecting the active config

```bash
trs doctor               # surface any unrecognized keys + shows loaded paths
trs stats                # the dashboard reflects limits on the output it logs
```

## When to skip the config file

If you're using default limits, **don't create one**. An empty or
defaults-only config adds noise without value. `trs doctor` flags
configs that contain only default values.

## See also

- [`docs/features/doctor.md`](./doctor.md) — health check; surfaces
  unrecognized config keys.
- [`docs/support/commands.md`](../support/commands.md) — which
  commands use which limits.
