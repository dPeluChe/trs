//! Every parser fixture, run through `trs parse`, must:
//! - print identical bytes on a second run (determinism);
//! - keep its savings within 2 points of `tests/compression_baseline.json`
//!   and its parser unchanged (the ratchet);
//! - keep the literals listed in `MUST_PRESERVE` (failures stay visible).
//!
//! After an intended change, `UPDATE_RATCHET=1 cargo test --test
//! cli_compression_ratchet` rewrites the baseline; review the diff.

use assert_cmd::Command;
use serde_json::{json, Map, Value};
use std::path::PathBuf;

/// Fixture file prefix to the `trs parse` arguments that handle it.
const PARSERS: &[(&str, &[&str])] = &[
    ("git_status_", &["git-status"]),
    ("git_diff_", &["git-diff"]),
    ("ls_", &["ls"]),
    ("grep_", &["grep"]),
    ("logs_", &["logs"]),
    ("tail_", &["logs"]),
    ("pytest_", &["test", "--runner", "pytest"]),
    ("jest_", &["test", "--runner", "jest"]),
    ("vitest_", &["test", "--runner", "vitest"]),
    ("bun_test_", &["test", "--runner", "bun"]),
    ("npm_test_", &["test", "--runner", "npm"]),
    ("pnpm_test_", &["test", "--runner", "pnpm"]),
];

/// Literals a failure fixture's summary must still show.
const MUST_PRESERVE: &[(&str, &[&str])] = &[
    (
        "pytest_all_failed.txt",
        &[
            "tests/test_main.py::test_one",
            "AssertionError: assert 1 == 2",
            "ValueError: Test error",
        ],
    ),
    (
        "jest_all_failed.txt",
        &["src/test1.test.js", "should work", "should pass"],
    ),
    (
        "vitest_all_failed.txt",
        &["test/test1.test.ts", "should work"],
    ),
    (
        "bun_test_all_failed.txt",
        &["test/test1.test.js", "AssertionError: test 1 failed"],
    ),
    (
        "npm_test_all_failed.txt",
        &["test/test2.test.js", "should pass"],
    ),
    (
        "pnpm_test_all_failed.txt",
        &["test/test2.test.js", "should pass"],
    ),
    (
        "git_status_conflict.txt",
        &[
            "src/conflict.rs",
            "src/new_file.rs",
            "src/removed_by_them.rs",
        ],
    ),
    (
        "logs_errors_only.txt",
        &["NullPointerException", "File not found: /data/config.json"],
    ),
];

const MAX_DROP_POINTS: f64 = 2.0;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")
}

fn parse(args: &[&str], file: &PathBuf) -> String {
    let out = Command::cargo_bin("trs")
        .unwrap()
        .arg("parse")
        .args(args)
        .arg("-f")
        .arg(file)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn saved(inp: u64, out: u64) -> f64 {
    if inp == 0 {
        0.0
    } else {
        100.0 * (1.0 - out as f64 / inp as f64)
    }
}

#[test]
fn fixtures_are_deterministic_ratcheted_and_keep_their_failures() {
    let mut files: Vec<_> = std::fs::read_dir(root().join("fixture_data"))
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    files.sort();

    let baseline_path = root().join("compression_baseline.json");
    let baseline: Value = std::fs::read_to_string(&baseline_path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| json!({}));
    let update = std::env::var_os("UPDATE_RATCHET").is_some();

    let (mut fresh, mut problems) = (Map::new(), Vec::new());
    for file in &files {
        let name = file.file_name().unwrap().to_string_lossy().into_owned();
        let Some((_, args)) = PARSERS.iter().find(|(p, _)| name.starts_with(p)) else {
            continue;
        };
        let (first, second) = (parse(args, file), parse(args, file));
        if first != second {
            problems.push(format!("{name}: two runs printed different bytes"));
        }
        if let Some((_, literals)) = MUST_PRESERVE.iter().find(|(n, _)| *n == name) {
            for lit in *literals {
                if !first.contains(lit) {
                    problems.push(format!("{name}: summary lost `{lit}`"));
                }
            }
        }
        let inp = std::fs::metadata(file).unwrap().len();
        let (handler, out) = (args.join(" "), first.len() as u64);
        if let Some(base) = baseline.get(&name) {
            let (b_in, b_out) = (base["in"].as_u64().unwrap(), base["out"].as_u64().unwrap());
            if base["handler"] != handler.as_str() {
                problems.push(format!(
                    "{name}: parser changed from {} to {handler}",
                    base["handler"]
                ));
            }
            let drop = saved(b_in, b_out) - saved(inp, out);
            if drop > MAX_DROP_POINTS {
                problems.push(format!(
                    "{name}: savings fell {drop:.1} points ({b_out} to {out} bytes)"
                ));
            }
        } else if !update {
            problems.push(format!(
                "{name}: not in the baseline, run with UPDATE_RATCHET=1"
            ));
        }
        fresh.insert(name, json!({ "handler": handler, "in": inp, "out": out }));
    }

    if update {
        let text = serde_json::to_string_pretty(&Value::Object(fresh)).unwrap() + "\n";
        std::fs::write(&baseline_path, text).unwrap();
        return;
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

/// Extension counts tie often; a hash-ordered map made the top list and the
/// JSON key order change from one run to the next.
#[test]
fn find_summary_is_stable_when_extension_counts_tie() {
    let input = "a.rs\nb.py\nc.go\nd.js\ne.md\nf.toml\ng.sh\nh.txt\n";
    let run = |json: bool| {
        let mut cmd = Command::cargo_bin("trs").unwrap();
        if json {
            cmd.arg("--json");
        }
        cmd.args(["parse", "find"]).write_stdin(input);
        String::from_utf8_lossy(&cmd.output().unwrap().stdout).into_owned()
    };
    for json in [false, true] {
        let first = run(json);
        assert!((0..8).all(|_| run(json) == first), "json={json}");
    }
}
