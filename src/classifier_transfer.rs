/// Compact git push/pull/fetch output.
///
/// Strips the remote URL (user already knows the repo), keeps the ref range and branch.
/// Examples:
///   "To https://github.com/user/repo.git\n   ae7dfe3..d6fd77d  main -> main"
///   → "pushed ae7dfe3..d6fd77d main -> main"
///
///   "From https://github.com/user/repo.git\n * branch  main -> FETCH_HEAD\nAlready up to date."
///   → "pulled (up to date)"
pub(crate) fn compact_git_transfer(combined: &str, subcmd: &str) -> String {
    let lines: Vec<&str> = combined
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        return format!("{} (no output)\n", subcmd);
    }

    // Check for errors
    if let Some(err) = lines
        .iter()
        .find(|l| l.starts_with("fatal:") || l.starts_with("error:"))
    {
        return format!("{}\n", err);
    }

    let verb = match subcmd {
        "push" => "pushed",
        "pull" => "pulled",
        "fetch" => "fetched",
        _ => subcmd,
    };

    let mut parts: Vec<String> = Vec::new();

    for line in &lines {
        // Skip "To/From https://..." lines
        if line.starts_with("To ") || line.starts_with("From ") {
            continue;
        }
        // Skip "Everything up-to-date"
        if line.contains("up-to-date") || line.contains("up to date") {
            return format!("{} (up to date)\n", verb);
        }
        // Ref range lines: "ae7dfe3..d6fd77d  main -> main"
        if line.contains("..") && line.contains("->") {
            parts.push(line.to_string());
            continue;
        }
        // New branch/tag: "* [new branch]  feature -> origin/feature"
        if line.contains("[new branch]") || line.contains("[new tag]") {
            let clean = line.trim_start_matches(|c: char| c == '*' || c == ' ');
            parts.push(clean.to_string());
            continue;
        }
        // Fast-forward / merge info
        if line.starts_with("Updating ") || line.starts_with("Fast-forward") {
            parts.push(line.to_string());
            continue;
        }
        // File change summary: "2 files changed, 10 insertions(+), 5 deletions(-)"
        if line.contains("file") && line.contains("changed") {
            parts.push(line.to_string());
            continue;
        }
    }

    if parts.is_empty() {
        format!("{} ok\n", verb)
    } else {
        format!("{} {}\n", verb, parts.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::compact_git_transfer;

    #[test]
    fn test_push_normal() {
        let input = "To https://github.com/user/repo.git\n   ae7dfe3..d6fd77d  main -> main\n";
        let result = compact_git_transfer(input, "push");
        assert_eq!(result, "pushed ae7dfe3..d6fd77d  main -> main\n");
        assert!(!result.contains("https://"));
    }

    #[test]
    fn test_push_up_to_date() {
        let input = "Everything up-to-date\n";
        let result = compact_git_transfer(input, "push");
        assert_eq!(result, "pushed (up to date)\n");
    }

    #[test]
    fn test_pull_already_up_to_date() {
        let input = "From https://github.com/user/repo.git\n * branch  main -> FETCH_HEAD\nAlready up to date.\n";
        let result = compact_git_transfer(input, "pull");
        assert_eq!(result, "pulled (up to date)\n");
    }

    #[test]
    fn test_pull_fast_forward() {
        let input = "From https://github.com/user/repo.git\n   ae7dfe3..d6fd77d  main -> origin/main\nUpdating ae7dfe3..d6fd77d\nFast-forward\n 2 files changed, 10 insertions(+), 5 deletions(-)\n";
        let result = compact_git_transfer(input, "pull");
        assert!(result.starts_with("pulled "));
        assert!(result.contains("Updating"));
        assert!(result.contains("2 files changed"));
        assert!(!result.contains("https://"));
    }

    #[test]
    fn test_fetch_new_branch() {
        let input = "From https://github.com/user/repo.git\n * [new branch]      feature -> origin/feature\n";
        let result = compact_git_transfer(input, "fetch");
        assert!(result.starts_with("fetched "));
        assert!(result.contains("[new branch]"));
        assert!(!result.contains("https://"));
    }

    #[test]
    fn test_push_fatal_error() {
        let input = "fatal: unable to access 'https://github.com/user/repo.git/': The requested URL returned error: 403\n";
        let result = compact_git_transfer(input, "push");
        assert!(result.starts_with("fatal:"));
    }

    #[test]
    fn test_push_empty_output() {
        let result = compact_git_transfer("", "push");
        assert_eq!(result, "push (no output)\n");
    }
}
