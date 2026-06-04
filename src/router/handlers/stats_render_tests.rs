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
