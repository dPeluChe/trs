# `trs stats` — token savings dashboard

Every trs invocation logs an entry to `~/.trs/history.jsonl`:
timestamp, command, input bytes, output bytes, duration. `trs stats`
reads that log and produces a dashboard of cumulative savings.

## Quick reference

```bash
trs stats              # summary dashboard (top 15 commands)
trs stats --history    # per-command log (most recent 20)
trs stats -n 30        # override row cap (top 30 in summary, last 30 in --history)
trs stats --by-agent   # breakdown by which AI agent triggered the run
trs stats --json       # machine-readable summary
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
==================================================
  AGENT           CALLS    SHARE  AVG -%       SAVED
──────────────────────────────────────────────────
  claude            1247    58.2%     71%       720K
  cursor             403    18.8%     68%       190K
  opencode           210     9.8%     77%       145K
  gemini              89     4.2%     65%        48K
  kilo                12     0.6%     72%         8K
  (untagged)         182     8.5%     44%        31K
```

Labels come from the `TRS_AGENT` env var that `trs rewrite` and the
OpenCode / Kilo plugin templates inject into the rewritten command
before the shell runs it. The shell strips the env-var assignment,
so it's transparent to git/cargo/etc downstream, and trs's tracker
picks it up when the rewritten invocation eventually logs.

### Which agents get attributed

| Agent | Signal | Label |
|---|---|---|
| Claude Code | `hook_event_name: PreToolUse` | `claude` |
| Gemini CLI | `hook_event_name: BeforeTool` | `gemini` |
| Cursor | `hook_event_name: preToolUse` | `cursor` |
| OpenCode | plugin template | `opencode` |
| Kilo Code | plugin template | `kilo` |
| Factory Droid | same wire format as Claude | `claude` (limitation) |
| Codex / Antigravity / Windsurf | no programmatic signal (rules-only) | `(untagged)` |

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
  "avg_reduction_pct": 77.8
}
```

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
