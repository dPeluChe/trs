//! `gh issue view` and `--comments` output, captured without a TTY as trs sees it.

use assert_cmd::Command;

fn parse(sub: &str, input: &str, json: bool) -> String {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    if json {
        cmd.arg("--json");
    }
    let out = cmd
        .args(["parse", sub])
        .write_stdin(input)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!("tests/fixture_data/{name}")).unwrap()
}

#[test]
fn an_issue_is_labeled_as_an_issue_and_keeps_its_body() {
    let out = parse("gh-issue-view", &fixture("gh_issue_view_real.txt"), false);
    assert!(out.starts_with("Issue #204: Claude Code hook"), "{out}");
    assert!(out.contains("[CLOSED] by cheetahbyte"), "{out}");
    assert!(out.contains("comments: 1"), "{out}");
    assert!(out.contains("## Reproduce"), "{out}");
    assert!(
        !out.contains("sub-issues") && !out.contains("blocked-by"),
        "{out}"
    );
}

#[test]
fn comments_keep_author_role_and_body() {
    let out = parse(
        "gh-issue-view",
        &fixture("gh_issue_view_comments_real.txt"),
        false,
    );
    assert!(
        out.starts_with("1 comment\n--\n@dPeluChe (owner)\n"),
        "{out}"
    );
    assert!(out.contains("Fixed in **v0.6.8**"), "{out}");
    assert!(out.contains("tracked in #58"), "{out}");
}

/// A body may hold its own `--` line; only a following header ends a comment.
#[test]
fn a_dash_line_inside_a_comment_does_not_split_it() {
    let input = "author:\ta\nassociation:\tmember\nedited:\tfalse\nstatus:\tapproved\n--\n\
                 before\n--\nafter\n--\nauthor:\tb\nassociation:\tnone\nedited:\ttrue\n\
                 status:\tnone\n--\nlast\n--\n";
    let out = parse("gh-pr-view", input, false);
    assert_eq!(
        out,
        "2 comments\n--\n@a (member, approved)\nbefore\n--\nafter\n--\n@b (edited)\nlast\n"
    );
    let json = parse("gh-pr-view", input, true);
    assert!(json.contains(r#""body":"before\n--\nafter""#), "{json}");
    assert!(json.contains(r#""status":"approved""#), "{json}");
}
