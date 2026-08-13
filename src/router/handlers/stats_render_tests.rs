use super::*;

#[test]
fn format_bypass_cell_zero_is_plain() {
    // Eye-skip rendering for the common case.
    assert_eq!(format_bypass_cell(0, 100), "0");
    assert_eq!(format_bypass_cell(0, 0), "0");
}

#[test]
fn format_bypass_cell_includes_rate_when_nonzero() {
    // Non-zero shows count and rate so the user can tell at a
    // glance whether bypass is rare or chronic.
    assert_eq!(format_bypass_cell(3, 142), "3 (2.1%)");
    assert_eq!(format_bypass_cell(50, 100), "50 (50.0%)");
}

#[test]
fn format_bypass_cell_zero_total_omits_rate() {
    // Defensive: avoid div-by-zero when total is 0 but bypass > 0
    // (shouldn't happen in practice — bypass entries also count
    // as commands — but the guard keeps the function total).
    assert_eq!(format_bypass_cell(2, 0), "2");
}

/// Reproduces the real shape that made the headline misleading: two enormous
/// uncompressed commands three weeks back, healthy work ever since. The
/// lifetime mean stays low forever; the recent window has to show the truth.
#[test]
fn recent_efficiency_ignores_an_old_outlier() {
    const DAY: u64 = 86_400;
    let now = 100 * DAY;
    let e = |days_ago: u64, inb: usize, outb: usize| HistoryEntry {
        ts: now - days_ago * DAY,
        cmd: "x".into(),
        in_bytes: inb,
        out_bytes: outb,
        saved_pct: 0,
        ms: 0,
        cwd: String::new(),
        agent: None,
        bypass: None,
    };
    let entries = vec![
        e(21, 380_000_000, 370_000_000), // the aws-shaped week: ~3% saved
        e(3, 1_000_000, 150_000),        // recent work: 85% saved
        e(1, 1_000_000, 150_000),
    ];

    use super::super::stats_efficiency::efficiency_since;
    let d7 = efficiency_since(&entries, now, 7).unwrap();
    let d30 = efficiency_since(&entries, now, 30).unwrap();
    assert!(d7 > 80.0, "last 7d should reflect recent work, got {d7}");
    assert!(d30 < 10.0, "30d still contains the outlier, got {d30}");

    // Empty window reports nothing rather than a fake 0%.
    assert!(efficiency_since(&entries, now, 0).is_none());
}
