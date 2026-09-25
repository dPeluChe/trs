use super::*;

#[test]
fn per_file_cap_counts_matches_not_their_context() {
    // With -C2 each match brings four context lines; counting those against
    // the per-file cap kept 5 matches out of 25.
    let input: String = (0..30)
        .map(|i| {
            let ln = i * 10 + 1;
            format!(
                "src/a.rs-{}-before\nsrc/a.rs:{}:hit {}\nsrc/a.rs-{}-after\n",
                ln - 1,
                ln,
                i,
                ln + 1
            )
        })
        .collect();
    let mut result = ParseHandler::parse_grep(&input).unwrap();
    ParseHandler::truncate_grep(&mut result, 50, 25);
    let kept = result.files[0]
        .matches
        .iter()
        .filter(|m| !m.is_context)
        .count();
    assert_eq!(kept, 25);
    assert!(result.is_truncated);
}
