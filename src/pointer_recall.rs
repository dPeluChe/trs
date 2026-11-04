//! How often a `[trs] full output: <path>` pointer is followed. The run that
//! prints one marks its history line (`ptr`); a later shell command that reads
//! a saved file shows up in the history by its path.

use crate::tracker::HistoryEntry;
use std::sync::atomic::{AtomicBool, Ordering};

static PRINTED: AtomicBool = AtomicBool::new(false);

/// Call when this process saved a raw output and is about to point at it.
pub(crate) fn note_pointer() {
    PRINTED.store(true, Ordering::Relaxed);
}

pub(crate) fn printed() -> bool {
    PRINTED.load(Ordering::Relaxed)
}

/// (runs that printed a pointer, shell commands that read a saved file back).
/// A read through the agent's own file tool never reaches the history, so the
/// second figure is a lower bound.
pub(crate) fn recall(entries: &[HistoryEntry]) -> (usize, usize) {
    let pointers = entries.iter().filter(|e| e.ptr == Some(true)).count();
    let reads = entries
        .iter()
        .filter(|e| e.bypass.is_none() && e.cmd.contains("/tee/") && e.cmd.contains(".log"))
        .count();
    (pointers, reads)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(cmd: &str, ptr: Option<bool>, bypass: Option<bool>) -> HistoryEntry {
        HistoryEntry {
            ts: 0,
            cmd: cmd.into(),
            in_bytes: 0,
            out_bytes: 0,
            saved_pct: 0,
            ms: 0,
            cwd: String::new(),
            agent: None,
            bypass,
            ok: None,
            ptr,
        }
    }

    #[test]
    fn counts_pointers_and_shell_reads_of_saved_files() {
        let entries = [
            entry("git show --stat", Some(true), None),
            entry("cargo test", Some(true), None),
            entry("git status", None, None),
            entry("cat /home/u/.trs/tee/17_git_show.log", None, None),
            entry("cat /home/u/.trs/tee/18_x.log", None, Some(true)),
        ];
        assert_eq!(recall(&entries), (2, 1));
    }
}
