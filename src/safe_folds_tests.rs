use super::*;

#[test]
fn consecutive_repeats_fold_to_a_count() {
    let text = "start\nretrying connection\nretrying connection\nretrying connection\ndone\n";
    assert_eq!(
        fold_repeats(text),
        "start\nretrying connection (x3)\ndone\n"
    );
}

#[test]
fn a_repeated_block_is_kept_once() {
    let text = "a\nerror: refused\nretry in 1s\nerror: refused\nretry in 1s\nerror: refused\nretry in 1s\nb";
    assert_eq!(
        fold_repeats(text),
        "a\nerror: refused\nretry in 1s\n(last 2 lines x3)\nb"
    );
}

#[test]
fn order_is_kept_so_non_consecutive_repeats_stay() {
    // A retry between two errors is not the same as three retries in a row.
    let text = "error: disk full\nretry\nerror: disk full\nretry again\nerror: disk full";
    assert_eq!(fold_repeats(text), text);
}

#[test]
fn every_fold_undoes_to_the_exact_input() {
    for text in [
        "x\nx\n",
        "one\ntwo\ntwo\ntwo\n\n\nthree\nthree",
        "a\nb\na\nb\na\nb\nc\nc\nc\nc",
        "  indented line here\n  indented line here\n",
        "",
        "\n\n\n",
    ] {
        assert_eq!(
            unfold(&fold_repeats(text)),
            text,
            "not reversible: {text:?}"
        );
    }
}

#[test]
fn input_that_already_looks_folded_is_left_alone() {
    // Otherwise `(x3)` in the output could be the tool's or ours.
    let text = "compiled module (x3)\nretry\nretry\nretry";
    assert_eq!(fold_repeats(text), text);
    let text = "(last 2 lines x9)\nok\nok\nok";
    assert_eq!(fold_repeats(text), text);
}

#[test]
fn folding_never_makes_output_larger() {
    // `ab (x2)` is longer than `ab\nab`.
    assert_eq!(fold_repeats("ab\nab"), "ab\nab");
    assert!(fold_repeats("a long enough line\na long enough line").contains("(x2)"));
}

#[test]
fn dense_lines_keep_head_tail_and_say_what_was_cut() {
    let b64 = "QUJD".repeat(200);
    let (out, cut) = elide_dense(&format!("before\n{b64}\nafter"));
    assert!(cut);
    assert!(out.contains("…[base64, 800 chars]…"), "{out}");
    assert!(
        out.starts_with("before\nQUJD") && out.ends_with("\nafter"),
        "{out}"
    );

    let uri = format!(
        "<img src=\"data:image/png;base64,{}\">",
        "iVBOR".repeat(100)
    );
    assert!(
        elide_dense(&uri).0.contains("[data URI image/png,"),
        "{}",
        elide_dense(&uri).0
    );

    let js = "function(e){return e&&e.__esModule?e:{default:e}};".repeat(10);
    assert!(elide_dense(&js).0.contains("[minified,"));
}

#[test]
fn ordinary_long_lines_and_tables_are_not_dense() {
    let prose =
        "This sentence is long but it has the normal amount of spaces between words. ".repeat(6);
    assert_eq!(elide_dense(&prose), (prose.clone(), false));
    let row = format!(
        "{}\t{}\t{}",
        "a".repeat(150),
        "b".repeat(150),
        "c".repeat(50)
    );
    assert_eq!(elide_dense(&row), (row.clone(), false));
    let short = "x".repeat(299);
    assert_eq!(elide_dense(&short), (short.clone(), false));
}

#[test]
fn eliding_is_safe_on_multibyte_text() {
    let line = "é".repeat(400);
    let (out, cut) = elide_dense(&line);
    assert!(cut && out.contains("[minified, 800 chars]"), "{out}");
}
