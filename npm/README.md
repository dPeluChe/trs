# trs: Token-Reducing Shell

Transform noisy terminal output into compact, structured signal.
A CLI toolkit for developers, automation pipelines, and AI agents.

**68-99% token savings** on common dev operations.

## Install

```bash
npm install -g @dpeluche/trs
```

Or with other package managers:

```bash
cargo install trs-cli    # from source (Rust required)
```

## Usage

```bash
# Git (compressed output)
trs git status
trs git diff
trs git log --oneline -20

# Test runners
trs cargo test / go test / pytest / jest / vitest / npm test

# Linters
trs cargo clippy / eslint / ruff / biome / golangci-lint

# Files & search
trs ls -la / find / grep / tree

# Project digest (LLM-ready codebase index)
trs ingest                    # digest current project
trs ingest --budget 128k      # fit to token budget
trs ingest --deps             # dependency graph only

# AI tool hooks (auto-rewrite commands through trs)
trs init claude               # Claude Code
trs init gemini               # Gemini CLI
trs init cursor               # Cursor
trs init codex                # Codex (AGENTS.md)

# Output-side savings (new in 0.5.7)
trs output-saver              # scan agents, suggest install
trs output-saver --install    # install rules block into agent configs

# Installation health + docs auditor
trs doctor                    # check install, PATH, deps
trs audit-docs                # lint CLAUDE.md / AGENTS.md for bloat
```

## How it works

trs wraps your existing commands, parses their output, and returns a compact structured version. No changes to your workflow, just prefix with `trs`.

```
$ git status                    # 2.1 KB raw output
$ trs git status                # 58 bytes (97% reduction)
main | clean | 0 staged
```

## Output formats

```bash
trs git status --json       # structured JSON
trs git status --compact    # human-readable (default)
trs git status --agent      # AI-optimized markdown
trs git status --csv        # tabular
trs git status --raw        # passthrough (tracked)
```

## Supported platforms

| Platform | Architecture | Notes |
|----------|-------------|-------|
| macOS    | x64, arm64  | shell launcher (~12ms overhead) |
| Linux    | x64, arm64  | shell launcher (~12ms overhead) |
| Windows  | x64         | `.cmd` launcher (requires Windows x64) |

> The launcher `exec`s the native binary directly, no Node.js at runtime,
> which is why startup is on par with a Homebrew install.

## Links

- [GitHub](https://github.com/dPeluChe/trs)
- [Documentation](https://github.com/dPeluChe/trs#readme)

A product by [Iteris](https://iteris.tech) · Published and maintained by [@dPeluChe](https://dpeluche.dev)

MIT License
