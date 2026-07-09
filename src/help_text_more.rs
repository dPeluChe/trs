//! Help text for exec / inspection commands (run/read/json/err/stats/
//! doctor/benchmark/diff/ingest). Split out of help.rs to keep it lean.

/// Help text for the run command.
#[allow(dead_code)]
pub const RUN_HELP: &str = "\
Execute a command and process its output.

The run command executes a system command and processes its output through
trs reducers for cleaner, more structured output.

USAGE:
    trs run <COMMAND> [ARGS]...

ARGUMENTS:
    <COMMAND>    The command to execute
    [ARGS]...    Arguments to pass to the command

EXAMPLES:
    # Run a command with compact output
    trs run ls -la

    # Run with JSON output
    trs --json run git status

    # Run npm test with structured output
    trs --json run npm test";

/// Help text for the read command.
#[allow(dead_code)]
pub const READ_HELP: &str = "\
Read a file with optional filtering to reduce token consumption.

FILTER LEVELS:
    none        Raw content (default)
    minimal     Strip comments, normalize blank lines
    aggressive  Signatures only — imports, function/class definitions

Data files (JSON, YAML, TOML, XML) are always passed through unmodified.

USAGE:
    trs read <FILE> [OPTIONS]

OPTIONS:
    -l, --level <LEVEL>    Filter level: none, minimal, aggressive
    -n, --lines <NUM>      Max lines from start
    -t, --tail <NUM>       Last N lines from end
    -N, --line-numbers     Show line numbers

EXAMPLES:
    trs read src/main.rs                      # Raw content
    trs read src/main.rs -l minimal           # Strip comments
    trs read src/main.rs -l aggressive        # Signatures only
    trs read src/main.rs -l aggressive -N     # Signatures + line numbers
    trs read src/main.rs --tail 50            # Last 50 lines
    trs --json read src/main.rs -l aggressive # JSON output";

/// Help text for the json command.
#[allow(dead_code)]
pub const JSON_HELP: &str = "\
Show JSON structure without values.

Reads JSON from a file or stdin and displays the structure: keys, types,
and array lengths. Reduces large API responses to a compact schema overview.

USAGE:
    trs json [OPTIONS] [-f <FILE>]

OPTIONS:
    -f, --file <FILE>    Input JSON file (stdin if not specified)
    -d, --depth <NUM>    Maximum depth to display

OUTPUT:
    Shows keys with their value types: String, Number, Bool, Null,
    Array[N], Object{N keys}. Long strings show length: String[1024].

EXAMPLES:
    # Inspect a JSON API response
    cat response.json | trs json

    # Inspect with depth limit
    trs json -f config.json --depth 2

    # From curl output
    curl -s https://api.github.com/users/octocat | trs json

    # JSON schema output
    cat data.json | trs --json json";

/// Help text for the err command.
#[allow(dead_code)]
pub const ERR_HELP: &str = "\
Run a command and show only errors and warnings.

Filters stdout and stderr to lines containing error, warning,
panic, fatal, or failed patterns. Includes 1 line of context
after each error for stack traces.

USAGE:
    trs err <COMMAND> [ARGS]...

EXAMPLES:
    trs err cargo build          # Show only build errors
    trs err npm install          # Show only install warnings/errors
    trs err make                 # Show only make failures
    trs err cargo test           # Show only test failures";

/// Help text for the stats command.
#[allow(dead_code)]
pub const STATS_HELP: &str = "\
Show token savings statistics.

Displays cumulative savings from using trs, grouped by command.
Data is stored in ~/.trs/history.jsonl.

USAGE:
    trs stats [OPTIONS]

OPTIONS:
    -H, --history    Show recent command history
    -p, --project    Filter to current project directory
        --json       Output in JSON format

EXAMPLES:
    trs stats                    # Global savings summary
    trs stats --history          # Recent commands with reduction %
    trs stats --project          # Savings for current project only
    trs stats --json             # JSON output for dashboards";

/// Help text for the doctor command.
#[allow(dead_code)]
pub const DOCTOR_HELP: &str = "\
Validate trs installation health.

Runs a series of checks to verify that trs is correctly installed,
runtime dependencies are available, and configuration is functional.

CHECKS:
    version      Binary version is readable
    binary       Binary path on disk
    PATH         trs is findable in PATH
    dep:git      git is available
    dep:rg       ripgrep (rg) is available
    config dir   ~/.trs/ exists and is writable
    history      history.jsonl is writable
    stdin pipe   stdin pipeline works (trs clean)
    hooks        AI tool hooks installed

STATUS MARKERS:
    +  PASS — check passed
    ~  WARN — non-critical issue
    !  FAIL — needs attention

