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

/// Canonical install script URL (curl|sh path).
const INSTALL_SCRIPT_URL: &str =
    "https://raw.githubusercontent.com/dPeluChe/trs/main/scripts/install.sh";

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
    /// Installed via `cargo install tars-cli` — lives under
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

pub(crate) fn run_upgrade(check_only: bool, skip_confirm: bool) {
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
            if check_only {
                return;
            }
            if !skip_confirm && !prompt_yes("Proceed?") {
                println!("aborted.");
                return;
            }
            run_shell(&cmd);
        }
        InstallMethod::Npm => {
            let cmd = format!("npm install -g {}@latest", NPM_PACKAGE);
            println!("Will run:");
            println!("  {}", cmd);
            if check_only {
                return;
            }
            if !skip_confirm && !prompt_yes("Proceed?") {
                println!("aborted.");
                return;
            }
            run_shell(&cmd);
        }
        InstallMethod::Cargo => {
            println!("trs upgrade does not support cargo installs yet");
            println!("(requires publishing to crates.io — tracked on the roadmap).");
            println!();
            println!("Run manually:");
            println!("  cargo install tars-cli --force");
        }
        InstallMethod::Brew => {
            println!("trs upgrade does not support Homebrew yet");
            println!("(tap not published — tracked on the roadmap).");
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
            println!("  cargo install tars-cli --force");
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

fn run_shell(cmd: &str) {
    println!();
    let status = Command::new("sh").arg("-c").arg(cmd).status();
    match status {
        Ok(s) if s.success() => {
            println!();
            println!("upgrade complete. Restart any open shells to pick up the new binary.");
        }
        Ok(s) => {
            eprintln!(
                "\nupgrade command exited with code {}",
                s.code().unwrap_or(-1)
            );
        }
        Err(e) => {
            eprintln!("\nfailed to spawn upgrade command: {}", e);
        }
    }
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
