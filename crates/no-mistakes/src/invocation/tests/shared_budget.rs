use super::*;
use std::io::{Read, Write};
use std::process::Stdio;
use wait_timeout::ChildExt;

// This exact test is also an isolated process whose stdin remains owned by its parent.
#[test]
fn watchdog_before_lock_path_subprocess_helper() {
    if std::env::var_os("NO_MISTAKES_BEFORE_LOCK_BUDGET_CONTROL").is_none() {
        return;
    }
    let json = std::env::var("NO_MISTAKES_BEFORE_LOCK_BUDGET_CONTROL").unwrap() == "json";
    let result = InvocationGuard::acquire_resolving_path_with_json(
        InvocationOptions {
            timeout: None,
            invocation_timeout: Some(Duration::from_millis(150)),
            ..InvocationOptions::default()
        },
        true,
        || {
            std::io::stdout()
                .write_all(b"owned-path-resolution-ready\n")
                .unwrap();
            std::io::stdout().flush().unwrap();
            let mut byte = [0];
            std::io::stdin().read_exact(&mut byte).unwrap();
            panic!("parent must not release blocked path resolution as a passing timeout control");
        },
        json,
    );
    panic!(
        "watchdog did not stop blocked resolution: {}",
        result.is_ok()
    );
}

#[test]
fn invocation_watchdog_owns_blocked_acquisition_before_the_lock_path_is_resolved() {
    for mode in ["text", "json"] {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "invocation::tests::shared_budget::watchdog_before_lock_path_subprocess_helper",
                "--nocapture",
            ])
            .env("NO_MISTAKES_BEFORE_LOCK_BUDGET_CONTROL", mode)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let status = child
            .wait_timeout(Duration::from_secs(3))
            .unwrap()
            .unwrap_or_else(|| {
                let _ = child.kill();
                let _ = child.wait();
                panic!("forced backstop: native watchdog did not own acquisition");
            });
        drop(input);
        let output = child.wait_with_output().unwrap();
        assert_eq!(status.code(), Some(124));
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("owned-path-resolution-ready"));
        if mode == "json" {
            let report: serde_json::Value = serde_json::from_str(
                stdout
                    .lines()
                    .find(|line| line.starts_with('{'))
                    .expect("JSON acquisition timeout"),
            )
            .unwrap();
            assert_eq!(report["error"]["kind"], "deadline");
            assert_eq!(report["error"]["phase"], "invocation.lock");
            assert_eq!(report["error"]["exitCode"], 124);
        } else {
            assert!(!stdout.contains("\"error\""));
        }
        assert!(String::from_utf8(output.stderr)
            .unwrap()
            .contains("150 milliseconds during invocation.lock"));
    }
}

#[cfg(unix)]
#[test]
fn term_resistant_registered_child_subprocess_helper() {
    use std::os::unix::net::UnixStream;
    let Some(path) = std::env::var_os("NO_MISTAKES_SHARED_BUDGET_SOCKET") else {
        return;
    };
    unsafe {
        nix::libc::signal(nix::libc::SIGTERM, nix::libc::SIG_IGN);
    }
    let mut stream = UnixStream::connect(path).unwrap();
    writeln!(stream, "{}:{}", std::process::id(), unsafe {
        nix::libc::getppid()
    })
    .unwrap();
    let mut byte = [0];
    // Only the native watchdog's SIGKILL or the parent's independent failure
    // cleanup ends this read; no timing sleep manufactures readiness or exit.
    let _ = stream.read(&mut byte);
}

#[cfg(unix)]
#[test]
fn registered_child_budget_subprocess_helper() {
    let Some(root) = std::env::var_os("NO_MISTAKES_SHARED_BUDGET_ROOT") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let _execution = InvocationGuard::acquire_at_path(
        InvocationOptions {
            timeout: None,
            invocation_timeout: Some(Duration::from_secs(1)),
            ..InvocationOptions::default()
        },
        &root.join("invocation.lock"),
        true,
    )
    .unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "invocation::tests::shared_budget::term_resistant_registered_child_subprocess_helper",
            "--nocapture",
        ])
        .env("NO_MISTAKES_SHARED_BUDGET_SOCKET", root.join("child.sock"));
    let result = command_output(&mut command);
    if let Err(error) = &result {
        if error.kind() == std::io::ErrorKind::TimedOut {
            eprintln!("native owned child deadline elapsed: {error}");
            std::process::exit(124);
        }
    }
    panic!(
        "deadline did not end owned blocked work: {}",
        result.is_ok()
    );
}

