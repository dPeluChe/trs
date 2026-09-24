//! pytest output captured from a real run (pytest 7.4, 2 of 72 tests failing),
//! not written by hand. Hand-written fixtures used `=== FAILURES ===`; real
//! banners are padded to the terminal width, so the parser never saw a failure
//! section, invented names like `test_71` in quiet mode, and panicked on the
//! padded summary line in the default mode.

use assert_cmd::Command;

fn run(fixture: &str) -> String {
    let input = std::fs::read_to_string(format!("tests/fixture_data/{fixture}")).unwrap();
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["parse", "test", "--runner", "pytest"])
        .write_stdin(input)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!err.contains("panicked"), "{fixture}: trs panicked\n{err}");
    text
}

#[test]
fn every_mode_names_the_real_failures_and_their_errors() {
    for fixture in [
        "pytest_real_default.txt",
        "pytest_real_quiet.txt",
        "pytest_real_verbose.txt",
    ] {
        let out = run(fixture);
        for must in [
            "70 passed, 2 failed",
            "test_invoice_total_rounding",
            "test_user_lookup_missing",
            "AssertionError: expected 0.31, got 0.3",
            "KeyError: 'beto'",
            "test_app.py:215",
        ] {
            assert!(out.contains(must), "{fixture}: missing `{must}` in\n{out}");
        }
        assert!(
            !out.contains("test_71") && !out.contains("def test_"),
            "{fixture}: invented name or source line instead of the error\n{out}"
        );
    }
}
