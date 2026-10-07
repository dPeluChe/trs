---
trigger: always_on
---

# trs (Token-Reducing Shell): terminal output optimization

Devin Local does not expose a pre-execution hook, so this rule is the way to
opt into trs. When running shell commands, prefix them with `trs` to get
compact, structured output.

```bash
# Instead of:
git status
cargo test
pnpm test

# Use:
trs git status
trs cargo test
trs pnpm test
```

Commands without a dedicated trs parser still get whitespace / ANSI
compression (~30-40% reduction). Pipes and chains are passed through unchanged.

## Output saver: keep replies cheap

These hold for every reply, not just the first. Don't drift back to
preambles and filler over a long session.

Keep replies under ~100 words unless the task needs more. Between tool
calls, stay under ~25 words. Match shape to task: a one-line question
gets a one-line answer, no headers.

Open with the answer or the diff. End when the answer ends.

- Result first; explanation only if non-obvious. State the finding, show
  the fix, stop.
- Let tool output speak for itself; don't restate or recap what the diff
  already shows.
- Structured output when the data is structured: bullets, tables, JSON.
  Prose only when the reader is human and the content is narrative.
- Don't invent abbreviations (cfg/impl/req/res) or causal arrows: the
  tokenizer splits them like the full word, so nothing is saved and
  clarity is lost.
- No em dashes. Use a comma, a colon, parentheses, a period, or "and".
  It is the top tell of generated text and almost nobody types one. Same
  for the en dash in prose; numeric ranges keep their hyphen.
- In prose (replies, docs, comments): concrete over abstract. No
  rule-of-three padding, no tier-1 slop (delve, leverage, robust,
  seamless, streamline, potenciar, impulsar). Terse and specific is what
  reads as human, and it is the same cut as cutting tokens.
- Never invent file paths, function names, or API fields. If unknown,
  say "UNKNOWN" or return null. Guessing costs more tokens than asking.
- Full clarity, never compressed, for security warnings, irreversible or
  destructive confirmations, and any multi-step order a misread would break.

## Code authoring

These apply when you write code, not just when you reply in chat.

- Reuse what's already here. A helper, type, or pattern a few files over
  beats re-implementing it. One pass: don't iterate on passing code, don't
  refactor / polish unless asked.
- Comments: none by default. If one is truly needed, write a terse WHY-only
  note (not a walkthrough), at most 3 lines, ~200 characters. Never
  paragraph docstrings or restate the code. A longer logic explanation goes
  to docs/ with a one-line pointer in the comment.
  - BAD: 8 lines narrating the algorithm step by step.
  - GOOD: `// OCC retry: Convex aborts concurrent writes; 3 attempts fit real load`

User instructions always override these rules.

## Keeping this file lean

Run `trs audit-docs` periodically to surface content that inflates every
agent session: duplicate sections, embedded code/SQL blocks that belong in
their own files, dead references. Every unnecessary token here loads on every
call.

Reference: https://github.com/dPeluChe/trs
