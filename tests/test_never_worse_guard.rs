//! Never-worse output guard (rtk 0.43.0 `af81b08`): the compressing path must
//! never emit MORE bytes than the raw command output. Generic compression
//! already self-guards; these tests cover the *parser* path, where a dedicated
//! parser can add summary/header overhead that exceeds a tiny raw output.

use assert_cmd::Command;
use std::fs;

fn trs() -> Command {
    Command::cargo_bin("trs").unwrap()
}

/// `ls` of a single short filename is the classic case: the raw output is
/// ~13 bytes, but the ls parser's `[N files, M dirs]` summary would make it
/// longer. The guard must fall back to raw.
#[test]
fn ls_single_file_never_grows() {
    let dir = std::env::temp_dir().join("trs_nw_ls_one");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("f.txt"), "").unwrap();

    let out = trs().arg("ls").current_dir(&dir).assert().success();
    let stdout = &out.get_output().stdout;

    // Raw `ls` on this dir is just "f.txt\n". trs must not exceed it.
    let raw_len = "f.txt\n".len();
    assert!(
        stdout.len() <= raw_len,
        "trs ls emitted {} bytes for a single-file dir, raw is {} — never-worse guard failed: {:?}",
        stdout.len(),
        raw_len,
        String::from_utf8_lossy(stdout)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Guard holds for a directory listing that groups into subdirs too: whatever
/// the parser emits, it never exceeds the raw byte count. (Compression itself
/// is covered by the parser suites; here we only assert the ceiling.)
#[test]
fn ls_with_subdirs_never_grows() {
    let dir = std::env::temp_dir().join("trs_nw_ls_dirs");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for i in 0..5 {
        fs::create_dir_all(dir.join(format!("sub_{i}"))).unwrap();
        fs::write(dir.join(format!("f_{i}.txt")), "").unwrap();
    }

    // Raw `ls` here is one entry per line; compute its exact byte length.
    let mut names: Vec<String> = Vec::new();
    for i in 0..5 {
        names.push(format!("f_{i}.txt"));
        names.push(format!("sub_{i}"));
    }
    names.sort();
    let raw_len: usize = names.iter().map(|n| n.len() + 1).sum();

    let out = trs().arg("ls").current_dir(&dir).assert().success();
    let stdout_len = out.get_output().stdout.len();
    assert!(
        stdout_len <= raw_len,
        "trs ls emitted {stdout_len} bytes, raw is {raw_len} — never-worse guard failed"
    );

    let _ = fs::remove_dir_all(&dir);
}
