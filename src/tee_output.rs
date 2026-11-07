//! The raw output of a run, saved where a cut summary can point to it.

/// Save full command output to ~/.trs/tee/ for failure recovery.
/// Returns the path to the saved file, or None if saving failed.
pub(crate) fn save_tee_output(cmd: &str, stdout: &str, stderr: &str) -> Option<String> {
    let tee_dir = crate::tracker::trs_dir()?.join("tee");

    // Create tee directory if needed
    std::fs::create_dir_all(&tee_dir).ok()?;

    // Clean old tee files (keep last N from config)
    let max_files = crate::config::config().limits.tee_max_files;
    if let Ok(entries) = std::fs::read_dir(&tee_dir) {
        let mut files: Vec<std::path::PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().map_or(false, |e| e == "log"))
            .collect();
        files.sort();
        if files.len() > max_files {
            for old in &files[..files.len() - max_files] {
                let _ = std::fs::remove_file(old);
            }
        }
    }

    // Generate filename: timestamp_command.log
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let safe_cmd = cmd
        .chars()
        .take(40)
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let filename = format!("{}_{}.log", timestamp, safe_cmd);
    let filepath = tee_dir.join(&filename);

    // Write stdout + stderr
    let mut content = String::new();
    if !stdout.is_empty() {
        content.push_str(stdout);
    }
    if !stderr.is_empty() {
        if !content.is_empty() {
            content.push_str("\n--- stderr ---\n");
        }
        content.push_str(stderr);
    }

    // Truncate if exceeds max size
    let max_bytes = crate::config::config().limits.tee_max_bytes;
    if max_bytes > 0 && content.len() > max_bytes {
        content.truncate(max_bytes);
        content.push_str(&format!("\n--- truncated at {} bytes ---", max_bytes));
    }

    // mode 0600: tee/*.log can carry response bodies that include
    // credentials or session tokens — user-only readable on Unix.
    write_user_only(&filepath, content.as_bytes()).ok()?;
    crate::pointer_recall::note_pointer();
    Some(filepath.to_string_lossy().to_string())
}

fn write_user_only(path: &std::path::Path, content: &[u8]) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    let mut opts = OpenOptions::new();
    opts.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts.open(path)?;
    f.write_all(content)
}
