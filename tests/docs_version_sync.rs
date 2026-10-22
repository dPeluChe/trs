//! The site and install docs show the current release in a few fixed spots.
//! They went stale (v0.5.12 on usetrs.dev while npm served 0.8.9), so the
//! release PR now fails until they follow Cargo.toml.

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[test]
fn pinned_versions_in_the_docs_are_the_current_release() {
    let tag = format!("v{VERSION}");
    for (file, needle) in [
        ("docs/index.html", format!("<span>{tag}</span>")),
        ("docs/index.html", format!("TRS_VERSION={tag}")),
        ("docs/support/install.md", format!("TRS_VERSION={tag}")),
        (
            "docs/support/install.md",
            format!("@dpeluche/trs@{VERSION}"),
        ),
        ("docs/support/install.md", format!("--version {VERSION}")),
        ("docs/install.sh", format!("TRS_VERSION={tag} ")),
    ] {
        assert!(
            read(file).contains(&needle),
            "{file} should contain `{needle}`: update the docs with the version bump"
        );
    }
}
