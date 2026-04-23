---
name: Strict clippy before push
description: Always run cargo clippy -D warnings + cargo fmt before git push in this repo
type: feedback
---

Always run `cargo clippy -- -D warnings` and `cargo fmt -- --check` before any `git push` in labs-tarscli.

**Why:** CI enforces `-D warnings`; pushing without running clippy locally causes red CI and wasted cycles.

**How to apply:** After any code change, run clippy + fmt before committing or pushing. If clippy gives warnings, fix them — don't use `#[allow(...)]` unless the warning is genuinely a false positive.
