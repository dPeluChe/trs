//! Output sink for parse handlers, so the compressing path can measure what a
//! parser produced and enforce a **never-worse guard**: trs must never emit
//! more bytes than the raw command output (a guarantee similar output-
//! compression tools also make). The parser
//! handlers print through [`emit`]; normally that goes straight to stdout, but
//! `execute_and_parse` wraps the parse in [`capture`] to get the formatted
//! string, compare it against the raw output, and print whichever is smaller.
//!
//! Thread-local so it needs no plumbing through `CommandContext` and stays
//! correct if parsing ever runs off-thread. Falls back to stdout when no
//! capture is active, so `trs parse …` run directly is unaffected.

use std::cell::RefCell;
use std::io::Write;

thread_local! {
    static SINK: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Emit parser output — into the active capture buffer if one is set, else
/// straight to stdout (the direct `trs parse` path).
pub(crate) fn emit(s: &str) {
    SINK.with(|sink| {
        let mut slot = sink.borrow_mut();
        match slot.as_mut() {
            Some(buf) => buf.push_str(s),
            None => {
                let _ = write!(std::io::stdout(), "{}", s);
            }
        }
    });
}

/// Run `f` with parser output captured; returns everything `emit`ted during
/// it. Nested captures are not expected (the parse path calls this once); an
/// inner call would simply reset the buffer.
pub(crate) fn capture<F: FnOnce()>(f: F) -> String {
    SINK.with(|sink| *sink.borrow_mut() = Some(String::new()));
    f();
    SINK.with(|sink| sink.borrow_mut().take().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_collects_emits_and_resets() {
        let out = capture(|| {
            emit("hello ");
            emit("world");
        });
        assert_eq!(out, "hello world");
        // After capture ends the sink is cleared — a later emit would go to
        // stdout, not linger in a buffer.
        SINK.with(|s| assert!(s.borrow().is_none()));
    }

    #[test]
    fn empty_capture_is_empty_string() {
        assert_eq!(capture(|| {}), "");
    }
}
