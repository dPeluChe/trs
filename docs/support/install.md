# Installing trs

Five install channels. All of them ship the same native binary
(~6 MB, zero runtime deps, ~12 ms startup). Pick whichever fits your
existing toolchain.

## Quick list

| Channel | One-liner |
|---|---|
| curl / sh | `curl -fsSL https://usetrs.dev/install.sh \| sh` |
| PowerShell | `irm https://usetrs.dev/install.ps1 \| iex` |
| npm | `npm install -g @dpeluche/trs` |
| cargo | `cargo install trs-cli` |
| Prebuilt binary | [GitHub Releases](https://github.com/dPeluChe/trs/releases) |

The `curl|sh` and `irm|iex` scripts are the recommended default: they
detect arch, download the right prebuilt binary, place it in
`~/.local/bin/` (or `$USERPROFILE\.local\bin\` on Windows), and add
that dir to PATH automatically if it isn't already.

## Platform support

| OS | Arch | How it's shipped |
|---|---|---|
| macOS | arm64 (Apple Silicon) | prebuilt binary + npm + cargo |
| macOS | x64 (Intel) | prebuilt binary + npm + cargo |
| Linux | x64 | prebuilt binary + npm + cargo |
| Linux | arm64 | prebuilt binary + npm + cargo |
| Windows | x64 | prebuilt binary + npm + cargo |

## Prebuilt binaries — manual install

If you prefer to pick the file yourself:

1. Go to <https://github.com/dPeluChe/trs/releases/latest>.
2. Download the asset matching your platform — `trs-darwin-arm64`,
   `trs-darwin-x64`, `trs-linux-arm64`, `trs-linux-x64`, or
   `trs-windows-x64.exe`.
3. Make it executable (`chmod +x trs-*` on Unix) and place it
   somewhere in your `PATH`. `~/.local/bin` is the convention.

Verify the install with:

```bash
trs --version
trs doctor          # full health check
```

## Pinning a specific version

Set `TRS_VERSION` before running the installer to pin rather than
taking the latest release:

```bash
TRS_VERSION=v0.5.9 curl -fsSL https://usetrs.dev/install.sh | sh
```

For Windows:

```powershell
$env:TRS_VERSION = 'v0.5.9'
irm https://usetrs.dev/install.ps1 | iex
```

For npm, use the standard tag syntax:

```bash
npm install -g @dpeluche/trs@0.5.9
```

For cargo, use the `--version` flag:

```bash
cargo install trs-cli --version 0.5.9
```

## Custom install directory

The `curl|sh` script picks `~/.local/bin` by default (XDG convention,
already in `PATH` on most modern shells). Override with
`TRS_INSTALL_DIR`:

```bash
TRS_INSTALL_DIR=/opt/trs curl -fsSL https://usetrs.dev/install.sh | sh
```

On Windows:

```powershell
$env:TRS_INSTALL_DIR = 'C:\tools\trs'
irm https://usetrs.dev/install.ps1 | iex
```

If the install dir isn't already in `PATH`, the installer adds it
automatically. Set `TRS_NO_MODIFY_PATH=1` to skip the PATH edit (you
take responsibility for adding it manually).

## Upgrading

```bash
trs upgrade --check    # show what would run (detects install channel)
trs upgrade            # perform the upgrade
trs upgrade --binary-only   # skip the hooks / output-saver refresh
```

`trs upgrade` auto-detects which channel installed the running binary
(via `current_exe()` path matching) and re-runs the matching install
command. See [`docs/features/upgrade.md`](../features/upgrade.md) for
the detection table, pre-refresh guards, and caveats for
cargo / Homebrew installs (not auto-upgradable yet).

## Shadowed installs (multi-channel)

If you install through two channels (npm *and* curl for example), the
one that appears first in `PATH` wins silently. `trs doctor` flags
duplicates explicitly:

```
WARN PATH  multiple trs binaries found
           active:    /Users/you/.local/bin/trs
           shadowed:  /Users/you/.nvm/.../node_modules/.bin/trs
           2 trs binaries in PATH — the first one wins
```

Fix by uninstalling the duplicate (`npm uninstall -g @dpeluche/trs`
or similar) or reordering `PATH`.

## Uninstall

| Channel | Command |
|---|---|
| curl / sh | `rm ~/.local/bin/trs` (or whichever `TRS_INSTALL_DIR` you used) |
| PowerShell | `Remove-Item $env:USERPROFILE\.local\bin\trs.exe` |
| npm | `npm uninstall -g @dpeluche/trs` |
| cargo | `cargo uninstall trs-cli` |

Remove hook integrations before (or after) uninstalling the binary:

```bash
trs init --remove --all         # before: cleans hooks using the current binary
trs output-saver --remove       # removes the rules block from agent configs
```

These can also be done manually — the integration files live under
`~/.claude/`, `~/.gemini/`, `~/.cursor/`, etc. Grep for `trs` in
`settings.json` / `hooks.json` to find any leftovers.

## Troubleshooting

- **`trs: command not found` after install** — re-source your shell
  profile (`source ~/.zshrc` / `source ~/.bashrc`) or open a new
  terminal. The installer adds the install dir to the profile but
  the current shell doesn't pick it up until it restarts.
- **Permission denied writing to `~/.local/bin`** — the dir doesn't
  exist or isn't writable. `mkdir -p ~/.local/bin` and retry.
- **Windows PATH not updating** — check `Get-Command trs` and confirm
  the install dir appears in `$env:Path`. The installer uses
  `SetEnvironmentVariable('Path', …, 'User')` which persists for new
  shells but leaves the current process's env untouched.
- **Still stuck?** Run `trs debug-info -o /tmp/trs.txt` and paste the
  file contents into a GitHub issue — the report bundles version,
  platform, doctor checks, and recent logs.
