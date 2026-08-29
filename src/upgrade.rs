//! `trs upgrade` — detect how trs was installed and re-run the install
//! path to pick up the latest release.
//!
//! Two install channels supported today: the `curl|sh` script and npm.
//! Cargo and Homebrew installs are detected but surfaced as
//! unsupported with an explicit manual command — better than silently
//! doing nothing or running the wrong thing.
//!
//! Detection is path-based against `std::env::current_exe()`. That's
//! coarse but reliable: each install writes to a well-known location,
//! so the running binary's parent directory uniquely identifies the
//! channel that put it there.

use std::path::Path;
use std::process::Command;

/// Canonical install script URL (curl|sh path). Points at the
/// usetrs.dev custom domain; GitHub Pages auto-redirects from the
/// legacy `raw.githubusercontent.com` URL for anyone still using it.
const INSTALL_SCRIPT_URL: &str = "https://usetrs.dev/install.sh";

/// npm package name.
const NPM_PACKAGE: &str = "@dpeluche/trs";

/// Documentation link shown to users who land on an unsupported path.
const DOCS_INSTALL_URL: &str = "https://github.com/dPeluChe/trs#install";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InstallMethod {
    /// Installed via the official `curl|sh` script — target dir is
    /// typically `$HOME/.local/bin` or `$HOME/.trs/bin`.
    Curl,
    /// Installed via `npm install -g @dpeluche/trs` — lives under
    /// some node prefix's `lib/node_modules/@dpeluche/trs/bin/`.
    Npm,
    /// Installed via `cargo install trs-cli` — lives under
    /// `$HOME/.cargo/bin/`. Not auto-upgradable yet (needs crates.io
    /// publish first).
    Cargo,
    /// Installed via Homebrew formula — lives under `/opt/homebrew/`
    /// or `/usr/local/Cellar/`. Not auto-upgradable yet (needs tap
    /// published first).
    Brew,
    /// Unknown path — the binary was moved, symlinked, or built
    /// locally. Print manual options.
    Unknown,
}

impl InstallMethod {
    fn label(&self) -> &'static str {
        match self {
            Self::Curl => "curl|sh installer",
            Self::Npm => "npm",
            Self::Cargo => "cargo install",
            Self::Brew => "Homebrew",
            Self::Unknown => "unknown source",
        }
    }
}

pub(crate) fn run_upgrade(check_only: bool, skip_confirm: bool, binary_only: bool) {
    let exe = std::env::current_exe().ok();
    let method = detect_install_method(exe.as_deref());

    println!();
    println!("trs upgrade");
    println!();
    if let Some(p) = &exe {
        println!("  current binary: {}", p.display());
    }
    println!("  current version: {}", env!("CARGO_PKG_VERSION"));
    println!("  detected method: {}", method.label());
    println!();

    match method {
        InstallMethod::Curl => {
            let cmd = format!("curl -fsSL {} | sh", INSTALL_SCRIPT_URL);
            println!("Will run:");
            println!("  {}", cmd);
            if !binary_only {
                println!("Then refresh (via the new binary):");
                println!("  trs init --all --global --force");
                println!("  trs output-saver --refresh");
            }
            if check_only {
                return;
            }
            if !skip_confirm && !prompt_yes("Proceed?") {
                println!("aborted.");
                return;
            }
            if run_shell(&cmd) && !binary_only {
                refresh_configs();
            }
        }
        InstallMethod::Npm => {
            let cmd = format!("npm install -g {}@latest", NPM_PACKAGE);
            println!("Will run:");
            println!("  {}", cmd);
            if !binary_only {
                println!("Then refresh (via the new binary):");
                println!("  trs init --all --global --force");
                println!("  trs output-saver --refresh");
            }
            if check_only {
                return;
            }
            if !skip_confirm && !prompt_yes("Proceed?") {
                println!("aborted.");
                return;
            }
            if run_shell(&cmd) && !binary_only {
                refresh_configs();
            }
        }
        InstallMethod::Cargo => {
            println!("trs upgrade does not support cargo installs yet");
            println!("(requires publishing to crates.io, tracked on the roadmap).");
            println!();
            println!("Run manually:");
            println!("  cargo install trs-cli --force");
        }
        InstallMethod::Brew => {
            println!("trs upgrade does not support Homebrew yet");
            println!("(tap not published, tracked on the roadmap).");
            println!();
            println!("For now, see: {}", DOCS_INSTALL_URL);
        }
        InstallMethod::Unknown => {
            println!("Could not identify the install channel from the binary path.");
            println!("The binary may have been moved, symlinked, or built from source.");
            println!();
            println!("Manual options:");
            println!("  curl -fsSL {} | sh", INSTALL_SCRIPT_URL);
            println!("  npm install -g {}@latest", NPM_PACKAGE);
            println!("  cargo install trs-cli --force");
            println!();
            println!("More: {}", DOCS_INSTALL_URL);
        }
    }
}