EXIT CODES:
    0  All checks passed (or only warnings)
    1  One or more checks failed

EXAMPLES:
    trs doctor                   # Run all checks
    trs doctor --json            # JSON output for CI pipelines";

/// Help text for the benchmark command.
#[allow(dead_code)]
pub const BENCHMARK_HELP: &str = "\
Benchmark a command showing compression metrics.

Runs a command both raw and through the trs pipeline, then reports
byte reduction, estimated token savings, and execution time.

USAGE:
    trs benchmark <COMMAND> [ARGS]...

OPTIONS:
    --repeat <N>    Number of iterations to average over (default: 1)
    --json          Output metrics in JSON format

OUTPUT:
    Raw output      Original byte count from the command
    Compressed      Byte count after trs processing
    Reduction       Percentage of bytes saved
    Est. tokens     Estimated token count (raw vs compressed)
    Time            Average execution time in milliseconds

EXAMPLES:
    trs benchmark git status
    trs benchmark git log -10 --json
    trs benchmark ls -la --repeat 5
    trs benchmark cargo test --repeat 3 --json";

/// Help text for the diff command.
#[allow(dead_code)]
pub const DIFF_HELP: &str = "\
Show raw vs compact output and exactly what trs dropped.

Runs a command once, captures the raw output and the compacted output
trs produces, then reports the size delta and lists the lines dropped or
collapsed — so you can trust what the agent actually sees.

USAGE:
    trs diff <COMMAND> [ARGS]...

OPTIONS:
    --json          Output the diff + metrics as JSON

OUTPUT:
    Header          raw vs compact bytes + estimated tokens saved
    Compact         the output the agent receives
    Dropped         lines present in raw but not in the compact output

EXAMPLES:
    trs diff git status
    trs diff cargo test
    trs diff --json git log -10        (flags go before the command)";

/// Help text for the ingest command.
#[allow(dead_code)]
pub const INGEST_HELP: &str = "\
Generate an LLM-ready digest of a project.

Walks the project directory, reads files with optional compression,
and produces a structured markdown digest optimized for AI context windows.

USAGE:
    trs ingest [PATH|URL|owner/repo] [OPTIONS]

OPTIONS:
    -l, --level <LEVEL>    Compression: full, minimal, aggressive (default: full)
    -b, --budget <TOKENS>  Token budget (e.g. 128k, 64000). Auto-truncates to fit.
    --changed              Only include files with uncommitted changes
    --since <REF>          Only include files changed since git ref (e.g. HEAD~5)
    -e, --exclude <PAT>    Exclude paths matching pattern (repeatable)
    -o, --output <FILE>    Write to file instead of stdout
    --ollama <MODEL>       Format digest with local Ollama model (e.g. llama3)
    --deps                 Output only the dependency graph (no file content)
    --tmp                  For URL input: shallow-clone into a tempdir (not saved)

EXAMPLES:
    trs ingest                              # full project digest (cwd)
    trs ingest owner/repo                   # clone via spark, then digest
    trs ingest github.com/o/r               # same, host-prefixed shorthand
    trs ingest https://github.com/o/r --tmp # ephemeral clone, auto-cleanup
    trs ingest --budget 128k                # fit to 128k token context
    trs ingest --changed                    # only uncommitted changes
    trs ingest --changed -l aggressive      # signatures of changed files
    trs ingest --since HEAD~5               # last 5 commits
    trs ingest src/ -e tests -e fixtures    # src/ only, exclude tests
    trs ingest -o digest.md                 # write to file
    trs ingest --ollama llama3              # LLM-formatted summary
    trs ingest --deps                       # import graph only, no file content
    trs ingest --budget 64k -l minimal      # compressed, budget-fitted

REMOTE INPUT:
    Remote refs accepted: https://... | git@... | github.com/o/r | o/r
    - With spark installed: clones to your spark-managed repos dir (persistent)
    - Without spark (or --tmp): shallow-clone to a tempdir, auto-deleted on exit
    - Local path wins when it exists: 'trs ingest owner/repo' uses ./owner/repo
      if that dir exists, otherwise resolves as a GitHub shorthand

OUTPUT:
    Structured markdown with:
    - Project metadata (files, tokens, compression level)
    - Budget usage (if --budget specified)
    - File tree
    - File contents (with code fences and language tags)
    - Changed file markers (if --changed/--since)

NOTES:
    - Respects .gitignore automatically
    - Skips binary files, lock files, and files >256KB
    - With --ollama: sends digest to local Ollama for a structured summary
      (requires Ollama running at localhost:11434)";
