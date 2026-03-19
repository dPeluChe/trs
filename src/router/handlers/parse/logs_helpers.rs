//! Helper functions for log parsing: timestamp extraction, log level detection,
//! and message extraction.

use super::super::types::*;
use super::ParseHandler;

impl ParseHandler {
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
    pub(crate) fn extract_message(
        line: &str,
        timestamp: &Option<String>,
        level: &LogLevel,
    ) -> String {
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
}
