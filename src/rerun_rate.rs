//! How often the next command in a directory repeats the previous one within
//! two minutes, split by how much trs cut from the first run. A fixed
//! compression that makes the agent ask again can cost more than it saves, so
//! the rate after a heavy cut is shown next to the rate after a light one.
//! Also finds failing commands repeated in a row. Local history only.

use crate::tracker::HistoryEntry;
use std::collections::HashMap;

const WINDOW_SECS: u64 = 120;
/// Runs cut by at least this much count as compressed.
const HEAVY_CUT_PCT: u8 = 50;
/// Runs cut by less than this count as left alone.
const LIGHT_CUT_PCT: u8 = 10;

#[derive(Debug, Default, PartialEq)]
pub(crate) struct Rate {
    pub(crate) reran: usize,
    pub(crate) total: usize,
}

impl Rate {
    pub(crate) fn pct(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.reran as f64 * 100.0 / self.total as f64
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Reruns {
    pub(crate) compressed: Rate,
    pub(crate) plain: Rate,
    /// (directory, repeats) for failing commands run again right after failing.
    pub(crate) failing_loops: Vec<(String, usize)>,
}

fn bucket<'a>(out: &'a mut Reruns, prev: &HistoryEntry) -> Option<&'a mut Rate> {
    if prev.saved_pct >= HEAVY_CUT_PCT {
        Some(&mut out.compressed)
    } else if prev.saved_pct < LIGHT_CUT_PCT {
        Some(&mut out.plain)
    } else {
        None
    }
}

pub(crate) fn analyze(entries: &[HistoryEntry]) -> Reruns {
    let mut runs: Vec<&HistoryEntry> = entries.iter().filter(|e| e.bypass.is_none()).collect();
    runs.sort_by_key(|e| e.ts);
    let mut out = Reruns::default();
    let mut loops: HashMap<&str, usize> = HashMap::new();
    let mut last_in_dir: HashMap<&str, &HistoryEntry> = HashMap::new();
    for e in runs {
        let Some(prev) = last_in_dir.insert(e.cwd.as_str(), e) else {
            continue;
        };
        let same = e.ts.saturating_sub(prev.ts) <= WINDOW_SECS && e.cmd == prev.cmd;
        if prev.ok == Some(false) {
            if same && e.ok == Some(false) {
                *loops.entry(e.cwd.as_str()).or_default() += 1;
            }
        } else if let Some(r) = bucket(&mut out, prev) {
            r.total += 1;
            r.reran += usize::from(same);
        }
    }
    let mut v: Vec<(String, usize)> = loops.into_iter().map(|(d, n)| (d.to_string(), n)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out.failing_loops = v;
    out
}

/// Printed under `trs stats`; silent until there is enough to compare.
pub(crate) fn print(r: &Reruns) {
    const MIN_RUNS: usize = 20;
    if r.compressed.total >= MIN_RUNS && r.plain.total >= MIN_RUNS {
        println!(
            "\nSame command rerun within {} s: {:.0}% after a cut of {}% or more ({} runs), {:.0}% after a light cut ({} runs).",
            WINDOW_SECS,
            r.compressed.pct(),
            HEAVY_CUT_PCT,
            r.compressed.total,
            r.plain.pct(),
            r.plain.total
        );
    }
    let loops: usize = r.failing_loops.iter().map(|(_, n)| n).sum();
    if loops >= 3 {
        println!(
            "Failing command repeated right after failing: {loops} times in {} director{} (most: {}).",
            r.failing_loops.len(),
            if r.failing_loops.len() == 1 { "y" } else { "ies" },
            r.failing_loops[0].0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(ts: u64, cwd: &str, cmd: &str, saved_pct: u8, ok: bool) -> HistoryEntry {
        HistoryEntry {
            ts,
            cwd: cwd.into(),
            cmd: cmd.into(),
            saved_pct,
            ok: (!ok).then_some(false),
            ..Default::default()
        }
    }

    #[test]
    fn a_repeat_right_after_a_heavy_cut_is_counted_apart_from_a_light_one() {
        let h = [
            run(0, "/a", "cargo test", 90, true),
            run(10, "/a", "cargo test", 90, true), // rerun after a cut
            run(20, "/a", "git status", 0, true),
            run(30, "/a", "git log", 0, true), // different command
            run(500, "/a", "ls", 0, true),     // outside the window
        ];
        let r = analyze(&h);
        assert_eq!(r.compressed, Rate { reran: 1, total: 2 });
        assert_eq!(r.plain, Rate { reran: 0, total: 2 }); // the last run has no successor
    }

    #[test]
    fn directories_do_not_mix_and_failures_are_not_a_rerun_of_a_cut() {
        let h = [
            run(0, "/a", "make", 90, true),
            run(5, "/b", "make", 90, true),
            run(10, "/a", "make", 0, false),
            run(20, "/a", "make", 0, false),
            run(30, "/a", "make", 0, false),
        ];
        let r = analyze(&h);
        assert_eq!(r.compressed, Rate { reran: 1, total: 1 });
        assert_eq!(r.failing_loops, [("/a".to_string(), 2)]);
    }

    #[test]
    fn bypass_entries_are_ignored() {
        let mut b = run(0, "/a", "x", 0, true);
        b.bypass = Some(true);
        assert_eq!(analyze(&[b]).plain.total, 0);
    }
}
