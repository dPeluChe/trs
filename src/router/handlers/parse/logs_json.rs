//! Structured-log (one-JSON-object-per-line) field extraction.
//!
//! pino, bunyan, winston, zap, zerolog, slog, python-json-logger and
//! `journalctl -o json` emit a JSON object per line where every key repeats —
//! large token bloat and unreadable for humans. We pull the human-relevant
//! fields (level, message, timestamp, logger, error) and drop the rest so the
//! existing level counting / fold / recent-critical logic works on the clean
//! message. Non-JSON or JSON lacking any log-shaped field returns None, so
//! arbitrary JSON the agent actually wanted passes through untouched.

use super::super::types::*;
use super::ParseHandler;

const MSG_KEYS: &[&str] = &["msg", "message", "text", "event", "short_message", "@m"];
const TS_KEYS: &[&str] = &[
    "ts",
    "time",
    "timestamp",
    "@timestamp",
    "@t",
    "datetime",
    "asctime",
];
const SRC_KEYS: &[&str] = &["logger", "name", "module", "caller", "source", "component"];
const ERR_KEYS: &[&str] = &["error", "err", "exception"];
const LEVEL_KEYS: &[&str] = &[
    "level",
    "lvl",
    "severity",
    "levelname",
    "loglevel",
    "level_name",
    "@l",
];

impl ParseHandler {
    pub(crate) fn try_parse_json_log_line(line: &str, line_number: usize) -> Option<LogEntry> {
        let t = line.trim();
        if !(t.starts_with('{') && t.ends_with('}')) {
            return None;
        }
        let value: serde_json::Value = serde_json::from_str(t).ok()?;
        let obj = value.as_object()?;

        let level = Self::json_log_level(obj);
        let msg = Self::json_first_str(obj, MSG_KEYS);
        // Require a log shape (a level or a message field); otherwise it's data.
        if level == LogLevel::Unknown && msg.is_none() {
            return None;
        }

        let timestamp = Self::json_first_str(obj, TS_KEYS);
        let source = Self::json_first_str(obj, SRC_KEYS);

        let mut message = msg.unwrap_or_else(|| "(no message)".to_string());
        if let Some(err) = Self::json_first_str(obj, ERR_KEYS) {
            let err = err.trim();
            if !message.contains(err) {
                let clipped: String = err.chars().take(160).collect();
                message.push_str(" | err: ");
                message.push_str(clipped.trim());
            }
        }
        if let Some(src) = &source {
            message.push_str(&format!(" ({src})"));
        }

        Some(LogEntry {
            line: t.to_string(),
            level,
            timestamp,
            source,
            message,
            line_number,
        })
    }

    fn json_first_str(
        obj: &serde_json::Map<String, serde_json::Value>,
        keys: &[&str],
    ) -> Option<String> {
        for k in keys {
            match obj.get(*k) {
                Some(serde_json::Value::String(s)) if !s.is_empty() => return Some(s.clone()),
                Some(serde_json::Value::Number(n)) => return Some(n.to_string()),
                _ => {}
            }
        }
        None
    }

    fn json_log_level(obj: &serde_json::Map<String, serde_json::Value>) -> LogLevel {
        for k in LEVEL_KEYS {
            match obj.get(*k) {
                Some(serde_json::Value::String(s)) => {
                    return Self::level_from_keyword(&s.to_uppercase())
                }
                Some(serde_json::Value::Number(n)) => {
                    if let Some(u) = n.as_u64() {
                        return Self::level_from_number(u);
                    }
                }
                _ => {}
            }
        }
        LogLevel::Unknown
    }

    /// JSON already isolated the level token, so match the bare word (no marker scan).
    fn level_from_keyword(up: &str) -> LogLevel {
        if up.starts_with("FATAL")
            || up.starts_with("CRIT")
            || up.starts_with("PANIC")
            || up.starts_with("EMERG")
            || up.starts_with("ALERT")
        {
            LogLevel::Fatal
        } else if up.starts_with("ERR") {
            LogLevel::Error
        } else if up.starts_with("WARN") {
            LogLevel::Warning
        } else if up.starts_with("INFO") || up.starts_with("NOTICE") {
            LogLevel::Info
        } else if up.starts_with("DEBUG") || up.starts_with("TRACE") || up.starts_with("DBG") {
            LogLevel::Debug
        } else {
            LogLevel::Unknown
        }
    }

    /// Numeric levels: pino/bunyan (10..=60) and syslog severity (0..=7).
    fn level_from_number(n: u64) -> LogLevel {
        match n {
            60 => LogLevel::Fatal,
            50 => LogLevel::Error,
            40 => LogLevel::Warning,
            30 => LogLevel::Info,
            10 | 20 => LogLevel::Debug,
            0..=2 => LogLevel::Fatal,
            3 => LogLevel::Error,
            4 => LogLevel::Warning,
            5..=6 => LogLevel::Info,
            7 => LogLevel::Debug,
            _ => LogLevel::Unknown,
        }
    }
}

#[cfg(test)]
#[path = "logs_json_tests.rs"]
mod tests;
