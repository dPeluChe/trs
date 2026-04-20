# `trs upgrade` — re-run the install pipeline for the latest release

`trs upgrade` detects how trs was installed on your machine and runs
the matching install command so you don't have to remember which
channel you used.

Added in v0.5.8.

## Quick reference

```bash
trs upgrade           # detect + confirm + run
trs upgrade -y        # skip confirmation (useful for scripts / cron)
trs upgrade --check   # show the detection result and planned command, don't run
```

## Detection logic

`trs upgrade` reads the running binary's path from
`std::env::current_exe()` and matches it against known install
locations:

| Path pattern | Channel | Upgrade command |
|---|---|---|
| `**/node_modules/@dpeluche/trs/**` | npm | `npm install -g @dpeluche/trs@latest` |
| `$HOME/.local/bin/trs`, `$HOME/.trs/bin/trs`, `$HOME/bin/trs` | curl\|sh | `curl -fsSL https://raw.githubusercontent.com/dPeluChe/trs/main/scripts/install.sh \| sh` |
| `$CARGO_HOME/bin/trs` or `$HOME/.cargo/bin/trs` | cargo | **not auto-upgradable yet** — run `cargo install tars-cli --force` |
| `/opt/homebrew/**`, `/usr/local/Cellar/**` | Homebrew | **not auto-upgradable yet** — tap not published |
| anything else | unknown | prints all manual options |

Supported channels run the actual upgrade shell command for you.
Unsupported channels print the manual command you need to run —
better than silently doing nothing.

## Why detection is path-based

It's the one signal every channel leaves consistently: each channel
writes the binary to a well-known location, and the kernel tells us
which path we're executing from. We don't need to parse package
managers, inspect lock files, or rely on environment variables.

The trade-off: if you move the binary or symlink it somewhere weird,
detection fails. In that case `trs upgrade` falls back to listing
every install option and lets you pick.

## Confirmation prompt

By default, `trs upgrade` asks before running anything destructive:

```
trs upgrade

  current binary: /Users/you/.local/bin/trs
  current version: 0.5.7
  detected method: curl|sh installer

Will run:
  curl -fsSL https://raw.githubusercontent.com/dPeluChe/trs/main/scripts/install.sh | sh

Proceed? [y/N]
```

Pass `-y` / `--yes` to skip the prompt. Pass `--check` to see what
would run without executing.

## Roadmap for unsupported channels

- **crates.io publish** — currently trs ships to npm and GitHub
  Releases only. Once `cargo install tars-cli` installs from the
  registry, upgrade will support cargo.
- **Homebrew tap** — low priority (npm + curl covers most users),
  tracked in [`docs/TASK_TODO.md`](../TASK_TODO.md).

Both are listed in the roadmap under Phase 1 — Release & Distribution.

## What happens after a successful upgrade

The shell command runs to completion, which typically:

- npm: downloads the new package + platform binary, overwrites the
  shim.
- curl|sh: fetches the latest release archive, verifies it,
  overwrites the binary in place.

After either, you may need to **restart open shells** for the new
binary to become active (the old binary stays loaded in the current
process's memory). New shells will see the updated version.

Verify:

```bash
trs --version
```

## Interaction with hooks

Upgrading the binary doesn't change your hook configurations — those
live in `~/.claude/`, `~/.gemini/`, etc., and point at the binary by
name (`trs rewrite`). The hook calls `trs` in PATH, which resolves
to the just-upgraded binary on the next shell invocation.

If a release ships improvements to the hook templates themselves
(not just the binary), re-run `trs init --all --global --force` to
refresh them. See [`trs init`](init.md#refreshing-hooks).

## See also

- [`trs doctor`](doctor.md) — confirms the upgrade worked and the
  new binary is healthy.
- [`trs init`](init.md) — once upgraded, may want `--force` to
  refresh hook templates if a new release changed them.
- [Install instructions in the README](../../README.md#install) —
  authoritative list of supported install channels.
