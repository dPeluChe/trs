# AGENTS.md — trs (TARS CLI)

## What is trs

A Rust CLI that transforms noisy terminal output into compact, structured signal.
Reduces token consumption by 68-90% for developers, AI agents, and automation pipelines.

## Architecture

```
src/
├── main.rs              # Entry point, mod declarations
├── cli.rs               # Cli struct, OutputFormat enum, flag precedence
├── commands.rs          # Commands enum, ParseCommands enum, TestRunner
├── classifier.rs        # Auto-detect command → parser routing
├── help.rs              # Help text for all commands
├── process.rs           # Process execution (spawn, capture, timeout)
├── formatter/           # 6 output formatters (compact, json, csv, tsv, agent, raw)
│   ├── mod.rs           # Formatter trait + select_formatter
│   ├── compact.rs       # Human-readable compact output
│   ├── json.rs          # Structured JSON
│   ├── csv.rs / tsv.rs  # Tabular formats
│   ├── agent.rs         # AI-optimized markdown
│   └── raw.rs           # Passthrough
├── reducer/             # Reducer framework (truncation, stats, output types)
├── schema/              # JSON schema types (git, fs, search, test, logs, process)
└── router/
    ├── mod.rs            # Router: dispatch commands to handlers
    └── handlers/
        ├── common.rs     # CommandContext, CommandError, CommandStats
        ├── types/        # Data structures (git, fs, grep, test runners, logs)
        ├── run.rs        # trs run <command>
        ├── search.rs     # trs search (ripgrep)
        ├── replace.rs    # trs replace
        ├── tail.rs       # trs tail
        ├── clean.rs      # trs clean
        ├── trim.rs       # trs trim
        ├── html2md.rs    # trs html2md
        ├── txt2md/       # trs txt2md (parser + format + detect)
        ├── isclean.rs    # trs is-clean
        └── parse/        # All input parsers
            ├── git_*.rs  # git status, diff, log, branch
            ├── ls.rs     # ls parser
            ├── grep*.rs  # grep parser + formatter
            ├── find.rs   # find parser
            ├── logs*.rs  # log parser + helpers + formatter
            ├── test*.rs  # pytest, jest, vitest, npm, pnpm, bun
            └── extra.rs  # tree, docker, deps, install, build, env
```

## Key Design Decisions

- **Auto-detect**: `trs git status` detects "git" + "status" and routes to git-status parser
- **Flags anywhere**: `trs git status --json` and `trs --json git status` both work
- **Pipe support**: `git status | trs parse git-status` also works
- **No runtime deps**: Single binary, ~7MB, works on macOS/Linux/Windows
- **Max 500 LOC per file**: 63 production files, none over 621 lines

## Development

```bash
cargo build                    # Build
cargo test                     # Run 2,399 tests
cargo install --path .         # Install globally
./scripts/benchmark.sh         # Compare vs rtk
```

## Testing

- 616 unit tests (src/)
- 685 CLI integration tests (tests/cli.rs)
- 14 additional test suites (replace, search, parser, etc.)
- Total: 2,399 tests, 0 failures, 0 warnings
