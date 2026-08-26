## What

<!-- One or two lines: what does this change and why. -->

## Checklist

- [ ] `cargo fmt` + `cargo clippy --all-targets -- -D warnings` clean (CI gates on this)
- [ ] `cargo test` green; new behavior has tests (parsers: basic / edge / empty, see CONTRIBUTING.md)
- [ ] Files stay under ~500 LOC
- [ ] Conventional Commit title (`feat(scope): …` / `fix(scope): …`), the changelog is generated from it
