---
name: Strict clippy before push
description: Always run cargo clippy -D warnings + cargo fmt before git push in this repo
type: feedback
---

Always run all three before any `git push` in labs-tarscli, in this order:
1. `cargo fmt -- --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`

**Why:** CI enforces both fmt and clippy -D warnings. Skipping fmt caused a red CI on v0.5.10 (#24 hotfix). Clippy alone is not enough.

**How to apply:** After any code change, run clippy + fmt before committing or pushing. If clippy gives warnings, fix them — don't use `#[allow(...)]` unless the warning is genuinely a false positive.
