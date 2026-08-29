# Changelog

All notable changes to **trs** are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.8.1] - 2026-08-29

### Bug Fixes

- **install:** Resolve the release from two sources, not one (#154)
- **init:** Refresh rules blocks on drift, centralize the sentinels, tidy the upgrade output (#155)

## [0.8.0] - 2026-08-28

### Features

- **output-saver:** No em dashes, no slop, and stop shipping the tell (#143)
- **parsers:** Compress du, lsof and pgrep (#147)
- **parsers:** Gh api boilerplate, plus a full documentation audit (#148)

### Bug Fixes

- **compress:** Never compress commands whose output is their layout (#146)
- **registry:** Stop the coverage report lying in both directions (#149)

### Refactor

- Split ten oversized files, and parse ollama (#150)

## [0.7.5] - 2026-08-13

### Features

- **aws:** Compress recursive s3 output from receipts to counts (#131)
- **find, stats:** Tier find output by size, add `stats --gaps` (#132)
- **stats:** Show recent efficiency next to the lifetime mean (#139)
- **stats:** Add --days, and window the gaps view by default (#140)

### Refactor

- **stats:** Fold --gaps into --coverage, document the windows (#141)

### CI / Build

- Split CI into a Tenki quick tier and a GitHub full gate (#130)
- Move the quick tier back to GitHub runners, cancel superseded runs (#135)
- Adopt tenki-standard-small-2c-4g as the quick-tier default (#138)

### Dependencies

- **deps:** Bump the cargo-minor group across 1 directory with 4 updates (#134)

## [0.7.4] - 2026-07-29

### Bug Fixes

- **rewrite:** Stop doing text surgery on commands we can't parse (#127)

### Dependencies

- **deps:** Bump the cargo-minor group across 1 directory with 7 updates (#123)

## [0.7.3] - 2026-07-28

### Bug Fixes

- **output:** Never claim success for a failed command; stop capturing compressed output (#124)

## [0.7.2] - 2026-07-28

### Features

- **ingest:** In-band budget warning for --agent large digests (#119)
- **output-saver:** Split "Code authoring" into its own section (#121)

## [0.7.1] - 2026-07-18

### Features

- **ingest:** --agent implies --print (digest to stdout) (#117)

## [0.7.0] - 2026-07-18

### Features

- **core:** Never-worse output guard + lock exit-code fidelity (rtk 0.43.0) (#105)
- **output-saver:** Sharpen rules with caveman/ponytail learnings (lean) (#104)
- **output-saver:** Add --verify to confirm agents picked up the block (#106)
- **ingest:** --html visual codebase report with dependency graph (#111)

### Bug Fixes

- **ingest:** Capture pub(crate)/pub(super) symbols in Rust digests (#109)

### Refactor

- **output:** Make trs init/upgrade/doctor output scannable (#103)
- Extract inline test modules to *_tests.rs (under 500 LOC) (#110)
- Apply ingest learnings — dedup + LOC splits (#112)

### Documentation

- Genericize competitor references in guard/exit-code comments (#107)

### CI / Build

- **release:** Pin npm to 11.x for publish (npm@latest 12.x needs node ≥22) (#116)

### Dependencies

- **deps:** Bump the cargo-minor group across 1 directory with 2 updates (#99)
- **deps:** Bump actions/cache from 5 to 6 (#97)
- **deps:** Bump actions/checkout from 6 to 7 (#98)
- **deps:** Bump actions/setup-node from 6 to 7 (#113)
- **deps:** Bump the cargo-minor group with 4 updates (#114)

## [0.6.17] - 2026-07-07

### Features

- **classifier:** Route by basename, bare py linters/formatters, timeout unwrap (#101)
- **devin-cli:** Devin CLI programmatic hook — 16th agent (#100)

### Documentation

- **diff:** Feature page for trs diff + llms.txt entry (#92)
- **roadmap:** OpenClaw + Hermes agent research — turnkey, gated on live validation (#93)

### Dependencies

- **deps:** Batch dependabot updates — actions + cargo (#94)

## [0.6.16] - 2026-06-09

### Features

- **rewrite:** Fail open on unrecognized hook_event_name (#77)
- **coverage:** Close field-data compression gaps (git commit/ls-files, cargo fmt, bash -c) (#78)
- **quality:** Signal-preservation harness + 3 parser bugs it caught (#79)
- **logs:** JSON structured-log field extraction (reducer + builtin tail) (#76)
- **devin:** Rename Windsurf agent to Devin Desktop + dual rules target (#75)
- **vscode:** VS Code Copilot agent support — 12th agent (#81)

### Documentation

- **roadmap:** VS Code Copilot research — turnkey spec, gated on live validation (#80)

## [0.6.15] - 2026-06-05

### Features

- **release:** Automate CHANGELOG + release notes with git-cliff (#72)

### Testing

- **ci:** Mark live-network html2md tests as #[ignore] (#73)

## [0.6.14] - 2026-06-04

### Refactor

- Split oversized modules under 500 LOC (7 modules incl audit_docs) + v0.6.14 (#71)

## [0.6.13] - 2026-06-04

### Features

- **pi:** Support Pi coding agent (pi.dev) via bash spawnHook extension (#69)

### Documentation

- **roadmap:** Mark trs diff shipped (v0.6.12); init.rs split done, remaining splits tracked

## [0.6.12] - 2026-06-02

### Features

- **diff:** Trs diff <cmd> — raw vs compact + exactly what was dropped (#66)

### Refactor

- Extract AiTool registry to src/ai_tool.rs (init.rs 711 -> 303) (#67)

## [0.6.11] - 2026-06-02

### Bug Fixes

- **doctor,install,upgrade:** PATH-dup detection, zshenv env-source, clean re-upgrade msg (#64)

## [0.6.10] - 2026-06-02

### Features

- **output-saver:** Cap agent code comments to brief reference notes (#62)

### Bug Fixes

- **#60:** Forward-slash path output on Windows (search/find) (#61)

## [0.6.9] - 2026-06-02

### Bug Fixes

- **#58:** Run trs on a 16 MB worker thread (Windows parse-test stack overflow) (#59)

## [0.6.8] - 2026-06-02

### Bug Fixes

- **#53:** Windows-safe OpenCode/Kilo plugins + shell-routed execution (#57)

## [0.6.7] - 2026-06-02

### Features

- **codex:** Enable PreToolUse rewrite hook (codex>=0.134) + AiTool identity registry (#55)

### Refactor

- Unify per-command knowledge into a single command registry (#54)

## [0.6.6] - 2026-05-23

### Features

- Agy feedback roundup — history ordering, install UX, find polish, monthly rotation (#50)

### Bug Fixes

- **antigravity:** Revert v0.6.5 jetski hook integration, move to rules-only (#51)

## [0.6.5] - 2026-05-22

### Bug Fixes

- **hooks:** Correct Antigravity (jetski) integration + Codex orphan sweep (#47)

### Documentation

- Update Antigravity (IDE + agy CLI) integration details (#48)

## [0.6.4] - 2026-05-20

### Features

- **init:** Antigravity 2.0 — IDE + CLI via shared Gemini hooks (#45)
- **release:** Migrate to npm Trusted Publishing (#46)

## [0.6.3] - 2026-05-19

### Features

- **output-saver:** Doctor validation + stronger research-backed prompt (#44)

## [0.6.1] - 2026-05-14

### Features

- **stats:** --coverage parser-gap analysis (#41)

## [0.6.0] - 2026-05-14

### Features

- Transparent_prefixes, init --dry-run, codex fixes (#38)

### Refactor

- Split init/rewrite, add  interactive command (#39)

## [0.5.16] - 2026-05-01

### Bug Fixes

- **template:** Tighter "Shell output" defensive line (#37)

## [0.5.15] - 2026-05-01

### Features

- **prompt+stats:** De-promote bypass to agents, add bypass telemetry (#36)

## [0.5.14] - 2026-04-30

### Bug Fixes

- **grep+build:** Two-pass grep scan, stderr combining for build tools (#35)
- **templates:** CODEX_AGENTS_SECTION imperative prefix; refresh Gemini hook format

### Documentation

- **output-saver:** Discourage trs raw for routine commands

## [0.5.13] - 2026-04-28

### Features

- Gh run view parser + npm/pnpm/bun run format routing (#32)
- Poetry run routing, lint:* variants, git pull parser, rtk v0.37.2 hook (#33)

### Documentation

- Add llms.txt for LLM agent discovery
- **llms.txt:** Shorten blockquote to spec-compliant 1-3 sentences
- **llms.txt:** Remove duplicate links, clean up structure

## [0.5.12] - 2026-04-24

### Bug Fixes

- **release:** Sync platform package versions after artifact assembly (#29)

### Documentation

- Update for v0.5.11 — trs.md, git grep, npm/pnpm run, @biomejs/biome (#27)

## [0.5.11] - 2026-04-24

### Features

- **output-saver:** Unified trs.md — input-rewrite + output-saver, trs init writes it (#25)

### Dependencies

- **deps:** Drop unused grep-matcher / grep-regex / grep-searcher (#22)

## [0.5.10] - 2026-04-23

### Features

- V0.5.10 — tsc parser, git push compression, ratio gate, common.rs split (#23)

### Bug Fixes

- **fmt:** Apply cargo fmt to extra_services, classifier, commands_parse, read_intercept (#24)

### Documentation

- **install:** Move installer to docs/ so Pages serves it
- Reorganize docs/ into support / features / development / roadmap
- Populate support/, features/, development/ with new references
- Restructure README + landing around agents / commands / features
- **readme:** Drop redundant Tech stack + Features, reorder install
- **readme:** Flip quick-start order, move config out, reference table
- **readme:** Drop Safety&quirks, reorder standalone, uniform link style
- **readme:** Uniform link styling in For contributors table
- **readme.es:** Clean up Spanglish, keep tech terms in English
- **readme:** Attribute Why to Iteris team + fix benchmarks link label
- **landing:** SEO polish + Features cleanup + Output saver section
- **roadmap:** Track documentation drift as deferred work
- **commands:** Stub redirects for v0.5.9-shipped doc URLs

## [0.5.9] - 2026-04-21

### Features

- **domain:** Migrate to usetrs.dev custom domain
- **stats:** Add -n/--limit; default top 15, history 20

### Bug Fixes

- **naming:** Catch remaining TarsCLI mentions + npm bug

### Documentation

- **readme:** Restructure install / quick start / dev sections
- **naming:** Anchor trs as Token-Reducing Shell, retire TARS CLI label
- **naming:** Sweep remaining TARS CLI refs in module doc-comments
- **readme:** Bold the T-R-S initials so the acronym is visible

### Other

- Tars-cli → trs-cli (Cargo package + all rustdoc paths)

## [0.5.8] - 2026-04-21

### Features

- **upgrade:** Chain hook + output-saver refresh after binary upgrade
- TRS_SKIP bypass + version upgrade hint + upgrade pre-refresh guards
- Python traceback + ps aux parsers, npx routing, stats display fixes
- Curl body compression + gh api routing
- Agent attribution — TRS_AGENT tag + stats --by-agent breakdown

### Documentation

- **roadmap:** Flag extra_download.rs for future split
- **v0.5.8:** Test count refresh, agent attribution docs, session log

### Other

- Trs upgrade + post-install UX polish

## [0.5.7] - 2026-04-20

### Features

- **stats:** Explicit date range + today breakdown + last command + history hint
- Ping handler + swift build classifier route
- Xcodebuild routing + BUILD FAILED sentinel
- Brew install/upgrade handler
- **safety:** Multilingual error detection + fail-open + credential preservation
- Trs audit-docs — static analysis of agent instruction files
- **audit-docs:** Detect embedded code/queries/tables that belong elsewhere
- **audit-docs:** Resolve code-fence symbols against project source
- Surface audit-docs from doctor + rules-file suggestions
- **doctor:** Always surface audit-docs hint, not just on bloat
- **init:** Add output hygiene block to rules templates
- **init:** Pre-install collision check for competing compressors
- **init:** Follow @imports in Claude/Gemini rule files
- **init:** Explicitly recommend --replace in collision report
- Trs output-saver — install output-reduction rules into agent configs
- **output-saver:** Add OpenCode support via ~/.config/opencode/AGENTS.md
- **output-saver:** Add Kilo and Droid support via AGENTS.md globals

### Bug Fixes

- **audit-docs:** Reduce noise from heading-only dups + npm-package false positives
- **init:** Scrub competitor hooks from their actual file on --replace

### Refactor

- **init:** Scan both home and project paths regardless of --global

### Documentation

- Capture user feedback for trs stats header enhancements
- **roadmap:** Phase 2.5 — ideas from token-optimizer competitor analysis
- Update roadmap + session log + AGENTS.md for v0.5.7
- **v0.5.7:** Roadmap, session log, agent integrations matrix
- **roadmap:** Add VSCode ecosystem + deferred agent work to Phase 3
- **narrative:** Rewrite origin story — price pressure, rtk inspiration, broader scope
- **narrative:** Trs as own initiative, single mention of rtk in the paragraph
- **narrative:** Credit rtk's growth trajectory, not just "another team"
- **narrative:** Rtk as validation, not trigger — disambiguate publish motive
- **narrative:** Fork-in-the-road framing — continue vs migrate
- **pre-release:** Resolve contradictions + retract false claims
- **attribution:** Clarify Iteris product / dPeluChe publisher relationship

### Other

- Cargo fmt

## [0.5.6] - 2026-04-18

### Bug Fixes

- **install:** Default to XDG ~/.local/bin — zero-friction install (#13)
- **v0.5.6:** Ingest levels differentiation + git branch compression (#14)

### Other

- 9-agent integration audit + honest-compression polish (#15)

## [0.5.5] - 2026-04-17

### Features

- **ingest:** URL + owner/repo shorthands + spark integration (#12)

## [0.5.4] - 2026-04-16

### Features

- **init:** Add Factory Droid + polish error UX
- **init:** Smart JSON merge preserves existing settings.json config
- **init:** Antigravity/Windsurf rules, install detection, status+usage default
- **ingest:** Clean --output contract — no shadow save when path is given
- **ingest:** Stdout = path by default, --print for content (Unix-style)

### Bug Fixes

- **page:** Real SVG flow diagram + preserve digest code-block newlines

### Documentation

- **page:** Reframe landing — laboratory-style benchmarks, updated install
- **page:** Major landing rework — flow, origin, collapsible demos, digest block, install tabs
- **page:** Add SVG flow diagram to the digest section
- **page:** Duplicate install tabs near hero + CTA banner before compression
- **page:** Remove redundant curl box from hero (install tabs are next)
- **page:** Tighten hero — UX review pass
- **seo:** Open Graph + Twitter Card + JSON-LD + og-image.png

## [0.5.3] - 2026-04-16

### Bug Fixes

- **npm:** Shell wrapper searches multiple candidate paths for platform binary

## [0.5.2] - 2026-04-16

### Features

- **ingest:** Stale detection, --since-last, --fresh check, HEAD sha in --list
- **stats:** Show date range and tokens-per-day in summary
- **rewrite:** Generalize chain handling + stats period format
- **install:** Add universal install script (curl|sh) for macOS/Linux + PowerShell for Windows

### Bug Fixes

- Clippy lints from Rust 1.95 (CI uses newer toolchain)

### Performance

- **npm:** Replace Node wrapper with shell script — 2.8x faster startup

### Documentation

- **benchmarks:** Move to docs/benchmarks + add chain-rewrite bench + README

## [0.5.1] - 2026-04-16

### Features

- **init:** Add --all flag to install hooks for all supported tools

### Bug Fixes

- **rewrite:** Use hookSpecificOutput format for Claude Code PreToolUse hooks
- **init:** Add restart notice after hook install + detect settings.json hooks

### Documentation

- Add trs init --all to README examples

## [0.5.0] - 2026-04-16

### Features

- **ingest:** Add dependency graph — header summary + --deps flag
- **go-test:** Add go test parser with verbose and default mode support
- **npm:** Migrate to optionalDependencies pattern (biome-style)

### Bug Fixes

- **deps:** Resolve @/ alias imports (Next.js/Vite) in dep graph
- **deps:** Add Go module-path import resolution
- **deps:** Resolve absolute Python package imports (from pkg.sub import)
- **ingest:** Skip 'archives' dir + Python absolute import resolution
- Remove unused re-export of extract_raw_imports from deps.rs
- UTF-8 truncation safety, pytest -q mode, Google Antigravity support
- Remove unreachable tsc pattern and unused variable in go_test parser
- **npm:** Use @dpeluche scope for all packages
- Remove file with invalid path (space prefix) that breaks Windows checkout

### Refactor

- **ingest:** Split deps.rs and collect.rs into smaller modules

### Documentation

- Add --deps flag to README, help text, and AGENTS.md ingest module listing
- Add go test, antigravity to README and AGENTS.md
- Mark go test and antigravity as completed in roadmap
- Add April 2026 completed tasks log
- Clean TASK_TODO — remove completed items, keep only pending

### Other

- Split monolithic router.rs into modular handler architecture
- Split all files to max 500 lines for opensource quality
- Apply cargo fmt

<!-- generated by git-cliff -->
