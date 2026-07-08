# `trs output-saver` — reduce tokens on the agent's replies

`trs rewrite` (wired up by [`trs init`](init.md)) compresses what
agents **see** — the output of the shell commands they run. Agents
still **emit** verbose replies: preambles ("Sure!"), narration
("Now I will…"), speculative suggestions, hallucinated file paths.

`trs output-saver` installs a short rules block into each supported
agent's global config so those replies come back tighter.

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
drifted** (stale block — run `--refresh`), so it can gate a script or CI
step. `trs doctor` surfaces the same drift signal inline.

## What the block says

Eight directives, roughly 250 tokens total. The exact text:

- **Numeric token budget.** "Keep replies under ~100 words unless the
  task needs more. Between tool calls, stay under ~25 words."
- **Task-shape calibration.** "Match shape to task — a one-line
  question gets a one-line answer, no headers."
- **Open and end positively.** "Open with the answer or the diff.
  End when the answer ends."
- **Result first; explanation only if non-obvious.** State the
  finding, show the fix, stop.
- **Let tool output speak for itself.** Don't restate or recap what
  the diff already shows.
- **Structured output when the data is structured.** Bullets, tables,
  JSON — prose only when the reader is human and the content is
  narrative.
- **Persistence.** The rules hold for every reply, not just the first —
  agents drift back to preambles over a long session unless reminded.
- **No invented abbreviations or causal arrows.** `cfg/impl/req/res` and
  `→` split into the same tokens as the full word, so they save nothing
  and cost clarity.
- **Never invent file paths, function names, or API fields.** If
  unknown, return `UNKNOWN` or `null` — guessing costs more tokens
  than asking.
- **Full clarity, never compressed, on safety.** Security warnings,
  irreversible/destructive confirmations, and any multi-step order a
  misread would break stay verbatim-clear.
- **Reuse before re-implementing.** A helper, type, or pattern already
  a few files over beats writing a new one.
- **One pass.** Don't iterate on passing code, don't refactor or
  polish unless asked.
- **Code comments: none by default.** One short line max if the WHY
  is non-obvious. Never multi-paragraph docstrings.

Plus an explicit user-override clause so the rules never fight a
user's deliberate instructions.

Run `trs output-saver --print` to see the exact text before
installing.

## Why these rules — research backing

The current rules block is the result of a 2026-Q2 research pass into
public prompt-engineering patterns for response-length reduction. Two
classes of sources informed each rule:

1. **Anthropic's own Claude Code system prompt** (publicly leaked and
   archived at [`asgeirtj/system_prompts_leaks`][leak-1] and
   [`Piebald-AI/claude-code-system-prompts`][leak-2]). Anthropic
   A/B-tested numeric token budgets against the qualitative "be
   concise" baseline and reported ~1.2% output token reduction in
   production. This is the strongest empirical signal available for a
   terminal-tooling agent like Claude Code — the closest match to
   trs's deployment shape.

2. **Positive vs negative instruction studies** ([eval.16x.engineer
   pink-elephant analysis][pink-1], [gadlet.com on negative
   prompting][pink-2]). InstructGPT-class models reliably comply less
   with "Don't do X" than with "Do Y" — the negation primes the
   forbidden behavior. Our previous block leaned heavily on negatives
   ("No preambles", "No narration", "Don't iterate"); the rewrite
   flips them where the positive alternative is unambiguous.

### Rule-by-rule provenance

| Rule | Source | Status |
|---|---|---|
| Numeric token budget | Claude Code system prompt | Empirically validated by Anthropic A/B |
| Task-shape calibration | Claude Code system prompt | Opinion-but-Anthropic-validated |
| "Open with the answer or the diff" | Pink-elephant studies (positive form) | Replaces older negative "No preambles / No narration" |
| Result first | Carryover from v0.5 | Internal opinion, no published study |
| Tool output speaks for itself | Pink-elephant rewrite | Positive form of old "No narration" |
| Structured when data is structured | Carryover from v0.5 | Internal opinion |
| Never invent | Carryover from v0.5 | Common LLM hallucination guard |
| Persistence | [caveman][cav] SKILL ("ACTIVE EVERY RESPONSE, no filler drift") | Counters verbosity regression over long sessions |
| No invented abbreviations / arrows | [caveman][cav] SKILL | Non-obvious tokenizer insight — `cfg/impl` split like the full word |
| Full clarity on safety | [caveman][cav] auto-clarity + [ponytail][pony] "when NOT to be lazy" | Guards against dangerous over-compression |
| Reuse before re-implementing | [ponytail][pony] ladder rung 2 ("the most common slop") | Code-quality lift |
| One pass | Carryover from v0.5 | Internal opinion |
| Code comments: none by default | Claude Code system prompt | Direct lift; addresses a known bloat source agents emit |

[cav]: https://github.com/juliusbrussee/caveman
[pony]: https://github.com/DietrichGebert/ponytail

### What was deliberately NOT added

- **`<answer>` XML delimiters** — Anthropic-documented for *parsing*
  structured outputs, not brevity. Wrong tool for this surface.
