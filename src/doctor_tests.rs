use super::*;

#[test]
fn test_check_version_passes() {
    let check = check_version();
    assert_eq!(check.status, CheckStatus::Pass);
    assert!(check.sub.iter().any(|s| s.contains("version:")));
    assert!(check.sub.iter().any(|s| s.contains("path:")));
}

#[test]
fn test_check_path_accessible() {
    let check = check_path_accessible();
    assert!(!check.detail.is_empty());
}

#[test]
fn test_check_dep_git() {
    let check = check_dep("git", "git", true, "install git");
    assert_eq!(check.status, CheckStatus::Pass);
    assert_eq!(check.name, "dep:git");
    assert!(check.sub.iter().any(|s| s.contains("git")));
}

#[test]
fn test_check_dep_rg_name() {
    let check = check_dep("rg", "ripgrep", false, "install ripgrep");
    assert_eq!(check.name, "dep:rg");
}

#[test]
fn test_check_dep_unknown_name() {
    let check = check_dep("nonexistent_xyz", "test", false, "install it");
    assert_eq!(check.name, "dep:other");
    assert_eq!(check.status, CheckStatus::Warn);
    assert_eq!(check.hint, "install it");
}

#[test]
fn test_check_dep_missing_required() {
    let check = check_dep("nonexistent_xyz", "test", true, "install it");
    assert_eq!(check.status, CheckStatus::Fail);
    assert_eq!(check.hint, "install it");
}

#[test]
fn test_check_config_dir() {
    let check = check_config_dir();
    assert_eq!(check.status, CheckStatus::Pass, "{}", check.detail);
}

#[test]
fn test_check_history_writable() {
    let check = check_history_writable();
    assert_eq!(check.status, CheckStatus::Pass);
}

#[test]
fn test_run_checks_returns_all() {
    let checks = run_checks();
    assert!(
        checks.len() >= 8,
        "expected at least 8 checks, got {}",
        checks.len()
    );

    let names: Vec<&str> = checks.iter().map(|c| c.name).collect();
    assert!(names.contains(&"trs binary"));
    assert!(names.contains(&"PATH"));
    assert!(names.contains(&"dep:git"));
    assert!(names.contains(&"dep:rg"));
    assert!(names.contains(&"config dir"));
    assert!(names.contains(&"history"));
    assert!(names.contains(&"stdin pipe"));
    assert!(names.contains(&"hooks"));
}

#[test]
fn test_check_status_display() {
    assert_eq!(format!("{}", CheckStatus::Pass), "PASS");
    assert_eq!(format!("{}", CheckStatus::Warn), "WARN");
    assert_eq!(format!("{}", CheckStatus::Fail), "FAIL");
}

#[test]
fn test_count_hooks_via_init() {
    let tools = AiTool::all_tools();
    let count: usize = tools.iter().filter(|t| check_tool(t)).count();
    assert!(count <= 6);
}

#[test]
fn test_summary_from_checks() {
    let checks = vec![
        Check::pass("a", "ok"),
        Check::warn("b", "meh"),
        Check::fail("c", "bad"),
        Check::pass("d", "ok"),
    ];
    let s = Summary::from_checks(&checks);
    assert_eq!(s.pass, 2);
    assert_eq!(s.warn, 1);
    assert_eq!(s.fail, 1);
    assert_eq!(s.total, 4);
}

#[test]
fn test_check_builders() {
    let c = Check::pass("test", "ok")
        .with_sub(vec!["line1".into()])
        .with_hint("do this");
    assert_eq!(c.status, CheckStatus::Pass);
    assert_eq!(c.sub.len(), 1);
    assert_eq!(c.hint, "do this");

    let c = Check::warn("test", "meh");
    assert_eq!(c.status, CheckStatus::Warn);
    assert!(c.hint.is_empty());

    let c = Check::fail("test", "bad").with_hint("fix it");
    assert_eq!(c.status, CheckStatus::Fail);
    assert_eq!(c.hint, "fix it");
}
