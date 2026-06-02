//! Help system module for trs (Token-Reducing Shell).
//!
//! This module contains comprehensive documentation and help text for all CLI commands,
//! flags, and usage examples.

/// Long about text for the main CLI.
pub const LONG_ABOUT: &str = "\
trs (Token-Reducing Shell) - Transform noisy terminal output into compact, structured signal

Reduces token consumption by 68-99% for developers, AI agents, and automation.
Just prefix any command with trs:

    trs git status               # 80% reduction
    trs git log -10              # 90% reduction
    trs ls -la                   # 82% reduction
    trs npm test                 # 90% reduction
    trs env                      # 68% reduction

FORMAT FLAGS (work before or after the command):
    --json     Structured JSON output
    --csv      CSV tabular output
    --tsv      TSV tabular output
    --agent    AI-optimized format
    --compact  Human-readable (default)
    --raw      Unprocessed passthrough
    --stats    Show reduction metrics

EXAMPLES:
    trs git status --json        # JSON output
    trs err cargo build          # Show only errors
    trs search src \"TODO\"        # Ripgrep search
    trs curl -I https://api.com  # HTTP headers compact
    trs stats                    # View token savings";

/// Help text for output format precedence.
#[allow(dead_code)]
pub const FORMAT_PRECEDENCE: &str = "\
OUTPUT FORMAT PRECEDENCE:
    When multiple format flags are specified, the following precedence applies:

    1. JSON (--json)     - Highest priority, most structured
    2. CSV (--csv)       - Structured tabular format
    3. TSV (--tsv)       - Tab-separated format
    4. Agent (--agent)   - AI-optimized format
    5. Compact (--compact) - Human-readable summary
    6. Raw (--raw)       - Unprocessed output

    Default: Compact (when no format flags are specified)

Examples:
    trs --json --csv search . \"test\"    # Uses JSON (higher precedence)
    trs --agent --compact search . \"x\"  # Uses Agent format
    trs search . \"pattern\"              # Uses Compact (default)";

/// Help text for the search command.
#[allow(dead_code)]
pub const SEARCH_HELP: &str = "\
Search for patterns in files using ripgrep-powered search.

The search command provides fast, intelligent pattern matching with support
for regular expressions and various output formats.

USAGE:
    trs search [OPTIONS] <PATH> <QUERY>

ARGUMENTS:
    <PATH>    Directory or file to search in
    <QUERY>   Search pattern (supports regular expressions)

