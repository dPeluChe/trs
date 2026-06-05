# Contributing to trs

Thanks for your interest in contributing! trs is a personal project that grew into something useful, and contributions are welcome — whether it's a new parser, a bug fix, or just better docs.

## Getting started

```bash
git clone https://github.com/dPeluChe/trs.git
cd trs
cargo build
cargo test
```

All three checks must pass before submitting a PR:

```bash
cargo fmt -- --check           # formatting
cargo clippy -- -D warnings    # no warnings allowed
cargo test                     # 2,186+ tests, 0 failures
```

The repo ships a pre-push hook that runs these automatically. Activate it once after cloning:

```bash
git config core.hooksPath .githooks
```

## Code guidelines

### File size
- **Max 500 lines per file.** If a file grows past this, split it.
- Rust allows multiple `impl` blocks in separate files — use this pattern.
- Tests go in `tests/` (integration) or `src/*_tests.rs` (unit).

### Naming
- Parser files: `{tool}_parse.rs` + `{tool}_format.rs` (e.g. `npm_parse.rs`, `npm_format.rs`)
- Test files: `test_{feature}_{category}.rs` (e.g. `test_replace_edge.rs`)
- Fixture data: `tests/fixture_data/{tool}_{scenario}.txt`

### Style
- Run `cargo fmt` before committing. No exceptions.
- No `unwrap()` in production code — use `?` or explicit error handling.
- `unwrap()` is fine in tests.
- Prefer simple code over clever code. Three similar lines > premature abstraction.
- Don't add doc comments to every function — only where the logic isn't obvious.

### Tests
- Every new parser needs at least 3 tests: basic input, edge case, empty input.
- Integration tests (in `tests/`) test the CLI binary end-to-end.
- Unit tests (in `src/`) test individual functions.
- Don't assert on timing (`duration_ms > 0`) — fast CI runners can complete in <1ms.
- Tests that hit the live network must be `#[ignore]`d (they're not a CI gate — a
  flaky endpoint shouldn't redden the build). Run them with `cargo test -- --ignored`.
- Fixture files go in `tests/fixture_data/`. Add `!tests/fixture_data/*.log` patterns to `.gitignore` if needed.

## Adding a new parser

This is the most common contribution. Here's the process:

### 1. Add the command variant

In `src/commands_parse.rs`, add a new variant to `ParseCommands`:

```rust
/// Parse {tool} output
ToolName {
    #[arg(short, long)]
    file: Option<PathBuf>,
},
```

### 2. Create the parser

Create `src/router/handlers/parse/{tool}.rs`:

```rust
use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
    pub(crate) fn handle_tool_name(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        // Parse the input...
        let output = "...";

        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("tool-name")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }
}
```

### 3. Wire it up

- `src/router/handlers/parse/mod.rs` — add `pub(crate) mod {tool};` and the dispatch match arm
- `src/classifier.rs` — add the subcommand → parser dispatch in `classify_command()`
- `src/classifier.rs` — add to `inject_file_path()`
- `src/command_registry.rs` — add **one row** to `REGISTRY` with the command's
  flat facts: aliases, `rewrite`/`known` flags, `keep_ratio`, and stderr policy.
  This is the single source of truth — keep_ratio, stderr handling, rewrite
  eligibility and coverage stats all read from it, so you no longer edit four
  files to add a command.

### 4. Handle stderr (if needed)

Some tools write their primary output (errors, warnings, results) to stderr
(clippy, eslint, gcc). Set the `stderr` field on the command's `REGISTRY` row
in `src/command_registry.rs` — `Stderr::Always`, or `Stderr::Subcmds(&[...])`
for subcommand-scoped cases like `cargo build` vs `cargo test`.

### 5. Add tests

At minimum:
```rust
#[test]
fn test_parse_tool_basic() { ... }

#[test]
fn test_parse_tool_empty() { ... }

#[test]
fn test_parse_tool_edge_case() { ... }
```

### 6. Update docs

- `README.md` — add the command to the appropriate section
- `AGENTS.md` — add the file to the architecture diagram
- `docs/roadmap/completed/YYMM.md` — document what you did and why

## Proposing new commands

Before building a new parser, open an issue with:

1. **Command**: what command are you parsing? (e.g. `terraform plan`)
2. **Raw output**: paste a sample of the raw output (anonymized)
3. **Proposed compact output**: what should trs show instead?
4. **Reduction estimate**: roughly what % smaller?

This helps us discuss the design before you invest time coding it.

### Good candidates for new parsers
- Commands with verbose, structured output (tables, logs, status reports)
- Commands that AI agents run frequently
- Commands where >50% of the output is noise

### Not a good fit
- Commands that already output compact data
- Commands where every byte matters (losing info is worse than saving tokens)
- Highly specialized tools used by <100 people

## Project structure

See [AGENTS.md](AGENTS.md) for the full architecture diagram. Key directories:

```
src/
├── command_registry.rs    # Single source of truth: per-command facts
│                          #   (aliases, rewrite/known, keep_ratio, stderr)
├── classifier.rs          # Subcommand → parser dispatch
├── router/handlers/parse/ # All input parsers live here
├── router/handlers/       # Built-in tools (search, replace, json, read...)
└── formatter/             # 6 output formats (compact, json, csv, tsv, agent, raw)

tests/
├── cli_*.rs               # CLI integration tests
├── test_*.rs              # Feature-specific tests
└── fixture_data/          # Test input files
```

## Commit messages

We use [Conventional Commits](https://www.conventionalcommits.org) — the
changelog and release notes are generated from them, so the prefix matters.

```
feat(kubectl): add parser for pod/service output
fix(git): correct status grouping for >20 files
docs: update README with benchmark table
refactor(classifier): split arg preprocessing into classifier_args
```

Types that surface in the changelog: `feat`, `fix`, `perf`, `refactor`,
`docs`, `test`, `ci`/`build`, `chore(deps)`. `chore`/`chore(release)`/`release`
are skipped. Breaking changes: add `!` after the type (`feat!:`) or a
`BREAKING CHANGE:` footer.

Include `Co-Authored-By` if pair-programming with AI.

## Releasing

The changelog is automated with [git-cliff](https://git-cliff.org)
(config: [`cliff.toml`](cliff.toml)) from the Conventional Commits above.

1. Bump the version in `Cargo.toml`, `npm/package.json`, and
   `npm/platforms/*/package.json`.
2. Regenerate the changelog: `git cliff -o CHANGELOG.md`
3. Commit (`chore(release): bump to vX.Y.Z`) and merge to `main`.
4. Tag and push: `git tag vX.Y.Z && git push origin vX.Y.Z`

The tag triggers `release.yml`, which builds the binaries, **regenerates the
GitHub Release notes for that tag with git-cliff** (grouped by type — not a
raw PR list), and publishes the npm packages. Devs watching the repo get the
release notification; npm and GitHub both show the same curated changelog.

## Questions?

Open an issue. There's no wrong question.
