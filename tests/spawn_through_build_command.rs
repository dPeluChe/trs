//! `Command::new` on a program name that is not a literal must not run what the
//! user typed: only `build_command` finds `.cmd` shims on Windows.

use std::path::Path;

/// Spawns whose program is not user input: the door itself, and trs re-running
/// its own binary or probing fixed names held in a variable.
const ALLOWED: &[(&str, &str)] = &[
    ("src/exec.rs", "cmd"),
    ("src/classifier.rs", "trs"),
    ("src/doctor_checks.rs", "cmd"),
    ("src/doctor_checks.rs", "&trs"),
];

fn sources(dir: &Path, found: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|e| e == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn user_programs_start_through_build_command() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    sources(&root.join("src"), &mut files);
    let mut bypasses = Vec::new();
    for file in files {
        let name = file.strip_prefix(root).unwrap().to_str().unwrap();
        let name = name.replace('\\', "/");
        for (number, line) in std::fs::read_to_string(&file).unwrap().lines().enumerate() {
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
