//! `trs discover` — Find missed token savings opportunities.
//!
//! Scans Claude Code conversation history (~/.claude/) to identify
//! commands that ran without trs and could have saved tokens.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// Known commands that trs can compress.
const KNOWN_COMMANDS: &[&str] = &[
    "git status",
    "git diff",
    "git log",
    "git branch",
    "git push",
    "git pull",
    "ls ",
    "find ",
    "grep ",
    "rg ",
    "cargo test",
    "cargo build",
    "cargo clippy",
    "npm test",
    "npm install",
    "pnpm ",
    "pytest",
    "jest",
    "vitest",
    "docker ps",
    "docker logs",
    "env",
    "wc ",
    "curl ",
    "wget ",
    "eslint",
    "ruff ",
    "biome ",
    "gh pr",
    "gh issue",
    "gh run",
    "ollama ",
    "kubectl ",
];

pub(crate) fn run_discover(all_projects: bool, since_days: usize) {
    let home = match std::env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => {
            eprintln!("HOME not set");
            return;
        }
    };

    let claude_dir = home.join(".claude");
    if !claude_dir.exists() {
        eprintln!("No Claude Code history found at ~/.claude/");
        eprintln!("This command scans Claude Code conversation transcripts.");
        return;
    }

    let mut missed: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_commands = 0usize;
    let mut trs_commands = 0usize;

    // Scan conversation files
    let projects_dir = claude_dir.join("projects");
    let dirs_to_scan: Vec<PathBuf> = if all_projects {
        fs::read_dir(&projects_dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| p.is_dir())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        // Current project only — find by cwd
        let cwd = std::env::current_dir().unwrap_or_default();
        let project_slug = cwd
            .to_string_lossy()
            .replace('/', "-")
            .trim_start_matches('-')
            .to_string();
        let project_dir = projects_dir.join(&project_slug);
        if project_dir.exists() {
            vec![project_dir]
        } else {
            // Try to find a matching dir
            fs::read_dir(&projects_dir)
                .ok()
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok().map(|e| e.path()))
                        .filter(|p| {
                            p.is_dir()
                                && p.file_name()
                                    .map(|n| n.to_string_lossy().contains(&project_slug))
                                    .unwrap_or(false)
                        })
                        .collect()
                })
                .unwrap_or_default()
        }
    };

    let cutoff =
        std::time::SystemTime::now() - std::time::Duration::from_secs(since_days as u64 * 86400);

    for dir in &dirs_to_scan {
        scan_directory(
            dir,
            &cutoff,
            &mut missed,
            &mut total_commands,
            &mut trs_commands,
        );
    }

    // Output results
    if missed.is_empty() && total_commands == 0 {
        println!("No commands found in history (last {} days).", since_days);
        return;
    }

    let missed_total: usize = missed.values().sum();
    let adoption_pct = if total_commands > 0 {
        trs_commands * 100 / total_commands
    } else {
        0
    };

    println!("trs discover — missed savings\n");
    println!(
        "Commands: {} total, {} via trs ({}% adoption)",
        total_commands, trs_commands, adoption_pct
    );
    println!("Missed: {} commands could have used trs\n", missed_total);

    if !missed.is_empty() {
        // Sort by count descending
        let mut sorted: Vec<(&String, &usize)> = missed.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));

        for (cmd, count) in sorted.iter().take(15) {
            println!("  {:>4}x  {}", count, cmd);
        }
        if sorted.len() > 15 {
            println!("  ...+{} more", sorted.len() - 15);
        }
    }
}

fn scan_directory(
    dir: &PathBuf,
    cutoff: &std::time::SystemTime,
    missed: &mut BTreeMap<String, usize>,
    total: &mut usize,
    trs_count: &mut usize,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Only scan .jsonl files (conversation transcripts)
        if path.extension().map_or(true, |e| e != "jsonl") {
            continue;
        }

        // Check file modification time
        if let Ok(meta) = path.metadata() {
            if let Ok(modified) = meta.modified() {
                if modified < *cutoff {
                    continue;
                }
            }
        }

        // Scan the file for bash commands
        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                // Look for bash tool invocations in JSONL
                if !line.contains("\"Bash\"") && !line.contains("\"command\"") {
                    continue;
                }

                // Extract command strings
                if let Some(cmd) = extract_command(line) {
                    *total += 1;

                    if cmd.starts_with("trs ") {
                        *trs_count += 1;
                        continue;
                    }

                    // Check if this is a command trs handles
                    for known in KNOWN_COMMANDS {
                        let k = known.trim();
                        if cmd.starts_with(known) || cmd == k {
                            let category =
                                k.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
                            *missed.entry(category).or_insert(0) += 1;
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// Extract a bash command string from a JSONL line.
fn extract_command(line: &str) -> Option<String> {
    // Look for "command":"..." pattern
    let marker = "\"command\":\"";
    let pos = line.find(marker)?;
    let start = pos + marker.len();
    let rest = &line[start..];

    // Find the closing quote (handle escaped quotes)
    let mut end = 0;
    let mut escaped = false;
    for ch in rest.chars() {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            break;
        }
        end += ch.len_utf8();
    }

    let cmd = &rest[..end];
    if cmd.is_empty() {
        return None;
    }

    Some(cmd.replace("\\n", "\n").replace("\\\"", "\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_command() {
        let line = r#"{"type":"tool_use","tool":"Bash","content":{"command":"git status"}}"#;
        assert_eq!(extract_command(line), Some("git status".into()));
    }

    #[test]
    fn test_extract_command_escaped() {
        let line = r#"{"command":"echo \"hello\""}"#;
        assert_eq!(extract_command(line), Some("echo \"hello\"".into()));
    }

    #[test]
    fn test_extract_command_none() {
        assert_eq!(extract_command("no command here"), None);
    }
}
