use assert_cmd::Command;

/// Every subcommand clap defines must also be listed in `is_external_fast_path`.
///
/// That list is hand-maintained and runs before clap for startup speed, so a
/// name missing from it does not produce an error: trs treats the subcommand
/// as an external binary to execute and reports `Command not found`. The new
/// command simply appears not to exist, which is how `trs report` behaved
/// until this test existed.
#[test]
fn every_clap_subcommand_takes_the_clap_path() {
    let help = Command::cargo_bin("trs")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();
    let help = String::from_utf8_lossy(&help.stdout);

    let names: Vec<String> = help
        .lines()
        .skip_while(|l| !l.starts_with("Commands:"))
        .skip(1)
        .take_while(|l| !l.starts_with("Options:"))
        .filter_map(|l| {
            let t = l.trim_start();
            let first = t.split_whitespace().next()?;
            (l.starts_with("  ") && first.chars().next()?.is_ascii_lowercase())
                .then(|| first.to_string())
        })
        .collect();

    assert!(names.len() > 10, "parsed too few subcommands: {names:?}");

    for name in &names {
        if name == "help" {
            continue;
        }
        let out = Command::cargo_bin("trs")
            .unwrap()
            .args([name, "--help"])
            .output()
            .unwrap();
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            !combined.contains("Command not found"),
            "`trs {name}` fell through to the external path. Add \"{name}\" to \
             is_external_fast_path in src/main_args.rs."
        );
    }
}
