//! Recent-window efficiency for `trs stats`.
//!
//! Split from `stats_render.rs` to keep it under the repo LOC limit, and
//! because it answers a different question: the lifetime mean says how trs has
//! done overall, this says how it is doing now.

use crate::tracker::HistoryEntry;

/// Print efficiency over the last 7 and 30 days, when those windows hold data.
pub(crate) fn print_recent(entries: &[HistoryEntry]) {
    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let (Some(d7), Some(d30)) = (
        efficiency_since(entries, now_ts, 7),
        efficiency_since(entries, now_ts, 30),
    ) {
        println!(
            "Recent:            {:.0}% last 7d \u{00b7} {:.0}% last 30d",
            d7, d30
        );
    }
}

/// Compression efficiency over the last `days`, or None when that window holds
/// no input. Separate from the lifetime mean on purpose: the lifetime figure is
/// cumulative, so a single rare command can hold it down permanently while
/// recent work runs far better.
pub(crate) fn efficiency_since(entries: &[HistoryEntry], now_ts: u64, days: u64) -> Option<f64> {
    let cutoff = now_ts.saturating_sub(days * 86_400);
    let (i, o) = entries
        .iter()
        .filter(|e| e.ts >= cutoff)
        .fold((0usize, 0usize), |(i, o), e| {
            (i + e.in_bytes, o + e.out_bytes)
        });
    (i > 0).then(|| 100.0 * (1.0 - o as f64 / i as f64))
}
