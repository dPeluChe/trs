use super::*;

#[test]
fn gradle_and_the_wrapper_route_here() {
    for cmd in ["gradle", "./gradlew", "gradlew.bat"] {
        assert!(
            matches!(
                crate::classifier::classify_command(cmd, &["build".to_string()]),
                Some(crate::ParseCommands::Gradle { .. })
            ),
            "{cmd}"
        );
    }
}

#[test]
fn reads_task_and_test_lines() {
    assert_eq!(
        task_line("> Task :api:test FAILED"),
        Some((":api:test", "FAILED"))
    );
    assert_eq!(task_line("> Task :classes"), Some((":classes", "")));
    assert_eq!(
        failed_test("CartTest > discount() FAILED"),
        Some(("CartTest", "discount()"))
    );
    assert_eq!(failed_test("> Task :test FAILED"), None);
}

#[test]
fn the_report_is_looked_up_in_the_failed_tasks_project() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("api/build/test-results/test");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("TEST-com.acme.api.TwoTest.xml"),
        r#"<testcase name="twice()" classname="com.acme.api.TwoTest" time="0.004">
    <failure message="org.opentest4j.AssertionFailedError: expected: &lt;5&gt; but was: &lt;4&gt;" type="x">trace</failure>
  </testcase>
  <testcase name="other()" classname="com.acme.api.TwoTest"><failure message="not this one"/></testcase>"#,
    )
    .unwrap();
    assert_eq!(
        report_message(root.path(), ":api:test", "TwoTest", "twice()").as_deref(),
        Some("expected: <5> but was: <4>")
    );
    assert_eq!(
        report_message(root.path(), ":web:test", "TwoTest", "twice()"),
        None
    );
}

#[test]
fn a_passing_test_in_the_report_adds_nothing() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("build/test-results/test");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("TEST-a.CartTest.xml"),
        r#"<testcase name="add()" classname="a.CartTest" time="0.1"/>
  <testcase name="sum()" classname="a.CartTest"><failure message="belongs to sum"/></testcase>"#,
    )
    .unwrap();
    assert_eq!(
        report_message(root.path(), ":test", "CartTest", "add()"),
        None
    );
}
