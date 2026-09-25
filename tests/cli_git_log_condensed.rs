//! `git log` captured from this repo's own history: long bodies, a GitHub
//! squash merge, a dependabot bump. The parser used to join every message
//! line with spaces and then take "the first line", which was the whole body.

use assert_cmd::Command;

fn parse() -> String {
    let input = std::fs::read_to_string("tests/fixture_data/git_log_real.txt").unwrap();
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["parse", "git-log"])
        .write_stdin(input)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn each_commit_keeps_what_an_agent_uses_and_says_how_to_get_the_rest() {
    let out = parse();
    // Subject on its own line, not glued to the body.
    assert!(
        out.lines().any(
            |l| l.starts_with("0175ae7 fix(parse): what trs lost or invented")
                && !l.contains("Each loss was found")
        ),
        "{out}"
    );
    // A squash merge's index of what went in.
    assert!(
        out.contains("  * perf(rewrite): stop wrapping ssh"),
        "{out}"
    );
    // The opening "why" of an ordinary commit.
    assert!(out.contains("Same repo state, o200k tokens"), "{out}");
    // The way back to the full message.
    assert!(
        out.contains("git show 0175ae7]") && out.contains("git show 787156f]"),
        "{out}"
    );
    // Process trailers are not content.
    assert!(!out.contains("Claude-Session"), "{out}");
}

#[test]
fn the_condensed_log_is_a_fraction_of_the_raw_one() {
    let raw = std::fs::read_to_string("tests/fixture_data/git_log_real.txt").unwrap();
    let out = parse();
    assert!(
        out.len() * 4 < raw.len(),
        "{} of {} bytes",
        out.len(),
        raw.len()
    );
}

/// A subject using the word "commit " made the oneline log read as the full
/// format, which found no `commit <hash>` line and printed nothing at all.
#[test]
fn oneline_log_survives_a_subject_that_says_commit() {
    let input = "18e820e feat(git-log): condense commit bodies\n0175ae7 fix: stop wrapping ssh\n";
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["parse", "git-log"])
        .write_stdin(input)
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.contains("18e820e") && out.contains("condense commit bodies"),
        "{out}"
    );
    assert!(out.contains("0175ae7"), "{out}");
}
