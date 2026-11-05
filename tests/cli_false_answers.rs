//! A summary must never deny what the raw output says.

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

mod support;

fn trs() -> Command {
    Command::cargo_bin("trs").unwrap()
}

fn numbered_file() -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    writeln!(f, "// note\nalpha\nbeta").unwrap();
    f
}

#[test]
fn matches_without_a_path_are_never_no_matches() {
    trs()
        .args(["parse", "grep"])
        .write_stdin("name = \"trs\"\nname = \"x\"\n")
        .assert()
        .success()
        .stdout("name = \"trs\"\nname = \"x\"\n");
}

#[test]
fn lines_with_a_colon_are_not_grouped_as_files() {
    for lines in ["3:name = \"trs\"\n4:name = \"x\"\n", "let x: u32 = 1;\n"] {
        trs()
            .args(["parse", "grep"])
            .write_stdin(lines)
            .assert()
            .stdout(lines);
    }
}

#[test]
fn a_lone_count_is_not_labelled_as_lines() {
    trs()
        .args(["parse", "wc"])
        .write_stdin("      19 w.txt\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("19 w.txt"))
        .stdout(predicate::str::contains("19L").not());
}

#[test]
fn unrecognized_lint_output_is_not_clean() {
    trs()
        .args(["parse", "lint"])
        .write_stdin("something went wrong\n")
        .assert()
        .success()
        .stdout("something went wrong\n");
}

#[cfg(unix)]
#[test]
fn a_failed_command_is_not_reported_clean() {
    trs()
        .args(["err", "sh", "-c", "echo boom >&2; exit 5"])
        .assert()
        .code(5)
        .stdout(predicate::str::contains("boom"))
        .stdout(predicate::str::contains("clean").not());
}

#[cfg(unix)]
#[test]
fn file_reads_stay_byte_exact() {
    let f = numbered_file();
    let path = f.path().to_str().unwrap();
    trs()
        .args(["cat", path])
        .assert()
        .stdout("// note\nalpha\nbeta\n");
    trs()
        .args(["head", "-n", "2", path])
        .assert()
        .stdout("// note\nalpha\n");
    trs()
        .args(["sed", "-n", "1,2p", path])
        .assert()
        .stdout("// note\nalpha\n");
}

#[cfg(unix)]
#[test]
fn a_non_utf8_argument_does_not_panic() {
    use std::os::unix::ffi::OsStrExt;
    let name = std::ffi::OsStr::from_bytes(b"g\xffx");
    trs().arg("cat").arg(name).assert().code(1);
}

#[test]
fn no_parser_turns_unread_output_into_an_empty_answer() {
    let unread = "zeta unreadable line 991\nanother strange line 77\n";
    for parser in [
        "git-status",
        "git-diff",
        "grep",
        "lint",
        "brew",
        "env",
        "docker-ps",
        "gh-pr",
        "gh-issue",
        "gh-run",
        "gh-pr-checks",
        "gh-pr-view",
        "gh-run-view",
        "tree",
        "db",
        "cargo-test",
        "go-test",
    ] {
        trs()
            .args(["parse", parser])
            .write_stdin(unread)
            .assert()
            .success()
            .stdout(unread);
    }
}

#[test]
fn unread_output_stays_valid_json() {
    let out = trs()
        .args(["--json", "parse", "git-status"])
        .write_stdin("Auf Branch main\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["output"], "Auf Branch main\n");
}

#[test]
fn a_really_clean_status_still_says_clean() {
    trs()
        .args(["parse", "git-status"])
        .write_stdin("On branch main\nnothing to commit, working tree clean\n")
        .assert()
        .stdout(predicate::str::contains("clean"));
    trs()
        .args(["parse", "git-status"])
        .write_stdin("")
        .assert()
        .stdout(predicate::str::contains("clean"));
}

#[test]
fn docker_ps_without_a_header_keeps_the_names() {
    trs()
        .args(["parse", "docker-ps"])
        .write_stdin("web\ndb\n")
        .assert()
        .stdout("web\ndb\n");
}

#[cfg(unix)]
#[test]
fn a_make_target_keeps_what_it_printed() {
    let dir = tempfile::tempdir().unwrap();
    let body =
        "for i in 1 2 3 4 5 6 7 8; do echo \"row $i: the value someone wanted to read\"; done";
    support::fake_program(&dir.path().join("make"), body);
    trs()
        .env("PATH", support::path_with(dir.path()))
        .args(["make", "report"])
        .assert()
        .success()
        .stdout(predicate::str::contains("row 8: the value someone wanted"));
}

#[cfg(unix)]
#[test]
fn git_runs_in_english_so_its_parsers_can_read_it() {
    let dir = tempfile::tempdir().unwrap();
    let git = dir.path().join("git");
    support::fake_program(&git, "echo \"language=$LANGUAGE\"");
    trs()
        .env("PATH", support::path_with(dir.path()))
        .env("LANGUAGE", "de")
        .args(["git", "version"])
        .assert()
        .stdout(predicate::str::contains("language=C"));
    trs()
        .env("LANGUAGE", "de")
        .args([git.to_str().unwrap(), "version"])
        .assert()
        .stdout(predicate::str::contains("language=C"));
}
