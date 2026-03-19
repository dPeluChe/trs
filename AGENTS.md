# AGENTS.md — trs (TARS CLI)

## What is trs

A Rust CLI that transforms noisy terminal output into compact, structured signal.
Reduces token consumption by 68-90% for developers, AI agents, and automation pipelines.

## Architecture

```
src/
├── main.rs                    # Entry point, mod declarations
├── cli.rs                     # Cli struct, OutputFormat enum, flag precedence
├── commands.rs                # Commands enum, TestRunner
├── commands_parse.rs          # ParseCommands enum
├── classifier.rs              # Auto-detect command → parser routing
├── classifier_exec.rs         # Execute → parse → format pipeline
├── classifier_transfer.rs     # Compact git push/pull/fetch output
├── config.rs                  # Config system (~/.trs/config.toml)
├── help.rs                    # Help text for all commands
├── process.rs                 # Process execution (spawn, capture, timeout)
├── process_helpers.rs         # Spawn error classification, output capture
├── tracker.rs                 # Token savings tracker (history.jsonl)
├── formatter/
│   ├── mod.rs                 # Formatter trait + select_formatter
│   ├── compact.rs             # Human-readable compact output
│   ├── compact_schema_git.rs  # Compact format: git status/diff schemas
│   ├── compact_schema_output.rs # Compact format: ls/grep/find/test/logs schemas
│   ├── json.rs                # Structured JSON
│   ├── agent.rs               # AI-optimized markdown
│   ├── agent_schema.rs        # Agent format: all schema types
│   ├── csv.rs / tsv.rs        # Tabular formats
│   ├── raw.rs                 # Passthrough
│   └── tests/                 # 6 test modules (150 tests)
├── reducer/
│   ├── mod.rs                 # Reducer framework (truncation, stats)
│   ├── output.rs / registry.rs
│   └── tests/                 # 6 test modules (93 tests)
├── schema/                    # JSON schema types (git, fs, search, test, logs, process)
└── router/
    ├── mod.rs                 # Router: dispatch commands to handlers
    ├── tests/                 # 14 test modules (225 tests)
    └── handlers/
        ├── common.rs          # CommandContext, CommandError, CommandStats
        ├── types/             # Data structures (git, fs, grep, test runners, logs)
        ├── run.rs             # trs run <command>
        ├── search.rs          # trs search (ripgrep)
        ├── replace.rs         # trs replace
        ├── tail.rs            # trs tail
        ├── clean.rs           # trs clean
        ├── trim.rs            # trs trim
        ├── json.rs            # trs json (structure without values)
        ├── read.rs            # trs read (handler + filter levels)
        ├── read_filters.rs    # Language detection, minimal/aggressive filters
        ├── html2md.rs         # trs html2md
        ├── txt2md/            # trs txt2md (detect_headings + detect_lists + format)
        ├── isclean.rs         # trs is-clean
        ├── err.rs             # trs err (error filter)
        ├── stats.rs           # trs stats (token savings dashboard)
        └── parse/             # All input parsers
            ├── git_*.rs       # git status, diff, log, branch
            ├── ls.rs          # ls parser
            ├── grep*.rs       # grep parser + formatter
            ├── find.rs        # find parser
            ├── logs*.rs       # log parser + helpers + formatter
            ├── {pytest,jest,vitest,npm,pnpm,bun}_{parse,format}.rs
            ├── extra_system.rs    # tree, docker, deps, install, build, wc
            ├── extra_download.rs  # curl/wget download handler
            ├── extra_env.rs       # env handler (grouped, filtered)
            ├── extra_services.rs  # gh pr/issue/run (truncated titles)
            └── extra_cargo_test.rs # cargo test parser

tests/
├── fixture_data/              # 160+ .txt/.html/.log fixture files
├── fixtures/                  # Fixture loader module (7 sub-modules)
├── cli_*.rs                   # 26 CLI integration test files
├── test_replace_*.rs          # 5 replace test files
├── test_search_*.rs           # 3 search test files
├── test_parser_*.rs           # 5 parser test files
├── test_signal_*.rs           # 3 signal preservation test files
├── test_clean_*.rs            # 3 clean test files
├── test_conversion_*.rs       # 3 conversion test files
├── test_run_*.rs              # 3 run test files
├── test_tail_*.rs             # 3 tail test files
└── ...                        # 70+ total test files
```

## Key Design Decisions

- **Auto-detect**: `trs git status` detects "git" + "status" and routes to git-status parser
- **Flags anywhere**: `trs git status --json` and `trs --json git status` both work
- **Pipe support**: `git status | trs parse git-status` also works
- **No runtime deps**: Single binary, ~7MB, works on macOS/Linux/Windows
- **Max 500 LOC per file**: 210+ .rs files, all under 506 lines (2 at 503-506)
- **Token tracking**: Every execution logged to ~/.trs/history.jsonl
- **3-tier fallback**: parser OK → degraded → truncated passthrough with `[trs:passthrough]`
- **Generic fallback**: commands without parser get whitespace/ANSI compression (20-40%)
- **Config system**: `~/.trs/config.toml` for tunable limits

## Development

```bash
cargo build                    # Build
cargo test                     # Run 2,017+ tests
cargo install --path .         # Install globally
./scripts/benchmark.sh         # Compare vs rtk (trs 13:4 rtk)
```

## Testing

- 658 unit tests (src/) across 30+ test modules
- 542 CLI integration tests (tests/cli_*.rs, 26 files)
- 812 additional integration tests (70+ test files)
- Total: 2,012 tests, 0 failures
