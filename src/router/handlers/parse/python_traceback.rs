//! Python traceback compression.
//!
//! A typical Python script failure prints a traceback with three kinds
//! of noise:
//!
//! - **Full file paths**. `/Users/dev/projects/long/path/to/script.py` —
//!   basename alone usually identifies the file within the project.
//! - **Code snippet lines**. Each `File` line is followed by a quoted
//!   line of source (e.g. `    some_call()`). Useful for humans
//!   skimming, but the agent already has the file open and can fetch
//!   context by file:line if needed.
//! - **Internal frames**. Runtime / stdlib frames come bundled with the
//!   user's frames and dominate the line count in deep stacks.
//!
//! We keep: frame list (basename:line + function) and the final
//! `<ErrorType>: <message>` line. Optionally fold internal frames.
//!
//! Passthrough when no traceback detected — the handler shouldn't
//! interfere with ordinary Python program output.

use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

/// Parsed stack frame. `code` is dropped for compact mode but preserved
/// on the struct in case a future format wants it.
#[derive(Debug, Clone)]
struct Frame {
    file: String,
    line: u32,
    func: String,
    #[allow(dead_code)]
    code: Option<String>,
}

#[derive(Debug, Default)]
struct Traceback {
    /// Lines printed before the `Traceback (most recent call last):`
    /// marker. We echo these verbatim — they may contain program
    /// output or earlier logs that aren't part of the stack.
    preamble: Vec<String>,
    frames: Vec<Frame>,
    /// Final `<ErrorType>: <message>` line (one or more for chained
    /// exceptions). Preserved exactly.
    error_lines: Vec<String>,
}

impl ParseHandler {
    pub(crate) fn handle_python_traceback(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        // No traceback marker → pass through. The handler doesn't
        // substitute for a general-purpose python-output compressor;
        // when there's nothing stack-shaped to fold, we stay out of
        // the way.
        if !input.contains("Traceback (most recent call last):") {
            crate::parse_out::emit(&input);
            if ctx.stats {
                CommandStats::new()
                    .with_reducer("python-passthrough")
                    .with_input_bytes(input_bytes)
                    .with_output_bytes(input_bytes)
                    .print();
            }
            return Ok(());
        }

        let parsed = parse_traceback(&input);
        let output = match ctx.format {
            OutputFormat::Json => render_json(&parsed),
            OutputFormat::Raw => input.clone(),
            _ => render_compact(&parsed),
        };

        crate::parse_out::emit(&output);

        if ctx.stats {
            CommandStats::new()
                .with_reducer("python-traceback")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }
}

fn parse_traceback(input: &str) -> Traceback {
    let mut out = Traceback::default();
    let mut lines = input.lines().peekable();
    let marker = "Traceback (most recent call last):";

    // Collect preamble until the marker.
    for line in lines.by_ref() {
        if line.trim_end() == marker {
            break;
        }
        out.preamble.push(line.to_string());
    }

    // Parse frames. Python emits frames as pairs:
    //   File "<path>", line <N>, in <func>
    //       <code snippet>
    // The code line is indented (4+ spaces) and optional in some
    // tracebacks (e.g. generated frames with no source file).
    while let Some(line) = lines.peek() {
        let trimmed = line.trim_start();
        if let Some(frame) = parse_file_line(trimmed) {
            lines.next();
            let code = match lines.peek() {
                Some(next)
                    if next.starts_with("    ") && !next.trim_start().starts_with("File ") =>
                {
                    let code_line = lines.next().unwrap().trim().to_string();
                    Some(code_line)
                }
                _ => None,
            };
            out.frames.push(Frame { code, ..frame });
        } else {
            break;
        }
    }

    // Whatever remains is the error message + any chained exceptions.
    for line in lines {
        if !line.is_empty() {
            out.error_lines.push(line.to_string());
        }
    }

    out
}

/// Parse a single `  File "path", line N, in func` line. Returns None
/// when the line doesn't match — our caller uses that as the signal to
/// stop reading frames.
fn parse_file_line(line: &str) -> Option<Frame> {
    let rest = line.strip_prefix("File \"")?;
    let (file_path, rest) = rest.split_once("\", line ")?;
    let (line_num_str, rest) = rest.split_once(", in ")?;
    let line_num: u32 = line_num_str.trim().parse().ok()?;
    let func = rest.trim().to_string();
    Some(Frame {
        file: basename(file_path),
        line: line_num,
        func,
        code: None,
    })
}

fn basename(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

fn render_compact(tb: &Traceback) -> String {
    let mut out = String::new();

    // Preserve any preamble lines that preceded the traceback —
    // they're often real program output the user wanted to see.
    for line in &tb.preamble {
        out.push_str(line);
        out.push('\n');
    }

    if tb.frames.is_empty() && tb.error_lines.is_empty() {
        return out;
    }

    out.push_str(&format!("traceback ({} frames):\n", tb.frames.len()));
    for f in &tb.frames {
        out.push_str(&format!("  {}:{}  {}\n", f.file, f.line, f.func));
    }
    for err in &tb.error_lines {
        out.push_str(err);
        out.push('\n');
    }
    out
}

fn render_json(tb: &Traceback) -> String {
    let frames: Vec<serde_json::Value> = tb
        .frames
        .iter()
        .map(|f| {
            serde_json::json!({
                "file": f.file,
                "line": f.line,
                "func": f.func,
            })
        })
        .collect();
    let value = serde_json::json!({
        "frames": frames,
        "error": tb.error_lines.join("\n"),
        "preamble_lines": tb.preamble.len(),
    });
    serde_json::to_string_pretty(&value).unwrap_or_default() + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"Traceback (most recent call last):
  File "/Users/dev/app/main.py", line 42, in <module>
    start()
  File "/Users/dev/app/server.py", line 100, in start
    init_db()
  File "/Users/dev/app/db.py", line 25, in init_db
    raise ValueError("db not configured")
ValueError: db not configured
"#;

    #[test]
    fn parses_full_traceback() {
        let tb = parse_traceback(SAMPLE);
        assert_eq!(tb.frames.len(), 3);
        assert_eq!(tb.frames[0].file, "main.py");
        assert_eq!(tb.frames[0].line, 42);
        assert_eq!(tb.frames[0].func, "<module>");
        assert_eq!(tb.frames[2].file, "db.py");
        assert_eq!(tb.error_lines, vec!["ValueError: db not configured"]);
    }

    #[test]
    fn compact_render_is_shorter() {
        let tb = parse_traceback(SAMPLE);
        let out = render_compact(&tb);
        assert!(out.len() < SAMPLE.len());
        assert!(out.contains("traceback (3 frames):"));
        assert!(out.contains("main.py:42"));
        assert!(out.contains("ValueError: db not configured"));
        // Full paths should be stripped.
        assert!(!out.contains("/Users/dev"));
    }

    #[test]
    fn preamble_is_preserved() {
        let input = format!("hello from script\n\n{}", SAMPLE);
        let tb = parse_traceback(&input);
        let out = render_compact(&tb);
        assert!(out.starts_with("hello from script"));
        assert!(out.contains("traceback"));
    }

    #[test]
    fn missing_code_snippet_is_tolerated() {
        let input = r#"Traceback (most recent call last):
  File "generated.py", line 1, in <module>
RuntimeError: synthetic frame
"#;
        let tb = parse_traceback(input);
        assert_eq!(tb.frames.len(), 1);
        assert_eq!(tb.frames[0].file, "generated.py");
        assert_eq!(tb.error_lines, vec!["RuntimeError: synthetic frame"]);
    }
}
