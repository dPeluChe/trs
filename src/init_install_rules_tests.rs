use super::*;

#[test]
fn refresh_sentinel_block_leaves_everything_outside_untouched() {
    // The file holds two trs blocks plus the user's own rules. A refresh that
    // ate the blank line between them would run three separate sets of
    // instructions together into one wall of prose, which is the opposite of
    // what a rules file is for.
    let content = "<!-- s -->\nold\n<!-- e -->\n\n## User rules\nkeep me\n";
    let out = refresh_sentinel_block(
        content,
        "<!-- s -->",
        "<!-- e -->",
        "<!-- s -->\nnew\n<!-- e -->",
    )
    .expect("sentinels are present");
    assert_eq!(
        out,
        "<!-- s -->\nnew\n<!-- e -->\n\n## User rules\nkeep me\n"
    );
}

#[test]
fn refresh_sentinel_block_refuses_to_pair_markers_from_different_blocks() {
    // A user documenting trs in their own rules file has the start marker
    // inside a fenced example. Pairing that first start with the real block's
    // end deletes everything between, which here is the user's own rules. This
    // was live and reported "(refreshed)" while doing it.
    let content = "\
```markdown
<!-- s -->
example
```

## MY CRITICAL RULES
1. never rm -rf

<!-- s -->
stale
<!-- e -->
";
    assert_eq!(
        refresh_sentinel_block(
            content,
            "<!-- s -->",
            "<!-- e -->",
            "<!-- s -->\nnew\n<!-- e -->"
        ),
        None,
        "two starts must decline, not splice across them"
    );
}

#[test]
fn refresh_sentinel_block_declines_two_well_formed_blocks() {
    // What the backticked-marker bug produced in the field. Refreshing only
    // the first and leaving the second stale is a half-fix that reports
    // success; declining says so instead.
    let content = "<!-- s -->\none\n<!-- e -->\n\n<!-- s -->\ntwo\n<!-- e -->\n";
    assert!(refresh_sentinel_block(content, "<!-- s -->", "<!-- e -->", "x").is_none());
}

#[test]
fn refresh_sentinel_block_declines_without_a_pair() {
    assert!(refresh_sentinel_block("no sentinels here", "<!-- s -->", "<!-- e -->", "x").is_none());
    // Opening sentinel but no close: replacing would swallow the rest of the
    // file, so decline rather than guess where the block ends.
    assert!(refresh_sentinel_block("<!-- s -->\nbody", "<!-- s -->", "<!-- e -->", "x").is_none());
}

#[test]
fn sentinel_markers_keep_their_exact_wire_form() {
    // These strings are matched against files already installed on people's
    // machines. Deriving them from a macro is fine; changing a byte is not,
    // because every existing block stops being recognised and gets a second
    // copy appended next to it.
    use crate::init_templates::{
        ANTIGRAVITY_RULES_SENTINEL_END, ANTIGRAVITY_RULES_SENTINEL_START,
        CODEX_AGENTS_SENTINEL_END, CODEX_AGENTS_SENTINEL_START,
    };
    use crate::output_saver::{SENTINEL_END, SENTINEL_START};

    assert_eq!(
        CODEX_AGENTS_SENTINEL_START,
        "<!-- trs:codex-rules:start v1 -->"
    );
    assert_eq!(CODEX_AGENTS_SENTINEL_END, "<!-- trs:codex-rules:end -->");
    assert_eq!(
        ANTIGRAVITY_RULES_SENTINEL_START,
        "<!-- trs:antigravity-rules:start v1 -->"
    );
    assert_eq!(
        ANTIGRAVITY_RULES_SENTINEL_END,
        "<!-- trs:antigravity-rules:end -->"
    );
    assert_eq!(SENTINEL_START, "<!-- trs:output-saver:start v1 -->");
    assert_eq!(SENTINEL_END, "<!-- trs:output-saver:end -->");
}

#[test]
fn the_trs_marker_catches_the_backticked_spelling() {
    // The codex rules prose writes it as `trs` with backticks. Missing that
    // is what let a second copy of the block get appended to an AGENTS.md
    // that already had one, costing 2189 bytes on every session.
    use crate::init::file_has_any_trs_marker;
    assert!(file_has_any_trs_marker(
        "uses `trs` (Token-Reducing Shell) for output"
    ));
    assert!(file_has_any_trs_marker(
        "uses trs (Token-Reducing Shell) for output"
    ));
    assert!(!file_has_any_trs_marker("nothing to do with the tool"));
}

#[test]
fn block_status_tells_the_four_states_apart() {
    use std::io::Write;
    let dir = std::env::temp_dir().join("trs_block_status_test");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("AGENTS.md");
    let start = "<!-- s -->";
    let end = "<!-- e -->";
    let section = "<!-- s -->\ncurrent\n<!-- e -->";

    let write = |body: &str| {
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(body.as_bytes()).unwrap();
    };

    write("# just my own notes\n");
    assert_eq!(block_status(&path, start, end, section), RulesBlock::Absent);

    write("<!-- s -->\ncurrent\n<!-- e -->");
    assert_eq!(
        block_status(&path, start, end, section),
        RulesBlock::Current
    );

    write("<!-- s -->\nolder text\n<!-- e -->");
    assert_eq!(
        block_status(&path, start, end, section),
        RulesBlock::Drifted
    );

    // The case that cost 2189 bytes a session and was invisible until someone
    // opened the file: two blocks, which refresh refuses to splice across.
    write("<!-- s -->\none\n<!-- e -->\n\n<!-- s -->\ntwo\n<!-- e -->");
    assert_eq!(
        block_status(&path, start, end, section),
        RulesBlock::Duplicated
    );

    write("## Terminal Output Optimization\n\nprose from before the sentinels\n");
    assert_eq!(block_status(&path, start, end, section), RulesBlock::Legacy);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn block_status_reports_a_missing_file_as_absent() {
    let missing = std::env::temp_dir().join("trs_no_such_agents_file_xyz.md");
    let _ = std::fs::remove_file(&missing);
    assert_eq!(
        block_status(&missing, "<!-- s -->", "<!-- e -->", "x"),
        RulesBlock::Absent
    );
}
