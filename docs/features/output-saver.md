# `trs output-saver`: reduce tokens on the agent's replies

`trs rewrite` (wired up by [`trs init`](init.md)) compresses what
agents **see**, the output of the shell commands they run.

`trs output-saver` installs a short rules block into each supported
agent's global config: answer concisely, never invent paths or API
fields, keep code comments short. It is deliberately small: in
pre-registered tests no longer block beat the line "Answer concisely."
([findings](../development/output-cost-findings.md)).

## Quick reference

```bash
trs output-saver                 # read-only scan of all agents
trs output-saver --install       # write to every detected agent
trs output-saver <agent> --install  # scope to one
trs output-saver --verify        # per-agent: block matches current canonical?
trs output-saver --refresh       # re-write the block where it's already present
trs output-saver --remove        # clean uninstall
trs output-saver --print         # dump the block to stdout (pipe-friendly)
```

`--verify` is the post-upgrade check: it confirms each agent's installed
block **byte-matches the current canonical text**, so you know every agent
picked up new rules after `trs upgrade`. It prints per-agent
`loaded` / `drifted` / `not installed` and **exits non-zero if any agent
drifted** (stale block, run `--refresh`), so it can gate a script or CI
step. `trs doctor` surfaces the same drift signal inline.

## What the block says

