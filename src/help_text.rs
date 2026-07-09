//! Help text for text-processing commands (search/replace/tail/clean/
//! parse/html2md/txt2md/trim). Split out of help.rs to keep it lean.

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
