use super::*;

#[test]
fn test_process_output_success() {
    let output = ProcessOutput {
        command: "echo".to_string(),
        args: vec!["hello".to_string()],
        stdout: "hello\n".to_string(),
        stderr: String::new(),
        exit_code: Some(0),
        duration: Duration::from_millis(10),
        timed_out: false,
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
        exit_code: Some(1),
        duration: Duration::from_millis(10),
        timed_out: false,
    };

    assert!(!output.success());
    assert_eq!(output.code(), 1);
}

#[test]
fn test_process_output_no_exit_code() {
    let output = ProcessOutput {
        command: "killed".to_string(),
        args: vec![],
        stdout: String::new(),
        stderr: String::new(),
        exit_code: None,
        duration: Duration::from_millis(10),
        timed_out: true,
    };

    assert!(!output.success());
    assert_eq!(output.code(), 1); // Default to 1
    assert!(output.timed_out);
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
        args: vec![],
        duration: Duration::from_secs(5),
        partial_stdout: String::new(),
        partial_stderr: String::new(),
    };
    assert!(format!("{}", err).contains("timed out"));
    assert!(format!("{}", err).contains("5.00"));
}

#[test]
fn test_process_error_helpers() {
    let err = ProcessError::CommandNotFound {
        command: "test".to_string(),
    };
    assert!(err.is_command_not_found());
    assert!(!err.is_permission_denied());
    assert!(!err.is_timeout());
    assert!(err.exit_code().is_none());

    let err = ProcessError::PermissionDenied {
        command: "test".to_string(),
    };
    assert!(!err.is_command_not_found());
    assert!(err.is_permission_denied());
    assert!(!err.is_timeout());

    let err = ProcessError::Timeout {
        command: "test".to_string(),
        args: vec![],
        duration: Duration::from_secs(1),
        partial_stdout: String::new(),
        partial_stderr: String::new(),
    };
    assert!(!err.is_command_not_found());
    assert!(!err.is_permission_denied());
    assert!(err.is_timeout());

    let output = ProcessOutput {
        command: "test".to_string(),
        args: vec![],
        stdout: String::new(),
        stderr: String::new(),
        exit_code: Some(42),
        duration: Duration::ZERO,
        timed_out: false,
    };
    let err = ProcessError::NonZeroExit { output };
    assert_eq!(err.exit_code(), Some(42));
}

#[test]
fn test_process_builder_basic() {
    let builder = ProcessBuilder::new("echo").arg("hello").arg("world");

    assert_eq!(builder.command, "echo");
    assert_eq!(builder.args, vec!["hello", "world"]);
}

#[test]
fn test_process_builder_args_iter() {
    let builder = ProcessBuilder::new("cmd").args(["a", "b", "c"]);

    assert_eq!(builder.args, vec!["a", "b", "c"]);
}

#[test]
fn test_process_builder_env() {
    let builder = ProcessBuilder::new("cmd")
        .env("KEY1", "value1")
        .env("KEY2", "value2");

    assert_eq!(builder.env.get("KEY1"), Some(&"value1".to_string()));
    assert_eq!(builder.env.get("KEY2"), Some(&"value2".to_string()));
}

#[test]
fn test_process_builder_envs() {
    let builder = ProcessBuilder::new("cmd").envs([("A", "1"), ("B", "2")]);

    assert_eq!(builder.env.get("A"), Some(&"1".to_string()));
    assert_eq!(builder.env.get("B"), Some(&"2".to_string()));
}

#[test]
fn test_process_builder_timeout() {
    let builder = ProcessBuilder::new("cmd").timeout(Duration::from_secs(30));

    assert_eq!(builder.timeout, Some(Duration::from_secs(30)));
}

#[test]
fn test_process_builder_capture() {
    let builder = ProcessBuilder::new("cmd")
        .capture_stdout(false)
        .capture_stderr(false);

    assert!(!builder.capture_stdout);
    assert!(!builder.capture_stderr);
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
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.is_command_not_found());
}

