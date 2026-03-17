use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::super::run::RunHandler;
use super::super::types::*;
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
    /// Handle the logs subcommand.
    pub(crate) fn handle_logs(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the log output
        let logs_output = Self::parse_logs(&input);

        // Format output based on the requested format
        let output = Self::format_logs(&logs_output, ctx.format);

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("logs")
                .with_output_mode(ctx.format)
                .with_input_bytes(input.len())
                .with_output_bytes(output.len())
                .with_items_processed(logs_output.total_lines)
                .with_extra("Debug", logs_output.debug_count.to_string())
                .with_extra("Info", logs_output.info_count.to_string())
                .with_extra("Warning", logs_output.warning_count.to_string())
                .with_extra("Error", logs_output.error_count.to_string())
                .with_extra("Fatal", logs_output.fatal_count.to_string())
                .with_extra(
                    "Repeated lines",
                    logs_output.repeated_lines.len().to_string(),
                );
            stats.print();
        }

        print!("{}", output);

        Ok(())
    }

    /// Parse log output into structured data.
    ///
    /// Supports various log formats:
    /// - Common timestamp formats (ISO 8601, syslog, etc.)
    /// - Log levels: DEBUG, INFO, WARN/WARNING, ERROR, FATAL/CRITICAL
    /// - Various formats: `[LEVEL]`, `LEVEL:`, `|LEVEL|`, etc.
    pub(crate) fn parse_logs(input: &str) -> LogsOutput {
        let mut logs_output = LogsOutput::default();
        let mut line_tracker: std::collections::HashMap<String, (usize, usize, usize)> =
            std::collections::HashMap::new();

        for (idx, line) in input.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            // Skip empty lines but count them
            if trimmed.is_empty() {
                continue;
            }

            // Track repeated lines
            let entry = line_tracker
                .entry(trimmed.to_string())
                .or_insert((0, line_num, line_num));
            entry.0 += 1;
            entry.2 = line_num;

            // Parse the log line
            let log_entry = Self::parse_log_line(trimmed, line_num);
            logs_output.entries.push(log_entry.clone());
            logs_output.total_lines += 1;

            // Count by level
            match log_entry.level {
                LogLevel::Debug => logs_output.debug_count += 1,
                LogLevel::Info => logs_output.info_count += 1,
                LogLevel::Warning => logs_output.warning_count += 1,
                LogLevel::Error => logs_output.error_count += 1,
                LogLevel::Fatal => logs_output.fatal_count += 1,
                LogLevel::Unknown => logs_output.unknown_count += 1,
            }

            // Track recent critical lines (ERROR and FATAL)
            if log_entry.level == LogLevel::Error || log_entry.level == LogLevel::Fatal {
                logs_output.recent_critical.push(log_entry.clone());
                // Keep only the most recent MAX_RECENT_CRITICAL entries
                if logs_output.recent_critical.len() > MAX_RECENT_CRITICAL {
                    logs_output.recent_critical.remove(0);
                }
            }
        }

        // Build repeated lines list (only lines repeated more than once)
        for (line, (count, first_line, last_line)) in line_tracker {
            if count > 1 {
                logs_output.repeated_lines.push(RepeatedLine {
                    line,
                    count,
                    first_line,
                    last_line,
                });
            }
        }

        // Sort repeated lines by first occurrence
        logs_output.repeated_lines.sort_by_key(|r| r.first_line);

        logs_output.is_empty = logs_output.entries.is_empty();
        logs_output
    }

    /// Parse a single log line.
    pub(crate) fn parse_log_line(line: &str, line_number: usize) -> LogEntry {
        let mut entry = LogEntry {
            line: line.to_string(),
            level: LogLevel::Unknown,
            timestamp: None,
            source: None,
            message: line.to_string(),
            line_number,
        };

        // Try to extract timestamp
        entry.timestamp = Self::extract_timestamp(line);

        // Try to extract log level
        entry.level = Self::detect_log_level(line);

        // Extract message (remove timestamp and level prefix)
        entry.message = Self::extract_message(line, &entry.timestamp, &entry.level);

        entry
    }

    /// Extract timestamp from a log line.
    pub(crate) fn extract_timestamp(line: &str) -> Option<String> {
        // Common timestamp patterns:
        // - ISO 8601: 2024-01-15T10:30:00, 2024-01-15 10:30:00
        // - Syslog: Jan 15 10:30:00
        // - Common: 2024/01/15 10:30:00, 01/15/2024 10:30:00
        // - Time only: 10:30:00, 10:30:00.123

        let chars: Vec<char> = line.chars().collect();

        // ISO 8601 with T separator: 2024-01-15T10:30:00
        // Format: YYYY-MM-DDTHH:MM:SS
        if chars.len() >= 19 {
            let potential = &line[..19.min(line.len())];
            if Self::is_iso8601_timestamp(potential) {
                // Check for milliseconds and timezone
                let mut end = 19;
                if line.len() > 19 {
                    let rest = &line[19..];
                    // Check for milliseconds
                    if rest.starts_with('.') {
                        let ms_end = rest
                            .find(|c: char| !c.is_ascii_digit())
                            .unwrap_or(rest.len().min(4));
                        end += 1 + ms_end;
                    }
                    // Check for timezone (Z or +/-HH:MM)
                    if end < line.len() {
                        let tz_part = &line[end..];
                        if tz_part.starts_with('Z') {
                            end += 1;
                        } else if tz_part.starts_with('+') || tz_part.starts_with('-') {
                            // Timezone offset like +00:00 or +0000
                            let tz_len =
                                if tz_part.len() >= 6 && tz_part.chars().nth(3) == Some(':') {
                                    6
                                } else if tz_part.len() >= 5 {
                                    5
                                } else {
                                    0
                                };
                            end += tz_len;
                        }
                    }
                }
                return Some(line[..end].to_string());
            }
        }

        // ISO 8601 with space separator: 2024-01-15 10:30:00
        if chars.len() >= 19 {
            let potential = &line[..19.min(line.len())];
            if Self::is_iso8601_space_timestamp(potential) {
                let mut end = 19;
                if line.len() > 19 && line[19..].starts_with('.') {
                    let rest = &line[19..];
                    let ms_end = rest
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(rest.len().min(4));
                    end += 1 + ms_end;
                }
                return Some(line[..end].to_string());
            }
        }

        // Slash date format: 2024/01/15 10:30:00
        if chars.len() >= 19 {
            let potential = &line[..19.min(line.len())];
            if Self::is_slash_date_timestamp(potential) {
                return Some(potential.to_string());
            }
        }

        // Syslog format: Jan 15 10:30:00
        if chars.len() >= 15 {
            let potential = &line[..15.min(line.len())];
            if Self::is_syslog_timestamp(potential) {
                return Some(potential.to_string());
            }
        }

        // Time only at start: 10:30:00 or 10:30:00.123
        if chars.len() >= 8 {
            let potential = &line[..8.min(line.len())];
            if Self::is_time_only(potential) {
                let mut end = 8;
                if line.len() > 8 && line[8..].starts_with('.') {
                    let rest = &line[8..];
                    let ms_end = rest
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(rest.len().min(4));
                    end += 1 + ms_end;
                }
                return Some(line[..end].to_string());
            }
        }

        None
    }

    /// Check if string is an ISO 8601 timestamp with T separator.
    pub(crate) fn is_iso8601_timestamp(s: &str) -> bool {
        // Format: YYYY-MM-DDTHH:MM:SS
        if s.len() < 19 {
            return false;
        }
        let bytes = s.as_bytes();
        // Check structure: XXXX-XX-XXTXX:XX:XX
        bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[10] == b'T'
            && bytes[13] == b':'
            && bytes[16] == b':'
            && bytes[0..4].iter().all(|b| b.is_ascii_digit())
            && bytes[5..7].iter().all(|b| b.is_ascii_digit())
            && bytes[8..10].iter().all(|b| b.is_ascii_digit())
            && bytes[11..13].iter().all(|b| b.is_ascii_digit())
            && bytes[14..16].iter().all(|b| b.is_ascii_digit())
            && bytes[17..19].iter().all(|b| b.is_ascii_digit())
    }

    /// Check if string is an ISO 8601 timestamp with space separator.
    pub(crate) fn is_iso8601_space_timestamp(s: &str) -> bool {
        // Format: YYYY-MM-DD HH:MM:SS
        if s.len() < 19 {
            return false;
        }
        let bytes = s.as_bytes();
        bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[10] == b' '
            && bytes[13] == b':'
            && bytes[16] == b':'
            && bytes[0..4].iter().all(|b| b.is_ascii_digit())
            && bytes[5..7].iter().all(|b| b.is_ascii_digit())
            && bytes[8..10].iter().all(|b| b.is_ascii_digit())
            && bytes[11..13].iter().all(|b| b.is_ascii_digit())
            && bytes[14..16].iter().all(|b| b.is_ascii_digit())
            && bytes[17..19].iter().all(|b| b.is_ascii_digit())
    }

    /// Check if string is a slash date timestamp.
    pub(crate) fn is_slash_date_timestamp(s: &str) -> bool {
        // Format: YYYY/MM/DD HH:MM:SS
        if s.len() < 19 {
            return false;
        }
        let bytes = s.as_bytes();
        bytes[4] == b'/'
            && bytes[7] == b'/'
            && bytes[10] == b' '
            && bytes[13] == b':'
            && bytes[16] == b':'
            && bytes[0..4].iter().all(|b| b.is_ascii_digit())
            && bytes[5..7].iter().all(|b| b.is_ascii_digit())
            && bytes[8..10].iter().all(|b| b.is_ascii_digit())
            && bytes[11..13].iter().all(|b| b.is_ascii_digit())
            && bytes[14..16].iter().all(|b| b.is_ascii_digit())
            && bytes[17..19].iter().all(|b| b.is_ascii_digit())
    }

    /// Check if string is a syslog timestamp.
    pub(crate) fn is_syslog_timestamp(s: &str) -> bool {
        // Format: Mon DD HH:MM:SS (e.g., "Jan 15 10:30:00")
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 3 {
            return false;
        }
        months.contains(&parts[0])
            && parts[1].parse::<u8>().is_ok()
            && parts[2].len() == 8
            && parts[2].contains(':')
    }

    /// Check if string is time only (HH:MM:SS).
    pub(crate) fn is_time_only(s: &str) -> bool {
        // Format: HH:MM:SS
        if s.len() < 8 {
            return false;
        }
        let bytes = s.as_bytes();
        bytes[2] == b':'
            && bytes[5] == b':'
            && bytes[0..2].iter().all(|b| b.is_ascii_digit())
            && bytes[3..5].iter().all(|b| b.is_ascii_digit())
            && bytes[6..8].iter().all(|b| b.is_ascii_digit())
    }

    /// Detect log level from a log line.
    pub(crate) fn detect_log_level(line: &str) -> LogLevel {
        let line_upper = line.to_uppercase();

        // Check for various level indicators in order of severity (highest first)
        // Patterns: [FATAL], FATAL:, |FATAL|, FATAL - etc.

        // Fatal/Critical - includes panic, crash, abort
        if Self::contains_level_marker(&line_upper, "FATAL")
            || Self::contains_level_marker(&line_upper, "CRITICAL")
            || Self::contains_level_marker(&line_upper, "CRIT")
            || Self::contains_error_keyword(&line_upper, "PANIC")
            || Self::contains_error_keyword(&line_upper, "CRASH")
            || Self::contains_error_keyword(&line_upper, "ABORT")
            || Self::contains_error_keyword(&line_upper, "EMERG")
            || Self::contains_error_keyword(&line_upper, "ALERT")
        {
            return LogLevel::Fatal;
        }

        // Error - includes exceptions, failures, and common error patterns
        if Self::contains_level_marker(&line_upper, "ERROR")
            || Self::contains_level_marker(&line_upper, "ERR")
            || Self::contains_error_keyword(&line_upper, "EXCEPTION")
            || Self::contains_error_keyword(&line_upper, "FAILED")
            || Self::contains_error_keyword(&line_upper, "FAILURE")
            || Self::contains_error_keyword(&line_upper, "STACK TRACE")
            || Self::contains_error_keyword(&line_upper, "BACKTRACE")
            || Self::contains_error_keyword(&line_upper, "SEGFAULT")
            || Self::contains_error_keyword(&line_upper, "SEG FAULT")
            || Self::contains_error_keyword(&line_upper, "NULL POINTER")
            || Self::contains_error_keyword(&line_upper, "ACCESS DENIED")
            || Self::contains_error_keyword(&line_upper, "TIMEOUT ERROR")
            || Self::contains_error_keyword(&line_upper, "CONNECTION REFUSED")
            || Self::contains_error_keyword(&line_upper, "CONNECTION ERROR")
        {
            return LogLevel::Error;
        }

        // Warning - includes deprecation, caution notices
        if Self::contains_level_marker(&line_upper, "WARN")
            || Self::contains_level_marker(&line_upper, "WARNING")
            || Self::contains_warning_keyword(&line_upper, "DEPRECATED")
            || Self::contains_warning_keyword(&line_upper, "CAUTION")
            || Self::contains_warning_keyword(&line_upper, "ATTENTION")
            || Self::contains_warning_keyword(&line_upper, "BE AWARE")
            || Self::contains_warning_keyword(&line_upper, "PLEASE NOTE")
            || Self::contains_warning_keyword(&line_upper, "SLOW QUERY")
            || Self::contains_warning_keyword(&line_upper, "SLOW REQUEST")
        {
            return LogLevel::Warning;
        }

        // Info
        if Self::contains_level_marker(&line_upper, "INFO")
            || Self::contains_level_marker(&line_upper, "NOTICE")
        {
            return LogLevel::Info;
        }

        // Debug
        if Self::contains_level_marker(&line_upper, "DEBUG")
            || Self::contains_level_marker(&line_upper, "TRACE")
            || Self::contains_level_marker(&line_upper, "VERBOSE")
        {
            return LogLevel::Debug;
        }

        LogLevel::Unknown
    }

    /// Check if line contains an error-related keyword.
    /// This is more lenient than contains_level_marker and looks for keywords
    /// anywhere in the line that typically indicate an error condition.
    pub(crate) fn contains_error_keyword(line_upper: &str, keyword: &str) -> bool {
        // Check for the keyword with word boundaries
        if line_upper.contains(keyword) {
            // Avoid false positives by checking context
            // For example, "no errors" should not be detected as an error
            let keyword_lower = keyword.to_lowercase();
            let negation_patterns = [
                format!("no {}", keyword_lower),
                format!("without {}", keyword_lower),
                format!("not {}", keyword_lower),
                format!("0 {}", keyword_lower),
                format!("zero {}", keyword_lower),
            ];
            for neg in negation_patterns {
                if line_upper.contains(&neg.to_uppercase()) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    /// Check if line contains a warning-related keyword.
    pub(crate) fn contains_warning_keyword(line_upper: &str, keyword: &str) -> bool {
        line_upper.contains(keyword)
    }

    /// Check if line contains a level marker.
    pub(crate) fn contains_level_marker(line_upper: &str, level: &str) -> bool {
        // Check for patterns like [LEVEL], LEVEL:, |LEVEL|, <LEVEL>, (LEVEL)
        // These are precise patterns that indicate a log level
        let patterns = [
            format!("[{}]", level),
            format!("{}:", level),
            format!("|{}|", level),
            format!("<{}>", level),
            format!("({})", level),
            format!("{} -", level),
            format!("{}]", level), // Level followed by closing bracket
        ];

        for pattern in patterns {
            if line_upper.contains(&pattern) {
                return true;
            }
        }

        // Check if line starts with level followed by space or colon
        if line_upper.starts_with(level) {
            let after_level = &line_upper[level.len()..];
            if after_level.starts_with(':')
                || after_level.starts_with(' ')
                || after_level.is_empty()
            {
                return true;
            }
        }

        false
    }

    /// Extract message by removing timestamp and level prefix.
    pub(crate) fn extract_message(line: &str, timestamp: &Option<String>, level: &LogLevel) -> String {
        let mut message = line.to_string();

        // Remove timestamp prefix
        if let Some(ts) = timestamp {
            if message.starts_with(ts) {
                message = message[ts.len()..].to_string();
            }
        }

        // Trim leading whitespace after timestamp removal
        message = message.trim_start().to_string();

        // Remove common level prefixes
        let level_patterns: &[&str] = match level {
            LogLevel::Debug => &[
                "[DEBUG]", "DEBUG:", "|DEBUG|", "<DEBUG>", "(DEBUG)", "DEBUG -", "DEBUG ",
            ],
            LogLevel::Info => &[
                "[INFO]", "INFO:", "|INFO|", "<INFO>", "(INFO)", "INFO -", "INFO ",
            ],
            LogLevel::Warning => &[
                "[WARN]",
                "[WARNING]",
                "WARN:",
                "WARNING:",
                "|WARN|",
                "|WARNING|",
                "<WARN>",
                "<WARNING>",
                "(WARN)",
                "(WARNING)",
                "WARN -",
                "WARNING -",
                "WARN ",
                "WARNING ",
            ],
            LogLevel::Error => &[
                "[ERROR]", "ERROR:", "|ERROR|", "<ERROR>", "(ERROR)", "ERROR -", "ERROR ", "[ERR]",
                "ERR:", "ERR ",
            ],
            LogLevel::Fatal => &[
                "[FATAL]",
                "FATAL:",
                "|FATAL|",
                "<FATAL>",
                "(FATAL)",
                "FATAL -",
                "FATAL ",
                "[CRITICAL]",
                "CRITICAL:",
                "[CRIT]",
                "CRIT:",
            ],
            LogLevel::Unknown => &[],
        };

        for pattern in level_patterns {
            let pattern_upper = pattern.to_uppercase();
            let message_upper = message.to_uppercase();
            if message_upper.starts_with(&pattern_upper) {
                message = message[pattern.len()..].to_string();
                break;
            }
        }

        // Clean up leading whitespace and separators
        message = message.trim().to_string();
        if message.starts_with('-') || message.starts_with(':') || message.starts_with(']') {
            message = message[1..].trim().to_string();
        }

        message
    }

    /// Format logs output for display.
    pub(crate) fn format_logs(logs_output: &LogsOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_logs_json(logs_output),
            OutputFormat::Csv => Self::format_logs_csv(logs_output),
            OutputFormat::Tsv => Self::format_logs_tsv(logs_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_logs_compact(logs_output),
            OutputFormat::Raw => Self::format_logs_raw(logs_output),
        }
    }

    /// Format logs output as JSON.
    pub(crate) fn format_logs_json(logs_output: &LogsOutput) -> String {
        let total_critical = logs_output.error_count + logs_output.fatal_count;
        serde_json::json!({
            "counts": {
                "total_lines": logs_output.total_lines,
                "debug": logs_output.debug_count,
                "info": logs_output.info_count,
                "warning": logs_output.warning_count,
                "error": logs_output.error_count,
                "fatal": logs_output.fatal_count,
                "unknown": logs_output.unknown_count,
            },
            "repeated_lines": logs_output.repeated_lines.iter().map(|r| serde_json::json!({
                "line": r.line,
                "count": r.count,
                "first_line": r.first_line,
                "last_line": r.last_line,
            })).collect::<Vec<_>>(),
            "recent_critical": logs_output.recent_critical.iter().map(|e| serde_json::json!({
                "line_number": e.line_number,
                "level": match e.level {
                    LogLevel::Debug => "debug",
                    LogLevel::Info => "info",
                    LogLevel::Warning => "warning",
                    LogLevel::Error => "error",
                    LogLevel::Fatal => "fatal",
                    LogLevel::Unknown => "unknown",
                },
                "timestamp": e.timestamp,
                "message": e.message,
            })).collect::<Vec<_>>(),
            "recent_critical_count": logs_output.recent_critical.len(),
            "total_critical": total_critical,
            "entries": logs_output.entries.iter().map(|e| serde_json::json!({
                "line_number": e.line_number,
                "level": match e.level {
                    LogLevel::Debug => "debug",
                    LogLevel::Info => "info",
                    LogLevel::Warning => "warning",
                    LogLevel::Error => "error",
                    LogLevel::Fatal => "fatal",
                    LogLevel::Unknown => "unknown",
                },
                "timestamp": e.timestamp,
                "message": e.message,
            })).collect::<Vec<_>>(),
        })
        .to_string()
    }

    /// Format logs output as CSV.
    pub(crate) fn format_logs_csv(logs_output: &LogsOutput) -> String {
        let mut result = String::new();
        result.push_str("line_number,level,timestamp,message\n");

        for entry in &logs_output.entries {
            let level_str = match entry.level {
                LogLevel::Debug => "debug",
                LogLevel::Info => "info",
                LogLevel::Warning => "warning",
                LogLevel::Error => "error",
                LogLevel::Fatal => "fatal",
                LogLevel::Unknown => "unknown",
            };
            let timestamp = entry.timestamp.as_deref().unwrap_or("");
            let message_escaped = RunHandler::escape_csv_field(&entry.message);
            result.push_str(&format!(
                "{},{},{},{}\n",
                entry.line_number, level_str, timestamp, message_escaped
            ));
        }

        result
    }

    /// Format logs output as TSV.
    pub(crate) fn format_logs_tsv(logs_output: &LogsOutput) -> String {
        let mut result = String::new();
        result.push_str("line_number\tlevel\ttimestamp\tmessage\n");

        for entry in &logs_output.entries {
            let level_str = match entry.level {
                LogLevel::Debug => "debug",
                LogLevel::Info => "info",
                LogLevel::Warning => "warning",
                LogLevel::Error => "error",
                LogLevel::Fatal => "fatal",
                LogLevel::Unknown => "unknown",
            };
            let timestamp = entry.timestamp.as_deref().unwrap_or("");
            let message_escaped = RunHandler::escape_tsv_field(&entry.message);
            result.push_str(&format!(
                "{}\t{}\t{}\t{}\n",
                entry.line_number, level_str, timestamp, message_escaped
            ));
        }

        result
    }

    /// Format logs output in compact format.
    pub(crate) fn format_logs_compact(logs_output: &LogsOutput) -> String {
        let mut output = String::new();

        if logs_output.is_empty {
            output.push_str("logs: empty\n");
            return output;
        }

        // Summary header
        output.push_str(&format!("lines: {}\n", logs_output.total_lines));

        // Level summary (only show non-zero counts)
        let mut level_parts = Vec::new();
        if logs_output.fatal_count > 0 {
            level_parts.push(format!("fatal:{}", logs_output.fatal_count));
        }
        if logs_output.error_count > 0 {
            level_parts.push(format!("error:{}", logs_output.error_count));
        }
        if logs_output.warning_count > 0 {
            level_parts.push(format!("warn:{}", logs_output.warning_count));
        }
        if logs_output.info_count > 0 {
            level_parts.push(format!("info:{}", logs_output.info_count));
        }
        if logs_output.debug_count > 0 {
            level_parts.push(format!("debug:{}", logs_output.debug_count));
        }
        if logs_output.unknown_count > 0 {
            level_parts.push(format!("other:{}", logs_output.unknown_count));
        }

        if !level_parts.is_empty() {
            output.push_str(&format!("levels: {}\n", level_parts.join(", ")));
        }

        // Repeated lines summary
        if !logs_output.repeated_lines.is_empty() {
            output.push_str(&format!(
                "repeated: {} unique lines ({} occurrences)\n",
                logs_output.repeated_lines.len(),
                logs_output
                    .repeated_lines
                    .iter()
                    .map(|r| r.count)
                    .sum::<usize>()
            ));
        }

        output.push('\n');

        // Show repeated lines
        if !logs_output.repeated_lines.is_empty() {
            output.push_str("repeated lines:\n");
            for repeated in &logs_output.repeated_lines {
                if repeated.count > 1 {
                    let preview = if repeated.line.len() > 60 {
                        format!("{}...", &repeated.line[..57])
                    } else {
                        repeated.line.clone()
                    };
                    output.push_str(&format!(
                        "  [x{}] {} (lines {}-{})\n",
                        repeated.count, preview, repeated.first_line, repeated.last_line
                    ));
                }
            }
            output.push('\n');
        }

        // Show recent critical lines (ERROR and FATAL)
        if !logs_output.recent_critical.is_empty() {
            let total_critical = logs_output.error_count + logs_output.fatal_count;
            let shown = logs_output.recent_critical.len();
            if shown < total_critical {
                output.push_str(&format!(
                    "recent critical ({} of {}):\n",
                    shown, total_critical
                ));
            } else {
                output.push_str(&format!("recent critical ({}):\n", shown));
            }
            for entry in &logs_output.recent_critical {
                let level_indicator = match entry.level {
                    LogLevel::Error => "[E]",
                    LogLevel::Fatal => "[F]",
                    _ => "[!]",
                };
                let preview = if entry.message.len() > 80 {
                    format!("{}...", &entry.message[..77])
                } else {
                    entry.message.clone()
                };
                output.push_str(&format!(
                    "  {} {} {}\n",
                    level_indicator, entry.line_number, preview
                ));
            }
            output.push('\n');
        }

        // Show entries with detected levels (collapse consecutive duplicates)
        let has_levels = logs_output
            .entries
            .iter()
            .any(|e| e.level != LogLevel::Unknown);
        if has_levels {
            output.push_str("entries:\n");
            // Collapse consecutive entries with same level and message
            let mut i = 0;
            while i < logs_output.entries.len() {
                let entry = &logs_output.entries[i];
                let level_indicator = match entry.level {
                    LogLevel::Debug => "[D]",
                    LogLevel::Info => "[I]",
                    LogLevel::Warning => "[W]",
                    LogLevel::Error => "[E]",
                    LogLevel::Fatal => "[F]",
                    LogLevel::Unknown => "   ",
                };

                // Count consecutive entries with same level and message
                let mut count = 1;
                let mut last_line = entry.line_number;
                while i + count < logs_output.entries.len() {
                    let next = &logs_output.entries[i + count];
                    if next.level == entry.level && next.message == entry.message {
                        count += 1;
                        last_line = next.line_number;
                    } else {
                        break;
                    }
                }

                let preview = if entry.message.len() > 80 {
                    format!("{}...", &entry.message[..77])
                } else {
                    entry.message.clone()
                };

                if count > 1 {
                    output.push_str(&format!(
                        "{} {}-{} {} [x{}]\n",
                        level_indicator, entry.line_number, last_line, preview, count
                    ));
                } else {
                    output.push_str(&format!(
                        "{} {} {}\n",
                        level_indicator, entry.line_number, preview
                    ));
                }

                i += count;
            }
        } else {
            // No levels detected, just show raw lines with line numbers (collapse consecutive duplicates)
            output.push_str("lines:\n");
            let mut i = 0;
            while i < logs_output.entries.len() {
                let entry = &logs_output.entries[i];

                // Count consecutive entries with same line content
                let mut count = 1;
                let mut last_line = entry.line_number;
                while i + count < logs_output.entries.len() {
                    let next = &logs_output.entries[i + count];
                    if next.line == entry.line {
                        count += 1;
                        last_line = next.line_number;
                    } else {
                        break;
                    }
                }

                let preview = if entry.line.len() > 80 {
                    format!("{}...", &entry.line[..77])
                } else {
                    entry.line.clone()
                };

                if count > 1 {
                    output.push_str(&format!(
                        "  {}-{} {} [x{}]\n",
                        entry.line_number, last_line, preview, count
                    ));
                } else {
                    output.push_str(&format!("  {} {}\n", entry.line_number, preview));
                }

                i += count;
            }
        }

        output
    }

    /// Format logs output as raw (original format).
    pub(crate) fn format_logs_raw(logs_output: &LogsOutput) -> String {
        let mut output = String::new();

        for entry in &logs_output.entries {
            output.push_str(&entry.line);
            output.push('\n');
        }

        output
    }
}