OPTIONS:
    -e, --extension <EXT>    Filter by file extension (e.g., \"rs\", \"ts\")
    -i, --ignore-case        Case-insensitive search
    -C, --context <NUM>      Number of context lines around matches
    --limit <NUM>            Maximum number of results to return

EXAMPLES:
    # Search for \"TODO\" in all Rust files
    trs search . \"TODO\" -e rs

    # Case-insensitive search with context
    trs search src \"error\" -i -C 2

    # Search with JSON output
    trs --json search . \"fn main\" --limit 50

    # Search and limit results
    trs search ./src \"import\" --limit 100";

/// Help text for the replace command.
#[allow(dead_code)]
pub const REPLACE_HELP: &str = "\
Search and replace patterns in files.

The replace command finds patterns in files and replaces them with a new string.
Use --dry-run to preview changes before applying them.

USAGE:
    trs replace [OPTIONS] <PATH> <SEARCH> <REPLACE>

ARGUMENTS:
    <PATH>     Directory or file to search in
    <SEARCH>   Pattern to search for (supports regular expressions)
    <REPLACE>  Replacement string

OPTIONS:
    -e, --extension <EXT>    Filter by file extension
    --dry-run, --preview     Preview changes without modifying files
    --count                  Output only the total replacement count

OUTPUT:
    Shows affected file count and replacement count in all formats.
    With --count, outputs only the total replacement count.

EXAMPLES:
    # Replace \"foo\" with \"bar\" in all files
    trs replace . \"foo\" \"bar\"

    # Preview changes in TypeScript files
    trs replace ./src \"oldName\" \"newName\" -e ts --preview

    # Replace with JSON output showing affected files
    trs --json replace . \"TODO\" \"DONE\"

    # Preview with dry-run (equivalent to --preview)
    trs replace ./src \"oldName\" \"newName\" -e ts --dry-run

    # Get just the count of replacements
    trs replace . \"TODO\" \"DONE\" --count";

/// Help text for the tail command.
#[allow(dead_code)]
pub const TAIL_HELP: &str = "\
Tail a file with compact log output.

The tail command reads the last lines of a file and can optionally filter
for error lines or follow the file for new content.

USAGE:
    trs tail [OPTIONS] <FILE>

ARGUMENTS:
    <FILE>    File to tail

OPTIONS:
    -n, --lines <NUM>    Number of lines to show (default: 10)
                         Supports -N shorthand (e.g., -5 for last 5 lines)
    -e, --errors         Filter for error lines only
    -f, --follow         Follow the file for new lines (streaming mode)

EXAMPLES:
    # Show last 20 lines of a log file
    trs tail /var/log/app.log -n 20

    # Show last 5 lines using shorthand
    trs tail /var/log/app.log -5

    # Show only error lines
    trs tail /var/log/app.log --errors

    # Follow log file in real-time
    trs tail /var/log/app.log --follow

    # Tail with JSON output
    trs --json tail /var/log/app.log -n 100";

/// Help text for the clean command.
#[allow(dead_code)]
pub const CLEAN_HELP: &str = "\
Clean and format text output.

The clean command processes text input to remove noise and normalize formatting.
It reads from stdin by default.

USAGE:
    trs clean [OPTIONS]

OPTIONS:
    -f, --file <FILE>           Input file (stdin if not specified)
    --no-ansi                   Remove ANSI escape codes
    --collapse-blanks           Collapse repeated blank lines
    --collapse-repeats          Collapse repeated lines
    --trim                      Trim whitespace from lines

EXAMPLES:
    # Clean output from a command
    some-command | trs clean --no-ansi --trim

    # Clean a log file
    trs clean -f app.log --collapse-blanks --collapse-repeats

    # Full cleanup
    cat messy.txt | trs clean --no-ansi --collapse-blanks --trim";

/// Help text for the parse command.
#[allow(dead_code)]
pub const PARSE_HELP: &str = "\
Parse structured input from stdin or file.

The parse command transforms output from common CLI tools into structured formats.
It supports various parsers for git, ls, grep, test runners, and logs.

USAGE:
    trs parse <PARSER> [OPTIONS]

PARSERS:
    git-status    Parse git status output
    git-diff      Parse git diff output
    ls            Parse ls output
    grep          Parse grep output
    test          Parse test runner output
    logs          Parse log/tail output

OPTIONS:
    -f, --file <FILE>    Input file (stdin if not specified)

TEST RUNNER OPTIONS:
    -t, --runner <RUNNER>    Test runner type (pytest, jest, vitest, npm, pnpm, bun)

EXAMPLES:
    # Parse git status
    git status | trs parse git-status

    # Parse git diff from file
    trs parse git-diff -f changes.diff

    # Parse pytest output with JSON format
    pytest | trs --json parse test --runner pytest

    # Parse ls output
    ls -la | trs parse ls";

/// Help text for the html2md command.
#[allow(dead_code)]
pub const HTML2MD_HELP: &str = "\
Convert HTML to Markdown.

The html2md command converts HTML content (from a file or URL) to clean Markdown.

USAGE:
    trs html2md <INPUT> [OPTIONS]

ARGUMENTS:
    <INPUT>    Input HTML file or URL

OPTIONS:
    -o, --output <FILE>    Output file (stdout if not specified)
    --metadata             Include metadata in JSON format

EXAMPLES:
    # Convert a URL to Markdown
    trs html2md https://example.com

    # Convert and save to file
    trs html2md https://example.com -o page.md

    # Convert local HTML file
    trs html2md index.html -o index.md

    # Include metadata
    trs html2md https://example.com --metadata";

/// Help text for the txt2md command.
#[allow(dead_code)]
pub const TXT2MD_HELP: &str = "\
Convert plain text to Markdown.

The txt2md command converts plain text to Markdown format, detecting patterns
like headings and lists.

USAGE:
    trs txt2md [OPTIONS]

OPTIONS:
    -i, --input <FILE>     Input text file (stdin if not specified)
    -o, --output <FILE>    Output file (stdout if not specified)

EXAMPLES:
    # Convert from stdin
    cat notes.txt | trs txt2md

    # Convert file to Markdown
    trs txt2md -i notes.txt -o notes.md

    # Convert and output as JSON
    trs --json txt2md -i notes.txt";

/// Help text for the trim command.
#[allow(dead_code)]
pub const TRIM_HELP: &str = "\
Trim whitespace from text lines.

The trim command removes leading and/or trailing whitespace from each line of text.
It reads from stdin by default.

USAGE:
    trs trim [OPTIONS]

OPTIONS:
    -f, --file <FILE>      Input file (stdin if not specified)
    --leading              Trim leading whitespace only
    --trailing             Trim trailing whitespace only (default when no flags)

EXAMPLES:
    # Trim all whitespace from stdin
    cat file.txt | trs trim

    # Trim whitespace from a file
    trs trim -f file.txt

    # Trim only leading whitespace
    cat file.txt | trs trim --leading

    # Trim only trailing whitespace
    cat file.txt | trs trim --trailing

    # With JSON output
    cat file.txt | trs --json trim";

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

/// Returns the help text for a specific command.
#[allow(dead_code)]
pub fn get_command_help(command: &str) -> Option<&'static str> {
    match command {
        "search" => Some(SEARCH_HELP),
        "replace" => Some(REPLACE_HELP),
        "tail" => Some(TAIL_HELP),
        "clean" => Some(CLEAN_HELP),
        "parse" => Some(PARSE_HELP),
        "html2md" => Some(HTML2MD_HELP),
        "txt2md" => Some(TXT2MD_HELP),
        "trim" => Some(TRIM_HELP),
        "run" => Some(RUN_HELP),
        "read" => Some(READ_HELP),
        "json" => Some(JSON_HELP),
        "err" => Some(ERR_HELP),
        "stats" => Some(STATS_HELP),
        "doctor" => Some(DOCTOR_HELP),
        "benchmark" => Some(BENCHMARK_HELP),
        "diff" => Some(DIFF_HELP),
        "ingest" => Some(INGEST_HELP),
        _ => None,
    }
}

/// Returns the format precedence help text.
#[allow(dead_code)]
pub fn get_format_precedence_help() -> &'static str {
    FORMAT_PRECEDENCE
}

#[cfg(test)]
#[path = "help_tests.rs"]
mod tests;
