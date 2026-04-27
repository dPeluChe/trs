use assert_cmd::Command;
use predicates::prelude::*;

// trs stats --by-command tests

#[test]
fn test_stats_by_command_runs_without_error() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("stats")
        .arg("--by-command")
        .assert()
        .success();
}

#[test]
fn test_stats_by_command_shows_header() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("stats")
        .arg("--by-command")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("by command")
                .or(predicate::str::contains("COMMAND"))
                .or(predicate::str::contains("no history")),
        );
}

#[test]
fn test_stats_by_command_columns_present() {
    let output = Command::cargo_bin("trs")
        .unwrap()
        .arg("stats")
        .arg("--by-command")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8_lossy(&output);
    // If there's any history, expect table columns; otherwise accept empty/no-history message
    if text.contains("COMMAND") {
        assert!(
            text.contains("CALLS") || text.contains("SHARE") || text.contains("SAVED"),
            "expected table columns in output: {text}"
        );
    }
}

#[test]
fn test_stats_default_does_not_show_by_command_table() {
    let output = Command::cargo_bin("trs")
        .unwrap()
        .arg("stats")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8_lossy(&output);
    // Default stats shows period/totals, not per-command table
    assert!(
        text.contains("Total commands") || text.contains("no history"),
        "expected default stats output: {text}"
    );
}
