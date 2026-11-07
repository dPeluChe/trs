//! CSV and TSV are data for a pipeline: every row, and a failure keeps the table.

use assert_cmd::Command;

mod support;

fn trs() -> Command {
    Command::cargo_bin("trs").unwrap()
}

fn tree_of(files: usize) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..files {
        support::write_file(dir.path(), &format!("f{i}.txt"), "needle\n");
    }
    dir
}

fn stdout(cmd: &mut Command) -> String {
    String::from_utf8_lossy(&cmd.output().unwrap().stdout).into_owned()
}

#[test]
fn search_tables_keep_every_file_unless_limited() {
    let dir = tree_of(60);
    let path = dir.path().to_str().unwrap();
    let all = stdout(trs().args(["--csv", "search", path, "needle"]));
    assert_eq!(all.lines().count(), 61, "{all}");
    let limited = stdout(trs().args(["--tsv", "search", path, "needle", "--limit", "5"]));
    assert_eq!(limited.lines().count(), 6, "{limited}");
    let compact = stdout(trs().args(["search", path, "needle"]));
    assert!(compact.contains("not shown"), "{compact}");
}

#[test]
fn parse_grep_tables_keep_every_match() {
    let input: String = (1..=300).map(|i| format!("a.rs:{i}:needle\n")).collect();
    let out = stdout(trs().args(["--csv", "parse", "grep"]).write_stdin(input));
    assert_eq!(out.lines().count(), 301, "{out}");
}

#[cfg(unix)]
#[test]
fn a_failed_command_keeps_its_table() {
    let dir = tree_of(1);
    let file = dir.path().join("f0.txt");
    let file = file.to_str().unwrap();
    let out = trs()
        .args(["--csv", "grep", "needle", file, "/no/such/file"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(2));
    assert!(text.starts_with("path,line_number,"), "{text}");

    let out = trs().args(["--csv", "grep", "zzz", file]).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("path,"));
}

#[cfg(unix)]
#[test]
fn a_failed_command_without_a_table_shows_its_raw_output() {
    let dir = tempfile::tempdir().unwrap();
    let body = "echo src/a.js; echo '  1:1  warning  unused var  no-unused'; exit 1\n";
    support::fake_program(&dir.path().join("eslint"), body);
    let out = trs()
        .env("PATH", support::path_with(dir.path()))
        .args(["--csv", "eslint", "src"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1));
    assert!(text.contains("unused var"), "{text}");
}

#[test]
fn a_match_that_reads_like_a_summary_is_still_a_row() {
    let out = stdout(
        trs()
            .args(["--csv", "parse", "grep"])
            .write_stdin("a.txt:12:we matched 3 files\n"),
    );
    assert_eq!(out.lines().count(), 2, "{out}");
}

#[cfg(unix)]
#[test]
fn err_tables_keep_every_line_of_a_failure() {
    let script = "i=0; while [ $i -lt 400 ]; do echo line$i; i=$((i+1)); done; exit 3";
    let out = stdout(trs().args(["--csv", "err", "sh", "-c", script]));
    assert_eq!(out.lines().count(), 401, "{out:.200}");
}
