//! What a parsed summary may claim once a child has run: its exit status is
//! the verdict, and a parser only describes the output.

use serde_json::Value;

/// `parsed` with the child's verdict, plus `full_output` when `raw` yields a
/// path. These live in the object because a text footer after it would break
/// the document. On success `success` only flips when the parser read nothing:
/// any other object may be the child's own data. Unchanged objects are
/// returned as written, since re-serializing sorts keys. None when `parsed`
/// is not a JSON object.
pub(crate) fn reconcile_json(
    parsed: &str,
    code: i32,
    raw: impl Fn() -> Option<String>,
) -> Option<String> {
    let parsed = parsed.trim();
    if !parsed.starts_with('{') {
        return None;
    }
    let mut value: Value = serde_json::from_str(parsed).ok()?;
    let object = value.as_object_mut()?;
    let failed = code != 0;
    let flag = |key: &str| object.get(key).and_then(Value::as_bool);
    let misread_pass = !failed && flag("success") == Some(false) && flag("is_empty") == Some(true);
    let raw = raw();
    if !failed && !misread_pass && raw.is_none() {
        return Some(parsed.to_string());
    }
    if failed {
        object.insert("exit_code".to_string(), Value::from(code));
    }
    if failed || misread_pass {
        object.insert("success".to_string(), Value::Bool(!failed));
    }
    if let Some(path) = raw {
        object.insert("full_output".to_string(), Value::from(path));
    }
    Some(value.to_string())
}

/// `text` as the single document `--json` promises. A JSON array the child
/// printed is its data and stays as written unless it needs a verdict; any
/// other text is wrapped.
pub(crate) fn json_document(text: &str, code: i32, raw: impl Fn() -> Option<String>) -> String {
    if let Some(object) = reconcile_json(text, code, &raw) {
        return object;
    }
    let path = raw();
    let trimmed = text.trim();
    if code == 0
        && path.is_none()
        && trimmed.starts_with('[')
        && serde_json::from_str::<Value>(trimmed).is_ok()
    {
        return trimmed.to_string();
    }
    let wrapped = serde_json::json!({ "output": text, "success": true }).to_string();
    reconcile_json(&wrapped, code, || path.clone()).unwrap_or(wrapped)
}

/// `document` with the child's stderr, where `2>&1` cannot split it from the
/// JSON. An array or a `stderr` key the child printed is its data, so that
/// stderr stays on stderr.
fn with_stderr(document: String, stderr: &str) -> String {
    if stderr.is_empty() {
        return document;
    }
    match serde_json::from_str(&document) {
        Ok(Value::Object(mut object)) if !object.contains_key("stderr") => {
            object.insert("stderr".to_string(), stderr.into());
            Value::Object(object).to_string()
        }
        _ => {
            eprint!("{stderr}");
            document
        }
    }
}

/// What ending a `--json` run needs: stdout holds one document, so the exit
/// status and the raw path go inside it rather than in a footer after it.
pub(crate) struct JsonRun<'a> {
    pub(crate) fcmd: String,
    pub(crate) stdout: &'a str,
    pub(crate) stderr: &'a str,
    /// The part of `stderr` the summary did not read.
    pub(crate) unread_stderr: &'a str,
    pub(crate) code: i32,
    pub(crate) in_bytes: usize,
    pub(crate) start: std::time::Instant,
    pub(crate) status: &'a std::process::ExitStatus,
}

impl JsonRun<'_> {
    /// The unread stderr at the size of a passthrough, and whether it lost text.
    fn trimmed_stderr(&self) -> (String, bool) {
        let max = crate::config::config().limits.passthrough_max_chars;
        let clean = crate::classifier_exec::generic_compress(self.unread_stderr);
        let dropped = crate::parse_out::take_dropped();
        match crate::passthrough_cut::cut(&clean, max) {
            Some(cut) => (cut, true),
            None => (clean, dropped),
        }
    }

    pub(crate) fn finish(&self, text: &str, cut: bool) -> ! {
        let (unread, unread_cut) = self.trimmed_stderr();
        let wants_raw = self.code != 0 || cut || unread_cut;
        let raw = || {
            wants_raw
                .then(|| {
                    crate::classifier_exec::save_tee_output(&self.fcmd, self.stdout, self.stderr)
                })
                .flatten()
        };
        let json = with_stderr(json_document(text, self.code, raw), &unread);
        println!("{json}");
        crate::classifier_exec::track(
            &self.fcmd,
            self.in_bytes,
            json.len(),
            self.start,
            self.status,
        );
        std::process::exit(self.code);
    }
}

/// A failure with no child output to wrap. `--json` promises one document on
/// stdout, so nothing goes to stderr where `2>&1` would break it.
pub(crate) fn exit_with_error(format: crate::OutputFormat, message: &str, code: i32) -> ! {
    if format == crate::OutputFormat::Json {
        let json = crate::formatter::JsonFormatter::format_error_with_code(message, code);
        println!("{json}");
    } else {
        eprintln!("{message}");
    }
    std::process::exit(code);
}

/// The format of a command that takes its own `--json` flag.
pub(crate) fn format_of(json: bool) -> crate::OutputFormat {
    if json {
        crate::OutputFormat::Json
    } else {
        crate::OutputFormat::Compact
    }
}

/// Report a command that never started: 127 and 126, as in a shell.
pub(crate) fn exit_on_spawn_failure(
    format: crate::OutputFormat,
    cmd: &str,
    error: &std::io::Error,
) -> ! {
    let (code, message) = crate::process::spawn_failure(cmd, error);
    exit_with_error(format, &message, code)
}

