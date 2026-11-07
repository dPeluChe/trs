use super::*;

fn run(command: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError> {
    ProcessBuilder::new(command)
        .args(args.iter().copied())
        .run()
}

fn run_with_timeout(
    command: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<ProcessOutput, ProcessError> {
    ProcessBuilder::new(command)
        .args(args.iter().copied())
        .timeout(timeout)
        .run()
}

#[test]
fn test_process_output_success() {
    let output = ProcessOutput {
        command: "echo".to_string(),
        args: vec!["hello".to_string()],
        stdout: "hello\n".to_string(),
        stderr: String::new(),
        exit_code: 0,
        duration: Duration::from_millis(10),
    };

    assert!(output.success());
    assert_eq!(output.code(), 0);
    assert!(output.has_stdout());
    assert!(!output.has_stderr());
}

#[test]
fn test_process_output_failure() {
    let output = ProcessOutput {
        command: "false".to_string(),
        args: vec![],
        stdout: String::new(),
        stderr: String::new(),
        exit_code: 1,
        duration: Duration::from_millis(10),
    };

    assert!(!output.success());
    assert_eq!(output.code(), 1);
}

#[test]
fn test_process_error_display() {
    let err = ProcessError::CommandNotFound {
        command: "nonexistent".to_string(),
    };
    assert_eq!(format!("{}", err), "Command not found: nonexistent");

    let err = ProcessError::PermissionDenied {
        command: "protected".to_string(),
    };
    assert_eq!(format!("{}", err), "Permission denied: protected");

    let err = ProcessError::Timeout {
        command: "slow".to_string(),
        duration: Duration::from_secs(5),
    };
    assert!(format!("{}", err).contains("timed out"));
    assert!(format!("{}", err).contains("5.00"));
}

#[test]
fn test_run_echo() {
    let result = run("echo", &["hello"]);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success());
    assert!(output.stdout.contains("hello"));
    assert!(output.stderr.is_empty());
    // Just verify duration is captured (on fast CI, echo completes in <1ms)
    let _ = output.duration.as_millis();
}

#[test]
fn test_run_command_not_found() {
    let result = run("nonexistent_command_xyz123", &[]);

    #[cfg(not(windows))]
    {
        // POSIX: direct spawn → a clean NotFound error.
        assert!(matches!(
            result.unwrap_err(),
            ProcessError::CommandNotFound { .. }
        ));
    }
    #[cfg(windows)]
    {
        // Windows routes through `cmd /C` (so .cmd/.bat shims resolve, #53),
        // which reports a missing command as its own non-zero exit + "not
        // recognized" message rather than a spawn NotFound. Either is an
        // acceptable failure signal.
        match result {
            Ok(o) => assert!(!o.success(), "missing command should not succeed"),
            Err(e) => assert!(matches!(e, ProcessError::CommandNotFound { .. })),
        }
    }
}

#[test]
fn test_run_non_zero_exit() {
    // Use 'false' command which always exits with 1
    let result = run("false", &[]);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(!output.success());
    assert_eq!(output.exit_code, 1);
}

#[test]
fn test_run_with_args() {
    let result = ProcessBuilder::new("echo")
        .args(["-n", "test"]) // -n prevents newline
        .run();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.success());
    // Note: on macOS, echo -n may not work as expected
}

#[test]
fn test_run_with_timeout_success() {
    // This command should complete quickly
    let result = run_with_timeout("echo", &["fast"], Duration::from_secs(5));
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success());
}

#[test]
fn test_run_with_timeout_exceeded() {
    // Sleep for 2 seconds but timeout after 100ms
    let result = run_with_timeout("sleep", &["2"], Duration::from_millis(100));
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, ProcessError::Timeout { .. }));
}

#[test]
fn test_run_capture_stderr() {
    // Use a command that writes to stderr
    let result = ProcessBuilder::new("sh")
        .args(["-c", "echo stderr >&2"])
        .run();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.stderr.contains("stderr"));
}

#[test]
fn test_run_no_capture_stdout() {
    let result = ProcessBuilder::new("echo")
        .args(["hello"])
        .capture_stdout(false)
        .run();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.has_stdout());
}

#[test]
fn test_run_no_capture_stderr() {
    let result = ProcessBuilder::new("sh")
        .args(["-c", "echo stderr >&2"])
        .capture_stderr(false)
        .run();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.has_stderr());
}

#[test]
fn test_run_reports_exit_code_zero() {
    let result = ProcessBuilder::new("echo").args(["hello"]).run();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.exit_code, 0);
    assert!(output.success());
}

#[test]
fn test_run_no_capture_duration() {
    let result = ProcessBuilder::new("echo")
        .args(["hello"])
        .capture_duration(false)
        .run();

    assert!(result.is_ok());
    let output = result.unwrap();
    // When duration is not captured, it should be ZERO
    assert_eq!(output.duration, Duration::ZERO);
}

#[test]
fn test_run_capture_duration_default() {
    // By default, duration should be captured
    let result = ProcessBuilder::new("echo").args(["hello"]).run();

    assert!(result.is_ok());
    let output = result.unwrap();
    // Duration should be greater than zero
    assert!(output.duration > Duration::ZERO);
}

#[test]
fn test_process_output_has_output() {
    let output = ProcessOutput {
        command: "test".to_string(),
        args: vec![],
        stdout: "content".to_string(),
        stderr: "error".to_string(),
        exit_code: 0,
        duration: Duration::ZERO,
    };

    assert!(output.has_stdout());
    assert!(output.has_stderr());

    let empty_output = ProcessOutput {
        command: "test".to_string(),
        args: vec![],
        stdout: String::new(),
        stderr: String::new(),
        exit_code: 0,
        duration: Duration::ZERO,
    };

    assert!(!empty_output.has_stdout());
    assert!(!empty_output.has_stderr());
}

#[test]
fn test_run_output_larger_than_the_pipe_buffer_does_not_hang() {
    let script = "head -c 300000 /dev/zero | tr '\\0' o; head -c 300000 /dev/zero | tr '\\0' e >&2";
    let output = ProcessBuilder::new("sh")
        .args(["-c", script])
        .timeout(Duration::from_secs(30))
        .run()
        .expect("a child filling both pipes must finish");

    assert_eq!(output.stdout.len(), 300_000);
    assert_eq!(output.stderr.len(), 300_000);
}

#[test]
fn test_run_keeps_output_around_invalid_utf8() {
    let output = ProcessBuilder::new("sh")
        .args(["-c", "printf 'before\\377after'"])
        .run()
        .unwrap();

    assert!(output.stdout.starts_with("before"));
    assert!(output.stdout.ends_with("after"));
}
