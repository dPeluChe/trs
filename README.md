# trs - TARS CLI

Transform noisy terminal output into compact, structured signal.

A CLI toolkit that reduces token consumption by **50-88%** for developers, automation pipelines, and AI agents.

## The Problem

Every terminal command produces verbose output. When AI agents run `git status`, `ls -la`, or `grep`, they consume hundreds of unnecessary tokens parsing boilerplate text like "use git add to stage..." or permission strings like `drwxr-xr-x`.

## The Solution

Just prefix your command with `trs`:

```
$ trs git status

branch: main
ahead: 1
counts: staged=2 unstaged=3
staged (2):
  M src/main.rs
  A src/new.rs

# vs raw git status: 370 bytes → 101 bytes (72% reduction)
```

## Install

```bash
# npm (recommended)
npm install -g tars-cli

# Try without installing
npx tars-cli git status

# From source (requires Rust)
cargo install --path .
```

## Quick Start

```bash
# Just prefix any command with trs
trs git status
trs git diff
trs ls -la
trs npm test

# Add --json for structured output
trs git status --json
trs git diff --json

# Format flags work before or after the command
trs --json git status    # also works
```

## Auto-Detect Commands

`trs` automatically detects the command and applies the right parser:

```bash
# Git
trs git status              # → git-status parser
trs git diff                # → git-diff parser
trs git log -10             # → git-log parser
trs git branch -a           # → git-branch parser

# File system
trs ls -la                  # → ls parser
trs tree -L 2               # → tree parser
trs find . -name "*.rs"     # → find parser

# Search
trs grep -rn "pattern" .    # → grep parser

# Test runners
trs pytest                  # → pytest parser
trs npm test                # → npm test parser
trs jest / vitest / bun test

# Package managers
trs npm ls                  # → dependency list
trs pip list                # → dependency list
trs npm install             # → install summary
trs pip install requests    # → install summary

# Build tools
trs cargo build             # → build output (errors/warnings)
trs tsc                     # → TypeScript errors
trs make                    # → build output

# Docker
trs docker ps               # → container status
trs docker logs <name>      # → log parser

# System
trs env                     # → sorted, truncated values
trs tail /var/log/app.log   # → log parser

# Unsupported commands pass through
trs echo "hello"            # → passthrough
```

## Built-in Tools

### search — Find patterns in files (ripgrep-powered)

```bash
trs search src "TODO" --extension rs
trs search . "handleError" -i --context 2
trs search . "import" --limit 50
```

### replace — Search and replace in files

```bash
trs replace src "oldName" "newName" --dry-run   # preview first
trs replace src "oldName" "newName"              # execute
trs replace . "TODO" "DONE" --count              # just the count
```

### tail — Smart log tailing

```bash
trs tail /var/log/app.log -n 20
trs tail /var/log/app.log --errors    # error lines only
trs tail /var/log/app.log --follow    # streaming mode
```

### clean — Strip noise from text

```bash
some-command | trs clean --no-ansi --collapse-blanks --trim
```

### html2md / txt2md — Convert to Markdown

```bash
trs html2md https://example.com
trs html2md page.html -o page.md
cat notes.txt | trs txt2md
```

### is-clean — Check git repo state

```bash
trs is-clean                  # exit 0 if clean, 1 if dirty
trs is-clean --json           # {"is_clean": true}
```

## Pipe Syntax

The pipe syntax also works for all parsers:

```bash
git status | trs parse git-status
git diff   | trs parse git-diff
ls -la     | trs parse ls
grep -rn "pattern" . | trs parse grep
cat app.log | trs parse logs
pytest | trs parse test --runner pytest
```

## Output Formats

Every command supports 6 output formats. Flags work before or after the command:

```bash
trs git status                # compact (default)
trs git status --json         # structured JSON
trs git status --csv          # CSV with headers
trs git status --tsv          # tab-separated
trs git status --agent        # AI-optimized text
trs git status --raw          # unprocessed passthrough
trs git status --stats        # show token reduction metrics
```

When multiple format flags are specified, precedence applies:
`--json` > `--csv` > `--tsv` > `--agent` > `--compact` > `--raw`

## Real-World Reductions

| Command | Raw | trs compact | Reduction |
|---------|-----|-------------|-----------|
| `git status` | 465 B | 131 B | **71%** |
| `git diff` | 34 KB | 125 B | **99%** |
| `git log -10` | 6.4 KB | 1.2 KB | **81%** |
| `ls -la` | 1.3 KB | 122 B | **57%** |
| `env` | 3.5 KB | 2.2 KB | **38%** |

Run the benchmark yourself:

```bash
./scripts/benchmark.sh
```

## For AI Agents

`trs` is designed as the output layer between system commands and LLMs:

```bash
# Structured JSON for programmatic consumption
trs git status --json
# → {"branch":"main","is_clean":false,"staged_count":2,...}

trs search src "handleError" --extension ts --json
# → {"files":[{"path":"src/handler.ts","matches":[...]}],...}

trs is-clean --json
# → {"is_clean":true}
```

## Tech Stack

- **Language**: Rust
- **Binary size**: ~6 MB (self-contained, no runtime dependencies)
- **CLI framework**: clap 4
- **Search engine**: ripgrep (grep crate)
- **HTML parsing**: htmd
- **HTTP client**: ureq

## License

MIT
