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
fn runners_reach_the_parser_of_what_they_run() {
    for (cmd, args) in [
        ("npx", "-y vitest run"),
        ("npx", "--yes -p vitest vitest run"),
        ("bunx", "--bun vitest"),
        ("uv", "run --with y --python 3.12 pytest -q"),
        ("poetry", "run pytest"),
        ("pipenv", "run pytest"),
        ("pnpm", "exec vitest"),
        ("yarn", "dlx vitest"),
    ] {
        assert!(
            matches!(
                classify_command(cmd, &argv(args)),
                Some(ParseCommands::Test { .. })
            ),
            "{cmd} {args}"
        );
    }
    for (cmd, args) in [
        ("nix", "develop -c cargo test"),
        ("nix", "develop .#ci --command cargo test"),
        ("nix", "shell nixpkgs#cargo -c cargo test"),
        ("mise", "exec rust@1.80 -- cargo test"),
        ("mise", "x -- cargo test"),
        ("direnv", "exec . cargo test"),
    ] {
        assert!(
            matches!(
                classify_command(cmd, &argv(args)),
                Some(ParseCommands::CargoTest { .. })
            ),
            "{cmd} {args}"
        );
    }
    assert!(matches!(
        classify_command("uv", &argv("run --no-project python a.py")),
        Some(ParseCommands::PythonTraceback { .. })
    ));
}

#[test]
fn a_runner_with_nothing_to_route_stays_out() {
    assert!(classify_command("just", &argv("test")).is_none());
    assert!(classify_command("npx", &argv("-c 'vitest run'")).is_none());
    assert!(classify_command("nix", &argv("develop")).is_none());
    assert!(classify_command("mise", &argv("run test")).is_none());
    assert!(classify_command("uv", &argv("run --with")).is_none());
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
