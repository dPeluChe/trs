# `trs upgrade` — re-run the install pipeline for the latest release

`trs upgrade` detects how trs was installed on your machine and runs
the matching install command so you don't have to remember which
channel you used.

Added in v0.5.8.

## Quick reference

```bash
trs upgrade                # detect + confirm + binary + refresh configs
trs upgrade -y             # skip confirmation (useful for scripts / cron)
trs upgrade --check        # dry-run: show detection + planned commands
trs upgrade --binary-only  # upgrade only the binary, skip config refresh
```

## What gets upgraded

By default `trs upgrade` runs three steps in order:

1. **Binary** — the shell install command for your detected channel
   (curl|sh or npm). See the detection table below.
2. **Hooks** — `trs init --all --global --force` refreshes every
   already-configured agent with the latest hook templates. Existing
   user-added hooks on the same event are preserved (the JSON merge
   only replaces trs's own entries).
3. **Output-saver** — `trs output-saver --refresh` re-installs the
   rules block **only** where it's already present. Agents that
   never had output-saver installed are left untouched.

Pass `--binary-only` to skip steps 2 and 3 — useful when you want to
upgrade the binary without re-touching any config files (e.g. you
have manual edits that shouldn't be overwritten).

The refresh steps run by spawning the **new** `trs` binary from PATH,
so they pick up whatever template changes shipped with the upgrade.

### Pre-refresh validations

Before `trs upgrade` touches any config file, it runs three guards:

1. **Spawn sanity** — invokes the new `trs --version` and confirms it
   executes cleanly. A corrupt binary aborts here instead of going on
   to write configs through a broken tool.
2. **Version bump** — confirms the new binary reports a version
   greater than the one that was running. Catches silent no-ops
   (npm shim pointing at an old cached package, curl install
   restoring same version). When they match, we skip the config
   refresh and tell the user — use `--binary-only` if this is
   intentional.
3. **JSON validity** — parses every hook config `init` would touch
   (`~/.claude/settings.json`, `~/.gemini/settings.json`, etc.). If
   any is corrupt, aborts with the exact file path so the user can
   fix it manually rather than have our merge layer compound the
   damage.

All three failures surface as explicit messages; the binary upgrade
itself has already happened and the two refresh commands are
idempotent, so the user can always re-run them by hand afterwards.

## Detection logic

`trs upgrade` reads the running binary's path from
`std::env::current_exe()` and matches it against known install
locations:

| Path pattern | Channel | Upgrade command |
|---|---|---|
| `**/node_modules/@dpeluche/trs/**` | npm | `npm install -g @dpeluche/trs@latest` |
| `$HOME/.local/bin/trs`, `$HOME/.trs/bin/trs`, `$HOME/bin/trs` | curl\|sh | `curl -fsSL https://usetrs.dev/install.sh \| sh` |
| `$CARGO_HOME/bin/trs` or `$HOME/.cargo/bin/trs` | cargo | **not auto-upgradable yet** — run `cargo install trs-cli --force` |
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
  curl -fsSL https://usetrs.dev/install.sh | sh

Proceed? [y/N]
```

Pass `-y` / `--yes` to skip the prompt. Pass `--check` to see what
would run without executing.

## Roadmap for unsupported channels

- **crates.io publish** — currently trs ships to npm and GitHub
  Releases only. Once `cargo install trs-cli` installs from the
  registry, upgrade will support cargo.
- **Homebrew tap** — low priority (npm + curl covers most users),
  tracked in [`docs/roadmap/TASK_TODO.md`](../roadmap/TASK_TODO.md).

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