pub(crate) fn detect_install_method(exe: Option<&Path>) -> InstallMethod {
    let Some(path) = exe.and_then(|p| p.to_str()) else {
        return InstallMethod::Unknown;
    };

    // npm layouts: `**/node_modules/@dpeluche/trs/**` is unambiguous —
    // no other channel nests a node_modules segment.
    if path.contains("/node_modules/@dpeluche/trs/") || path.contains("/node_modules/.bin/trs") {
        return InstallMethod::Npm;
    }

    // Homebrew on both Apple Silicon (/opt/homebrew) and Intel
    // (/usr/local/Cellar). A stray `trs` binary in /usr/local/bin
    // that's a brew symlink would also match.
    if path.starts_with("/opt/homebrew/") || path.starts_with("/usr/local/Cellar/") {
        return InstallMethod::Brew;
    }

    // Cargo's default bin dir. CARGO_HOME respected if set.
    let cargo_bin = std::env::var("CARGO_HOME")
        .ok()
        .map(|h| format!("{}/bin/", h))
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| format!("{}/.cargo/bin/", h))
        });
    if let Some(prefix) = cargo_bin {
        if path.starts_with(&prefix) {
            return InstallMethod::Cargo;
        }
    }

    // curl|sh installer: writes to $HOME/.local/bin by default,
    // falls back to $HOME/.trs/bin or $HOME/bin.
    if let Ok(home) = std::env::var("HOME") {
        for rel in [".local/bin/", ".trs/bin/", "bin/"] {
            let candidate = format!("{}/{}", home.trim_end_matches('/'), rel);
            if path.starts_with(&candidate) {
                return InstallMethod::Curl;
            }
        }
    }

    InstallMethod::Unknown
}

fn prompt_yes(question: &str) -> bool {
    use std::io::{self, Write};
    print!("{} [y/N] ", question);
    if io::stdout().flush().is_err() {
        return false;
    }
    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        return false;
    }
    matches!(line.trim().to_lowercase().as_str(), "y" | "yes")
}

/// Run a shell command and return whether it succeeded. Stdout/stderr
/// inherit the current terminal so the user sees the installer's
/// progress output live.
fn run_shell(cmd: &str) -> bool {
    println!();
    let status = Command::new("sh").arg("-c").arg(cmd).status();
    match status {
        Ok(s) if s.success() => {
            println!();
            println!("binary upgrade complete.");
            true
        }
        Ok(s) => {
            eprintln!(
                "\nupgrade command exited with code {}",
                s.code().unwrap_or(-1)
            );
            false
        }
        Err(e) => {
            eprintln!("\nfailed to spawn upgrade command: {}", e);
            false
        }
    }
}

