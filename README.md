# trs - TARS CLI

Transform noisy terminal output into compact, structured signal.

A CLI toolkit that reduces token consumption by **50-88%** for developers, automation pipelines, and AI agents.

## The Problem

Every terminal command produces verbose output. When AI agents run `git status`, `ls -la`, or `grep`, they consume hundreds of unnecessary tokens parsing boilerplate text like "use git add to stage..." or permission strings like `drwxr-xr-x`.

## The Solution

`trs` sits between commands and consumers, stripping noise and delivering structured signal:

```
git status                          →  branch: main
  On branch main                       ahead: 1
  Your branch is ahead of               counts: staged=2 unstaged=3
  'origin/main' by 1 commit.           staged (2):
  (use "git push" to publish...)          M src/main.rs
  Changes to be committed:                A src/new.rs
    (use "git restore --staged...")
    modified:   src/main.rs
    new file:   src/new.rs
  Changes not staged for commit:
    (use "git add <file>...")
    ...
  (370 bytes)                        (101 bytes — 72% reduction)
```

## Install

```bash
# From source
cargo install --path .

# Or build and use directly
cargo build --release
./target/release/trs --help
```

## Quick Start

```bash
# Parse git output
git status | trs parse git-status
git diff   | trs parse git-diff

# Search codebase (ripgrep-powered)
trs search src "TODO" --extension rs
trs search . "import" --extension ts -i

# All commands support multiple output formats
git status | trs --json parse git-status
git status | trs --csv  parse git-status
git status | trs --agent parse git-status
```

## Commands

### parse — Structure terminal output

```bash
# Git
git status | trs parse git-status
git diff   | trs parse git-diff

# File listing
ls -la | trs parse ls

# Search results
grep -rn "pattern" src/ | trs parse grep

# Logs
cat app.log | trs parse logs

# Test runners (pytest, jest, vitest, npm, pnpm, bun)
pytest | trs parse test --runner pytest
npm test | trs parse test --runner npm
```

### search — Find patterns in files

```bash
trs search <path> <query> [options]

# Basic search
trs search . "TODO"

# Filter by extension
trs search src "fn main" --extension rs

# Case insensitive with context
trs search . "error" -i --context 2

# Limit results
trs search . "import" --limit 50
```

### replace — Search and replace in files

```bash
trs replace <path> <search> <replace> [options]

# Preview changes (dry run)
trs replace src "oldName" "newName" --dry-run

# Replace in specific file types
trs replace . "foo" "bar" --extension ts

# Get just the count
trs replace . "TODO" "DONE" --count
```

### tail — Smart log tailing

```bash
# Last 20 lines
trs tail /var/log/app.log -n 20

# Errors only
trs tail /var/log/app.log --errors

# Follow (streaming)
trs tail /var/log/app.log --follow
```

### clean — Strip noise from text

```bash
# Remove ANSI color codes
some-command | trs clean --no-ansi

# Collapse blank lines + trim whitespace
cat messy.txt | trs clean --collapse-blanks --trim

# Full cleanup
noisy-command | trs clean --no-ansi --collapse-blanks --collapse-repeats --trim
```

### html2md / txt2md — Convert to Markdown

```bash
# HTML to Markdown
trs html2md https://example.com
trs html2md page.html -o page.md

# Plain text to Markdown (detects headings, lists)
cat notes.txt | trs txt2md
```

### run — Execute and structure

```bash
# Run any command through trs
trs run echo "hello"
trs --json run git status
trs --json run npm test
```

## Output Formats

Every command supports 6 output formats via global flags:

| Flag | Format | Best for |
|------|--------|----------|
| (default) | Compact | Human reading |
| `--json` | JSON | Programmatic consumption, AI agents |
| `--csv` | CSV | Spreadsheets, data pipelines |
| `--tsv` | TSV | Tab-separated processing |
| `--agent` | Agent | AI-optimized structured text |
| `--raw` | Raw | Unprocessed passthrough |

```bash
# Same command, different formats
git status | trs parse git-status              # compact (default)
git status | trs --json parse git-status       # structured JSON
git status | trs --csv parse git-status        # CSV with headers
git status | trs --agent parse git-status      # AI-friendly
```

When multiple format flags are specified, precedence applies:
`--json` > `--csv` > `--tsv` > `--agent` > `--compact` > `--raw`

## Stats

Add `--stats` to any command to see token reduction metrics:

```bash
git status | trs --stats parse git-status
# Output:
#   Stats:
#     Reducer: git-status
#     Output mode: compact
#     Input bytes: 370
#     Output bytes: 101
#     Reduction: 72.7%
#     Input tokens (est.): 92
#     Output tokens (est.): 25
#     Token reduction: 72.8%
```

## Real-World Reductions

| Command | Raw | trs compact | Reduction |
|---------|-----|-------------|-----------|
| `git status` | 370 B | 101 B | **72%** |
| `git diff` (2 files) | 617 B | 74 B | **88%** |
| `ls -la` (1029 entries) | 108 KB | 54 KB | **49%** |
| HTML page → Markdown | 455 B | 306 B | **32%** |

## For AI Agents

`trs` is designed to be the output layer between system commands and LLMs:

```bash
# Agent runs a command, gets structured JSON instead of raw text
git status | trs --json parse git-status
# → {"branch":"main","is_clean":false,"staged_count":2,"unstaged_count":3,...}

# Search codebase, get parseable results
trs --json search src "handleError" --extension ts
# → {"files":[{"path":"src/handler.ts","matches":[{"line_number":42,...}]},...]}

# Check repo state with a single boolean
trs --json is-clean
# → {"is_clean":true}
```

## Tech Stack

- **Language**: Rust
- **CLI framework**: clap 4
- **Search engine**: ripgrep (grep crate)
- **HTML parsing**: htmd
- **HTTP client**: ureq

## License

MIT
