# CLAUDE.md

Agent instructions for this repo live in [`AGENTS.md`](./AGENTS.md). Start
there. This file carries only the machine-readable config the hooks and the
ship gate read.

Personal agent setup (skills, flow imports) belongs in your own global config,
not here: this file is committed and shared by every contributor.

## ship config

```yaml
# Mirrors .github/workflows/ci.yml so a local gate fails whatever CI would.
# --all-targets is a superset of CI's plain clippy: it also lints tests.
lint: cargo fmt -- --check && cargo clippy --all-targets -- -D warnings
# No separate typecheck: clippy type-checks the crate, so a `cargo check`
# step would just repeat the same work.
build: cargo build
# --no-fail-fast so one failing suite doesn't mask the rest (matters most
# on the blocking windows-latest job).
test: cargo test --no-fail-fast
# pre-push deliberately not delegated: it already mirrors CI and is stricter.
hooks_skip: pre-push: "already mirrors CI (fmt + clippy + full suite), stricter than the ship-config gate"
merge_policy: ask   # auto | ask
loc_limit: 500
simplify: 500       # run /simplify only if changed LOC > N (off = only on request)
```

## Git hooks

`core.hooksPath` points at the tracked `.githooks/`, so lefthook's own stubs
in `.git/hooks/` are never invoked. `.githooks/pre-commit` and
`.githooks/commit-msg` delegate to lefthook explicitly; both no-op when
lefthook isn't installed, so nobody is blocked by an optional tool.

`.githooks/pre-push` deliberately does NOT delegate: it already mirrors CI
(`fmt` + `clippy` + the full test suite), which is stricter than the
ship-config gate, and delegating would run clippy twice.

## Release

Tagging `vX.Y.Z` triggers `.github/workflows/release.yml` (5 binaries + npm
publish + GitHub Release). Bump `Cargo.toml`, regenerate `CHANGELOG.md` with
`git cliff -o CHANGELOG.md --tag vX.Y.Z`, merge, then tag. A tagged run uses
the workflow at the tag's commit, so fixing a failed release means re-tagging,
not re-running.
