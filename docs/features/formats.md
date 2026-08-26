# Output formats

Every trs command supports six output formats. Pick the one that
matches the consumer: humans / agents read compact, scripts read
json/csv/tsv, pipelines sometimes want raw passthrough.

| Flag | Name | Who it's for |
|---|---|---|
| *(default)* | compact | humans + agents, terse single-pass form |
| `--json` | JSON | scripts, dashboards, anything structured |
| `--csv` | CSV | spreadsheets, basic data import |
| `--tsv` | TSV | tab-friendly tooling (`cut -f`, spreadsheets) |
| `--agent` | agent-optimized markdown | LLMs specifically, same compact form with marker syntax for section parsing |
| `--raw` | raw passthrough | unchanged, no compression, still tracked in stats |

Flags work anywhere in the invocation: `trs --json git status` and
`trs git status --json` are equivalent.

## Examples

### Compact (default)

```
$ trs git status
main [ahead 1]
unstaged (3):
  M src/main.rs
  M src/lib.rs
  A src/new.rs
```

### JSON

```
$ trs git status --json
{
  "branch": "main",
  "ahead": 1,
  "behind": 0,
  "unstaged": [
    { "path": "src/main.rs", "status": "M" },
    { "path": "src/lib.rs",  "status": "M" },
    { "path": "src/new.rs",  "status": "A" }
  ]
}
```

### CSV

```
$ trs git status --csv
status,path
M,src/main.rs
M,src/lib.rs
A,src/new.rs
```

### TSV

Identical to CSV but with tabs as separators. Preferred when the
data itself might contain commas.

### Agent

```
$ trs git status --agent
### git status
- branch: main
- ahead: 1
- unstaged:
  - M src/main.rs
  - M src/lib.rs
  - A src/new.rs
```

The agent format is closest to compact but uses explicit Markdown
headings and bullet syntax. Useful when the agent is parsing the
output to extract specific fields via pattern matching.

### Raw

```
$ trs git status --raw
On branch main
Your branch is ahead of 'origin/main' by 1 commit.
  (use "git push" to publish your local commits)

Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
        modified:   src/main.rs
        modified:   src/lib.rs

Untracked files:
  (use "git add <file>..." to include in what will be committed)
        src/new.rs

no changes added to commit (use "git add" and "git commit -a" to commit them)
```

Raw bypasses the parser entirely and emits the tool's native output.
Still gets tracked in `trs stats` with the raw size as input
(for anomaly detection in the dashboard).

## When to use what

- **Default (compact)**: Claude / Gemini / Cursor agents, humans
  eyeballing a terminal. Best token-efficiency per line of signal.
- **JSON**: scripts, CI jobs, dashboards. `trs stats --json | jq`
  and similar pipelines.
- **CSV / TSV**: spreadsheets, BI tools, anything that expects
  tabular import. Rare in agent workflows.
- **Agent**: agents that are doing *further parsing* of the output
  (extracting fields into their own state). The marker syntax makes
  regex / pattern-matching simpler than the compact form.
- **Raw**: debugging ("did the parser drop something?"), piping
  into another tool that expects the canonical format.

## Built-in tools vs wrapped commands

Built-in trs commands (`trs json`, `trs search`, `trs stats`, etc.)
produce the same six formats via the same flags.

Wrapped commands (`trs git status`, `trs cargo test`, etc.) produce
the six formats from the parsed structured output. For commands that
fall to the **generic compression** path (no dedicated parser), only
compact and raw are meaningful, json/csv/tsv are skipped with a
warning since there's no structured data to serialize.

## See also

- [`docs/support/commands.md`](../support/commands.md): which
  commands have dedicated parsers (and thus full format support) vs
  generic fallback.
- [`docs/features/stats.md`](./stats.md), `trs stats --json` format
  example with the full schema.
