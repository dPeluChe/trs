use super::*;

fn argv(s: &str) -> Vec<String> {
    s.split_whitespace().map(String::from).collect()
}

#[test]
fn ls_files_routes_to_find() {
    assert!(matches!(
        classify_command("git", &argv("ls-files --others --exclude-standard")),
        Some(ParseCommands::Find { .. })
    ));
}

#[test]
fn commit_routes_to_parser() {
    assert!(matches!(
        classify_command("git", &argv("commit -m msg")),
        Some(ParseCommands::GitCommit { .. })
    ));
    // Structured-output flags stay passthrough.
    assert!(classify_command("git", &argv("commit --porcelain")).is_none());
}

#[test]
fn cargo_fmt_routes_to_parser() {
    assert!(matches!(
        classify_command("cargo", &argv("fmt --check")),
        Some(ParseCommands::Fmt { .. })
    ));
    assert!(matches!(
        classify_command("cargo", &argv("fmt")),
        Some(ParseCommands::Fmt { .. })
    ));
}

#[test]
fn bash_c_simple_command_unwraps() {
    assert!(matches!(
        classify_command("bash", &["-c".into(), "git status".into()]),
        Some(ParseCommands::GitStatus { .. })
    ));
    assert!(matches!(
        classify_command("sh", &["-c".into(), "cargo test --lib".into()]),
        Some(ParseCommands::CargoTest { .. })
    ));
}

#[test]
fn absolute_path_routes_by_basename() {
    // Field data: `/opt/homebrew/bin/gh` averaged 40 KB/cmd uncompressed
    // because the classifier matched the full path, not `gh`.
    assert!(matches!(
        classify_command("/opt/homebrew/bin/gh", &argv("pr list")),
        Some(ParseCommands::GhPr { .. })
    ));
    assert!(matches!(
        classify_command("/usr/bin/git", &argv("status")),
        Some(ParseCommands::GitStatus { .. })
    ));
    assert!(matches!(
        classify_command("./node_modules/.bin/eslint", &argv("src")),
        Some(ParseCommands::Lint { .. })
    ));
}

#[test]
fn bare_python_linters_and_formatters_route() {
    for c in ["flake8", "mypy"] {
        assert!(
            matches!(
                classify_command(c, &argv("src")),
                Some(ParseCommands::Lint { .. })
            ),
            "{c} should route to Lint"
        );
    }
    for c in ["black", "isort"] {
        assert!(
            matches!(
                classify_command(c, &argv(".")),
                Some(ParseCommands::Fmt { .. })
            ),
            "{c} should route to Fmt"
        );
    }
}

#[test]
fn timeout_unwraps_inner_command() {
    assert!(matches!(
        classify_command("timeout", &argv("30 cargo test")),
        Some(ParseCommands::CargoTest { .. })
    ));
    // Option with a separate value, then duration with a unit suffix.
    assert!(matches!(
        classify_command("timeout", &argv("-s KILL 5s git status")),
        Some(ParseCommands::GitStatus { .. })
    ));
    // No duration / no inner command → passthrough.
    assert!(classify_command("timeout", &argv("--help")).is_none());
    assert!(classify_command("timeout", &argv("30")).is_none());
}

#[test]
fn bash_c_compound_or_quoted_stays_generic() {
    for script in [
        "echo a; git status",
        "git status | head",
        "ls && pwd",
        "echo \"hi\"",
        "node -e 'console.log(1)'",
        "VAR=$(date) printenv",
    ] {
        assert!(
            classify_command("bash", &["-c".into(), script.into()]).is_none(),
            "should stay generic: {script}"
        );
    }
}

#[test]
fn git_show_blob_form_is_not_a_diff() {
    // `git show <rev>:<path>` prints raw file contents. Routing it to the diff
    // parser yielded "diff: empty" — silently destroying the file when the
    // caller redirected it into one.
    let blob = |a: &[&str]| {
        classify_command("git", &a.iter().map(|s| s.to_string()).collect::<Vec<_>>()).is_none()
    };
    assert!(blob(&["show", "main:src/App.tsx"]));
    assert!(blob(&["show", "HEAD:Cargo.toml"]));
    assert!(blob(&["show", "origin/main:a/b.ts"]));
    // Ordinary commit views still route to the diff parser.
    assert!(!blob(&["show", "HEAD"]));
    assert!(!blob(&["show", "--stat", "HEAD~2"]));
}
