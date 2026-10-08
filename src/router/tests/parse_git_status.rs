use super::*;

fn untracked(input: &str) -> Vec<String> {
    ParseHandler::parse_git_status(input)
        .unwrap()
        .untracked
        .into_iter()
        .map(|e| e.path)
        .collect()
}

/// With `advice.statusHints=false` the footer has no `(use "git add"...)`, and
/// it was counted as an untracked file.
#[test]
fn the_footer_is_never_an_untracked_file() {
    let footers = [
        "nothing added to commit but untracked files present",
        "nothing added to commit but untracked files present (use \"git add\" to track)",
        "no hay nada agregado al commit pero hay archivos sin seguimiento",
        "nichts zum Commit vorgemerkt, aber es gibt unversionierte Dateien",
    ];
    for footer in footers {
        let input = format!("On branch main\nUntracked files:\n\ta.txt\n\tdir/b.txt\n\n{footer}\n");
        assert_eq!(untracked(&input), ["a.txt", "dir/b.txt"], "{footer}");
    }
}

#[test]
fn untracked_entries_with_spaces_in_the_name_are_kept() {
    let input = "On branch main\nUntracked files:\n\tmy notes.txt\n\nnothing added to commit but untracked files present\n";
    assert_eq!(untracked(input), ["my notes.txt"]);
}
