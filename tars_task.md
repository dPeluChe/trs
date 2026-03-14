# TARS CLI Development Tasks

Binary command: `trs`  
Language: Rust

TARS is a CLI toolkit that transforms noisy terminal output into compact and structured signal for developers, automation pipelines, and AI agents.

The project includes three functional layers:

1. Command wrappers (git, ls, diff, etc.)
2. Parsers (stdin → structured output)
3. Tools (search, replace, clean, conversions)

---

# Core CLI Architecture

- [x] initialize Rust CLI project for `tars-cli`
- [x] configure binary command name as `trs`
- [x] implement CLI argument parsing
- [x] implement global flags: `--raw`, `--compact`, `--json`, `--csv`, `--tsv`, `--agent`
- [x] define output flag precedence rules
- [x] implement CLI help system
- [x] implement command-specific help
- [x] implement command routing system for subcommands

---

# Command Execution Layer

- [x] implement process execution module
- [x] support executing real system commands
- [x] capture `stdout`
- [x] capture `stderr`
- [x] capture exit code
- [x] capture execution duration
- [x] propagate exit codes correctly
- [x] support passing command arguments safely
- [x] handle command not found errors
- [x] handle permission errors
- [x] implement optional timeout handling

---

# Command Classifier

- [x] implement command classifier module
- [x] detect `git status`
- [x] detect `git diff`
- [x] detect `ls`
- [x] detect `find`
- [x] detect `grep`
- [x] detect test runners (`pytest`, `jest`, `vitest`, `npm test`, `pnpm test`, `bun test`)
- [x] detect log/tail style output
- [x] implement fallback passthrough for unsupported commands

---

# Reducer Framework

- [x] define reducer interface
- [x] implement reducer input structure
- [x] implement reducer output structure
- [x] support compact output
- [x] support structured output
- [x] support truncation detection
- [x] ensure reducers never hide errors
- [x] ensure reducers preserve exit codes

---

# Git Status Reducer

- [x] parse `git status` output
- [x] extract branch name
- [x] extract ahead/behind counts
- [x] extract staged file count
- [x] extract unstaged file count
- [x] extract untracked file count
- [x] detect clean repository state
- [x] detect dirty repository state
- [x] group modified file paths
- [x] implement compact output
- [x] implement JSON output
- [x] handle detached HEAD state

---

# Git Diff Reducer

- [x] parse `git diff` output
- [x] extract changed file list
- [x] extract insertions and deletions summary
- [x] group diff results by file
- [x] implement compact summary
- [x] implement JSON structured output
- [x] implement truncation for very large diffs

---

# LS Reducer

- [x] parse `ls` output
- [x] separate directories from files
- [x] detect hidden files
- [x] group common generated directories
- [x] implement compact output
- [x] implement JSON output
- [x] handle symbolic links
- [x] handle permission denied entries

---

# Grep Reducer

- [x] parse grep results
- [x] group matches by file
- [x] preserve line numbers
- [x] collapse repeated context lines
- [x] implement compact summary
- [x] implement JSON output
- [x] support truncation for large result sets

---

# Test Runner Reducers

- [x] parse pytest output
- [x] parse jest output
- [x] parse vitest output
- [x] parse npm test output
- [x] parse pnpm test output
- [x] parse bun test output
- [x] extract passed test count
- [x] extract failed test count
- [x] extract skipped test count
- [x] extract execution duration
- [x] extract failing test identifiers
- [x] implement compact success summary
- [x] implement failure-focused summary

---

# Log / Tail Reducer

- [x] parse log streams
- [x] detect repeated log lines
- [x] collapse repeated entries
- [x] detect error/warning levels
- [x] maintain most recent critical lines
- [x] implement compact log output
- [x] implement JSON summary

---

# Formatter System

- [x] implement raw formatter
- [x] implement compact formatter
- [x] implement JSON formatter
- [x] implement CSV formatter
- [x] implement TSV formatter
- [x] implement agent formatter
- [x] define stable JSON schema for each reducer

---

# STDIN Parsers

- [x] implement `trs parse` command
- [x] implement `trs parse git-status`
- [x] implement `trs parse ls`
- [x] implement `trs parse grep`
- [x] implement `trs parse logs`
- [x] support reading input from stdin
- [x] handle malformed input gracefully

---

# Search Tool

- [x] implement `trs search`
- [x] support syntax `trs search <path> <query>`
- [x] integrate high-performance search using ripgrep or Rust equivalent crates
- [x] group matches by file
- [x] return line numbers
- [x] include short match excerpts
- [x] return total match count
- [x] return total files matched
- [x] support `--json` output
- [x] support `--compact` output
- [x] implement result truncation threshnew
- [x] support ignoring common directories (`.git`, `node_modules`, `dist`, `build`, etc.)

---

# Replace Tool

- [x] implement `trs replace`
- [x] support syntax `trs replace <path> <search> <replace>`
- [x] support optional extension filter
- [x] support preview mode
- [x] return affected file count
- [x] return replacement count
- [x] support JSON output

---

# Tail Tool

- [x] implement `trs tail`
- [x] support syntax `trs tail <file>`
- [x] support compact log output
- [x] support filtering for error lines
- [x] support last N lines option
- [x] support streaming mode

---

# Clean Tool

- [x] implement `trs clean`
- [x] remove ANSI escape codes
- [x] collapse repeated blank lines
- [x] trim whitespace
- [x] collapse repeated lines
- [x] support stdin input
- [x] support compact output

---

# HTML to Markdown Tool

- [x] implement `trs html2md`
- [x] support local HTML files
- [x] support URL input
- [x] convert headings
- [x] convert links
- [x] convert lists
- [x] remove unnecessary HTML noise
- [x] support JSON metadata output

---

# TXT to Markdown Tool

- [x] implement `trs txt2md`
- [x] detect simple heading patterns
- [x] convert text sections to markdown headings
- [x] convert lists
- [x] normalize spacing
- [x] support stdin input

---

# Command Statistics

- [ ] implement `--stats` flag
- [ ] measure original output size
- [ ] measure reduced output size
- [ ] estimate token reduction
- [ ] display reducer used
- [ ] display output mode used

---

# Testing

- [ ] create fixtures for `git status`
- [ ] create fixtures for `git diff`
- [ ] create fixtures for `ls`
- [ ] create fixtures for `grep`
- [ ] create fixtures for test runners
- [ ] create fixtures for logs
- [ ] add reducer tests
- [ ] add formatter tests
- [ ] add command execution tests
- [ ] add parser tests
- [ ] add search tool tests
- [ ] add replace tool tests
- [ ] add tail tool tests
- [ ] add clean tool tests
- [ ] add conversion tool tests

---

# Output Comparison Tests

- [ ] compare raw vs reduced output size for `git status`
- [ ] compare raw vs reduced output size for `git diff`
- [ ] compare raw vs reduced output size for `ls`
- [ ] compare raw vs reduced output size for search results
- [ ] validate signal preservation after reduction

---

# Local Execution

- [ ] ensure binary runs as `trs`
- [ ] validate CLI works locally
- [ ] validate command routing
- [ ] validate structured output modes