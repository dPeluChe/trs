# Competitor GitHub issue sweep, 2026-11-21

What competing projects' public issue trackers reveal about fidelity bugs,
feature requests and operational pain that trs can learn from. Sources:
`gh issue list` on each repo (open + closed since ~2026-09-22), cross-checked
against trs source. Issue numbers refer to the named repo.

Repos covered: `rtk-ai/rtk` (~87k stars, ~1,700 open issues — the dominant
signal), `alexgreensh/token-optimizer`, `AgusRdz/chop` (0 open issues),
`yamadashy/repomix`, `headroomlabs-ai/headroom`, `open-compress/claw-compactor`
(2 issues, nothing actionable), `DeusData/codebase-memory-mcp`,
`earendil-works/pi` (agent, not a compressor — ecosystem signal only).

## Confirmed gaps in trs (verified against source)

### Fidelity bugs trs shares

1. **`git status` footer match drops real files.** `git_status.rs:242` skips
   any line *containing* "nothing to commit" / "working tree clean". A file
   named `nothing to commit, working tree clean` (porcelain `??` entry or
   tab-indented long entry) is silently dropped. Same class as rtk #5139,
   which drops every entry after such a file. Fix: match footer lines by
   position (last line / outside file-entry context), not `contains()`.

2. **`git diff` strips `\r`.** `str::lines()` removes `\r\n`, so a
   line-ending-only change renders identical `-`/`+` lines with no signal.
   Same as rtk #5084. Minor: detect `\r` in raw input and mark the file
   (e.g. `␍` suffix or a "line endings changed" note).

3. **Test parsers don't surface flaky/retried counts.** No `flaky` handling
   in `jest_parse.rs`, `vitest_parse.rs`, `pytest_parse.rs`. rtk shipped this
   bug three times (#5142 playwright `PASS (0) FAIL (0)`, #3588 jest,
   #3935 vitest, #4694/#4420 cargo nextest dropping `N failed`/`N skipped`).
   When the playwright parser lands (TASK_TODO Phase 2), flaky must be a
   first-class count. Vitest/jest print `flaky` in the summary line;
   nextest prints `FLAKY`.

4. **Plugin PATH resolution.** `init_templates_plugins.rs` spawns bare
   `"trs"`; GUI-launched agents (OpenCode Desktop from Finder) may lack
   `~/.local/bin` on PATH → silent no-rewrite. Same as rtk #4462
   (`resolveRtkPath` missed PATH-cased dirs on macOS GUI). Fix: resolve trs
   against common install locations before spawn, or extend `trs doctor`
   (already warns "hook installed, never fired") to name the cause.

5. **`nix develop -c` / `timeout N` hide inner program from permission
   gate.** Already in TASK_TODO; rtk #5157/#5062 confirm the class is active
   across tools: deny rules written against `rm -rf` don't see
   `nix develop -c rm -rf d` or `timeout 5 rm -rf d`, and the rewritten
   `trs …` form matches no rule either. `strip_wrappers` covers
   `uv run --with` but not these.

6. **Hermes plugin mutation mechanism unverified.** rtk #3797: Hermes
   `pre_tool_call` ignores in-place `args` mutation → rewrites never apply.
   trs's Hermes integration is pending live validation; verify the mutation
   surface actually works before claiming support.

### Measured-data additions to existing TODOs

7. **Token estimator (bytes/4).** Already a TASK_TODO item. rtk #5135 adds
   real numbers: on 655 proxied Claude Code requests, bytes/4 estimate ran
   0.42–0.64× of billed input tokens (median 0.50); per-output ratios 1.7–4.1.
   bytes/4 *understates* absolute tokens ~2× on current Claude models.
   Percentages are safer than absolute counts, but `trs stats`/`gain`-style
   claims should carry the "est." label the TODO already plans.

8. **Output-token side effect.** rtk #582 (11 comments): compressed tool
   output made the model write +50% output tokens → +18% total cost. trs
   measures input savings and rerun rate, but nothing measures whether
   compressed input makes agents *talk more*. `transcript_output.py` already
   measures output tokens for the output-saver block; the same harness could
   A/B rewrite on/off. Supports keeping savings claims scoped to shell output.

9. **Homebrew tap.** Already low-priority TODO. rtk #5194 shows the carrying
   cost: `conflicts_with` fights other taps, untrusted-tap formula loading.
   Validates keeping it deprioritized or skipping.

## Checked: trs already ahead or equivalent

- **Rewrite conservatism.** trs refuses `$(...)`, pipes, redirects, `;`,
  loops, `case`, heredocs, multiline (`rewrite_decide.rs`). rtk's issue
  stream is dominated by corruption in exactly these shapes: #3197 (`;;` →
  `; ;` syntax error, 12 comments), #5090 (backslash-newline collapsed inside
  quotes/comments), #5110 (`$(...)`/loop bodies silently unrewritten),
  #1282 (17 comments: truncation when stdout isn't a TTY corrupts pipes).
  trs's "lose the savings, keep the equivalence" posture is the right call
  and is the strongest differentiator to document.
- **Permission rules.** trs applies deny/ask/allow to the *original* command
  (`rewrite_permissions.rs`, `rewrite_gate.rs`) with wrapper stripping —
  rtk #5157 is the same bug trs fixed by design. rtk #582's unconditional
  `allow` is what trs's per-agent `Gate` avoids.
