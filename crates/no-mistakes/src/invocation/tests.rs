use super::deadline::{active_deadline, Deadline};
use super::*;
use serde_json::Value;
use std::process::Command;
use std::time::Instant;

fn deadline_test_lock() -> &'static std::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
}

#[test]
fn napi_options_default_and_strip_controls() {
    let (json, options) = extract_napi_options(
        r#"{"timeout":4,"lockTimeout":5,"failOnLock":true,"root":"."}"#.to_string(),
    )
    .unwrap();
    assert_eq!(options.timeout, Some(Duration::from_secs(4)));
    assert_eq!(options.lock_timeout, Some(Duration::from_secs(5)));
    assert!(options.fail_on_lock);
    assert_eq!(
        serde_json::from_str::<Value>(&json).unwrap(),
        serde_json::json!({"root":"."})
    );
}

#[test]
fn napi_zero_and_null_disable_timeouts() {
    let (_, options) =
        extract_napi_options(r#"{"timeout":0,"lockTimeout":null}"#.to_string()).unwrap();
    assert_eq!(options.timeout, None);
    assert_eq!(options.lock_timeout, None);
}

#[test]
fn napi_missing_controls_use_defaults() {
    let (_, options) = extract_napi_options("{}".to_string()).unwrap();
    assert_eq!(options, InvocationOptions::default());
}

#[test]
fn napi_controls_validate_types() {
    for json in [
        r#"{"timeout":-1}"#,
        r#"{"timeout":1.5}"#,
        r#"{"lockTimeout":"30"}"#,
        r#"{"failOnLock":1}"#,
        "[]",
        "not-json",
    ] {
        assert!(extract_napi_options(json.to_string()).is_err(), "{json}");
    }
}

#[test]
fn lock_contention_supports_fail_timeout_and_release() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("invocation.lock");
    let held = acquire_lock(&path, None, false).unwrap();
    let busy = acquire_lock(&path, None, true).unwrap_err();
    assert_eq!(
        busy.downcast_ref::<InvocationError>().unwrap().kind(),
        InvocationErrorKind::LockBusy
    );
    assert_eq!(timeout_exit_code(&busy), None);
    let timeout = acquire_lock(&path, Some(Duration::from_millis(1)), false).unwrap_err();
    assert_eq!(
        timeout.downcast_ref::<InvocationError>().unwrap().kind(),
        InvocationErrorKind::LockTimeout
    );
    assert_eq!(timeout_exit_code(&timeout), Some(124));
    drop(held);
    assert!(acquire_lock(&path, Some(Duration::from_secs(1)), false).is_ok());
}

#[test]
fn cli_defaults_and_zero_values_have_napi_parity() {
    assert_eq!(
        InvocationArgs::default().options(),
        InvocationOptions::default()
    );
    assert_eq!(
        InvocationArgs {
            timeout: 0,
            lock_timeout: 0,
            fail_on_lock: true,
        }
        .options(),
        InvocationOptions {
            timeout: None,
            lock_timeout: None,
            fail_on_lock: true,
        }
    );
}

#[test]
fn disabled_deadline_allows_timeout_check() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _guard = DeadlineGuard::install(None).unwrap();
    check_timeout().unwrap();
}

#[test]
fn expired_deadline_returns_timeout_exit_code() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let previous = {
        let mut active = active_deadline().write().unwrap();
        active.replace(Deadline {
            expires_at: Instant::now(),
            timeout: Duration::from_secs(3),
        })
    };
    let error = check_timeout().unwrap_err();
    {
        let mut active = active_deadline().write().unwrap();
        *active = previous;
    }
    assert_eq!(timeout_exit_code(&error), Some(124));
    assert!(error.to_string().contains("3 seconds"));
}

#[test]
fn invocation_guard_installs_deadline_after_lock_acquisition() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let directory = tempfile::tempdir().unwrap();
    let guard = InvocationGuard::acquire_at_path(
        InvocationOptions {
            timeout: Some(Duration::from_secs(5)),
            lock_timeout: Some(Duration::from_secs(1)),
            fail_on_lock: false,
        },
        &directory.path().join("invocation.lock"),
    )
    .unwrap();
    check_timeout().unwrap();
    drop(guard);
    assert!(active_deadline().read().unwrap().is_none());
}

#[test]
fn oversized_deadline_is_rejected() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let result = DeadlineGuard::install(Some(Duration::MAX));
    let Err(error) = result else {
        panic!("an oversized timeout should fail");
    };
    assert!(error.to_string().contains("too large"));
}

#[test]
fn lock_open_errors_include_the_path() {
    let directory = tempfile::tempdir().unwrap();
    let error = acquire_lock(directory.path(), None, false).unwrap_err();
    assert!(error
        .to_string()
        .contains(&directory.path().display().to_string()));
}

#[test]
fn lock_path_and_directory_errors_are_contextualized() {
    let path = lock_path().unwrap();
    assert_eq!(path.file_name().unwrap(), "invocation.lock");

    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("not-a-directory");
    std::fs::write(&file, "occupied").unwrap();
    let error = super::lock::create_lock_directory(&file).unwrap_err();
    assert!(error.to_string().contains(&file.display().to_string()));
}

#[test]
fn lock_wait_polls_before_acquiring() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("invocation.lock");
    let held = acquire_lock(&path, None, false).unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(10));
        drop(held);
    });
    let acquired = acquire_lock(&path, None, false).unwrap();
    drop(acquired);
    release.join().unwrap();
}

#[cfg(unix)]
#[test]
fn command_output_captures_output_with_and_without_deadline() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for timeout in [None, Some(Duration::from_secs(5))] {
        let _guard = DeadlineGuard::install(timeout).unwrap();
        let mut command = Command::new("sh");
        command.args(["-c", "printf stdout; printf stderr >&2"]);
        let output = command_output(&mut command).unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"stdout");
        assert_eq!(output.stderr, b"stderr");
    }
}

#[cfg(unix)]
#[test]
fn command_output_terminates_child_at_deadline() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _guard = DeadlineGuard::install(Some(Duration::from_millis(1))).unwrap();
    let mut command = Command::new("sleep");
    command.arg("10");
    let error = command_output(&mut command).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
}

#[cfg(unix)]
#[test]
fn child_wait_errors_cleanup_the_child() {
    use std::process::Stdio;

    // `wait_timeout` errors are OS-level and cannot be induced reliably, so exercise
    // the shared cleanup path with the same live child and reader handles.
    let mut command = Command::new("sh");
    command
        .args(["-c", "printf stdout; printf stderr >&2; sleep 10"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdout_reader = std::thread::spawn(move || super::child::read_pipe(stdout));
    let stderr_reader = std::thread::spawn(move || super::child::read_pipe(stderr));
    let error = std::io::Error::other("synthetic wait failure");
    let result = super::child::cleanup_wait_error(child, stdout_reader, stderr_reader, error);
    assert_eq!(result.unwrap_err().to_string(), "synthetic wait failure");
}

#[test]
fn expired_deadline_prevents_child_spawn() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let previous = {
        let mut active = active_deadline().write().unwrap();
        active.replace(Deadline {
            expires_at: Instant::now(),
            timeout: Duration::from_secs(1),
        })
    };
    let mut command = Command::new("definitely-not-a-real-no-mistakes-command");
    let error = command_output(&mut command).unwrap_err();
    *active_deadline().write().unwrap() = previous;
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
}