#[cfg(unix)]
#[test]
fn shared_deadline_closes_a_real_term_resistant_registered_child_channel() {
    use std::os::unix::net::{UnixListener, UnixStream};
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("child.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "invocation::tests::shared_budget::registered_child_budget_subprocess_helper",
            "--nocapture",
        ])
        .env("NO_MISTAKES_SHARED_BUDGET_ROOT", directory.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let wrapper_pid = child.id();
    let reader = match std::thread::Builder::new().spawn(move || -> std::io::Result<Vec<u8>> {
        let (mut stream, _) = listener.accept()?;
        stream.set_read_timeout(Some(Duration::from_secs(3)))?;
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes)?;
        Ok(bytes)
    }) {
        Ok(reader) => reader,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("observer startup failed: {error}");
        }
    };
    let (status, forced) = match child.wait_timeout(Duration::from_secs(4)) {
        Ok(Some(status)) => (Some(status), false),
        _ => {
            let _ = child.kill();
            (child.wait().ok(), true)
        }
    };
    // Wake an unaccepted listener on any startup failure. If the real child
    // connected, the observer's bounded error drops its stream to release it.
    let _ = UnixStream::connect(&socket);
    let observed = reader.join();
    let output = child.wait_with_output();
    assert!(!forced, "forced outer cleanup is not qualification");
    assert_eq!(status.unwrap().code(), Some(124));
    let bytes = observed
        .unwrap()
        .expect("owned child must close the channel without observer timeout fallback");
    let identity = String::from_utf8(bytes).unwrap();
    let (pid, parent) = identity
        .trim()
        .split_once(':')
        .expect("real child identity");
    assert!(pid.parse::<u32>().unwrap() > 0);
    assert_eq!(parent.parse::<u32>().unwrap(), wrapper_pid);
    let stderr = String::from_utf8(output.unwrap().stderr).unwrap();
    assert!(
        stderr.contains("command timed out after 1 seconds")
            || stderr.contains("native owned child deadline elapsed"),
        "{stderr}"
    );
}

#[test]
fn shared_invocation_budget_expires_while_waiting_without_changing_active_napi_holder() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _active =
        DeadlineGuard::install_for_invocation(Some(Duration::from_secs(30)), None).unwrap();
    let before = active_deadline().read().unwrap().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("invocation.lock");
    let _holder = acquire_lock(&path, None, false).unwrap();
    let result = InvocationGuard::acquire_at_path(
        InvocationOptions {
            timeout: Some(Duration::from_secs(30)),
            invocation_timeout: Some(Duration::from_secs(1)),
            lock_timeout: Some(Duration::from_secs(2)),
            ..InvocationOptions::default()
        },
        &path,
        false,
    );
    let error = result
        .err()
        .expect("waiter must not acquire the held native lock");
    assert_eq!(timeout_exit_code(&error), Some(124));
    assert_eq!(
        error.downcast_ref::<InvocationError>().unwrap().kind(),
        InvocationErrorKind::CommandTimeout
    );
    let earlier_lock = InvocationGuard::acquire_at_path(
        InvocationOptions {
            timeout: Some(Duration::from_secs(30)),
            invocation_timeout: Some(Duration::from_secs(1)),
            lock_timeout: Some(Duration::from_millis(1)),
            ..InvocationOptions::default()
        },
        &path,
        false,
    )
    .err()
    .expect("explicit shorter lock deadline must still apply");
    assert_eq!(
        earlier_lock
            .downcast_ref::<InvocationError>()
            .unwrap()
            .kind(),
        InvocationErrorKind::LockTimeout
    );
    let after = active_deadline().read().unwrap().unwrap();
    assert_eq!(before.expires_at, after.expires_at);
    assert_eq!(before.timeout, after.timeout);
    assert_eq!(before.owner, after.owner);
    assert!(!after.committed);
    check_timeout().unwrap();
}

#[test]
fn expired_invocation_budget_cannot_get_a_fresh_command_deadline_after_acquiring_lock() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let directory = tempfile::tempdir().unwrap();
    let result = InvocationGuard::acquire_at_path(
        InvocationOptions {
            timeout: Some(Duration::from_secs(30)),
            invocation_timeout: Some(Duration::ZERO),
            ..InvocationOptions::default()
        },
        &directory.path().join("invocation.lock"),
        false,
    );
    let error = result
        .err()
        .expect("expired total budget must fail before work");
    assert_eq!(timeout_exit_code(&error), Some(124));
    assert!(
        error.to_string().contains("invocation deadline elapsed"),
        "{error}"
    );
    assert!(active_deadline().read().unwrap().is_none());
}

