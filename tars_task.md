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
- [ ] parse npm test output
- [ ] parse pnpm test output
- [ ] parse bun test output
- [ ] extract passed test count
- [ ] extract failed test count
- [ ] extract skipped test count
- [ ] extract execution duration
- [ ] extract failing test identifiers
- [ ] implement compact success summary
- [ ] implement failure-focused summary

---

# Log / Tail Reducer

- [ ] parse log streams
- [ ] detect repeated log lines
- [ ] collapse repeated entries
- [ ] detect error/warning levels
- [ ] maintain most recent critical lines
- [ ] implement compact log output
- [ ] implement JSON summary

---

# Formatter System

- [ ] implement raw formatter
- [ ] implement compact formatter
- [ ] implement JSON formatter
- [ ] implement CSV formatter
- [ ] implement TSV formatter
- [ ] implement agent formatter
- [ ] define stable JSON schema for each reducer

---

# STDIN Parsers

- [ ] implement `trs parse` command
- [ ] implement `trs parse git-status`
- [ ] implement `trs parse ls`
- [ ] implement `trs parse grep`
- [ ] implement `trs parse logs`
- [ ] support reading input from stdin
- [ ] handle malformed input gracefully

---

# Search Tool

- [ ] implement `trs search`
- [ ] support syntax `trs search <path> <query>`
- [ ] integrate high-performance search using ripgrep or Rust equivalent crates
- [ ] group matches by file
- [ ] return line numbers
- [ ] include short match excerpts
- [ ] return total match count
- [ ] return total files matched
- [ ] support `--json` output
- [ ] support `--compact` output
- [ ] implement result truncation threshold
- [ ] support ignoring common directories (`.git`, `node_modules`, `dist`, `build`, etc.)

---

# Replace Tool

- [ ] implement `trs replace`
- [ ] support syntax `trs replace <path> <search> <replace>`
- [ ] support optional extension filter
- [ ] support preview mode
- [ ] return affected file count
- [ ] return replacement count
- [ ] support JSON output

---

# Tail Tool

- [ ] implement `trs tail`
- [ ] support syntax `trs tail <file>`
- [ ] support compact log output
- [ ] support filtering for error lines
- [ ] support last N lines option
- [ ] support streaming mode

---

# Clean Tool

- [ ] implement `trs clean`
- [ ] remove ANSI escape codes
- [ ] collapse repeated blank lines
- [ ] trim whitespace
- [ ] collapse repeated lines
- [ ] support stdin input
- [ ] support compact output

---

# HTML to Markdown Tool

- [ ] implement `trs html2md`
- [ ] support local HTML files
- [ ] support URL input
- [ ] convert headings
- [ ] convert links
- [ ] convert lists
- [ ] remove unnecessary HTML noise
- [ ] support JSON metadata output

---

# TXT to Markdown Tool

- [ ] implement `trs txt2md`
- [ ] detect simple heading patterns
- [ ] convert text sections to markdown headings
- [ ] convert lists
- [ ] normalize spacing
- [ ] support stdin input

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