/// A trs line on stdout, except where it would be a bogus row in a table.
pub(crate) fn notice(format: crate::OutputFormat, line: &str) {
    if matches!(format, crate::OutputFormat::Csv | crate::OutputFormat::Tsv) {
        eprint!("{line}");
    } else {
        print!("{line}");
    }
}

/// `[trs] exit N`, with the raw path when there is one.
pub(crate) fn exit_line(code: i32, raw: Option<&str>) -> String {
    match raw {
        Some(path) => format!("[trs] exit {} · full output: {}\n", code, path),
        None => format!("[trs] exit {}\n", code),
    }
}

/// Terminal failure footer. Goes through `notice` and never returns.
///
/// stdout, not stderr: agents run commands with `2>/dev/null` to keep their
/// context clean, which silently drops a stderr-only notice exactly when it
/// matters most. The exit code makes the status verifiable instead of
/// inferred, and the tee path is the escape hatch to the full raw output.
pub(crate) fn emit_failure_footer(
    format: crate::OutputFormat,
    code: i32,
    fcmd: &str,
    stdout: &str,
    stderr: &str,
) -> ! {
    let raw = crate::classifier_exec::save_tee_output(fcmd, stdout, stderr);
    notice(format, &exit_line(code, raw.as_deref()));
    std::process::exit(code);
}

/// Does this compressed output actually show that something went wrong?
/// Backstop for parsers that summarize without consulting the exit status —
/// when the answer is "no" we fall back to raw, so a miss costs tokens, never
/// truth.
///
/// Counts of zero don't count: "0 errors" contains "error" while asserting the
/// opposite, and reading it as failure evidence is what lets a clean summary
/// stand in for a failed command.
pub(crate) fn failure_is_visible(parsed: &str) -> bool {
    if parsed.contains('✗') {
        return true;
    }
    let lower = parsed.to_ascii_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    const MARKERS: &[&str] = &["fail", "error", "panic", "fatal", "abort", "refus"];
    words.iter().enumerate().any(|(i, w)| {
        if !MARKERS.iter().any(|m| w.starts_with(m)) {
            return false;
        }
        !matches!(
            i.checked_sub(1).and_then(|p| words.get(p)),
            Some(&"0") | Some(&"no")
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{json_document, reconcile_json, with_stderr};

    #[test]
    fn text_is_wrapped_with_the_verdict() {
        let failed: serde_json::Value =
            serde_json::from_str(&json_document("hi\n", 3, || Some("/t.log".into()))).unwrap();
        assert_eq!(failed["output"], "hi\n");
        assert_eq!(failed["success"], false);
        assert_eq!(failed["exit_code"], 3);
        assert_eq!(failed["full_output"], "/t.log");
        assert_eq!(
            json_document("hi\n", 0, || None),
            r#"{"output":"hi\n","success":true}"#
        );
    }

    #[test]
    fn a_childs_array_stays_as_written_unless_it_failed() {
        assert_eq!(json_document("[1, 2]\n", 0, || None), "[1, 2]");
        assert!(json_document("[1, 2]\n", 2, || None).contains("\"exit_code\":2"));
    }

    #[test]
    fn exit_status_overrides_either_wrong_verdict() {
        let failed = reconcile_json(r#"{"success":true}"#, 3, || Some("/t.log".into())).unwrap();
        assert_eq!(
            failed,
            r#"{"exit_code":3,"full_output":"/t.log","success":false}"#
        );
        let misread = r#"{"is_empty":true,"success":false}"#;
        let passed = reconcile_json(misread, 0, || None).unwrap();
        assert_eq!(passed, r#"{"is_empty":true,"success":true}"#);
    }

    #[test]
    fn a_failure_always_says_so() {
        let out = reconcile_json("{\"name\":\"x\"}", 2, || None).unwrap();
        assert_eq!(out, r#"{"exit_code":2,"name":"x","success":false}"#);
    }

    #[test]
    fn the_childs_own_data_survives_a_clean_exit() {
        let data = r#"{"success":false,"id":7}"#;
        assert_eq!(reconcile_json(data, 0, || None).unwrap(), data);
    }

    #[test]
    fn a_matching_verdict_keeps_the_parser_text() {
        let parsed = "{\"z\":1,\"success\":true}\n";
        let kept = reconcile_json(parsed, 0, || None).unwrap();
        assert_eq!(kept, parsed.trim());
    }

    #[test]
    fn a_success_that_is_not_a_bool_is_left_alone() {
        let out = reconcile_json(r#"{"success":"yes"}"#, 0, || None).unwrap();
        assert_eq!(out, r#"{"success":"yes"}"#);
    }

    #[test]
    fn the_childs_stderr_joins_a_document_without_replacing_its_data() {
        let doc = with_stderr(r#"{"output":"hi"}"#.to_string(), "warn\n");
        assert_eq!(doc, r#"{"output":"hi","stderr":"warn\n"}"#);
        let own = r#"{"stderr":"mine"}"#;
        assert_eq!(with_stderr(own.to_string(), "warn\n"), own);
        assert_eq!(with_stderr(own.to_string(), ""), own);
    }

    #[test]
    fn only_a_json_object_is_reconciled() {
        assert_eq!(reconcile_json("[1,2]", 1, || None), None);
        assert_eq!(reconcile_json("plain text", 1, || None), None);
    }
}
