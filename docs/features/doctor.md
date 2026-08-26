# `trs doctor`: installation health check

`trs doctor` runs 10 checks that cover every surface where a trs
install can go wrong: the binary, the PATH, dependencies, the config
dir, the history file, the hook pipeline, agent integrations, and
agent-doc budgets. Use it after a fresh install, when an agent hook
suddenly stops firing, or when debugging CI.

## Quick reference

```bash
trs doctor          # human-readable report
trs doctor --json   # machine-readable; exits non-zero on any Fail
```

## What each check does

| Check | What it verifies |
|---|---|
| `trs binary` | Reports version + binary path (never fails, always informational). |
| `trs in PATH` | `which -a trs` / `where trs`. Warns if multiple binaries exist so a shadowed install doesn't silently win. |
| `git available` | `git --version`. Required, several parsers assume git is present. |
| `ripgrep available` | `rg --version`. Not required but needed for `trs search`. |
| `config directory` | `~/.trs/` exists and is writable. Creates it if missing. |
| `history writable` | `~/.trs/history.jsonl` can be written. Reports current size. |
| `stdin pipeline functional` | Pipes `"doctor probe\n"` through `trs clean` and confirms roundtrip. Catches PATH vs executable-bit issues. |
| `AI tool hooks` | Count of detected agents with trs hooks installed (see [`trs init`](init.md)). |
| `output saver` | Count of agents with the output-saver block installed (see [`trs output-saver`](output-saver.md)). Warns if zero. |
| `agent docs` | Scans CLAUDE.md / AGENTS.md / rules files in cwd and reports total token budget. Warns if any single file exceeds ~5k tokens. |

## Reading the report

```
TRS Doctor: Installation Health Check

  ✓ trs binary
    version: 0.5.8
    path:    /Users/you/.local/bin/trs
  ✓ trs in PATH
    path: /Users/you/.local/bin/trs
  ✓ git available
    version: git version 2.52.0
  ✓ ripgrep available
    version: ripgrep 15.1.0
  ✓ config directory
    /Users/you/.trs
  ✓ history writable
    size: 586.5K tracked
  ✓ stdin pipeline functional
  ✓ AI tool hooks (9/9 configured)
  ~ output-saver not installed  → `trs output-saver --install` adds anti-preamble rules to agent configs
  ✓ 2 files, 1.8k tokens loaded per agent session  → run `trs audit-docs` to review duplicates / dead refs / embedded bloat

  ───────────────────────────────────
  9 passed   0 failed   1 warnings
```

Markers:

- `✓` pass: the check succeeded.
- `~` warn: something worth attention, but not blocking. Reported
  with a hint about how to act.
- `✗` fail: something broken. Exits non-zero in JSON mode so CI
  catches it.

## JSON mode

```bash
trs doctor --json
```

Returns a structured report suitable for piping into other tools.
Exits with status 1 if any check failed. Useful in provisioning
scripts:

```bash
if ! trs doctor --json > /dev/null; then
  echo "trs install is broken"
  exit 1
fi
```

## Typical fixes

- **`trs not found in PATH`**: The binary isn't reachable from this
  shell. Re-run the installer, or add `~/.local/bin` to `PATH`
  explicitly. See [`trs upgrade`](upgrade.md) for a one-command
  refresh.
- **Multiple `trs` binaries in PATH**: You have both the curl install
  and an npm or cargo install on the box. Uninstall the duplicates
  so the first one wins deterministically.
- **`stdin pipeline failed`**: The binary's executable bit was
  stripped (sometimes happens after file-sync tools). Re-install.
- **`no AI tool hooks installed`**: Run `trs init --all --global`.
- **`output-saver not installed`**: Run `trs output-saver --install`
  (check-first, it won't write without confirmation).
- **`agent docs oversized`**: Run `trs audit-docs` to find the
  biggest offenders.

## See also

- [`trs init`](init.md): install the input-side hooks that `doctor`
  reports on.
- [`trs output-saver`](output-saver.md): install the output-side
  rules block that `doctor` reports on.
- [`trs audit-docs`](audit-docs.md): the tool that fixes the
  oversized-docs warning.
- [`trs upgrade`](upgrade.md): re-run the install pipeline.
