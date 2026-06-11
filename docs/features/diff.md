# `trs diff` — audit what compression drops

`trs diff <cmd>` runs a command twice — raw and through trs — and shows
both sides: the byte/token savings, the compact output the agent would
see, and every line that was dropped or collapsed. It exists so you
never have to *trust* the compression: you can check any command in
two seconds.

## Quick reference

```bash
trs diff git status          # raw vs compact + dropped lines
trs diff cargo test          # works with any supported command
trs diff git status --json   # machine-readable report
```

## What it shows

```
trs diff: git status
──────────────────────────────────────────────────
raw:          449 B  ~112 tok
compact:      135 B  ~34 tok   (70% smaller, 78 tok saved)
──────────────────────────────────────────────────
compact output (what the agent sees):
  main
  unstaged (3):
    M components/Button.tsx
    ...
──────────────────────────────────────────────────
dropped / collapsed (12 lines in raw, not in compact):
  − On branch main
  − (use "git add <file>..." to update what will be committed)
  ...
```

Three sections:

1. **Header** — raw vs compact size in bytes and estimated tokens,
   plus the reduction percentage.
2. **Compact output** — exactly what the agent receives when the hook
   rewrites this command.
3. **Dropped / collapsed** — every raw line that did not survive,
   prefixed with `−`. This is the audit trail: errors, file paths and
   failure lines should never appear here. If they do, that's a bug —
   please [report it](https://github.com/dPeluChe/trs/issues).

## When to use it

- **Before trusting trs on a new command** — see what the parser keeps.
- **Debugging an agent that "missed" something** — confirm whether the
  signal was in the compact output or genuinely dropped.
- **Contributing a parser** — the dropped-lines list is the fastest way
  to tune what a new parser should preserve (see
  [CONTRIBUTING.md](https://github.com/dPeluChe/trs/blob/main/CONTRIBUTING.md)).

## Escape hatches

If you ever want the raw output in an agent session: prefix the command
with `TRS_SKIP=1`, or use `trs --raw <cmd>`. Nothing is stored — trs is
a one-shot filter, the raw output always remains one re-run away.
