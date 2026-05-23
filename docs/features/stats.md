# `trs stats` — token savings dashboard

Every trs invocation logs an entry to `~/.trs/history.jsonl`:
timestamp, command, input bytes, output bytes, duration. `trs stats`
reads that log and produces a dashboard of cumulative savings.

The active file rolls into a month-stamped archive
(`~/.trs/history.YYYY-MM.jsonl`) at the first append of each new month.
`trs stats` reads the active file plus every archive transparently, so
your cumulative numbers don't reset. Use `trs history --prune
--older-than 90` to retire archives older than your retention window.

`trs stats --history` lists commands **newest first** (top of output is
the most recent run). Same ordering in `--json` mode.

## Quick reference

```bash
trs stats              # summary dashboard (top 15 commands)
trs stats --history    # per-command log (most recent 20)
trs stats -n 30        # override row cap (top 30 in summary, last 30 in --history)
trs stats --by-agent   # breakdown by which AI agent triggered the run
trs stats --by-command # breakdown by normalized command family (e.g. "git diff", "npm run lint")
trs stats --coverage   # parser-gap analysis (what compresses well, what falls through)
trs stats --json       # machine-readable summary (combines with any of the above)
```

## Summary (default)

```
trs savings — Apr 15 23:04 → Apr 20 17:12 (5 days)
────────────────────────────────────────────────────
  input:       4.2 MB    output:  930 KB     saved: 3.3 MB
  tokens in:   1.0M      out:     232K        saved: 800K (77%)
  commands:    1,247     today:   23          avg:   ~165k tokens/day
  last:        git status (2m ago)

Top Commands
───────────────────────────────
  cargo test              91x  -99%  95k saved
  git status              203x -76%  42k saved
  cargo clippy            34x  -89%  38k saved
  …

For full history: trs stats --history
```

Header explains the measurement period explicitly so you know whether
you're looking at a week or a month of usage. Today's count and the
period average are shown side-by-side for a quick "am I above or
below average" read.

## History view

```bash
trs stats --history         # last 20 (default)
trs stats --history -n 50   # last 50
```

Shows the most recent commands (default 20, override with `-n`):

```
Recent Commands (today: 23 commands)
────────────────────────────────────────────────────────────────
  Apr 20 16:40  trs git status              124 ->   104  -16%   34ms
  Apr 20 16:40  trs cargo build            55K  ->  58   -99%  4484ms
  Apr 20 16:45  cargo build                169  ->  16   -90%  4484ms
  …
```

If a logged command's first token is an absolute path (e.g. a hook
invoking `/Users/you/.local/bin/trs …`), the display collapses it to
the basename so you still see what was run. The full path is preserved
in `~/.trs/history.jsonl` if you need it.

## `--by-agent` — attribution breakdown

```bash
trs stats --by-agent
```

Shows which AI agent triggered each execution. Added in v0.5.8.

```
trs Token Savings — by agent
============================================================
  AGENT           CALLS    SHARE  AVG -%       SAVED      BYPASS
────────────────────────────────────────────────────────────
  claude            1247    58.2%     71%       720K   3 (0.2%)
  cursor             403    18.8%     68%       190K           0
  opencode           210     9.8%     77%       145K           0
  gemini              89     4.2%     65%        48K   1 (1.1%)
  kilo                12     0.6%     72%         8K           0
  (untagged)         182     8.5%     44%        31K           0
```

Labels come from the `TRS_AGENT` env var that `trs rewrite` and the
OpenCode / Kilo plugin templates inject into the rewritten command
before the shell runs it. The shell strips the env-var assignment,
so it's transparent to git/cargo/etc downstream, and trs's tracker
picks it up when the rewritten invocation eventually logs.

### The BYPASS column

`BYPASS` counts how many commands the agent prefixed with
`TRS_SKIP=1` — those skip trs entirely, so we never see the output
and can't compress it. The column shows the count plus the rate as
a fraction of the agent's total calls (`3 (0.2%)`); `0` is rendered
as a plain zero so the eye skips over the common case.

We log bypass observations even though we don't see the output, so
the dashboard can answer: "is this agent reaching for the escape
hatch on routine commands?" High rates (>5%) usually mean the
agent's prompt promotes bypass too aggressively — refresh
`~/.<agent>/trs.md` via `trs output-saver --refresh` to ship the
current minimal template.

Bypass entries carry zero in/out byte counts, so they don't affect
SAVED / AVG -% — they only contribute to CALLS and BYPASS.

### Which agents get attributed

| Agent | Signal | Label |
|---|---|---|
| Claude Code | `hook_event_name: PreToolUse` | `claude` |
| Gemini CLI | `hook_event_name: BeforeTool` | `gemini` |
| Cursor | `hook_event_name: preToolUse` | `cursor` |
| OpenCode | plugin template | `opencode` |
| Kilo Code | plugin template | `kilo` |
| Factory Droid | same wire format as Claude | `claude` (limitation) |
| Antigravity IDE / CLI (`agy`) | `ANTIGRAVITY_CONVERSATION_ID` env on Claude-shaped event | `antigravity` |
| Codex / Windsurf | no programmatic signal (rules-only) | `(untagged)` |

Direct-shell invocations (`trs git status` typed manually) also land
under `(untagged)`. We don't invent a label where we don't have
honest data.

### Droid attribution caveat

Factory Droid uses Claude's hook wire format verbatim (same
`hook_event_name: PreToolUse` envelope), so our dispatcher can't
distinguish the two at rewrite time. Both show up as `claude`. If
you need separation, you currently need to eyeball `cwd` paths or
look at the hour of the day. A future release could disambiguate
via a install-time flag or a second detection path.

## `--by-command` — command family breakdown

```bash
trs stats --by-command
```

Groups history entries by normalized command family (binary + up to 2
meaningful subcommands) and ranks by total tokens saved. Useful for
spotting which commands run most and which give the best reduction.

```
trs Token Savings — by command
==================================================
  COMMAND                CALLS   SHARE  AVG -%       SAVED