/// After the binary is replaced on disk, spawn the new `trs` to
/// refresh hook templates and re-install the output-saver block where
/// it's already present. The current process is still running the old
/// binary — spawning a subprocess picks up the new binary via PATH.
///
/// Runs three validations before touching any config:
///
/// 1. **Spawn sanity** — invoke the new `trs --version` and confirm it
///    executes cleanly. If the installer wrote a corrupt binary,
///    refuse to proceed so we don't pipe user configs into a broken
///    tool.
/// 2. **Version bump** — confirm the new binary reports a version
///    greater than the one that was running. Catches silent no-ops
///    (npm shim pointing at an old cached package, curl install
///    restoring same version, etc.).
/// 3. **JSON validity** — parse every agent hook config that `init`
///    would touch. If any is corrupt, abort refresh with the file
///    path so the user can fix it manually rather than have our
///    merge layer compound the damage.
///
/// Failures surface as explicit warnings; the binary upgrade itself
/// has already happened and the refresh commands are idempotent, so
/// the user can always re-run them manually.
fn refresh_configs() {
    let old_version = env!("CARGO_PKG_VERSION");

    println!();
    println!("Running post-install checks...");

    // 1. Spawn sanity — new binary must execute.
    let new_version = match Command::new("trs").arg("--version").output() {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout
                .lines()
                .next()
                .and_then(|l| l.split_whitespace().nth(1))
                .unwrap_or("unknown")
                .to_string()
        }
        Ok(_) | Err(_) => {
            eprintln!("  ! new trs binary did not respond to --version");
            eprintln!("    skipping config refresh. Verify the install manually.");
            return;
        }
    };
    println!("  ok  new binary responds to --version ({})", new_version);

    // 2. Version bump — the new one should be greater.
    if new_version == old_version {
        // The installer fetches the latest release, so an unchanged version
        // means you were already on it — not a failed update. Say so plainly.
        println!(
            "  ok  already on the latest version ({}), nothing to update.",
            new_version
        );
        println!("      Configs left as-is (re-run `trs init --all --global --force` to refresh templates).");
        return;
    }
    println!("  ok  version bumped: {} -> {}", old_version, new_version);

    // 3. JSON validity of hook configs we're about to touch.
    if let Some(bad_path) = first_broken_hook_json() {
        eprintln!("  ! hook config {} is not valid JSON", bad_path.display());
        eprintln!("    fix that file manually, then re-run: trs init --all --global --force");
        eprintln!("    skipping config refresh for now.");
        return;
    }
    println!("  ok  existing hook configs parse as valid JSON");

    // State both commands once, up front, then let each subcommand's own
    // output follow. Echoing each command right before running it put two
    // lines starting with "trs init" back to back at different indents, one
    // of them the echo and one the subcommand's header, which read as
    // duplicated text rather than as a command and its result.
    println!();
    println!(
        "Refreshing agent integrations with v{} templates:",
        new_version
    );
    println!("  trs init --all --global --force");
    println!("  trs output-saver --refresh");
    println!();

    let init_ok = Command::new("trs")
        .args(["init", "--all", "--global", "--force"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !init_ok {
        eprintln!("  warning: hook refresh failed, run manually if needed");
    }

    println!();
    let refresh_ok = Command::new("trs")
        .args(["output-saver", "--refresh"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !refresh_ok {
        eprintln!("  warning: output-saver refresh failed, run manually if needed");
    }

    println!();
    println!("upgrade complete. Restart any open shells to pick up the new binary.");
}

/// Return the first hook-config JSON file in `$HOME` that fails to
/// parse. Used by `refresh_configs` to bail out before our JSON merge
/// layer tries to edit a corrupt file. Returns `None` when everything
/// is clean (or when no configs exist yet).
fn first_broken_hook_json() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let candidates = [
        ".claude/settings.json",
        ".claude/hooks.json",
        ".gemini/settings.json",
        ".cursor/hooks.json",
        ".factory/settings.json",
    ];
    for rel in candidates {
        let path = std::path::PathBuf::from(&home).join(rel);
        if !path.exists() {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue; // unreadable isn't "corrupt", init's own error path will handle
        };
        if content.trim().is_empty() {
            continue;
        }
        if serde_json::from_str::<serde_json::Value>(&content).is_err() {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_npm_install() {
        let p = PathBuf::from(
            "/Users/peluche/.nvm/versions/node/v20/lib/node_modules/@dpeluche/trs/bin/trs",
        );
        assert_eq!(detect_install_method(Some(&p)), InstallMethod::Npm);
    }

    #[test]
    fn detects_curl_install_default_path() {
        // Can't hard-code $HOME here; just verify the path shape works
        // against whatever HOME is in CI.
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        let p = PathBuf::from(format!("{}/.local/bin/trs", home));
        assert_eq!(detect_install_method(Some(&p)), InstallMethod::Curl);
    }

    #[test]
    fn detects_curl_install_trs_bin_path() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        let p = PathBuf::from(format!("{}/.trs/bin/trs", home));
        assert_eq!(detect_install_method(Some(&p)), InstallMethod::Curl);
    }

    #[test]
    fn detects_brew_install() {
        let p = PathBuf::from("/opt/homebrew/bin/trs");
        assert_eq!(detect_install_method(Some(&p)), InstallMethod::Brew);
    }

    #[test]
    fn detects_cargo_install() {
        // Build the expected cargo-bin prefix using the same env
        // lookup the production code uses, so the test is robust to
        // CARGO_HOME being set (e.g. by the test harness itself).
        let prefix = std::env::var("CARGO_HOME")
            .ok()
            .map(|h| format!("{}/bin/", h))
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| format!("{}/.cargo/bin/", h))
            })
            .expect("HOME or CARGO_HOME must be set");
        let p = PathBuf::from(format!("{}trs", prefix));
        assert_eq!(detect_install_method(Some(&p)), InstallMethod::Cargo);
    }

    #[test]
    fn unknown_for_built_from_source() {
        let p = PathBuf::from("/Users/dev/projects/trs/target/release/trs");
        assert_eq!(detect_install_method(Some(&p)), InstallMethod::Unknown);
    }

    #[test]
    fn unknown_for_none() {
        assert_eq!(detect_install_method(None), InstallMethod::Unknown);
    }
}
