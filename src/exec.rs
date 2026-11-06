//! Command-execution helper: build the platform-appropriate `Command` for an
//! external program. On Windows route through the shell (#53); POSIX spawns
//! directly. Every program trs does not own starts here;
//! `tests/spawn_through_build_command.rs` keeps it that way.

use std::ffi::OsStr;

/// Build the `Command` used to run an external program through trs.
///
/// On Windows, direct `Command::new(cmd)` only resolves `.exe` — it fails on
/// the `.cmd`/`.bat` PATHEXT shims that front most JS tooling (`npm`, `npx`,
/// `yarn`, `pnpm`, `tsc`, `eslint`, …), on `.ps1` scripts, and on shell
/// builtins. That's the deeper half of issue #53: even after the plugin stops
/// emitting a POSIX `VAR=value` prefix, `trs npm …` / `trs foo.ps1` would die
/// with "command not found". So on Windows we route through the shell the way
/// the user's own shell would: PowerShell for `.ps1`, `cmd /C` otherwise (which
/// honors PATHEXT and builtins). POSIX is unchanged — direct spawn.
pub(crate) fn build_command(
    cmd: impl AsRef<OsStr>,
    args: &[impl AsRef<OsStr>],
) -> std::process::Command {
    let cmd = cmd.as_ref();
    let mut command = platform_command(cmd, args);
    if crate::text_util::cmd_basename(&cmd.to_string_lossy()) == "git" {
        // Parsers read git's English; LANGUAGE leaves encoding and sort order alone.
        command.env("LANGUAGE", "C");
    }
    command
}

fn platform_command(cmd: &OsStr, args: &[impl AsRef<OsStr>]) -> std::process::Command {
    use std::process::Command;
    #[cfg(windows)]
    {
        if cmd.to_string_lossy().to_ascii_lowercase().ends_with(".ps1") {
            let mut c = Command::new("powershell");
            c.args(["-NoProfile", "-File"]).arg(cmd);
            c.args(args);
            return c;
        }
        let mut c = Command::new("cmd");
        c.arg("/C").arg(cmd).args(args);
        c
    }
    #[cfg(not(windows))]
    {
        let mut c = Command::new(cmd);
        c.args(args);
        c
    }
}

#[cfg(test)]
mod build_command_tests {
    use super::build_command;

    #[test]
    #[cfg(not(windows))]
    fn posix_spawns_directly() {
        let c = build_command("npm", &["install", "--save"]);
        assert_eq!(c.get_program(), "npm");
        let args: Vec<_> = c
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["install", "--save"]);
    }

    #[test]
    #[cfg(windows)]
    fn windows_routes_cmd_and_powershell() {
        // .cmd/.bat shims + builtins go through `cmd /C`.
        let c = build_command("npm", &["install"]);
        assert_eq!(c.get_program(), "cmd");
        let args: Vec<_> = c
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["/C", "npm", "install"]);

        // .ps1 scripts go through PowerShell -File.
        let c = build_command(r"C:\srv\start.ps1", &["--host", "127.0.0.1"]);
        assert_eq!(c.get_program(), "powershell");
        let args: Vec<_> = c
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            vec![
                "-NoProfile",
                "-File",
                r"C:\srv\start.ps1",
                "--host",
                "127.0.0.1"
            ]
        );
    }
}
