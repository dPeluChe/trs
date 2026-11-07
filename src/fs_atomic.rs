//! Writes for files a user also edits (`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`):
//! never a half-written file, and never over text another writer just added.

use std::fs;
use std::io;
use std::path::Path;

/// Replace `path` with `content` through a temp file in the same directory.
/// A symlinked file is written through, so the link survives.
pub(crate) fn write_atomic(path: &Path, content: &str) -> io::Result<()> {
    let target = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let name = target.file_name().unwrap_or_default().to_string_lossy();
    let tmp = target.with_file_name(format!(".{name}.trs-tmp-{}", std::process::id()));
    fs::write(&tmp, content)?;
    if let Ok(meta) = fs::metadata(&target) {
        let _ = fs::set_permissions(&tmp, meta.permissions());
    }
    fs::rename(&tmp, &target).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })
}

/// `write_atomic`, but only if the file still holds `base`, the text the new
/// content was built from. Two `init` runs racing on one file would otherwise
/// each overwrite the other's read-modify-write.
/// `content` takes the line endings of `base`, so a CRLF file stays CRLF.
pub(crate) fn write_if_unchanged(path: &Path, base: &str, content: &str) -> io::Result<()> {
    if fs::read_to_string(path).is_ok_and(|now| now != base) {
        return Err(io::Error::other(format!(
            "{} changed while trs was updating it; nothing written, run the command again",
            path.display()
        )));
    }
    write_atomic(
        path,
        &crate::text_util::with_endings_of(base, content.to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_content_and_leaves_no_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("AGENTS.md");
        fs::write(&p, "old").unwrap();
        write_atomic(&p, "new").unwrap();
        assert_eq!(fs::read_to_string(&p).unwrap(), "new");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn refuses_when_the_file_changed_since_it_was_read() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("CLAUDE.md");
        fs::write(&p, "mine, edited meanwhile").unwrap();
        let err = write_if_unchanged(&p, "what trs read", "block").unwrap_err();
        assert!(err.to_string().contains("changed while trs"));
        assert_eq!(fs::read_to_string(&p).unwrap(), "mine, edited meanwhile");
    }

    #[test]
    fn an_edit_keeps_the_files_line_endings() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("CLAUDE.md");
        fs::write(&p, "mine\r\n").unwrap();
        write_if_unchanged(&p, "mine\r\n", "mine\n@trs.md\n").unwrap();
        assert_eq!(fs::read_to_string(&p).unwrap(), "mine\r\n@trs.md\r\n");
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_is_written_through_and_stays_a_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("AGENTS.md");
        let link = dir.path().join("CLAUDE.md");
        fs::write(&real, "old").unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();
        write_atomic(&link, "new").unwrap();
        assert!(fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(fs::read_to_string(&real).unwrap(), "new");
    }
}
