//! Gradle 8.14 output captured from real runs, single and multi-project.
//! `gradle_project/` holds the JUnit XML report the failing run wrote.

use assert_cmd::Command;

fn gradle_in(dir: &str, fixture: &str) -> String {
    let input = std::fs::read_to_string(format!("tests/fixture_data/{fixture}")).unwrap();
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["parse", "gradle"])
        .current_dir(dir)
        .write_stdin(input)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn gradle(fixture: &str) -> String {
    gradle_in(".", fixture)
}

#[test]
fn a_failed_test_gets_its_message_from_the_report() {
    let out = gradle_in(
        "tests/fixture_data/gradle_project",
        "gradle_real_test_fail.txt",
    );
    assert!(out.contains("CartBTest > discount() FAILED"), "{out}");
    assert!(
        out.contains("10% off 1000 ==> expected: <905> but was: <900>"),
        "message not read from the XML report:\n{out}"
    );
    // No report for CartDTest here: the line stays as Gradle printed it.
    assert!(
        out.contains("java.lang.IllegalArgumentException at CartDTest.java:6"),
        "{out}"
    );
    assert!(out.contains("16 tests completed, 2 failed"), "{out}");
    assert!(out.contains("BUILD FAILED"), "{out}");
    assert!(!out.contains("* Try:") && !out.contains("--scan"), "{out}");
}

#[test]
fn quiet_tasks_are_counted_and_failed_ones_kept() {
    let out = gradle("gradle_real_multi_test_fail.txt");
    assert!(out.starts_with("(49 tasks ok)\n"), "{out}");
    assert!(!out.contains("UP-TO-DATE"), "{out}");
    assert!(out.contains("> Task :api:test FAILED"), "{out}");
    assert!(out.contains("> Task :billing:test FAILED"), "{out}");
    assert!(out.contains("TwoTest > twice() FAILED"), "{out}");
    assert!(
        out.contains("Execution failed for task ':billing:test'."),
        "{out}"
    );
}

#[test]
fn a_compile_error_is_shown_once_with_its_source_line() {
    let out = gradle("gradle_real_compile_error.txt");
    assert_eq!(
        out.matches("Cart.java:8: error: cannot find symbol")
            .count(),
        1,
        "{out}"
    );
    assert!(out.contains("return totl() - total()"), "{out}");
    assert!(out.contains("symbol:   method totl()"), "{out}");
    assert!(
        out.contains("Execution failed for task ':compileJava'."),
        "{out}"
    );
}

#[test]
fn warnings_keep_the_location_not_the_echoed_source() {
    let out = gradle("gradle_real_multi_build_pass.txt");
    assert!(
        out.contains("api/src/main/java/com/acme/api/Use.java:2: warning: [deprecation] old() in Svc has been deprecated"),
        "{out}"
    );
    assert!(!out.contains("public class Use"), "{out}");
    assert!(out.contains("BUILD SUCCESSFUL"), "{out}");
}

#[test]
fn an_up_to_date_build_is_one_count_and_the_result() {
    let out = gradle("gradle_real_multi_uptodate.txt");
    assert!(
        out.starts_with("(") && out.contains("tasks ok)\nBUILD SUCCESSFUL"),
        "{out}"
    );
    assert_eq!(out.lines().count(), 2, "{out}");
}
