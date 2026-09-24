//! Commands the hook must leave alone because something other than the
//! agent, or nothing trs can read, consumes their output.

use super::maybe_rewrite;

/// Every pipeline here returned a wrong answer through trs before this was
/// fixed, measured against the raw command on the trs repo itself: `find | wc
/// -l` said 14 instead of 241, `git diff | grep -c '^+'` said 0 instead of
/// 804, `ps aux | grep` missed a running process, `gh --json | python3` threw.
#[test]
fn test_piped_output_is_left_raw() {
    for cmd in [
        "git log | grep fix",
        "find . -name '*.rs' | xargs wc",
        "find src -name '*.rs' | wc -l",
        "git diff HEAD~3 | grep -c '^+'",
        "git diff --stat | tail -1",
        "ps aux | grep zellij",
        "gh pr view 1 --json title | python3 -c 'import json,sys; json.load(sys.stdin)'",
        "cargo test 2>&1 | grep 'test result'",
        "git status | head -3",
        "git log | head -5 | tail -1",
        "cargo test 2>&1 | tail -30",
        "git status |& head",
        "cd /tmp && git log | head",
    ] {
        assert_eq!(maybe_rewrite(cmd), None, "must not rewrite: {}", cmd);
    }
}

#[test]
fn test_pipe_characters_that_are_not_pipes_still_rewrite() {
    assert_eq!(
        maybe_rewrite("grep -n \"a|b\" src/main.rs"),
        Some("trs grep -n \"a|b\" src/main.rs".into())
    );
    assert_eq!(
        maybe_rewrite("git log --grep='x|y'"),
        Some("trs git log --grep='x|y'".into())
    );
    assert_eq!(
        maybe_rewrite("cargo test || echo failed"),
        Some("trs cargo test || echo failed".into())
    );
}

#[test]
fn ssh_is_left_alone() {
    assert_eq!(maybe_rewrite("ssh prod 'docker ps'"), None);
    assert_eq!(maybe_rewrite("ssh -o BatchMode=yes host uptime"), None);
    // Only the remote shell is skipped, not every word that starts with ssh.
    assert!(maybe_rewrite("sshfs host:/ /mnt").is_some());
}
