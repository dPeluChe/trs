# What actually moves agent cost: findings from the output-saver tests

Measured 2026-11-04 on Claude Code with sonnet. Read this before changing the
output-saver block or claiming an output-side saving. Per-test numbers and the
public claims live in [the output-saver doc](../features/output-saver.md#measuring-impact).

## The short version

- Reply rules don't pay. No rule block beat the one line "Answer concisely."
  on tokens, cost, turns or correct answers.
- Asking for brevity at all does work: about half the output tokens of no
  instruction on chat-style prompts.
- Output is a small part of the bill. Turns and context size are the lever.
- The model already reads the code. Telling it to verify changed nothing we
  could measure.
- So the block is now 698 bytes: the one line, "never invent", the comment rule.

## Where the money goes

From local transcripts, 105,592 model calls, at price ratios of 1 (input), 1.25
(cache write), 0.1 (cache read) and 5 (output):

| Part of the bill | Share |
| --- | --- |
| Cache reads (rereading the context every turn) | 76% |
| Cache writes | 16.4% |
| Output | 7.6% |

Inside output tokens: hidden thinking about 50-58% (a residual estimate, so an
upper bound), tool arguments 35% (73% of Bash tokens are heredocs), reply prose
6-10%. A rule that shortens prose touches about 0.5% of total cost.

A public paired study (arXiv 2607.12161) reached the same place from the input
side: reducing tool output is not a reliable predictor of billed cost. A
published agentic A/B of a telegraphic reply style found -8.5% output with no
quality loss, far below the 65% its authors quote from chat prompts, while a
system prompt aimed at fewer steps cut median cost 17.9%.

## The tests

| Test | Runs | Arms | Result |
| --- | --- | --- | --- |
| Chat replies, no tools (`output_saver_ab.py`) | 40 per arm | none, one line, block | Run 2, with the longer block: vs none -235 tokens (CI -326 to -141), vs one line -22 (CI -80 to +39), fidelity 100% in every arm. Run 1, older block: -237 and -20, fidelity 95%, see the feature doc |
| Small repo tasks, tools on (`output_saver_agentic.py`) | 24 per arm | none, one line, 3 blocks | No arm differs on cost or turns. Blocks about $0.004 per run above none |
| Stale docs vs code (`--suite stale`) | 40 per arm | none, one line, block | Correct: 95%, 95%, 90%. Block vs one line -5 points (CI -17.5 to +5) |

The third test also gave the only clear cost signal, and it favors the one
line. Same runs:

| Arm | Cost | Turns | Output tokens | Both values shown |
| --- | --- | --- | --- | --- |
| none | $0.048 | 3.5 | 541 | 95% |
| "Answer concisely." | $0.045 | 2.9 | 363 | 95% |
| block | $0.049 | 3.4 | 409 | 82% |

The one line beats none on cost (-$0.003, CI -$0.005 to -$0.001) and turns
(-0.68, CI -1.28 to -0.05), and beats the block on cost (-$0.005, CI -$0.007 to
-$0.002). These comparisons were not the pre-registered ones and many were
looked at, so treat them as a lead to retest, not a result.

Rules tried and dropped, none with a measured gain: word budgets, shape and
style rules, "fewer turns" (batching, bounded reads, no polling), "read once",
"verify against code", "prefer the newest source". The "fewer turns" rules
moved turns by +0.25 (CI -0.67 to +1.13).

## What we don't know

- All tasks are short (2 to 7 turns) on one model at low effort. Long sessions,
  where drift and context growth matter, are untested.
- Weaker models may need the rules this one doesn't. Nothing here covers them.
- 40 runs per arm only show gaps of about 25 points, and runs of one task are
  not independent. A null is not proof of no effect.
- The stale-doc control was near the ceiling, and almost every miss was one trap
  task (a default port changed by an override).
- The shipped 698-byte block was not itself rerun. Its rules are a subset of
  the tested block plus the control line.
- The comment rule and "never invent" have never been measured. They stay
  because they cover what the one line doesn't, and the comment limits are
  mirrored by other configs.

## Pre-registered: "thorough and bold" (2026-11-06)

Claim under test: "Be thorough in your analysis and bold in your
recommendations." beats "Answer concisely." Arms: `none`, `concise`, `tweet`
(the phrase alone), `bold` ("Answer concisely. Be bold in your
recommendations."). Sonnet, low effort, repo and stale suites.

- Primary: correct answers and cost per run, each arm against `concise`.
- A new arm replaces `concise` only if its correct-answer CI excludes zero in
  its favor and its cost is at most 10% above `concise`.
- If neither arm wins, the block stays as is and this section records the numbers.
- The stale control sits near the ceiling, so a null here can't rule out a gain.
  Neither suite scores recommendations; output tokens and cost show the price.

## How to run the next test

1. Commit the criteria first, including what happens if the change loses.
2. Compare to "Answer concisely.", not only to no instruction. Beating nothing
   is easy; most of the gain comes from asking for brevity at all.
3. Pilot one round. If the control is near 100%, the test can't show a gain:
   harden it once, commit, and stop tuning.
4. Read every failure. A first sweep was invalid because the Bash allowlist
   lacked `sed`: 50 of 120 runs hit a denial and every failure was one of them.
5. Isolate each run: `claude -p` in a temp repo with the arm as `CLAUDE.md` and
   `--setting-sources project`. Subagents can't do this, they inherit the global
   config and the trs hook.
6. Use `--permission-mode acceptEdits` with a scoped `--allowedTools`, never
   `bypassPermissions`.
7. Cap spend with `--total-cap-usd`. About 120 runs cost $6 to $8 at low effort.

## Where to go next

In order of expected value:

1. **Cut turns from the tool side, not with reply rules.** Check the size first:
   delta for repeated test runs was dropped because test reruns are 0.9% of
   shell output at most, with a median of 327 bytes. Top shell output is `find`,
   `grep` and `curl`, but 100 runs make up 68-98% of each, so look at those
   one-offs before building filters.
2. **Test long sessions.** Build a 30+ turn task and rerun the arms. That is
   where context growth lives and where the one-line lead can be confirmed.
3. **Test a weaker model.** If rules help anywhere, it is likely there.
4. **Keep the always-loaded stack small.** Every byte in it is reread each turn.
5. **Heredoc-heavy tool arguments** are a bigger output share than prose. Worth
   a look, but no idea so far survives the fact that trs can't edit what the
   model generates.

Don't add a line to the block without a test it passes. Size a feature against
local history and transcripts before building it, and test the estimate: a loose
regex said piped checks that could be fixed were 17.7% of Bash calls, the real
rewrite reached 3.5%.