- **`git show rev:path` blob form.** trs detects it and passes through raw
  (`classifier.rs:90-99`); rtk #5070 truncated blobs to ~8KB breaking
  scripts. trs also handles `git -C`, `git grep`, `git ls-files`, `npm test`,
  `python -m pytest`, `bash -c` single commands, `timeout`, runner unwrap
  (`uv run`, `npx`, `nix develop`, `mise`, `direnv`).
- **Release targets.** gnu/musl-free matrix (gnu + apple + msvc) avoids rtk's
  musl SIGILL-on-no-AVX bugs (#5043, #5115).
- **`trs rewrite` argv.** rtk #5127 (`rtk rewrite --help` eaten by clap) is
  N/A: trs reads the command from stdin, never argv.
- **Token counting.** Both use bytes/4; rtk is building a proxy-counted exact
  path (#5135). trs's TODO already covers labeling estimates; a proxy is in
  the do-not-copy list.
- **Drift gates.** headroom #4833 (compression-ratio drift gate) and #4834
  (benchmark suite) are features trs already has:
  `tests/cli_compression_ratchet.rs` + `benchmark.sh` + `quality_harness.rs`.
- **Lossless-fold caution.** headroom's fold bugs (#4900 timestamps split
  into fake grep headers, #4874 JSON-looking strings converted to values,
  #4796 CCR marker reuse after eviction) validate trs's `safe_folds` +
  `anchor_guard` conservatism.
- **`git branch` remote dedup.** trs splits local/remote and dedupes shadowed
  names (`git_branch.rs`); rtk #5184 listed remote branches plain git
  doesn't.

## Evaluated: current trs behavior is correct, no change

- **`rg -l` / `grep -l` / `--files-with-matches`.** Bare-path output fails
  `parse_grep_line` → `has_unread` → raw passthrough. Zero savings but the
  path list stays byte-usable for the next command — which is the payload.
  rtk #5197 shows the failure mode of "compressing" it (grouped by
  directory → unusable paths). Leave as is; revisit only if a count-only
  variant (`--count`) proves useful — that one *can* compress safely.
- **`git merge-base` / `rev-parse` / `describe`.** rtk #5200 requests them;
  outputs are tiny, generic fallback suffices.
- **pytest "no tests collected".** trs renders `no tests found` via
  `is_empty`; a passing run always has `summary.total > 0`. rtk #5016's
  false-negative was a different defect (passed run reported as collected-
  nothing). Worth one fixture test pinning "collected 0 items" → exit-code
  preserved, but not a parser bug.

## repomix → `trs ingest` (existing TODOs confirmed + new)

Existing TODOs (`--grep`, secret scanning, `--audit-loss`) map to repomix's
MCP `grep_repomix_output`/`read_repomix_output` tools and secret scanning.
New signals from open issues:

- **Legacy encodings dropped as binary** (#1878, #1966): Shift-JIS/EUC-KR/
  GBK files never reach the encoding detector. Check whether `trs ingest`
  collect skips non-UTF-8 files silently — same failure mode.
- **Entity-level packing** (#1642, functions/classes granularity) and
  **graph-aware selection** (#1682, file + dependency cone): both need AST/
  symbol graphs beyond trs's import-graph scope. Consistent with the
  deferred symbol-level dead-code decision; keep deferred.
- **Ignore-file fidelity** (#1873 `.git/info/exclude` negations inert,
  #1870 patterns only top-level, #1872 ancestor `.ignore` not applied to
  subdirectory targets): audit `ingest/collect*` ignore handling against
  git's actual semantics — cheap correctness win if gaps exist.
- **MCP sandbox mode** (#1754), **SSE transport** (#582): relevant only if
  the deferred trs-MCP question resolves yes.

## headroom → mostly validation, two ecosystem notes

- Confirms trs's measurement rigor is ahead: their open bugs are exactly the
  fidelity failures trs's ratchet/anchor-guard/quality-harness exist to
  prevent.
- #4663 "Support rtk proxy": headroom integrates rtk's proxy — ecosystem
  evidence that proxy-layer compression is where the industry is heading;
  trs's do-not-copy stance stays, but worth noting in positioning docs.
- #4712 per-project memory keyed on git repo: different layer (semantic
  memory), not shell output. No action.

## pi / token-optimizer / codebase-memory-mcp → ecosystem signals

- pi #12165: `searchTools()` returns 36KB of tool descriptions per call
  (~25% of a long session's context); #11386 configurable MCP output limit;
  #11529 tool order busts prompt cache. All agent-side context costs trs
  can't reach — supports the "trs covers shell output only" scoping.
- token-optimizer: PostToolUse bash compression archives the original behind
  an expand pointer — same recovery model as trs `tee`/`ptr`. Their open
  issue queue is almost entirely dashboard/context-tracking (out of scope).
  #313: a no-op synchronous PreToolUse hook still adds latency — trs rewrite
  at ~4ms is fine; keep the plugin spawn timeout generous (3s) but never add
  per-call work without a cheap gate (cbm #3005 lesson: hook-augment spawned
  per tool call even with an empty index).

## Suggested TASK_TODO additions

See `docs/roadmap/TASK_TODO.md` § "Phase 2.10: Competitor issue sweep".
Priority order: (1) git-status footer `contains()` bug, (2) flaky counts in
test parsers + playwright-when-written, (3) plugin PATH fallback, (4) git
diff `\r` marker, (5) `nix/timeout` deny-rule gap (already tracked), (6)
ingest ignore/encoding audit, (7) output-token A/B for rewrite.