#[test]
fn invocation_milliseconds_are_validated_and_removed_before_command_options() {
    let (value, options) =
        extract_napi_options_value(r#"{"root":"owned","timeout":2,"invocationTimeoutMs":3500}"#)
            .unwrap();
    assert_eq!(value, serde_json::json!({"root":"owned"}));
    assert_eq!(options.timeout, Some(Duration::from_secs(2)));
    assert_eq!(
        options.invocation_timeout,
        Some(Duration::from_millis(3500))
    );
    for value in [
        "0",
        "-1",
        "1.5",
        "true",
        "\"1\"",
        "9007199254740992",
        "null",
    ] {
        let error =
            extract_napi_options_value(format!("{{\"invocationTimeoutMs\":{value}}}")).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("invocationTimeoutMs must be a positive safe integer"),
            "{error}"
        );
    }
    let (_, options) =
        extract_napi_options_value(r#"{"invocationTimeoutMs":3500,"profile":"ci"}"#).unwrap();
    assert_eq!(options.invocation_timeout, None);
}

#[test]
fn oversized_shared_deadline_fails_before_lock_path_side_effects() {
    let touched = std::cell::Cell::new(false);
    let result = InvocationGuard::acquire_resolving_path(
        InvocationOptions {
            invocation_timeout: Some(Duration::MAX),
            ..InvocationOptions::default()
        },
        false,
        || {
            touched.set(true);
            panic!("invalid duration must fail before path effects");
        },
    );
    let error = result
        .err()
        .expect("unrepresentable deadline must be rejected");
    assert!(error
        .to_string()
        .contains("invocation timeout is too large"));
    assert!(!touched.get());
}

// A native channel timeout starts after the guard captures its deadline, so
// path resolution necessarily consumes that budget without polling or sleeps.
fn resolve_after_budget(path: &std::path::Path, budget: Duration) -> Result<std::path::PathBuf> {
    let (sender, receiver) = std::sync::mpsc::channel::<()>();
    let outcome = receiver.recv_timeout(budget);
    drop(sender);
    assert_eq!(outcome, Err(std::sync::mpsc::RecvTimeoutError::Timeout));
    Ok(path.to_path_buf())
}

#[test]
fn elapsed_resolution_budget_precedes_fail_fast_lock_contention() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("held.lock");
    let holder = acquire_lock(&path, None, false).unwrap();
    let budget = Duration::from_millis(10);
    let result = InvocationGuard::acquire_resolving_path(
        InvocationOptions {
            invocation_timeout: Some(budget),
            fail_on_lock: true,
            ..InvocationOptions::default()
        },
        false,
        || resolve_after_budget(&path, budget),
    );
    drop(holder);
    let error = result.err().expect("expired acquisition must fail");
    assert_eq!(
        error.downcast_ref::<InvocationError>().unwrap().kind(),
        InvocationErrorKind::CommandTimeout
    );
    assert!(active_deadline().read().unwrap().is_none());
}

#[test]
fn elapsed_resolution_budget_does_not_create_a_lock_file() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("never-opened.lock");
    let budget = Duration::from_millis(10);
    let result = InvocationGuard::acquire_resolving_path(
        InvocationOptions {
            invocation_timeout: Some(budget),
            ..InvocationOptions::default()
        },
        false,
        || resolve_after_budget(&path, budget),
    );
    let error = result.err().expect("expired acquisition must fail");
    assert_eq!(
        error.downcast_ref::<InvocationError>().unwrap().kind(),
        InvocationErrorKind::CommandTimeout
    );
    assert!(!path.exists(), "expired work must not open the lock path");
}

#[test]
fn unexpired_fail_fast_contention_keeps_its_original_error_kind() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("held.lock");
    let holder = acquire_lock(&path, None, false).unwrap();
    let result = InvocationGuard::acquire_at_path(
        InvocationOptions {
            invocation_timeout: Some(Duration::from_secs(30)),
            lock_timeout: Some(Duration::ZERO),
            fail_on_lock: true,
            ..InvocationOptions::default()
        },
        &path,
        false,
    );
    drop(holder);
    let error = result
        .err()
        .expect("held lock must reject fail-fast acquisition");
    assert_eq!(
        error.downcast_ref::<InvocationError>().unwrap().kind(),
        InvocationErrorKind::LockBusy
    );
}

#[test]
fn delayed_lock_timeout_keeps_the_deadline_that_actually_limited_acquisition() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("held.lock");
    let _holder = acquire_lock(&path, None, false).unwrap();
    for invocation_limited in [false, true] {
        let budget = Duration::from_millis(10);
        let deadline = Deadline {
            expires_at: clock::now() + budget,
            timeout: budget,
            owner: None,
            committed: false,
        };
        let native_error = acquire_lock(&path, Some(Duration::from_millis(1)), false);
        assert_eq!(
            native_error
                .as_ref()
                .unwrap_err()
                .downcast_ref::<InvocationError>()
                .unwrap()
                .kind(),
            InvocationErrorKind::LockTimeout
        );
        // Delay classification past both deadlines using an owned channel, as a
        // descheduled waiter would; selection, not wake-up timing, owns the error.
        resolve_after_budget(&path, budget).unwrap();
        let error = super::super::acquisition::classify_lock_result(
            native_error,
            Some(deadline),
            invocation_limited,
        )
        .unwrap_err();
        assert_eq!(
            error.downcast_ref::<InvocationError>().unwrap().kind(),
            if invocation_limited {
                InvocationErrorKind::CommandTimeout
            } else {
                InvocationErrorKind::LockTimeout
            }
        );
    }
}
