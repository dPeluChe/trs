//! What a parsed summary may claim once a child has run: its exit status is
//! the verdict, and a parser only describes the output.

use serde_json::Value;

/// `parsed` with `success` set by the child's status, plus `exit_code` on
/// failure and `full_output` when `raw` is given. These live in the object
/// because a text footer after it would make the output unparseable. None
/// when `parsed` is not a JSON object.
pub(crate) fn reconcile_json(parsed: &str, code: i32, raw: Option<&str>) -> Option<String> {
    let mut value: Value = serde_json::from_str(parsed.trim()).ok()?;
    let object = value.as_object_mut()?;
    let mut changed = false;
    let succeeded = code == 0;
    if object.get("success").and_then(Value::as_bool) == Some(!succeeded) {
        object.insert("success".to_string(), Value::Bool(succeeded));
        changed = true;
    }
    if !succeeded {
        object.insert("exit_code".to_string(), Value::from(code));
        changed = true;
    }
    if let Some(path) = raw {
        object.insert("full_output".to_string(), Value::from(path));
        changed = true;
    }
    if changed {
        Some(value.to_string())
    } else {
        Some(parsed.trim().to_string())
    }
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
    use super::reconcile_json;

    #[test]
    fn exit_status_overrides_either_wrong_verdict() {
        let failed = reconcile_json(r#"{"success":true}"#, 3, Some("/t.log")).unwrap();
        assert_eq!(
            failed,
            r#"{"exit_code":3,"full_output":"/t.log","success":false}"#
        );
        let passed = reconcile_json(r#"{"success":false}"#, 0, None).unwrap();
        assert_eq!(passed, r#"{"success":true}"#);
    }

    #[test]
    fn a_matching_verdict_keeps_the_parser_text() {
        let parsed = "{\"z\":1,\"success\":true}\n";
        let kept = reconcile_json(parsed, 0, None).unwrap();
        assert_eq!(kept, parsed.trim());
    }

    #[test]
    fn a_success_that_is_not_a_bool_is_left_alone() {
        let out = reconcile_json(r#"{"success":"yes"}"#, 0, None).unwrap();
        assert_eq!(out, r#"{"success":"yes"}"#);
    }

    #[test]
    fn only_a_json_object_is_reconciled() {
        assert_eq!(reconcile_json("[1,2]", 1, None), None);
        assert_eq!(reconcile_json("plain text", 1, None), None);
    }
}
