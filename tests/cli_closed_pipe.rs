#![cfg(unix)]
//! `trs <cmd> | head` ends like any other tool whose reader left: no panic
//! text, status 141 (128 + SIGPIPE).

use std::io::Read;
use std::process::{Command, Stdio};

#[test]
fn a_reader_that_leaves_early_is_not_a_panic() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_trs"))
        .args(["seq", "1", "200000"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut out = child.stdout.take().unwrap();
    let mut first = [0u8; 8];
    out.read_exact(&mut first).unwrap();
    drop(out);

    let status = child.wait().unwrap();
    let mut err = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut err)
        .unwrap();
    assert!(!err.contains("panicked"), "{err}");
    assert_eq!(status.code(), Some(141), "{err}");
}
