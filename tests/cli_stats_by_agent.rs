//! `trs stats --by-agent` must say which stretch of history it covers, and
//! `--days` must narrow it: the table is read as "my agents lately".

use assert_cmd::Command;

fn entry(ts: u64, agent: Option<&str>) -> String {
    let agent = agent
        .map(|a| format!(",\"agent\":\"{a}\""))
        .unwrap_or_default();
    format!("{{\"ts\":{ts},\"cmd\":\"git status\",\"in_bytes\":400,\"out_bytes\":100,\"saved_pct\":75,\"ms\":3,\"cwd\":\"/x\"{agent}}}\n")
}

fn by_agent(args: &[&str]) -> String {
    let home = tempfile::tempdir().unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let day = 86400;
    let history = [
        entry(now - 40 * day, Some("claude")),
        entry(now - 40 * day, None),
        entry(now - 60, Some("opencode")),
    ]
    .concat();
    std::fs::write(home.path().join("history.jsonl"), history).unwrap();
    let out = Command::cargo_bin("trs")
        .unwrap()
        .env("TRS_HOME", home.path())
        .args(["stats", "--by-agent"])
        .args(args)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn the_table_states_its_period_and_days_narrows_it() {
    let all = by_agent(&[]);
    assert!(all.contains("Period: "), "{all}");
    assert!(all.contains("(40 days)"), "{all}");
    assert!(
        all.contains("  (untagged) ") && all.contains("  claude "),
        "{all}"
    );

    let week = by_agent(&["--days", "7"]);
    assert!(week.contains("(1 day)"), "{week}");
    assert!(week.contains("opencode"), "{week}");
    assert!(
        !week.contains("  claude ") && !week.contains("  (untagged) "),
        "{week}"
    );
}
