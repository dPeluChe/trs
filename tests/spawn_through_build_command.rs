//! `Command::new` on a program name that is not a literal must not run what the
//! user typed: only `build_command` finds `.cmd` shims on Windows.

mod support;

/// Spawns whose program is not user input: the door itself, and trs re-running
/// its own binary or probing fixed names held in a variable.
const ALLOWED: &[(&str, &str)] = &[
    ("src/exec.rs", "cmd"),
    ("src/classifier.rs", "trs"),
    ("src/doctor_checks.rs", "cmd"),
    ("src/doctor_checks.rs", "&trs"),
];

#[test]
fn user_programs_start_through_build_command() {
    let mut bypasses = Vec::new();
    for (name, text) in support::src_files() {
        for (number, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for (start, _) in line.match_indices("Command::new(") {
                let rest = &line[start + "Command::new(".len()..];
                let program = rest.split(')').next().unwrap();
                let allowed = ALLOWED.contains(&(name.as_str(), program));
                if !program.starts_with('"') && !allowed {
                    bypasses.push(format!("{name}:{}: {program}", number + 1));
                }
            }
        }
    }
    assert!(bypasses.is_empty(), "use build_command: {bypasses:#?}");
}
