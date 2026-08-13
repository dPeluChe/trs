//! Recent-window efficiency for `trs stats`.
//!
//! Split from `stats_render.rs` to keep it under the repo LOC limit, and
//! because it answers a different question: the lifetime mean says how trs has
//! done overall, this says how it is doing now.

use crate::tracker::HistoryEntry;

/// Print efficiency over the last 7 and 30 days, when those windows hold data.
pub(crate) fn print_recent(entries: &[HistoryEntry]) -> Option<f64> {
    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    for days in [7u64, 30] {
        if let Some((saved, pct)) = window_totals(entries, now_ts, days) {
            println!(
                "Last {:<2} days:      {} saved \u{00b7} {:.0}%",
                days,
                crate::tracker::format_bytes_human(saved / 4),
                pct
            );
        }
    }
    efficiency_since(entries, now_ts, 30)
}

/// `(bytes saved, percent saved)` inside the window, or None when it is empty.
fn window_totals(entries: &[HistoryEntry], now_ts: u64, days: u64) -> Option<(usize, f64)> {
    let cutoff = now_ts.saturating_sub(days * 86_400);
    let (i, o) = entries
        .iter()
        .filter(|e| e.ts >= cutoff)
        .fold((0usize, 0usize), |(i, o), e| {
            (i + e.in_bytes, o + e.out_bytes)
        });
    (i > 0).then(|| (i.saturating_sub(o), 100.0 * (1.0 - o as f64 / i as f64)))
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

/// The efficiency bar, labelled with the window it actually covers. The label
/// matters: an unlabelled percentage over 120 days reads as today's number.
pub(crate) fn print_bar(avg_pct: f64, window_days: Option<u64>) {
    let filled = (avg_pct / 5.0).round() as usize;
    let filled = filled.min(20);
    let empty = 20 - filled;
    println!(
        "Efficiency: {}{} {:.0}% ({})",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty),
        avg_pct,
        match window_days {
            Some(d) => format!("last {}d", d),
            None => "lifetime".to_string(),
        }
    );
}