Two headings, three rules. Size and cost: see [What the block costs](#what-the-block-costs).

- **`Answer concisely.`** One line, because longer reply rules never beat it
  (see [Measuring impact](#measuring-impact)).
- **Never invent file paths, function names, or API fields.** If unknown, say
  `UNKNOWN` or return `null`.
- **`## Code authoring`: code comments.** None by default. If one is truly
  needed, a terse WHY-only note (not a walkthrough), at most 3 lines, ~200
  characters; never paragraph docstrings or restated code. A longer logic
  explanation goes to `docs/` with a one-line pointer in the comment. A minimal
  BAD/GOOD pair anchors the calibration.

These code-comment limits (WHY-only, ~3 lines, ~200 chars) are the
**canonical** source, downstream configs that replicate them should
mirror this text rather than fork it, and this block changing is the
signal to re-sync.

Plus an explicit user-override clause so the rules never fight a
user's deliberate instructions.

Run `trs output-saver --print` to see the exact text before installing.

## Why it is this short

Earlier versions had numeric word budgets, shape rules, style rules, turn-saving
rules and answer checks, drawn from published agent prompts and prompt-engineering
studies. Measured against a one-line control, none of it paid its input cost, so
the block keeps only what the one line cannot say.

## Coverage matrix

| Agent | Mechanism | Path |
|---|---|---|
| Claude Code | Standalone file + `@import` | `~/.claude/trs.md` + line in `~/.claude/CLAUDE.md` |
| Gemini CLI | Standalone file + `@import` | `~/.gemini/trs.md` + line in `~/.gemini/GEMINI.md` |
| Cursor | Local plugin with an always-apply rule | `~/.cursor/plugins/local/trs/` (`.cursor-plugin/plugin.json` + `rules/trs-output-saver.mdc`) |
| Codex | Inline with sentinels | `~/.codex/AGENTS.md` |
| Devin Desktop (ex-Windsurf) | Inline with sentinels | `~/.codeium/windsurf/memories/global_rules.md` |
| OpenCode | Inline with sentinels | `~/.config/opencode/AGENTS.md` |
| Kilo Code | Inline with sentinels | `~/.config/kilo/AGENTS.md` |
| Factory Droid | Inline with sentinels | `~/.factory/AGENTS.md` |
| Antigravity IDE | Standalone file + `@import` (shared with Gemini) | `~/.gemini/trs.md` + line in `~/.gemini/GEMINI.md` |
| Antigravity CLI (`agy`) | Standalone file + `@import` (shared with Gemini) | `~/.gemini/trs.md` + line in `~/.gemini/GEMINI.md` |
| VS Code Copilot / Copilot CLI | Own instructions file with `applyTo: "**"`, read by both | `~/.copilot/instructions/trs.instructions.md` (`$COPILOT_HOME/instructions/` when set) |

Codex, OpenCode, Kilo, and Droid are signatories of the
[`AGENTS.md` convention](https://factory.ai/news/agents-md), which is
why they share the same install mechanism.

**Grok Build** loading is confirmed on grok 1.0.70. **Kiro and OpenClaw** are
unverified on real binaries: paths in the
[output-saver matrix](../development/agent-integrations.md#output-saver-matrix).

**Antigravity IDE + CLI (`agy`)** share Gemini's `GEMINI.md`/`trs.md`
for the output-saver only. Their *hooks* are jetski-specific and live
in `~/.gemini/antigravity-{ide,cli}/hooks.json`, see
[supported agents](../support/agents.md#antigravity-ide--antigravity-cli-agy).

## How the install is idempotent

Inline installs wrap the block in HTML comment sentinels:

```
<!-- trs:output-saver:start v1 -->
## Output saver: keep replies cheap
…
<!-- trs:output-saver:end -->
```

Running `--install` again detects the sentinels and replaces the
content between them, no duplication, no accidental user-content
loss. The sentinel carries a version tag (`v1`) so we can migrate
block content in future releases without breaking detection.

The `@import` mechanism for Claude/Gemini is naturally idempotent:
re-install overwrites `trs.md` and re-adds the import line only if
missing. If the legacy `trs-output-saver.md` file exists it is deleted
and its import line replaced with `@trs.md` automatically.

## Check-first semantics

A bare `trs output-saver` never writes. It reports what install would
do for each agent and prints the exact commands to commit or remove
the block. This mirrors `trs audit-docs` and `trs doctor`, nothing
destructive happens without an explicit flag.

Sample output:

```
trs output-saver: scan

  + Claude Code     already installed
  . Gemini CLI      not yet installed
  - Cursor          not detected on this system
  + Codex           already installed
  + Antigravity IDE already installed (shared with Gemini)
  + Antigravity CLI already installed (shared with Gemini)

  1 installable, 4 already installed, 1 not detected, 0 unsupported
```

## `--refresh`: pick up template changes without adding new installs

```bash
trs output-saver --refresh
```

Re-installs the block **only** where `trs output-saver` already
reports `AlreadyInstalled`. Agents that never had the block are
skipped silently. This is the mode `trs upgrade` calls automatically
to refresh templates without surprising you by adding rules to agents
you deliberately didn't opt into.

## `--remove` behavior

For `@import` and `RulesDir` installs, `--remove` deletes the
standalone file and strips the import line from the parent config.
For `InlineFile` installs, the content between the sentinels is
removed along with the sentinels themselves, surrounding user
content is preserved exactly.

## What the block costs

It is input on every session, whether or not it helps that turn.
Measured on this version (`trs output-saver --print`, and the file `trs init
--global` writes for Claude Code):

| What | Bytes | Tokens (bytes / 4, an estimate) |
| --- | --- | --- |
| The rules block (`--print`, inline installs) | 698 | ~175 |
| `~/.claude/trs.md` (block plus the "Shell output" introduction) | 1,726 | ~430 |

Earlier versions of this page said ~300 tokens; that was wrong for both. A
unit test (`the_block_stays_inside_the_size_the_doc_quotes`) now fails when
the block grows past these sizes without this table being updated.

With prompt caching the block sits in the cached prefix, so after the first
request it is billed at a fraction of normal input. At typical API ratios
(output about 5x input, cached read about 0.1x input; check your provider's
price list) ~430 cached tokens cost about as much per turn as ~9 output
tokens, so the block pays for itself only if it removes more than that from
each reply. On a flat-rate plan the price is the context window, not money.

## Measuring impact

The block doesn't compress existing output; it steers what gets generated.

**A/B with a control** (`docs/development/benchmarks/output_saver_ab.py`,
2026-11-04, Claude Code 2.1.318 with `--model sonnet`, no tools, isolated
sessions, 8 prompts x 5 runs = 40 replies per arm; three of the prompts are
fidelity cases: a negation, a number, a destructive-action warning):

| Arm | Output tokens / reply | Words / reply | Fidelity cases passed |
| --- | --- | --- | --- |
| no block | 425 | 177 | 100% |
| "Answer concisely." (one line) | 208 | 86 | 100% |
| the trs block at that date | 188 | 70 | 95% (1 miss in 40; the same case reran 5 of 5) |

- The block against no instruction: **-237 output tokens per reply** (95% CI
  -331 to -137).
- The block against the one-line "Answer concisely.": **-20 per reply, CI -79
  to +42**. Not distinguishable. Most of the saving comes from asking for
  brevity at all, not from the block's other rules.
- Preamble, closing filler and em dashes were already near zero in every arm on
  this model, so the block's rules against them had nothing to remove here.

Read it with these limits: one model, eight short prompts with no tools or
long sessions, N=5 per prompt, and `output_tokens` includes thinking, not only
the prose the block steers. It says nothing about the code-authoring rules
(comment style, no invented paths), which this test does not exercise. Per-call
billing sees no benefit either: output is a small share of most bills, and the
block it tested added about 590 to 850 input tokens to every session.
Rerun it on your model before relying on the block for cost.

The `trs stats` dashboard tracks input-side savings from `trs rewrite` but
cannot measure output-side savings, they happen outside any process we run.
To estimate them from your own Claude Code transcripts:

```bash
python3 docs/development/benchmarks/transcript_output.py --since YYYY-MM-DD
```

It compares `output_tokens` per assistant message before and after the date
(default: when `~/.claude/trs.md` was created), per model, with a bootstrap 95%
confidence interval on the difference. It is observational: the task mix
changed between the periods, and `output_tokens` includes thinking and tool
arguments, so the figure is an estimate and a hint, not proof. Transcripts only
go back as far as Claude Code kept them; if none predate the install there is
nothing to compare, and the script says so.

**Agentic A/B** (`docs/development/benchmarks/output_saver_agentic.py`,
2026-11-04, `--model sonnet --effort low`, tools on, 4 small repo tasks x 6
runs = 24 per arm, a 30-module filler repo, 120 runs, about 6.6 USD). Mean
per run, success checked by each task's own test:

| Arm | Cost | Turns | Output tokens | Success |
| --- | --- | --- | --- | --- |
| none | $0.053 | 4.4 | 527 | 96% |
| "Answer concisely." | $0.053 | 4.5 | 491 | 100% |
| previous block | $0.057 | 4.4 | 489 | 92% |
| longer block (style rules plus a "Fewer turns" section; not shipped) | $0.056 | 4.6 | 488 | 100% |
| telegraphic-style rules | $0.056 | 4.8 | 535 | 96% |

No arm differs from another in cost or turns: every 95% CI (bootstrap on the
difference of means) spans zero. The block arms sit about $0.004 (7%) above
"none", with a CI that just touches zero, which is what their input cost
would predict. The "Fewer turns" rules did not reduce turns here (+0.25,
CI -0.67..+1.13 against none). A first sweep was discarded: its Bash
allowlist lacked `sed`, 50 of 120 runs hit a permission denial and every
failure was one of them.

What this does not show: tasks of 2 to 7 turns leave little to batch, one
model at low effort, no long sessions.

**Did the longer block earn its place?** No. Criteria were committed before
the runs (2026-11-04): against "Answer concisely.", a 2,334-byte block (word
budgets, style rules, "read once / verify against code / prefer the newest
source") had to win on concision or on correct answers, with a 95% CI excluding
zero, or the block would shrink to the one-line control plus the comment rule
and the "never invent" rule. That is the block shipped now.

| Test | "Answer concisely." | Longer block | Difference (95% CI) |
| --- | --- | --- | --- |
| Concision: output tokens per reply (`output_saver_ab.py --n 5`, 40 per arm) | 213 | 192 | -22 (-80 to +39) |
| Quality: correct answers (`output_saver_agentic.py --suite stale --n 5`, 40 per arm) | 95% | 90% | -5 points (-17.5 to +5) |
| Same runs: cost per run | $0.045 | $0.049 | +$0.005 (+$0.002 to +$0.007) |

No instruction at all scored 95% on the quality test too. Both pilots had the
control near the ceiling, the fixture was hardened once before the sweep, and
almost every miss was one trap task (a default port changed by an override), so
this test could not show a small gain. Runs of one task are not independent, and
the shipped short block was not itself rerun. On this model the agent already
read the code without being told to.

## Interaction with `trs init`

For Claude Code and Gemini CLI, `trs init --global` now writes `trs.md`
automatically alongside the hook config, you get both input compression
and output-saver rules in one command. For all other agents the two
installs remain independent: you can have hooks without output-saver
rules, and vice versa.

## See also

- [What actually moves agent cost](../development/output-cost-findings.md): the findings behind this block and how to run the next test.
- [`trs init`](init.md): the input-side compression via hooks.
- [`trs audit-docs`](audit-docs.md): audit CLAUDE.md / AGENTS.md
  files for bloat; pairs well with output-saver.
- [`trs doctor`](doctor.md): reports output-saver coverage alongside
  hook coverage.
