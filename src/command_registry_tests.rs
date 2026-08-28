//! Golden tests pinning the registry to the exact behavior of the four
//! hand-maintained tables it replaced. If any ratio / stderr policy / known
//! drifts, these fail.

use super::*;

fn r(cmd: &str, sub: &str) -> f64 {
    keep_ratio(cmd, sub)
}

#[test]
fn keep_ratio_git_subcommands() {
    assert_eq!(r("git", "status"), 0.20);
    assert_eq!(r("git", "diff"), 0.10);
    assert_eq!(r("git", "log"), 0.10);
    assert_eq!(r("git", "branch"), 0.11);
    assert_eq!(r("git", "show"), 0.10);
    assert_eq!(r("git", "stash"), 0.10);
    assert_eq!(r("git", "pull"), 0.15);
    assert_eq!(r("git", "fetch"), 0.15);
    assert_eq!(r("git", "grep"), 0.40);
    assert_eq!(r("git", "commit"), 0.20);
    // Unknown git subcommand → command default (was the `_ => 0.50` arm).
    assert_eq!(r("git", "rebase"), 0.50);
}

#[test]
fn keep_ratio_flat_commands() {
    assert_eq!(r("ls", ""), 0.18);
    assert_eq!(r("lsd", "anything"), 0.18);
    assert_eq!(r("exa", "x"), 0.18);
    assert_eq!(r("eza", "x"), 0.18);
    assert_eq!(r("tree", ""), 0.30);
    assert_eq!(r("find", ""), 0.52);
    assert_eq!(r("fd", ""), 0.52);
    assert_eq!(r("grep", ""), 0.40);
    assert_eq!(r("rg", ""), 0.40);
    assert_eq!(r("ag", ""), 0.40);
    assert_eq!(r("env", ""), 0.32);
    assert_eq!(r("printenv", ""), 0.32);
    assert_eq!(r("make", ""), 0.15);
    assert_eq!(r("gcc", ""), 0.15);
    assert_eq!(r("g++", ""), 0.15);
    assert_eq!(r("tsc", ""), 0.15);
    assert_eq!(r("pytest", ""), 0.10);
    assert_eq!(r("jest", ""), 0.10);
    assert_eq!(r("vitest", ""), 0.10);
    assert_eq!(r("wget", ""), 0.15);
    assert_eq!(r("curl", ""), 0.15);
    assert_eq!(r("eslint", ""), 0.15);
    assert_eq!(r("biome", ""), 0.15);
    assert_eq!(r("ruff", ""), 0.15);
    assert_eq!(r("pylint", ""), 0.15);
    assert_eq!(r("golangci-lint", ""), 0.15);
}

#[test]
fn keep_ratio_package_manager_overrides() {
    // install/i across the shared family.
    for cmd in ["npm", "pnpm", "yarn", "pip", "pip3", "cargo"] {
        assert_eq!(r(cmd, "install"), 0.20, "{cmd} install");
        assert_eq!(r(cmd, "i"), 0.20, "{cmd} i");
    }
    // ls/list/tree/freeze only for npm/pip/pip3/cargo (NOT pnpm/yarn/bun).
    for cmd in ["npm", "pip", "pip3", "cargo"] {
        assert_eq!(r(cmd, "ls"), 0.40, "{cmd} ls");
        assert_eq!(r(cmd, "list"), 0.40, "{cmd} list");
        assert_eq!(r(cmd, "tree"), 0.40, "{cmd} tree");
        assert_eq!(r(cmd, "freeze"), 0.40, "{cmd} freeze");
    }
    // pnpm/yarn do NOT get the list ratio → command default.
    assert_eq!(r("pnpm", "list"), 0.50);
    assert_eq!(r("yarn", "list"), 0.50);
    // bun install was never in the install family → default.
    assert_eq!(r("bun", "install"), 0.50);
}

#[test]
fn keep_ratio_test_and_run() {
    for cmd in ["npm", "pnpm", "bun", "yarn"] {
        assert_eq!(r(cmd, "test"), 0.10, "{cmd} test");
        assert_eq!(r(cmd, "run"), 0.15, "{cmd} run");
    }
}

#[test]
fn keep_ratio_cargo_specific() {
    assert_eq!(r("cargo", "clippy"), 0.15);
    assert_eq!(r("cargo", "build"), 0.10);
    assert_eq!(r("cargo", "check"), 0.10);
    assert_eq!(r("cargo", "test"), 0.05);
    assert_eq!(r("cargo", "tree"), 0.40);
    assert_eq!(r("cargo", "install"), 0.20);
}

