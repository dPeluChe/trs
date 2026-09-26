use super::*;

fn parse(input: &str) -> String {
    format_maven_compact(&parse_maven(input, "/work/shop/"))
}

#[test]
fn mvn_and_the_wrapper_route_here() {
    for cmd in ["mvn", "./mvnw", "mvnw.cmd"] {
        assert!(
            matches!(
                crate::classifier::classify_command(cmd, &["test".to_string()]),
                Some(crate::ParseCommands::Maven { .. })
            ),
            "{cmd}"
        );
    }
}

#[test]
fn surefire_2_failure_headers_are_read_too() {
    assert_eq!(
        failed_test("discount(com.acme.CartTest)  Time elapsed: 0.01 sec  <<< FAILURE!"),
        Some(("CartTest.discount".to_string(), "FAIL"))
    );
    assert_eq!(
        failed_test("com.acme.CartTest.add -- Time elapsed: 0.001 s <<< ERROR!"),
        Some(("CartTest.add".to_string(), "ERROR"))
    );
    assert_eq!(
        failed_test("Tests run: 4, Failures: 1, Errors: 0, Skipped: 0 <<< FAILURE! -- in X"),
        None
    );
}

#[test]
fn counts_sum_across_modules_and_skip_per_class_lines() {
    let out = parse(
        "[INFO] Tests run: 3, Failures: 0, Errors: 0, Skipped: 1, Time elapsed: 0.02 s -- in a.OneTest\n\
         [WARNING] Tests run: 9, Failures: 0, Errors: 0, Skipped: 3\n\
         [ERROR] Tests run: 9, Failures: 1, Errors: 2, Skipped: 0\n",
    );
    assert_eq!(out, "tests: 18 run, 1 failed, 2 errors, 3 skipped\n");
}

#[test]
fn paths_are_shown_relative_to_the_project() {
    let out =
        parse("[ERROR] /work/shop/src/main/java/A.java:[3,9] ';' expected\n[INFO] BUILD FAILURE\n");
    assert_eq!(
        out,
        "[ERROR] src/main/java/A.java:[3,9] ';' expected\nBUILD FAILURE\n"
    );
}

#[test]
fn unknown_lines_are_kept() {
    let out =
        parse("[INFO] +- com.google.guava:guava:jar:33.2.0-jre:compile\nhello from exec:java\n");
    assert_eq!(
        out,
        "+- com.google.guava:guava:jar:33.2.0-jre:compile\nhello from exec:java\n"
    );
}

#[test]
fn test_logs_keep_warnings_and_errors() {
    let out = parse(
        "[INFO]  T E S T S\n\
         2026-09-25T18:52:17.209-06:00  INFO 1 --- [main] c.a.App : Starting App\n\
         18:52:16.607 [main] DEBUG org.x.Y -- noise\n\
         2026-09-25T18:52:18.100-06:00  WARN 1 --- [main] c.a.Db : pool exhausted\n\
         2026-09-25T18:52:18.200-06:00 ERROR 1 --- [main] c.a.Api : request failed\n\
         [INFO] Results:\n",
    );
    assert!(
        out.contains("WARN 1 --- [main] c.a.Db : pool exhausted"),
        "{out}"
    );
    assert!(
        out.contains("ERROR 1 --- [main] c.a.Api : request failed"),
        "{out}"
    );
    assert!(
        !out.contains("Starting App") && !out.contains("noise"),
        "{out}"
    );
    assert!(out.contains("(hid 2 lines: test logs"), "{out}");
}

#[test]
fn logs_outside_tests_are_kept() {
    let line = "2026-09-25T18:52:19.102-06:00  INFO 1 --- [main] o.s.b.w.e.t.TomcatWebServer : Tomcat started on port 8080 (http)";
    assert_eq!(parse(line), format!("{line}\n"));
}

#[test]
fn jvm_notices_are_hidden_anywhere_but_app_warnings_are_not() {
    let out = parse(
        "[INFO] BUILD SUCCESS\n\
         WARNING: A terminally deprecated method in sun.misc.Unsafe has been called\n\
         OpenJDK 64-Bit Server VM warning: Sharing is only supported for boot loader classes\n\
         WARNING: Connection pool is almost full\n",
    );
    assert!(
        out.contains("WARNING: Connection pool is almost full"),
        "{out}"
    );
    assert!(!out.contains("Unsafe") && !out.contains("Sharing"), "{out}");
    assert!(out.contains("(hid 2 lines"), "{out}");
}
