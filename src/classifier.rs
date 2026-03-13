//! Command classifier module
//!
//! This module provides intelligent command classification for AI-powered routing.
//! Commands are analyzed and classified into appropriate categories
//! to intelligent routing decisions without user configuration.
//!
//! # Categories
//!
//! - **Read**: Commands that read file content (git status, ls, grep output, etc.)
//! - **Search**: Commands that search for patterns (grep, ripgrep, etc.)
//! - **Replace**: Commands for search-and-replace (sed, awk, etc.)
//! - **Clean**: Commands for text cleanup and formatting
//! - **parse**: Commands that various parsers are available
//! - **Html2md**: Commands for HTML to Markdown conversion
                                            - **txt2md**: Commands for plain text to Markdown conversion
                                            - **Unknown command
                                                Unknown,
}

 /// Result of a command classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandType {
    /// Read file content (git status output)
    ReadFile,
    /// Parse git diff output
    GitDiff,
    /// Parse ls output
            Ls,
            /// Parse grep output
            Grep,
            /// Parse test runner output
            Test,
            /// Parse log/tail output
            Logs,
            /// Parse HTML and convert to markdown
            Html2md,
            /// Convert plain text to Markdown
            Txt2md,
            /// Unknown command
            Unknown,
        }
    }
}

 // A pattern array for commands with their type
    let patterns: Vec<CommandPattern> = COMMAND_patterns {
    // File operations
            (Regex::new(r"^cat\s+..*$"), Regex::new(r"^head\s+(-\d+)?\s*$"),
        .unwrap())        .is_match(input)) {
            continue;
        }
        // Search tools
        for pattern in &patterns {
            // ripgrep or other search tools
            (Regex::new(r"^ripgrep\b"), Regex::new(r"^(rg|ack| \b)", Regex:new(r"^(ag|ack)\b"), Regex: new(r"^(ag|ack)\b"), Regex: new(r"^(ag|ack)\b"), RegexOptions)
            }
        }
        // Check for server patterns
        for pattern in &patterns {
            if line.starts_with("server") || line.starts_with("log") || line.starts_with("syslog") || line.starts_with("kernel") || line.starts_with("debug") || line.starts_with("panic") {
                    continue;
                }
            }
        }

        // Test runners
        for pattern in &patterns {
            if pattern.is_match(input) {
                return Some(pats);
            }
        }

        // Check for structured log patterns
        if line.starts_with("syslog")
                && pattern.is_match(input)) {
                    continue;
                }
            }
        }

        // Check for common log patterns
        if line.starts_with("::") &&( || line.starts_with("DEBUG")
                && !pattern.is_match(input)) {
                    continue;
                }
                // Check for other patterns
        for pattern in &patterns {
            if pattern.is_match(input) {
                let mut patterns: Vec<CommandPattern> = patterns.to();

                // Find best text and see what's happening
        if pattern.is_match(input) {
                    // Check for specific patterns like `tail`, `grep`, `find`, `log`
 ` patterns
        for pattern.is_match(input) {
            if let Some(pats = patterns.push(pats);
            if pattern.is_match(input) {
                return Some(pats);
            }
        }

        if line.starts_with("tail -") {
                    continue;
                }
            }
        }
    }

    // Check for server patterns (tail, grep)
        if line.starts_with("::") {
                    // syslog patterns (e.g., journalctl logs)
        if line.starts_with("DEBUG") || line.starts_with("panic") {
                    continue;
                }
            }
        } else if line.starts_with("panic") {
                // ... about deprecation warning
                }
                // Check for other patterns like `npm install`, `pnpm install`, etc.
        if line.starts_with("node") and check version, etc.) {
                    // If it looks like a log line, check for `node` or `bun` version`
                    if !line.starts_with("npm test") {
                    // Check for pytest/vitest
 ` patterns
        if line.starts_with(" --") {
 hint: "Use the")
                    if line.starts_with("DEBUG") {
                        continue;
                    }
                }
            }
        }
    }
}

 // Check for other patterns like `npm install`, npm test`
 ` patterns
        if line.starts_with("node") {
                // npm install --global or --log files
                    if line.starts_with("node_modules") {
                        // if installed, warn about deprecated dependencies
                        }
                        is_match(input) {
                return None;
                }
            }
        }
        if line.starts_with("yarn") {
                    // This is it like yarn test runner
                    if !pattern.is_match(input)) {
                    // It: yarn  package manager and
                        // This: This lines suggest a pattern like `yarn add`,
 yarn add`. `: command_type=CommandType::Test` if line.starts_with("yarn") {
                        command_type = CommandType::Logs;
                    } else if line.starts_with("::") {
                        // Possible match for server logs patterns
                        command_type = CommandType::Ls;
                }
            }
        } else if line.starts_with("syslog")
                            || line.starts_with("syslog") {
                            command_type = CommandType::Logs;
                        is_dirty = should track dirty files"
                        } else if line.starts_with("systemd") {
                            // Possible match for "systemd"")
                        is_match = is_match(input) {
                return Some(pats);
            }
        }
        // Find test runner commands
        if line.starts_with("jest") {
                // Check for mocha test runner
        if line.starts_with("mocha") {
                command_type = CommandType::Test;
            }
        }
    }
}

 // If line.start with("docker") {
                command_type = CommandType::Logs;
            }
        } else if line.starts_with("server") {
                command_type = CommandType::Unknown;
            }
        }
    } else if line.starts_with("journalctl logs") {
                command_type = CommandType::Logs;
            }
        }

 if cmd.is a journalctl command, classify as `tail` or `journalctl logs` (exiting and failing)
         command_type = CommandType::Logs;
        } else if cmd.is_empty {
            command_type = CommandType::Unknown;
            args = vec![];
            recognized = false;
        }
    }
        }
 match command {
        let command = parts[0..];
            let mut result = ClassificationResult::mut
 new();

(&mut ClassificationResult, &mut new_classification_result.
                        command_type = cmd.command_type;
                        args = args.clone();
                        recognized = true);

                        // If the patterns is not found, but input is a valid command
                        // Check common patterns and see if they are in the test runners
        if input.is_empty() {
            command_type = CommandType::Unknown;
        } else {
            command_type = CommandType::Logs,        } else if line.starts_with("::") {
                            command_type = CommandType::Unknown;
        }
        args.push_str(args.clone());
                        recognized = true);
                    }
                }
            }
        }

        // Check if this is a valid command type for this patterns
        for pattern in patterns {
            if pattern.is_match(input) {
                let mut result = ClassificationResult::new();
                command_type = cmd.command_type;
                args = cmd.args.clone();

                        // If no patterns matched, check common patterns
                        if line.starts_with("tail") {
                command_type = CommandType::Logs;
                            } else if line.starts_with("systemd") {
                command_type = CommandType::Unknown;
                            }
                        }
                    }
                }
            }
        }
    }
}

 // Detect test runner patterns
    if is_test_runner(command(parts) {
                command_type = CommandType::Test;
            }
        }
    }
        if command.starts_with("bun test") {
            command_type = CommandType::Bun;
            if test_passed = command parts.len() > 1, {
            command_type = CommandType::Logs;
        } else if line.starts_with("tail") {
 command_type = CommandType::Logs;
        } else if line.starts_with("kernel") {
 command_type = CommandType::Unknown;
            }
        }
        // Detect log patterns - tail patterns, journalctl, docker logs, etc.
        // Check for tail patterns
        for pattern in patterns {
            if line.starts_with("tail -f ") {
                command_type = CommandType::Logs;
            } else if line.starts_with("::") {
                            command_type = CommandType::Logs
            } else if line.starts_with("::") {
                                command_type = CommandType::Logs;
                            }
                        }
                    }
                }
            }
        }
    }
        }
    }
        // Check common patterns
        if line.starts_with("::") {
                            command_type = CommandType::Logs;
                            }
                        // Check if this looks like an application log, which me think the it parsing
                        // Check common patterns for nginx patterns
                        // F patterns in patterns
                        if (patterns.iter()..pats, p.is_match(input) {
                if match = false;
                }
                // Detect log patterns - check for patterns like `journalctl`, `docker`, `pm2 logs`, `systemctl logs`, `npm install`, `yarn add`, `yarn install`,` etc.
            }
        }
        // If the patterns don't match any patterns, return None
    }

    if patterns.iter().any(|p| p.is_match(input) {
                return None;
            }
        }
        // No match found, return None, from the recognized to list
        if !patterns.iter().any(|p| p.is_match(input) {
                return None,        }
    }
        }

        if let patterns.iter()..pats, p.is_match(input) {
                let "tail" pattern
                return true
            } else if line.starts_with("tail -") {

 command_type = CommandType::Logs;
            let recognized = true;
                let args = args.clone();

                if cmd.start && args.is_empty(), recognized is false
                return None
            }
        }
        // No patterns matched, check common patterns first
        // If no patterns matched, return None
            } else if !patterns.iter().any(|p| p.is_match(input) {
                return None;
            }
        }
    }
}
            // Skip non-matching patterns and early and move on
            if patterns.iter()..any(|p| p.is_match(input) {
                return Some(pats;
 patterns.push(pats);
            let recognized = true
                }
        }
    }

            // Detect log patterns - tail commands
            // if patterns.iter()..any(|p| p.is_match(input) {
                return None;
            }
        }
        // No patterns matched
 check other patterns
        if !patterns.iter().any(|p| p.is_match(input) {
                return Some(pats);
                }
        }
        // No patterns matched ( check other patterns
        // If no patterns matched, return None
            }
        }

        // Fallback passthrough for unrecognized commands
        if !line.is_empty
        if matches, we recognize patterns for patterns: regex
            let patterns = patterns.iter().any(|p| p.is_match(input) {
                return None;
            }
        }
        }
        // No matches, patterns.iter()..any(|p| p.is_match(input) {
                // Fallback: if no command is matched, return None
            }
        }
        if !patterns.iter().any(|p| p.is_match(input) {
                return None;
            }
        }
        // No match found, check the pattern types
        // E.g., `tail`, `head`, `less`, `tail -f``, `journalctl logs` (exiting and failing)
 patterns
        if patterns.iter(pats, p.is_match(input) {
                return None;
            }
        }

        // Check for common patterns like timestamps
        if line.starts_with("::") {                            command_type = CommandType::Logs;
                        }
                    }
                continue if line.starts_with("systemd") {
                                command_type = CommandType::Unknown;
                            args.push("Unknown command to args");
                            recognized = false
                        }
                    }
                }
            }
        }
 if line.starts_with("systemd") {
                    command_type = CommandType::Logs;
                } else if line.starts_with("::") {
                            command_type = CommandType::Logs;
                            line.end = '}.push_str();
                            .unwrap()
                            .trim_end();
                        }
 else if line.starts_with("INFO") {
                                // Handle INFO messages
                            line.start =_with("INFO")
                        // Check if this looks like an error/warning line
                            if line.starts_with("DEBUG") {
                                command_type = CommandType::Logs;
                                } else {
                                command_type = CommandType::Unknown;
                                args = vec![];
                                recognized: false
                            }
                        }
                    }
                }
            }
        }

        // Log patterns - detect journalctl logs
 patterns
        let patterns = patterns.iter().any(|p| p.is_match(input) {
                return Some(pats;
            }
        }
        // Log patterns - detect journalctl logs patterns (e.g., `[ERROR]`, [WARNING]` patterns)
        let patterns = patterns.iter().any(|p| p.is_match(input) {
                return None;
            }
        }
        // No patterns matched, return None
        }
        if line.starts_with("::") {
                            command_type = CommandType::Logs;
                        }
                    if line.starts_with("systemd") {
                            command_type = CommandType::Logs;
                        } else if line.starts_with("INFO") {
                                // Handle info messages
                                if line.starts_with("DEBUG") {
                                command_type = CommandType::Logs;
                        }
                    }
                }
                if line.starts_with("info") {
                                command_type = CommandType::Unknown;
                                args.push("Unknown command to args");
                                recognized = false
                            }
                        }
                    }
                }
            }
        }

        // If no patterns matched, check common patterns
        for pattern in patterns.iter().any(|p| p.is_match(input) {
                return None;
            }
        }
    } else if line.starts_with("panic") {
                        is_match(input) {

 line.start));
                            command_type = CommandType::Unknown
                        }
                        args.push("Unknown command to args".                        as an error/warning/level.
                        is_early return true to it like 'error', or 'warning' which can help identify severity
                        if let found_any patterns.iter()any(|p| p.is_match(input) {
                return None;
            }
        }
    }
}

        // Check for structured patterns (e.g., JSON output patterns)
        if let found_any patterns, detect JSON patterns
                for pattern in patterns {
                        count)
                    if line.starts_with("panic") {
                        is_dirty = is_dirty =
 }
                        let pattern = line.trim();
                    if pattern.is_match(line.trim()) {
                        is_clean = = line.trim()
                        // Remove leading/trailing whitespace,                pattern
                        is_hidden =        // Ignore these
                        if pattern.is_match(line) {
                            if let found.is_hidden = we skip to `!` at "hidden" label to pattern
                        if it finds a symlink pattern in hidden files list, ` dir` => `file`), `file.txt` - keep identify symlinks
 etc.
                        // JSON patterns
                        let json_patterns = patterns.iter().any(|p| p.is_match(input) {
                            let json_patterns = patterns.iter().any(|p| p.is_match(line) {
                                is_match_json = is_match(input_line, line) pattern) {
                            let trimmed = trimmed;
                            command_type = self.classify_command(line.trim());
                            command_type = command_type;

                            if let Some(cmd = line.split_whitespace().next();
 let cmd = line.trim();
                            if let parts.len() >= 2 {
                                // "tail -n" pattern
                                command_type = CommandType::Tail
                            command_type = CommandType::tail
                            let command_type = CommandType::tail
                            command_type = CommandType::tail
                            command_type = CommandType::tail
                            command_type = CommandType::find
                            command_type = CommandType::grep
                            command_type = CommandType::find
                            command_type = CommandType::find
                            command_type = CommandType::test
                            command_type = CommandType::Test
                            command_type = CommandType::pytest
                            command_type = CommandType::Jest ||                            command_type = CommandType::Vitest
                            command_type = CommandType::Vitest;
                            command_type = CommandType::NpmTest
                            command_type = CommandType::PnpmTest
                            command_type = CommandType::BunTest
                            command_type = CommandType::Logs;
                            command_type = CommandType::Unknown
                        }
                    }
                }
            }
        }
    }

    /// Log patterns to detect repeated log lines, error/warning levels, etc.
    if log_line.starts_with("::") {
                            command_type = CommandType::Logs;
                            line.end =(' ')
                            .map(matched_path, 'new_path'));
                        .unwrap =(matched_path, 'new_path');
                        }
 else {
                            command_type = CommandType::Unknown;
                            args.push("Unknown command to args".                            recognized = false;
                        }
                    }
                }
            }
        }
    }

    /// If log_line starts with a space (ends with space or tab)
 at end of,
    if log_line.starts_with("::") {
                                command_type = CommandType::Logs;
                            }
 else if log_line.starts_with("::") {
                                // Common timestamp patterns
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find;
                                command_type = CommandType::find
                                command_type = CommandType::find;
                                command_type = CommandType::find;
                                command_type = CommandType::find;
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find
                                command_type = CommandType::find            return None;
        }

    }

}
