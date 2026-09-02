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

/// Every command the hint advertises must exist in the clap tree. This pins the
/// hint text against the command definitions; it does NOT cover the
/// hand-maintained fast-path list in main_args.rs that caused #158, which
/// tests/cli_subcommands_reach_clap.rs exercises against the real binary.
#[test]
fn every_command_the_install_hint_advertises_exists() {
    use clap::CommandFactory;

    let advertised: Vec<&str> = crate::report::HINT
        .lines()
        .filter_map(|l| l.trim().strip_prefix("trs "))
        .filter_map(|rest| rest.split("  ").next())
        .collect();
    assert!(
        !advertised.is_empty(),
        "hint advertises no commands, the parse stopped matching:\n{}",
        crate::report::HINT
    );

    for path in &advertised {
        let mut node = crate::cli::Cli::command();
        for (depth, name) in path.split_whitespace().enumerate() {
            let Some(sub) = node.find_subcommand(name).cloned() else {
                panic!(
                    "the install hint advertises `trs {path}`, but `{name}` is not a subcommand \
                     of `trs {}`. Either fix the hint in src/report.rs or add the command.",
                    path.split_whitespace()
                        .take(depth)
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            };
            node = sub;
        }
    }
}
