//! One place that decides what a trs sentinel looks like.
//!
//! Three blocks are spliced into files trs does not own (the codex and
//! antigravity rules sections, and the output-saver block), and each needs a
//! start/end pair to find itself again on the next install. Declaring six
//! loose string constants let the format drift silently; declaring the name
//! and deriving the pair means a change to the shape is one edit and no
//! writer can invent a variant.
//!
//! The `v1` in the start marker is the block-format version, not trs's. Bump
//! it only when old blocks must be recognised as incompatible rather than
//! merely stale, since installs in the field are matched against it.

/// Declare a `(START, END)` sentinel pair for a named block, and optionally
/// the section itself so the bytes written and the bytes searched for are
/// literally the same expansion.
///
/// The pair-only arm is for blocks whose body is assembled elsewhere. Prefer
/// the arm that takes `$body`: deriving the constants while the section still
/// spells its own markers by hand leaves exactly the drift this module is
/// supposed to close, and that drift is invisible until someone's installed
/// block stops being recognised and gets a duplicate appended beside it.
macro_rules! trs_sentinels {
    ($start:ident, $end:ident, $name:literal) => {
        pub(crate) const $start: &str = concat!("<!-- trs:", $name, ":start v1 -->");
        pub(crate) const $end: &str = concat!("<!-- trs:", $name, ":end -->");
    };
    ($start:ident, $end:ident, $section:ident, $name:literal, $body:literal) => {
        trs_sentinels!($start, $end, $name);
        pub(crate) const $section: &str = concat!(
            "<!-- trs:",
            $name,
            ":start v1 -->\n",
            $body,
            "\n<!-- trs:",
            $name,
            ":end -->\n"
        );
    };
}

pub(crate) use trs_sentinels;