#[test]
fn keep_ratio_misc_overrides() {
    assert_eq!(r("go", "test"), 0.08);
    assert_eq!(r("go", "build"), 0.50);
    assert_eq!(r("docker", "ps"), 0.30);
    assert_eq!(r("docker", "logs"), 0.50);
    assert_eq!(r("docker", "build"), 0.50);
    assert_eq!(r("gh", "pr"), 0.30);
    assert_eq!(r("gh", "issue"), 0.30);
    assert_eq!(r("gh", "run"), 0.30);
    assert_eq!(r("poetry", "install"), 0.20);
    assert_eq!(r("poetry", "add"), 0.20);
    assert_eq!(r("poetry", "update"), 0.20);
    assert_eq!(r("poetry", "run"), 0.15);
    assert_eq!(r("poetry", "lock"), 0.50);
}

#[test]
fn keep_ratio_default_for_unknown_and_no_entry_commands() {
    assert_eq!(r("totally-unknown", ""), 0.50);
    // Commands present in classify but never in keep_ratio → default.
    assert_eq!(r("ack", ""), 0.50);
    assert_eq!(r("tail", ""), 0.50);
    assert_eq!(r("cmake", ""), 0.50);
    assert_eq!(r("clang", ""), 0.50);
    assert_eq!(r("javac", ""), 0.50);
    assert_eq!(r("swift", "build"), 0.50);
    assert_eq!(r("xcodebuild", ""), 0.50);
    assert_eq!(r("wc", ""), 0.50);
    assert_eq!(r("ps", ""), 0.50);
    assert_eq!(r("ping", ""), 0.50);
    assert_eq!(r("brew", ""), 0.50);
    assert_eq!(r("python", ""), 0.50);
    assert_eq!(r("python3", ""), 0.50);
    assert_eq!(r("npx", ""), 0.50);
    assert_eq!(r("uv", ""), 0.50);
    assert_eq!(r("ollama", ""), 0.50);
    assert_eq!(r("kubectl", ""), 0.50);
}

#[test]
fn combine_stderr_always_commands() {
    for cmd in [
        "eslint",
        "biome",
        "ruff",
        "pylint",
        "golangci-lint",
        "make",
        "cmake",
        "gcc",
        "g++",
        "clang",
        "javac",
        "xcodebuild",
    ] {
        assert!(combine_stderr(cmd, ""), "{cmd} should merge stderr");
        assert!(combine_stderr(cmd, "anything"), "{cmd} any subcmd");
    }
}

#[test]
fn combine_stderr_subcommand_scoped() {
    assert!(combine_stderr("cargo", "clippy"));
    assert!(combine_stderr("cargo", "build"));
    assert!(combine_stderr("cargo", "check"));
    assert!(!combine_stderr("cargo", "test"));
    assert!(!combine_stderr("cargo", "run"));

    assert!(combine_stderr("go", "build"));
    assert!(!combine_stderr("go", "test"));

    assert!(combine_stderr("swift", "build"));
    assert!(!combine_stderr("swift", "test"));
}

#[test]
fn combine_stderr_never_commands() {
    for cmd in ["git", "ls", "npm", "docker", "curl", "tsc", "pytest"] {
        assert!(!combine_stderr(cmd, ""), "{cmd} should NOT merge stderr");
    }
    assert!(!combine_stderr("totally-unknown", ""));
}

/// The exact set of binaries `is_known_binary` returned before the registry.
const GOLDEN_KNOWN: &[&str] = &[
    "git",
    "ls",
    "lsd",
    "exa",
    "eza",
    "tree",
    "find",
    "fd",
    "grep",
    "rg",
    "ag",
    "ack",
    "tail",
    "cargo",
    "npm",
    "pnpm",
    "bun",
    "yarn",
    "pip",
    "pip3",
    "pytest",
    "jest",
    "vitest",
    "make",
    "cmake",
    "tsc",
    "gcc",
    "g++",
    "clang",
    "javac",
    "docker",
    "gh",
    "env",
    "printenv",
    "wc",
    "wget",
    "curl",
    "eslint",
    "biome",
    "ruff",
    "pylint",
    "golangci-lint",
    "swift",
    "xcodebuild",
    "ping",
    "brew",
    "python",
    "python3",
    "npx",
    "ps",
    "uv",
    "trs",
    "cat",
    "head",
    "cd",
    "echo",
    "go",
    "poetry",
    "aws",
    // Parsed, so counted as handled: journalctl shares the Logs parser and
    // the db clients route to the Db parser.
    "journalctl",
    // `list`/`ps`/`pull` are parsed; the rest of ollama falls to generic.
    "ollama",
    "psql",
    "mysql",
    "sqlite3",
    "mariadb",
    "bunx",
    "du",
    "lsof",
    "pgrep",
    // Verbatim: handled by being left alone, so coverage counts them as
    // handled rather than reporting them as missing parsers. Same treatment
    // cat/head/echo already get above.
    "awk",
    "sed",
    "base64",
    "basenc",
    "column",
    "comm",
    "cut",
    "expand",
    "fold",
    "hexdump",
    "iconv",
    "join",
    "jq",
    "nl",
    "od",
    "paste",
    "printf",
    "rev",
    "sort",
    "strings",
    "tac",
    "tr",
    "unexpand",
    "uniq",
    "xxd",
    "yq",
];