──────────────────────────────────────────────────
  find                     555   42.1%     48%        3.3M
  git diff                 102    7.7%     91%        1.2M
  npm run lint              48    3.6%     82%         420K
  gh pr checks              31    2.4%     78%         310K
  …
```

Normalization strips paths, flags, and IDs — `git diff HEAD~1` and
`git diff main..feature` both count as `git diff`. `npm run lint`
and `pnpm run lint` are separate entries (binary is kept).

## `--coverage` — parser-gap analysis

```bash
trs stats --coverage           # human-readable, three tiers
trs stats --coverage --json    # machine-readable (use this to share)
trs stats --coverage -n 20     # row cap per tier (default 15)
```

Aggregates every entry by `(binary, subcommand)` and surfaces three
tiers:

1. **Gaps** — high-volume subcommands with poor compression. These are
   the highest-leverage parser additions. Sample row:
   `poetry run  count=769  avg_in=11186  %low=51%`.
2. **Unrecognized binaries** — commands trs has no dedicated parser
   for. Falls through to generic ANSI / whitespace compression
   (~30-40%). Adding the binary to `REWRITE_PREFIXES` ensures even
   that minimum kicks in.
3. **Well-covered** — top by volume, low `%low` rate. Confirmation
   the existing parsers are doing their job.

A row qualifies as "low" when its `saved_pct < 10`. A row enters tier 1
or 2 only if at least 40% of its entries are low AND the average input
is ≥ 256 bytes (small outputs intrinsically can't be compressed much).

### Reporting a parser gap

Run `--coverage --json` and paste the output into an issue:

```bash
trs stats --coverage --json > coverage.json
gh issue create --repo dPeluChe/trs --title "parser-gap: poetry run" --body-file coverage.json
```

The JSON is self-contained (`trs_version`, entry range, tier rows with
binary/sub/count/in_bytes/avg_in/low_pct/sample). No cwd paths or
secrets leak — `sample` is truncated to 70 chars and may include flag
patterns, so glance over it before sharing if your project layouts
contain sensitive names.

## JSON mode

```bash
trs stats --json
```

Structured output suitable for dashboards or CI:

```json
{
  "total_commands": 1247,
  "period_start": "2026-04-15",
  "period_end": "2026-04-20",
  "period_days": 5,
  "tokens_per_day": 160000,
  "input_bytes": 4200000,
  "output_bytes": 930000,
  "saved_bytes": 3270000,
  "input_tokens": 1050000,
  "output_tokens": 232500,
  "saved_tokens": 817500,
  "avg_reduction_pct": 77.8,
  "bypass_count": 4
}
```

`bypass_count` is the number of `TRS_SKIP=1` observations across the
window — same signal as the BYPASS column in `--by-agent`, but
aggregated. Useful for dashboards that want a single bypass-rate
metric (`bypass_count / total_commands`).

## What gets tracked

Every invocation of trs that produces output (compressed or not)
writes a line to history.jsonl. Commands that failed, commands routed
through the generic fallback, commands that emitted zero savings —
all logged, so the dashboard reflects real usage rather than only the
"wins."

Failed passthrough (when a parser errors out and trs falls back to
raw output) is logged with identical input/output bytes so the
reduction shows as 0%. That's intentional — lying about savings would
defeat the point of the dashboard.

## Clearing / trimming history

```bash
rm ~/.trs/history.jsonl        # start over
```

No built-in retention policy yet — history grows indefinitely. A log
rotation helper is on the roadmap. Typical sizes: a few MB after a
few months of heavy use.

## See also

- [`trs doctor`](doctor.md) — reports history.jsonl size and
  writability.
- [`trs discover`](../../README.md) — scans your prior shell history
  for commands where trs would have saved tokens but wasn't used.
