//! `gh pr/issue/run list` captured from real runs (non-TTY, tab separated).
//! The columns are not what the parsers once assumed: a PR row has the
//! branch, not an author; an issue row has the state before the title; a run
//! row has the id that `gh run view` needs.

use assert_cmd::Command;

fn parse(kind: &str, fixture: &str) -> String {
    let input = std::fs::read_to_string(format!("tests/fixture_data/{fixture}")).unwrap();
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["parse", kind])
        .write_stdin(input)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn lines(out: &str) -> Vec<&str> {
    out.lines().skip(1).collect()
}

#[test]
fn a_pr_row_keeps_its_whole_branch_and_its_state() {
    let out = parse("gh-pr", "gh_pr_list_real.txt");
    let rows = lines(&out);
    assert_eq!(rows.len(), 8, "{out}");
    let row = rows.iter().find(|r| r.contains("#182")).unwrap();
    assert!(row.contains("[MERGED]"), "{row}");
    assert!(row.contains("(release/v0.8.6)"), "{row}");
    // The slug after the first `/` used to be cut off.
    assert!(out.contains("(fix/closed-pipe-exits-quietly)"), "{out}");
    assert!(out.contains("(dependabot/cargo/rustls-0.23.45)"), "{out}");
    // Titles of ~90 characters are shown whole.
    assert!(
        out.contains("route by the real command (fix/exec-explicit-format-and-path-invocation)"),
        "{out}"
    );
}

#[test]
fn a_state_every_row_shares_moves_to_the_header() {
    let input = std::fs::read_to_string("tests/fixture_data/gh_pr_list_real.txt").unwrap();
    let open_only: String = input
        .lines()
        .filter(|l| l.contains("\tOPEN\t"))
        .map(|l| format!("{l}\n"))
        .collect();
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["parse", "gh-pr"])
        .write_stdin(open_only)
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(out.starts_with("pull requests: 3 (OPEN)\n"), "{out}");
    assert!(!out.contains("[OPEN]"), "{out}");
}

#[test]
fn an_issue_row_keeps_its_title_not_just_the_state() {
    let out = parse("gh-issue", "gh_issue_list_real.txt");
    // It used to print `#14621 OPEN`: the state was read as the title.
    assert!(
        out.contains("#14621 [OPEN] `gh` skill description does not tell agents when to use it"),
        "{out}"
    );
    assert!(out.contains("(enhancement, gh-pr)"), "labels lost:\n{out}");
    assert!(
        out.contains("[CLOSED] gh skill: document that issue view"),
        "{out}"
    );
}

#[test]
fn a_run_row_keeps_the_id_the_next_command_needs() {
    let out = parse("gh-run", "gh_run_list_real.txt");
    let rows = lines(&out);
    assert_eq!(rows.len(), 10, "{out}");
    for id in ["37702445661", "37684851221", "37684768104"] {
        assert!(out.contains(id), "run id {id} missing:\n{out}");
    }
    let failed = rows.iter().find(|r| r.contains("37702018784")).unwrap();
    assert!(failed.starts_with("  - "), "failure marker: {failed}");
    assert!(
        failed.contains("(CI, fix/exec-explicit-format-and-path-invocation, 5m1s)"),
        "{failed}"
    );
    assert!(rows[0].starts_with("  ~ 37702445661"), "{}", rows[0]);
}

#[test]
fn json_carries_the_same_fields() {
    let input = std::fs::read_to_string("tests/fixture_data/gh_run_list_real.txt").unwrap();
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["parse", "gh-run", "--json"])
        .write_stdin(input)
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let first = &v["runs"][0];
    assert_eq!(first["id"], "37702445661");
    assert_eq!(first["workflow"], "CI");
    assert_eq!(first["branch"], "fix/closed-pipe-exits-quietly");
}