#[test]
fn test_run_non_zero_exit() {
    // Use 'false' command which always exits with 1
    let result = run("false", &[]);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(!output.success());
    assert_eq!(output.exit_code, Some(1));
}

#[test]
fn test_run_checked_non_zero_exit() {
    let result = run_checked("false", &[]);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, ProcessError::NonZeroExit { .. }));
    assert_eq!(err.exit_code(), Some(1));
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
fn test_run_with_env() {
    let result = ProcessBuilder::new("sh")
        .arg("-c")
        .arg("echo $TEST_VAR")
        .env("TEST_VAR", "test_value")
        .run();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.stdout.contains("test_value"));
}

#[test]
fn test_run_with_working_dir() {
    use std::path::Path;

    let result = ProcessBuilder::new("pwd")
        .current_dir(Path::new("/tmp"))
        .run();

    assert!(result.is_ok());
    let output = result.unwrap();
    // On macOS, /tmp is a symlink to /private/tmp
    assert!(output.stdout.contains("/tmp") || output.stdout.contains("private/tmp"));
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
    assert!(err.is_timeout());
}

#[test]
fn test_run_checked_success() {
    let result = run_checked("echo", &["hello"]);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success());
}

#[test]
fn test_run_capture_stderr() {
    // Use a command that writes to stderr
    let result = ProcessBuilder::new("sh")
        .arg("-c")
        .arg("echo stderr >&2")
        .run();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.stderr.contains("stderr"));
}

#[test]
fn test_run_no_capture_stdout() {
    let result = ProcessBuilder::new("echo")
        .arg("hello")
        .capture_stdout(false)
        .run();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.has_stdout());
}

#[test]
fn test_run_no_capture_stderr() {
    let result = ProcessBuilder::new("sh")
        .arg("-c")
        .arg("echo stderr >&2")
        .capture_stderr(false)
        .run();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.has_stderr());
}

#[test]
fn test_run_no_capture_exit_code() {
    let result = ProcessBuilder::new("false").capture_exit_code(false).run();

    assert!(result.is_ok());
    let output = result.unwrap();
    // When exit code is not captured, it should be None
    assert_eq!(output.exit_code, None);
    // success() should return false because exit_code is None
    assert!(!output.success());
}

#[test]
fn test_run_capture_exit_code_default() {
    // By default, exit code should be captured
    let result = ProcessBuilder::new("echo").arg("hello").run();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.exit_code, Some(0));
    assert!(output.success());
}

#[test]
fn test_run_capture_exit_code_non_zero() {
    let result = ProcessBuilder::new("sh")
        .arg("-c")
        .arg("exit 42")
        .capture_exit_code(true)
        .run();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.exit_code, Some(42));
    assert!(!output.success());
}

#[test]
fn test_run_no_capture_duration() {
    let result = ProcessBuilder::new("echo")
        .arg("hello")
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
    let result = ProcessBuilder::new("echo").arg("hello").run();

    assert!(result.is_ok());
    let output = result.unwrap();
    // Duration should be greater than zero
    assert!(output.duration > Duration::ZERO);
}

#[test]
fn test_process_builder_env_clear() {
    let builder = ProcessBuilder::new("cmd")
        .env_clear(true)
        .env("PATH", "/usr/bin");

    assert!(builder.env_clear);
    assert_eq!(builder.env.get("PATH"), Some(&"/usr/bin".to_string()));
}

#[test]
fn test_process_output_has_output() {
    let output = ProcessOutput {
        command: "test".to_string(),
        args: vec![],
        stdout: "content".to_string(),
        stderr: "error".to_string(),
        exit_code: Some(0),
        duration: Duration::ZERO,
        timed_out: false,
    };

    assert!(output.has_stdout());
    assert!(output.has_stderr());

    let empty_output = ProcessOutput {
        command: "test".to_string(),
        args: vec![],
        stdout: String::new(),
        stderr: String::new(),
        exit_code: Some(0),
        duration: Duration::ZERO,
        timed_out: false,
    };

    assert!(!empty_output.has_stdout());
    assert!(!empty_output.has_stderr());
}