#[test]
fn is_known_binary_matches_golden_set_exactly() {
    // Every golden name is known.
    for name in GOLDEN_KNOWN {
        assert!(is_known_binary(name), "{name} should be known");
    }
    // Nothing outside the golden set is known. Walk every registered name.
    for spec in REGISTRY {
        for name in spec.names {
            let expected = GOLDEN_KNOWN.contains(name);
            assert_eq!(
                is_known_binary(name),
                expected,
                "{name}: known flag disagrees with golden set"
            );
        }
    }
    // A clearly-unknown binary is not known.
    // `bash` stays unknown on purpose: the classifier arm only helps for
    // `bash -c "<one simple command>"`; `bash script.sh` gets the generic
    // reducer like anything else.
    assert!(!is_known_binary("bash"));
    assert!(!is_known_binary("kubectl"));
    assert!(!is_known_binary("totally-unknown"));
}

#[test]
fn no_duplicate_command_names() {
    let mut seen: Vec<&str> = Vec::new();
    for spec in REGISTRY {
        for name in spec.names {
            assert!(!seen.contains(name), "duplicate command name: {name}");
            seen.push(name);
        }
    }
}

#[test]
fn verbatim_commands_are_counted_as_known() {
    for name in VERBATIM_COMMANDS {
        // Declared as known so `stats --coverage` stops reporting them as
        // parser gaps: trs handles them, by deliberately not touching them.
        assert!(is_known_binary(name), "should be known: {name}");
    }
}

#[test]
fn a_caller_selected_field_list_is_left_alone() {
    // `gh api --jq '{name, url}'` returns valid JSON, so the gh-api pruner
    // would happily strip `url` back out: a key the caller named on purpose.
    // Caught as a live regression while sharing the verbatim predicate.
    assert!(is_verbatim_invocation(
        "gh",
        " api repos/o/r --jq '{name, url}'"
    ));
    assert!(is_verbatim_invocation("gh", " api repos/o/r -q .name"));
    assert!(is_verbatim_invocation(
        "gh",
        " api repos/o/r --template '{{.name}}'"
    ));
    // Without a selector the response is GitHub's full body: prune it.
    assert!(!is_verbatim_invocation("gh", " api repos/o/r"));
    // Keyed by subcommand, not just binary: `-t` is `--template` on `gh api`
    // but `--title` on these three, which would otherwise lose compression.
    assert!(!is_verbatim_invocation("gh", " pr create -t Title -b body"));
    assert!(!is_verbatim_invocation(
        "gh",
        " release create v1.0 -t Title"
    ));
    assert!(!is_verbatim_invocation("gh", " issue create -t Title"));
    // And it does not leak to other tools.
    assert!(!is_verbatim_invocation("npm", " test --jq x"));
}

#[test]
fn verbatim_gate_sees_through_a_shell_wrapper() {
    // `bash -c "column -t x"` must be left alone like a bare `column -t x`:
    // wrapping costs the child its tty, so `column` falls back to 80 columns
    // before anything downstream can help it.
    assert!(is_verbatim_invocation("column", " -t data.tsv"));
    assert!(is_verbatim_invocation("bash", " -c \"column -t data.tsv\""));
    assert!(is_verbatim_invocation("sh", " -c 'awk NR<=4 f.py'"));
    assert!(is_verbatim_invocation("bash", " -lc \"cut -c1-20 f.py\""));
    // Still compressible: the inner command is not verbatim.
    assert!(!is_verbatim_invocation("bash", " -c \"ls -la src\""));
    assert!(!is_verbatim_invocation("bash", " script.sh"));
    assert!(!is_verbatim_invocation("npm", " test"));
}

#[test]
fn compressible_commands_stay_out_of_the_verbatim_class() {
    // Regression guard: widening VERBATIM_COMMANDS to a command that has a
    // parser silently drops its compression instead of failing loudly.
    for name in ["ls", "git", "cargo", "npm", "grep", "find", "du", "poetry"] {
        assert!(!is_verbatim_command(name), "must stay compressible: {name}");
    }
}

#[test]
fn bunx_dispatches_like_npx() {
    let npx = spec("npx").expect("npx in registry");
    let bunx = spec("bunx").expect("bunx in registry");
    assert!(std::ptr::eq(npx, bunx), "bunx must share npx's spec row");
}
