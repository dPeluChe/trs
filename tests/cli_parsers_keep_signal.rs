//! Parsers checked against output captured from the real tools. Each case is a
//! loss found by comparing trs with raw output, not by reading the parser.

use assert_cmd::Command;

fn parse(sub: &str, fixture: &str) -> String {
    let input = std::fs::read_to_string(format!("tests/fixture_data/{fixture}")).unwrap();
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["parse", sub])
        .write_stdin(input)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Non-TTY gh separates metadata from the body with `--`, not a `body:` key,
/// so the body never surfaced: an agent reading a PR got its title only.
#[test]
fn gh_pr_view_keeps_the_body() {
    let out = parse("gh-pr-view", "gh_pr_view_real.txt");
    assert!(out.contains("PR #162"), "{out}");
    assert!(out.contains("changes: +76 -83"), "{out}");
    assert!(
        out.contains("## Evidence") && out.contains("rewrote the **first segment**"),
        "{out}"
    );
}

/// The PORTS cell holds spaces, so only the header offsets can cut it out.
#[test]
fn docker_ps_keeps_the_ports() {
    let out = parse("docker-ps", "docker_ps_real.txt");
    assert!(
        out.contains("app-postgres-1") && out.contains("5442->5432/tcp"),
        "{out}"
    );
    assert!(
        !out.contains("0.0.0.0:"),
        "IPv4/IPv6 duplicate not folded:\n{out}"
    );
}

/// `--stat` bars are scaled to the terminal: counting `+`/`-` in them gave
/// README.md +7/-1 for a +45/-1 change, and the status letter was a guess.
#[test]
fn git_show_stat_reports_git_numbers_and_the_commit() {
    let out = parse("git-diff", "git_show_stat_real.txt");
    assert!(
        out.contains("465e586 · dPeluChe"),
        "commit header lost:\n{out}"
    );
    assert!(
        out.contains("fix(benchmark): report what trs printed"),
        "{out}"
    );
    for line in ["README.md | 46", "truth.py | 192", "src/benchmark.rs | 86"] {
        assert!(out.contains(line), "missing exact `{line}` in\n{out}");
    }
    assert!(
        !out.contains("(+7/-1)") && !out.contains("+ docs/"),
        "invented split or status:\n{out}"
    );
}
