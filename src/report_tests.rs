use super::scrub;

#[test]
fn scrub_rewrites_the_home_directory() {
    // The leak that motivated this command: a coverage report off a real
    // machine carried a full `/Users/<name>/...` path in a command sample.
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return;
    }
    let line = format!("ls {}/projects/client-work", home);
    let out = scrub(&line);
    assert!(out.starts_with("ls ~/"), "home not rewritten: {out}");
    assert!(!out.contains(&home), "home still present: {out}");
}

// Credential-shape coverage lives in `tracker_tests.rs`, which already has
// six cases against `redact_secrets`. Repeating them here would duplicate the
// coverage and plant a second set of fake credentials for the secret scanner
// to trip over, which is exactly what it did.

#[test]
fn scrub_cannot_detect_an_internal_hostname_and_does_not_pretend_to() {
    // Documented limit, asserted so nobody later claims the payload is safe
    // to post unread. `ssh <internal-host>` is not a pattern: it is a fact
    // about someone's network. The preview and the confirm prompt are what
    // protect this, not the redactor.
    let line = "ssh prod-db-01 sudo cat /etc/nftables.conf";
    assert_eq!(scrub(line), line);
}

#[test]
fn scrub_leaves_ordinary_command_lines_alone() {
    for line in ["cargo test --no-fail-fast", "git status --porcelain"] {
        assert_eq!(scrub(line), line);
    }
}

/// The install and upgrade tails print this hint, and #158 shipped a `trs
/// report` that answered "Command not found". Anything the hint tells a user
/// to run is resolved against the real clap tree, so advertising a command
/// that does not exist fails here instead of in someone's terminal.
#[test]
fn every_command_the_install_hint_advertises_exists() {
    use clap::CommandFactory;

    let hint = crate::report::hint();
    let advertised: Vec<Vec<&str>> = hint
        .lines()
        .filter_map(|l| l.trim().strip_prefix("trs "))
        .map(|l| {
            l.split("  ")
                .next()
                .unwrap_or("")
                .split_whitespace()
                .collect()
        })
        .filter(|w: &Vec<&str>| !w.is_empty())
        .collect();
    assert_eq!(
        advertised.len(),
        2,
        "hint should advertise both reports:\n{hint}"
    );

    for path in &advertised {
        let mut node = crate::cli::Cli::command();
        for (depth, name) in path.iter().enumerate() {
            let found = node.find_subcommand(name).cloned();
            match found {
                Some(sub) => node = sub,
                None => panic!(
                    "the install hint advertises `trs {}`, but `{name}` is not a subcommand of \
                     `trs {}`. Either fix the hint in src/report.rs or add the command.",
                    path.join(" "),
                    path[..depth].join(" ")
                ),
            }
        }
    }
}
