use assert_cmd::Command;
use predicates::prelude::*;

// git pull / git fetch parser tests

#[test]
fn test_git_pull_already_up_to_date() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-pull")
        .write_stdin("Already up to date.\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("already up to date"));
}

#[test]
fn test_git_pull_with_changes() {
    let input = "remote: Enumerating objects: 5, done.\n\
remote: Counting objects: 100% (5/5), done.\n\
remote: Compressing objects: 100% (3/3), done.\n\
remote: Total 3 (delta 2), reused 0 (delta 0), pack-reused 0\n\
Unpacking objects: 100% (3/3), done.\n\
From https://github.com/dPeluChe/trs\n\
   abc1234..def5678  main -> origin/main\n\
Updating abc1234..def5678\n\
Fast-forward\n\
 src/classifier.rs | 10 ++++++++++\n\
 src/main.rs       |  5 +++++\n\
 2 files changed, 15 insertions(+)\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-pull")
        .write_stdin(input)
        .assert()
        .success()
        // remote progress lines stripped
        .stdout(predicate::str::contains("remote:").not())
        .stdout(predicate::str::contains("Unpacking").not())
        // useful info kept
        .stdout(predicate::str::contains("github.com/dPeluChe/trs"))
        .stdout(predicate::str::contains("2 files changed"));
}

#[test]
fn test_git_pull_strips_remote_noise() {
    let input = "remote: Enumerating objects: 100, done.\n\
remote: Counting objects: 100% (100/100), done.\n\
remote: Compressing objects: 100% (80/80), done.\n\
remote: Total 80 (delta 40), reused 0 (delta 0), pack-reused 0\n\
Receiving objects: 100% (80/80), 12.50 KiB | 2.00 MiB/s, done.\n\
Resolving deltas: 100% (40/40), done.\n\
From https://github.com/example/repo\n\
 * [new branch]  feature/foo -> origin/feature/foo\n";

    let output = Command::cargo_bin("trs")
        .unwrap()
        .arg("parse")
        .arg("git-pull")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let ratio = output.len() as f64 / input.len() as f64;
    assert!(
        ratio < 0.5,
        "expected >50% reduction on remote-noise input, got {ratio:.2}"
    );
}

#[test]
fn test_git_pull_json_format() {
    let input = "From https://github.com/dPeluChe/trs\n\
   abc1234..def5678  main -> origin/main\n\
Updating abc1234..def5678\n\
Fast-forward\n\
 file.rs | 5 +++++\n\
 1 file changed, 5 insertions(+)\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-pull")
        .arg("--json")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"from\""))
        .stdout(predicate::str::contains("\"branches\""));
}

#[test]
fn test_git_fetch_new_branch() {
    let input = "remote: Counting objects: 3, done.\n\
From https://github.com/example/repo\n\
 * [new branch]      feat/new-feature -> origin/feat/new-feature\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-pull")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("remote:").not())
        .stdout(predicate::str::contains("feat/new-feature"));
}
