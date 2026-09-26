//! Maven output captured from real runs (Maven 3.9, surefire 3.2, JUnit 5,
//! Spring Boot 3.3): single module, multi-module reactor, Spring Boot tests.

use assert_cmd::Command;

fn maven(fixture: &str) -> String {
    let input = std::fs::read_to_string(format!("tests/fixture_data/{fixture}")).unwrap();
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["parse", "maven"])
        .write_stdin(input.clone())
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(out.len() * 3 < input.len(), "cut less than 2/3:\n{out}");
    out
}

#[test]
fn each_failed_test_keeps_its_message_and_the_project_frame() {
    let out = maven("mvn_real_test_fail.txt");
    assert!(out.contains("FAIL CartBTest.discount"), "{out}");
    assert!(
        out.contains("10% off 1000 ==> expected: <905> but was: <900>"),
        "{out}"
    );
    assert!(
        out.contains("at com.acme.shop.CartBTest.discount(CartBTest.java:7)"),
        "{out}"
    );
    assert!(out.contains("ERROR CartDTest.emptyIsZero"), "{out}");
    assert!(
        out.contains("IllegalArgumentException: negative price: -5"),
        "{out}"
    );
    assert!(
        out.contains("at com.acme.shop.Cart.add(Cart.java:6)"),
        "{out}"
    );
    assert!(
        !out.contains("org.junit.jupiter.api.AssertEquals"),
        "framework frames kept:\n{out}"
    );
    assert!(
        out.contains("(+8 frames)"),
        "cut frames not counted:\n{out}"
    );
    assert!(
        out.contains("tests: 16 run, 1 failed, 1 errors, 0 skipped"),
        "{out}"
    );
    assert!(out.contains("BUILD FAILURE"), "{out}");
    // The Results list repeats the detailed failures.
    assert_eq!(out.matches("expected: <905>").count(), 1, "{out}");
}

#[test]
fn colored_output_parses_like_plain() {
    let out = maven("mvn_real_test_fail_color.txt");
    assert!(!out.contains('\u{1b}'), "{out}");
    assert!(out.contains("FAIL CartBTest.discount"), "{out}");
    assert!(out.contains("BUILD FAILURE"), "{out}");
}

#[test]
fn reactor_names_the_failed_module_and_how_to_resume() {
    let out = maven("mvn_real_multi_test_fail.txt");
    assert!(out.contains("FAIL TwoTest.twice"), "{out}");
    assert!(
        out.contains("modules: api FAILURE, web SKIPPED (4 total)"),
        "{out}"
    );
    assert!(out.contains("resume with: mvn <args> -rf :api"), "{out}");
    // core ran 9 and api ran 9 before the build stopped.
    assert!(
        out.contains("tests: 18 run, 1 failed, 0 errors, 6 skipped"),
        "{out}"
    );
}

#[test]
fn a_compile_error_is_shown_once_with_its_symbol() {
    let out = maven("mvn_real_multi_compile_error.txt");
    assert_eq!(
        out.matches("Svc.java:[6,46] cannot find symbol").count(),
        1,
        "{out}"
    );
    assert!(out.contains("symbol:   variable undefinedVar"), "{out}");
    assert!(out.contains("modules: web FAILURE (4 total)"), "{out}");

    let out = maven("mvn_real_testcompile_error.txt");
    assert!(
        out.contains("CartCTest.java:[5,94] cannot find symbol"),
        "{out}"
    );
    assert!(out.contains("symbol:   method totall()"), "{out}");
}

#[test]
fn spring_boot_test_logs_are_counted_not_shown() {
    let out = maven("mvn_real_boot_test_fail.txt");
    assert!(out.contains("FAIL ApiTests.getsOrder"), "{out}");
    assert!(
        out.contains("expected: <order-8> but was: <order-7>"),
        "{out}"
    );
    assert!(!out.contains("Tomcat started"), "{out}");
    assert!(!out.contains(":: Spring Boot ::"), "{out}");
    assert!(out.contains("(hid 34 lines: test logs"), "{out}");
}

#[test]
fn download_progress_becomes_a_count() {
    let out = maven("mvn_real_install_progress.txt");
    assert!(!out.contains("Progress ("), "{out}");
    assert!(out.starts_with("downloaded 11 files\n"), "{out}");
    assert!(
        out.contains("Building jar: /home/dev/shop/target/shop-1.0.0-SNAPSHOT.jar"),
        "{out}"
    );
    assert!(out.contains("BUILD SUCCESS"), "{out}");
}

#[test]
fn a_passing_build_is_the_counts_and_the_result() {
    let out = maven("mvn_real_test_pass.txt");
    assert_eq!(
        out,
        "tests: 16 run, 0 failed, 0 errors, 0 skipped\nBUILD SUCCESS (0.738 s)\n"
    );
}
