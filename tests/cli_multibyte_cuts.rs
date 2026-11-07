//! A fixed-width cut that lands inside a multibyte char must not panic.

use assert_cmd::Command;

fn run(args: &[&str], stdin: &str) -> std::process::Output {
    Command::cargo_bin("trs")
        .unwrap()
        .args(args)
        .write_stdin(stdin.to_string())
        .output()
        .unwrap()
}

fn assert_ran(out: &std::process::Output) {
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("panicked"), "{stderr}");
    assert_ne!(out.status.code(), Some(101), "{stderr}");
}

#[test]
fn logs_with_a_char_across_the_timestamp_width() {
    let line = format!("{}ñ rest\n", "a".repeat(18));
    assert_ran(&run(&["parse", "logs"], &line.repeat(2)));
    assert_ran(&run(&["parse", "logs"], "2024-01-15T10:30:00+0ññ x\n"));
}

#[test]
fn logs_with_long_multibyte_messages() {
    let line = format!("ERROR {}{}\n", "b".repeat(70), "ñ".repeat(9));
    assert_ran(&run(&["parse", "logs"], &line.repeat(3)));
    assert_ran(&run(&["--json", "parse", "logs"], &line.repeat(3)));
}

#[test]
fn a_title_cut_inside_a_char() {
    let text = format!("{}{}\n\nbody\n", "d".repeat(76), "ñ".repeat(4));
    assert_ran(&run(&["--json", "txt2md"], &text));
}
