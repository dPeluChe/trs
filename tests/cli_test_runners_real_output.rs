//! Test-runner output captured from real runs, so the fixtures carry what the
//! tools actually print (padding, order, which stream) rather than what a
//! parser author expected.

use assert_cmd::Command;

fn parse(args: &[&str], fixture: &str) -> String {
    let input = std::fs::read_to_string(format!("tests/fixture_data/{fixture}")).unwrap();
    let out = Command::cargo_bin("trs")
        .unwrap()
        .arg("parse")
        .args(args)
        .write_stdin(input)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn section<'a>(out: &'a str, from: &str, to: &str) -> &'a str {
    let start = out
        .find(from)
        .unwrap_or_else(|| panic!("no `{from}` in\n{out}"));
    let rest = &out[start..];
    &rest[..rest.find(to).unwrap_or(rest.len())]
}

/// bun prints each failure's details BEFORE its `(fail)` line. Reading them
/// as trailing details pinned the second failure's diff onto the first.
#[test]
fn bun_failures_carry_their_own_reason() {
    let out = parse(&["test", "--runner", "bun"], "bun_test_real_failures.txt");
    let first = section(&out, "formats currency", "rejects expired");
    assert!(
        first.contains(r#"Expected: "1,234.50""#) && first.contains(r#"Received: "1234.5""#),
        "{out}"
    );
    assert!(first.contains("math.test.ts:82:"), "location lost:\n{out}");
    assert!(
        !first.contains("\"expired\""),
        "second failure's diff leaked into the first:\n{out}"
    );
    let second = section(&out, "rejects expired", "\n\n");
    assert!(
        second.contains("\"expired\"") && second.contains("math.test.ts:83:"),
        "{out}"
    );
    assert!(
        !out.contains("82 | test("),
        "numbered source context should be dropped:\n{out}"
    );
}

/// assert_eq! prints the two values after the message; they are the fix.
#[test]
fn cargo_test_failures_keep_the_compared_values() {
    let out = parse(&["cargo-test"], "cargo_test_real_failures.txt");
    assert!(
        out.contains("addition drifted") && out.contains("(left: 4) (right: 5)"),
        "{out}"
    );
    assert!(out.contains("src/main.rs:67:45"), "{out}");
    assert!(out.contains("ParseIntError"), "{out}");
}
