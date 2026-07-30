use super::{prefix_of, progress_line};

#[test]
fn parses_s3_progress_lines() {
    assert_eq!(
        progress_line("delete: s3://bucket/logs/a.log"),
        Some(("delete", "s3://bucket/logs/a.log".to_string()))
    );
    // `copy`/`move` carry two targets; the source is what we group by.
    assert_eq!(
        progress_line("copy: s3://src/a.txt to s3://dst/a.txt"),
        Some(("copy", "s3://src/a.txt".to_string()))
    );
    // Not a receipt line — must stay content.
    assert_eq!(
        progress_line("Completed 3 file(s) with ~2 file(s) remaining"),
        None
    );
    assert_eq!(progress_line("An error occurred (AccessDenied)"), None);
}

#[test]
fn groups_by_bucket_and_directory() {
    assert_eq!(prefix_of("s3://b/logs/2026/a.log"), "s3://b/logs/2026/");
    assert_eq!(prefix_of("s3://b/top.txt"), "s3://b/");
    assert_eq!(prefix_of("s3://bucket-only"), "s3://bucket-only");
    // Non-s3 targets pass through rather than being mangled.
    assert_eq!(prefix_of("./local/file.txt"), "./local/file.txt");
}

#[test]
fn verbs_cover_the_recursive_operations() {
    for verb in ["delete", "upload", "download", "copy", "move"] {
        let line = format!("{}: s3://b/k", verb);
        assert!(progress_line(&line).is_some(), "unhandled verb: {}", verb);
    }
}
