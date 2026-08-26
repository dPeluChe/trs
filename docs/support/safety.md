# Safety guarantees

trs sits between the user / agent and the underlying tools, so the
worst failure mode we can have is corrupting output or breaking an
exit-code contract. These are the guarantees trs holds to keep
that from happening.

## Command hygiene

- **`--no-verify` blocked on `git commit` / `git push`.** Pre-commit
  and pre-push hooks are there for a reason, agents that default to
  bypassing them can ship broken code. trs refuses the bypass; users
  who explicitly want it can still invoke git directly (`TRS_SKIP=1`).
- **`--json` / `--porcelain` passthrough.** When the wrapped tool
  already has a structured mode, trs doesn't re-parse its output, 
  the structured form passes through untouched.
- **Exit codes always propagated.** If the wrapped command exits 1,
  trs exits 1. Scripts and CI relying on exit codes keep working.
- **Never silently change semantics.** No rewrites that would change
  what the tool does, only how its output is presented.

## Parser safety

- **Parser errors fall back to truncated passthrough.** If a parser
  errors mid-parse, trs emits the raw output (truncated to
  `passthrough_max_chars`) rather than silently losing content. The
  full raw output is always saved to `~/.trs/tee/` regardless.
- **`trs read` never strips content from data files.** JSON, YAML,
  TOML, XML, and CSV files are returned verbatim, stripping
  comments or "aggressive" signature extraction only applies to
  source-code files where the syntax supports it unambiguously.
- **Verbatim commands are handed back byte for byte.** `awk`, `cut`,
  `tr`, `sort`, `uniq`, `column`, `fold`, `nl`, `iconv`, `xxd`, `od`,
  `hexdump`, `base64`, `jq`, `yq`, `printf` and friends re-lay-out
  their input, so runs of spaces and blank lines are the payload, not
  noise. Generic compression collapses exactly those, which turned
  `awk 'NR<=4' x.py` into Python with every indent flattened to one
  space. trs neither rewrites these at the hook nor compresses them
  when called directly, at any output size. The same holds through a
  shell wrapper (`bash -c "cut -c1-20 x.py"`); a compound script
  (`cd x && awk …`) is the known gap, only its first command is seen.

## Failure recovery

- **`~/.trs/tee/` log dir.** Every command run stores its full stdout
  + stderr here, keyed by timestamp. Recovers content when a parser
  truncates or the agent loses the intermediate output.
- **`trs debug-info` bundles.** When reporting a bug or asking for
  help, `trs debug-info -o /tmp/trs.txt` packages version, platform,
  doctor checks, recent history, and the three most recent tee logs
  into a single paste-ready report.

## Install-time safety

- **Collision detection in `trs init`.** Before writing any hook
  config, trs scans for hooks from other token-compression tools
  (following `@imports` too) and aborts by default. Running two
  compressors on the same command risks garbled output. `--replace`
  cleanly scrubs the competitor's hook; `--force` installs alongside
  (not recommended).
- **Pre-upgrade validation.** `trs upgrade` runs three guards before
  touching any config file: spawn sanity (new binary executes),
  version bump (not a silent no-op), and JSON validity on hook
  configs that would be edited. See [`docs/features/upgrade.md`](../features/upgrade.md).

## See also

- [`docs/features/doctor.md`](../features/doctor.md): health check
  that surfaces any config drift or unexpected state.
- [`docs/features/upgrade.md`](../features/upgrade.md): upgrade
  pre-flight validations.
- [`docs/support/other-token-savers.md`](./other-token-savers.md), 
  migration notes from other tools in the space.