- **CoT (chain-of-thought) suppression in the prompt** — extended
  thinking is controlled by API parameters (`max_thinking_tokens`),
  not by reply-text instructions. Out of scope here.
- **GPT-5 `<verbosity>low</verbosity>` tag** — empirically validated
  by OpenAI for Codex / GPT-5+ agents, but requires per-agent
  conditional content (Claude doesn't honor it). Worth a follow-up
  with agent-specific templates; not in the current single-template
  shape.

[leak-1]: https://github.com/asgeirtj/system_prompts_leaks/blob/main/Anthropic/claude-code.md
[leak-2]: https://github.com/Piebald-AI/claude-code-system-prompts
[pink-1]: https://eval.16x.engineer/blog/the-pink-elephant-negative-instructions-llms-effectiveness-analysis
[pink-2]: https://gadlet.com/posts/negative-prompting/

## Coverage matrix

| Agent | Mechanism | Path |
|---|---|---|
| Claude Code | Standalone file + `@import` | `~/.claude/trs.md` + line in `~/.claude/CLAUDE.md` |
| Gemini CLI | Standalone file + `@import` | `~/.gemini/trs.md` + line in `~/.gemini/GEMINI.md` |
| Cursor | Auto-loaded rules file | `~/.cursor/rules/trs-output-saver.mdc` |
| Codex | Inline with sentinels | `~/.codex/AGENTS.md` |
| Devin Desktop (ex-Windsurf) | Inline with sentinels | `~/.codeium/windsurf/memories/global_rules.md` |
| OpenCode | Inline with sentinels | `~/.config/opencode/AGENTS.md` |
| Kilo Code | Inline with sentinels | `~/.config/kilo/AGENTS.md` |
| Factory Droid | Inline with sentinels | `~/.factory/AGENTS.md` |
| Antigravity IDE | Standalone file + `@import` (shared with Gemini) | `~/.gemini/trs.md` + line in `~/.gemini/GEMINI.md` |
| Antigravity CLI (`agy`) | Standalone file + `@import` (shared with Gemini) | `~/.gemini/trs.md` + line in `~/.gemini/GEMINI.md` |

Codex, OpenCode, Kilo, and Droid are signatories of the
[`AGENTS.md` convention](https://factory.ai/news/agents-md), which is
why they share the same install mechanism.

**Antigravity IDE + CLI (`agy`)** share Gemini's `GEMINI.md`/`trs.md`
for the output-saver only. Their *hooks* are jetski-specific and live
in `~/.gemini/antigravity-{ide,cli}/hooks.json` — see
[supported agents](../support/agents.md#antigravity-ide--antigravity-cli-agy).

## How the install is idempotent

Inline installs wrap the block in HTML comment sentinels:

```
<!-- trs:output-saver:start v1 -->
## Output saver — keep replies cheap
…
<!-- trs:output-saver:end -->
```

Running `--install` again detects the sentinels and replaces the
content between them — no duplication, no accidental user-content
loss. The sentinel carries a version tag (`v1`) so we can migrate
block content in future releases without breaking detection.

The `@import` mechanism for Claude/Gemini is naturally idempotent:
re-install overwrites `trs.md` and re-adds the import line only if
missing. If the legacy `trs-output-saver.md` file exists it is deleted
and its import line replaced with `@trs.md` automatically.

## Check-first semantics

A bare `trs output-saver` never writes. It reports what install would
do for each agent and prints the exact commands to commit or remove
the block. This mirrors `trs audit-docs` and `trs doctor` — nothing
destructive happens without an explicit flag.

Sample output:

```
trs output-saver — scan

  + Claude Code     already installed
  . Gemini CLI      not yet installed
  - Cursor          not detected on this system
  + Codex           already installed
  + Antigravity IDE already installed (shared with Gemini)
  + Antigravity CLI already installed (shared with Gemini)

  1 installable, 4 already installed, 1 not detected, 0 unsupported
```

## `--refresh` — pick up template changes without adding new installs

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
removed along with the sentinels themselves — surrounding user
content is preserved exactly.

## Measuring impact

The block doesn't compress existing output; it steers what gets
generated. Exact savings depend on your agent, model, and prompts.
Anecdotally: ~30-40% fewer tokens on simple tasks (where preambles
are proportionally the bulk of replies), less on long reasoning
replies where the compression target is narration and speculation.

The `trs stats` dashboard tracks input-side savings from `trs rewrite`
but cannot measure output-side savings — they happen outside any
process we run. If you want to measure, compare before/after
token-usage numbers in your agent's own dashboard.

## Interaction with `trs init`

For Claude Code and Gemini CLI, `trs init --global` now writes `trs.md`
automatically alongside the hook config — you get both input compression
and output-saver rules in one command. For all other agents the two
installs remain independent: you can have hooks without output-saver
rules, and vice versa.

## See also

- [`trs init`](init.md) — the input-side compression via hooks.
- [`trs audit-docs`](audit-docs.md) — audit CLAUDE.md / AGENTS.md
  files for bloat; pairs well with output-saver.
- [`trs doctor`](doctor.md) — reports output-saver coverage alongside
  hook coverage.
