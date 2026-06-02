//! Golden tests pinning the registry to the exact behavior of the four
//! hand-maintained tables it replaced. If any ratio / stderr policy / known
//! / rewrite flag drifts, these fail.

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
    // Unknown git subcommand → command default (was the `_ => 0.50` arm).
    assert_eq!(r("git", "commit"), 0.50);
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
    // Commands present in classify/rewrite but never in keep_ratio → default.
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
    "ollama",
    "kubectl",
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
    "sed",
    "cd",
    "echo",
    "go",
    "poetry",
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
    assert!(!is_known_binary("psql"));
    assert!(!is_known_binary("bash"));
    assert!(!is_known_binary("journalctl"));
    assert!(!is_known_binary("totally-unknown"));
}

#[test]
fn rewrite_eligibility_matches_legacy_prefixes() {
    // Commands that were in REWRITE_PREFIXES.
    for cmd in [
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
        "ollama",
        "kubectl",
        "swift",
        "xcodebuild",
        "ping",
        "brew",
        "python",
        "python3",
        "npx",
        "ps",
        "uv",
        "bash",
        "node",
        "awk",
        "du",
        "jq",
    ] {
        assert!(is_rewrite_command(cmd), "{cmd} should be rewrite-eligible");
    }
    // Commands that were NOT in REWRITE_PREFIXES (still wrapped by catch-all,
    // but not part of the documented explicit set).
    for cmd in ["go", "poetry", "psql", "journalctl", "cat", "cd"] {
        assert!(
            !is_rewrite_command(cmd),
            "{cmd} should not be in explicit set"
        );
    }
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
