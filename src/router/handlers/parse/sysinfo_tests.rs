use super::{compress_du, compress_lsof, compress_pgrep, short_cmd, size_bytes, DU_ROWS};

// ---- du ----

#[test]
fn du_sorts_by_size_descending_and_totals() {
    let out = compress_du("8.0K\tsrc/a.rs\n 20M\tsrc/b.rs\n1.5G\tsrc/c.rs\n").unwrap();
    let first = out.lines().next().unwrap();
    assert!(
        first.contains("c.rs"),
        "largest must come first, got: {first}"
    );
    assert!(out.contains("3 entries"), "missing total line: {out}");
}

#[test]
fn du_summarizes_the_tail_beyond_the_cap() {
    let input: String = (0..DU_ROWS + 5)
        .map(|i| format!("{}K\tpath/{}\n", i + 1, i))
        .collect();
    let out = compress_du(&input).unwrap();
    assert!(out.contains("+5 smaller"), "tail not summarized: {out}");
    assert_eq!(
        out.lines().filter(|l| l.contains("path/")).count(),
        DU_ROWS,
        "must print exactly the cap"
    );
}

#[test]
fn du_declines_shapes_it_would_only_make_worse() {
    // `du -sh .` is a single line: there is nothing to sort or summarize.
    assert_eq!(compress_du("1.2G\t.\n"), None);
    // Not du output at all (a permission error, for instance).
    assert_eq!(compress_du("du: cannot read 'x'\nsecond line\n"), None);
    assert_eq!(compress_du(""), None);
}

// ---- lsof ----

const LSOF_SAMPLE: &str = "\
COMMAND     PID    USER   FD   TYPE             DEVICE SIZE/OFF   NODE NAME
Superset    671 peluche   59u  IPv4 0xfbf316ffa4f0b6da      0t0    TCP 127.0.0.1:49153 (LISTEN)
Superset    671 peluche   78u  IPv4 0xd1c60a47d7279cad      0t0    TCP 127.0.0.1:51741 (LISTEN)
rapportd    673 peluche    9u  IPv4 0x2ad7e262eaf7e19c      0t0    TCP *:56165 (LISTEN)
";

#[test]
fn lsof_folds_the_descriptors_of_one_process_into_one_row() {
    let out = compress_lsof(LSOF_SAMPLE).unwrap();
    assert_eq!(
        out.lines().filter(|l| l.starts_with("Superset")).count(),
        1,
        "two descriptors of one pid must fold into one row: {out}"
    );
    assert!(
        out.contains("49153") && out.contains("51741"),
        "both addresses must survive: {out}"
    );
    assert!(
        out.contains("2 processes, 3 file descriptors"),
        "bad footer: {out}"
    );
    assert!(out.len() < LSOF_SAMPLE.len(), "must be smaller than raw");
}

#[test]
fn lsof_keeps_the_whole_name_cell_including_its_spaces() {
    // NAME is the last column and contains spaces, so a whitespace split
    // would keep only "(LISTEN)" and throw away the address.
    let out = compress_lsof(LSOF_SAMPLE).unwrap();
    assert!(
        out.contains("TCP 127.0.0.1:49153 (LISTEN)"),
        "name truncated: {out}"
    );
}

#[test]
fn lsof_declines_without_the_standard_header() {
    // `lsof -F` machine format has no header row.
    assert_eq!(compress_lsof("p671\ncSuperset\nfcwd\n"), None);
    assert_eq!(compress_lsof(""), None);
}

// ---- pgrep ----

#[test]
fn pgrep_collapses_identical_command_lines() {
    let input = "\
20163 node /tmp/.npm-cache/_npx/abc/node_modules/.bin/context7-mcp
20275 node /tmp/.npm-cache/_npx/abc/node_modules/.bin/context7-mcp
20209 node /tmp/.npm-cache/_npx/def/node_modules/.bin/other-mcp
";
    let out = compress_pgrep(input).unwrap();
    assert!(
        out.contains("20163,20275"),
        "pids of one command must merge: {out}"
    );
    assert!(out.contains("3 processes"), "bad footer: {out}");
    assert!(out.len() < input.len(), "must be smaller than raw");
}

#[test]
fn pgrep_declines_bare_pid_output() {
    // `pgrep` without -l prints pids only: already minimal, and there is no
    // command column to group on.
    assert_eq!(compress_pgrep("20163\n20275\n"), None);
    assert_eq!(compress_pgrep(""), None);
}

// ---- helpers ----

#[test]
fn short_cmd_drops_the_argv0_path_but_keeps_the_arguments() {
    assert_eq!(
        short_cmd("/usr/local/bin/node app.js --port 3000"),
        "node app.js --port 3000"
    );
    assert_eq!(short_cmd("/opt/x/y/daemon"), "daemon");
    assert!(
        short_cmd(&"x".repeat(200)).ends_with('…'),
        "long lines must be capped"
    );
}

#[test]
fn size_bytes_reads_both_human_and_block_forms() {
    assert_eq!(size_bytes("1K"), Some(1024.0));
    assert_eq!(size_bytes("2.5M"), Some(2.5 * 1024.0 * 1024.0));
    // `du -s` without -h reports 1K blocks, not bytes.
    assert_eq!(size_bytes("4"), Some(4096.0));
    assert_eq!(size_bytes("nope"), None);
    assert_eq!(size_bytes(""), None);
}

#[test]
fn restructuring_output_may_be_longer_than_its_input() {
    // du sorts and totals, pgrep groups: for both, the reordering IS the
    // product, so a size guard would hand back the raw output and throw the
    // ordering away. These handlers use `emit_restructured` for that reason;
    // this test pins the case that would silently regress if one of them were
    // switched to `emit_compressed`.
    let du_in = "4.0K\ta\n8.0K\tb\n";
    let du_out = compress_du(du_in).unwrap();
    assert!(du_out.len() > du_in.len(), "the guard would suppress this");
    assert!(
        du_out.lines().next().unwrap().ends_with("8.0K  b"),
        "largest first: {du_out}"
    );

    let pg_in = "101 node\n102 node\n103 python3\n";
    let pg_out = compress_pgrep(pg_in).unwrap();
    assert!(pg_out.len() > pg_in.len(), "the guard would suppress this");
    assert!(pg_out.contains("101,102"), "grouping lost: {pg_out}");
}
