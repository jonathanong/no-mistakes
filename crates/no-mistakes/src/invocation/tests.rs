#[cfg(unix)]
use super::child::process_tree::signals;
use super::deadline::{active_deadline, Deadline};
use super::*;
use serde_json::Value;
use std::process::Command;
use std::time::Instant;
use wait_timeout::ChildExt;

fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/invocation")
        .join(name)
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
    let _guard =
        DeadlineGuard::install_with_owner(None, Some(std::thread::current().id())).unwrap();
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
            owner: Some(std::thread::current().id()),
            committed: false,
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
fn io_deadlines_return_timeout_exit_code() {
    let error = anyhow::Error::new(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "child deadline elapsed",
    ));
    assert_eq!(timeout_exit_code(&error), Some(124));
}

#[test]
fn deadline_owned_by_another_thread_does_not_affect_this_thread() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let foreign_owner = std::thread::spawn(|| std::thread::current().id())
        .join()
        .unwrap();
    let previous = active_deadline().write().unwrap().replace(Deadline {
        expires_at: Instant::now(),
        timeout: Duration::from_secs(1),
        owner: Some(foreign_owner),
        committed: false,
    });
    check_timeout().unwrap();
    assert_eq!(super::deadline::remaining_timeout().unwrap(), None);
    *active_deadline().write().unwrap() = previous;
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
        true,
    )
    .unwrap();
    check_timeout().unwrap();
    drop(guard);
    assert!(active_deadline().read().unwrap().is_none());
}

#[test]
fn cli_and_napi_entrypoints_select_distinct_parent_signal_policies() {
    let cli = include_str!("../main.rs");
    let napi = include_str!("../napi_api/async_task.rs");

    assert!(cli.contains("ExecutionGuard::acquire_for_cli"));
    assert!(napi.contains("InvocationGuard::acquire(invocation_options)"));
    assert!(!napi.contains("InvocationGuard::acquire_for_cli"));
}

#[test]
fn oversized_deadline_is_rejected() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let result =
        DeadlineGuard::install_with_owner(Some(Duration::MAX), Some(std::thread::current().id()));
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
fn lock_system_errors_include_the_path() {
    let path = Path::new("synthetic-invocation.lock");
    let error = super::lock::lock_error(path, std::io::Error::other("synthetic lock failure"));
    assert!(error.to_string().contains(&path.display().to_string()));
    assert!(format!("{error:#}").contains("synthetic lock failure"));
}

#[test]
fn lock_path_and_directory_errors_are_contextualized() {
    let path = lock_path().unwrap();
    assert_eq!(path.file_name().unwrap(), "invocation.lock");

    let file = fixture_path("not-a-directory");
    let error = super::lock::create_lock_directory(&file).unwrap_err();
    assert!(error.to_string().contains(&file.display().to_string()));
}

#[test]
fn lock_released_after_deadline_still_times_out() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("invocation.lock");
    let held = acquire_lock(&path, None, false).unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(10));
        drop(held);
    });
    let error = acquire_lock(&path, Some(Duration::from_millis(1)), false).unwrap_err();
    release.join().unwrap();
    assert_eq!(
        error.downcast_ref::<InvocationError>().unwrap().kind(),
        InvocationErrorKind::LockTimeout
    );
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
        let _guard =
            DeadlineGuard::install_with_owner(timeout, Some(std::thread::current().id())).unwrap();
        let mut command = Command::new("sh");
        command.args([
            "-c",
            "if read line; then exit 1; else printf stdout; printf stderr >&2; fi",
        ]);
        let output = command_output(&mut command).unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"stdout");
        assert_eq!(output.stderr, b"stderr");
    }
}

#[cfg(windows)]
#[test]
fn command_output_resumes_child_after_job_assignment() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _guard = DeadlineGuard::install_with_owner(
        Some(Duration::from_secs(5)),
        Some(std::thread::current().id()),
    )
    .unwrap();
    let mut command = Command::new("cmd.exe");
    command.args(["/D", "/C", "<NUL set /P =stdout & <NUL set /P =stderr 1>&2"]);

    // The Windows path creates the process suspended, attaches its job, and
    // must resume it before waiting for output.
    let output = command_output(&mut command).unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"stdout");
    assert_eq!(output.stderr, b"stderr");
}

#[cfg(unix)]
#[test]
fn command_output_terminates_child_at_deadline() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _guard = DeadlineGuard::install_with_owner(
        Some(Duration::from_millis(1)),
        Some(std::thread::current().id()),
    )
    .unwrap();
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
    let process_tree = super::child::ProcessTree::attach(&child);
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdout_reader = super::child::spawn_reader(stdout);
    let stderr_reader = super::child::spawn_reader(stderr);
    let error = std::io::Error::other("synthetic wait failure");
    let result =
        super::child::cleanup_wait_error(child, &process_tree, stdout_reader, stderr_reader, error);
    assert_eq!(result.unwrap_err().to_string(), "synthetic wait failure");
}

#[test]
fn child_cleanup_errors_preserve_the_original_error_kind() {
    let error = super::child::cleanup_result(
        std::io::Error::new(std::io::ErrorKind::TimedOut, "deadline"),
        Some(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "kill denied",
        )),
    )
    .unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert!(error.to_string().contains("kill denied"));
}

#[test]
fn disconnected_child_output_reader_reports_broken_pipe() {
    let (sender, receiver) = std::sync::mpsc::channel::<std::io::Result<Vec<u8>>>();
    drop(sender);
    let error = super::child::receive_reader(&receiver).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
}

#[test]
fn disconnected_child_output_reader_with_deadline_is_not_a_timeout() {
    let _guard = install_test_deadline(Duration::from_secs(1)).unwrap();
    let (sender, receiver) = std::sync::mpsc::channel::<std::io::Result<Vec<u8>>>();
    drop(sender);

    let error = super::child::receive_reader(&receiver).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
    assert_eq!(timeout_exit_code(&error.into()), None);
}

#[test]
fn child_output_reader_returns_bytes_without_deadline() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _guard =
        DeadlineGuard::install_with_owner(None, Some(std::thread::current().id())).unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    sender.send(Ok(vec![1, 2, 3])).unwrap();
    assert_eq!(super::child::receive_reader(&receiver).unwrap(), [1, 2, 3]);
}

#[cfg(unix)]
#[test]
fn command_output_deadline_kills_descendants_holding_output_pipes() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _guard = DeadlineGuard::install_with_owner(
        Some(Duration::from_millis(50)),
        Some(std::thread::current().id()),
    )
    .unwrap();
    let started = Instant::now();
    let mut command = Command::new("sh");
    command.args(["-c", "sleep 10 &"]);
    let error = command_output(&mut command).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[cfg(unix)]
#[test]
fn child_process_group_receives_forwarded_termination_signal() {
    use std::io::Read;

    let mut command = Command::new("sh");
    command
        .args([
            "-c",
            "trap 'exit 42' TERM; sleep 10 & worker=$!; printf x; wait \"$worker\"",
        ])
        .stdout(std::process::Stdio::piped());
    super::child::configure_process_group(&mut command);
    let mut child = command.spawn().unwrap();
    let process_tree = super::child::ProcessTree::attach(&child);
    let mut ready = [0u8; 1];
    child.stdout.take().unwrap().read_exact(&mut ready).unwrap();
    assert_eq!(ready, *b"x");
    signals::forward_signal(child.id() as i32, signal_hook::consts::SIGTERM);
    let Some(status) = child.wait_timeout(Duration::from_secs(1)).unwrap() else {
        process_tree.terminate(&mut child).unwrap();
        panic!("forwarded signal did not terminate the child process group");
    };
    drop(process_tree);
    assert_eq!(status.code(), Some(42));
}

#[cfg(unix)]
#[test]
fn signal_listener_forwards_before_terminating_parent() {
    let signals = signal_hook::iterator::Signals::new([signal_hook::consts::SIGUSR1]).unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    let registry = std::sync::Arc::new(signals::SignalRegistry::new());
    let thread = signals::spawn_signal_listener(signals, registry, move |signal| {
        sender.send(signal).unwrap()
    });

    unsafe {
        nix::libc::raise(signal_hook::consts::SIGUSR1);
    }

    assert_eq!(
        receiver.recv_timeout(Duration::from_secs(1)).unwrap(),
        signal_hook::consts::SIGUSR1
    );
    thread.join().unwrap();
}

#[cfg(unix)]
#[test]
fn signal_registry_tracks_multiple_process_groups_independently() {
    let registry = std::sync::Arc::new(signals::SignalRegistry::new());
    let first = registry.register(11);
    let second = registry.register(22);
    assert_eq!(registry.snapshot(), [11, 22]);

    drop(first);
    assert_eq!(registry.snapshot(), [22]);
    drop(second);
    assert!(registry.snapshot().is_empty());
    signals::forward_signal_to_groups(&[], signal_hook::consts::SIGTERM);
}

#[cfg(unix)]
#[test]
fn signal_forwarder_subprocess_helper() {
    if std::env::var_os("NO_MISTAKES_SIGNAL_FORWARDER_SUBPROCESS").is_none() {
        return;
    }

    let mut command = Command::new("sh");
    command.args(["-c", "while :; do sleep 1; done"]);
    super::child::configure_process_group(&mut command);
    let mut child = command.spawn().unwrap();
    let _forwarder = super::child::ParentSignalForwardingGuard::install(true).unwrap();
    let _process_tree = super::child::ProcessTree::attach(&child);
    unsafe {
        nix::libc::raise(signal_hook::consts::SIGTERM);
    }
    let _ = child.wait();
    panic!("the default SIGTERM handler did not terminate the process");
}

#[cfg(unix)]
#[test]
fn signal_forwarder_preserves_parent_default_termination() {
    use std::os::unix::process::ExitStatusExt;

    let mut subprocess = Command::new(std::env::current_exe().unwrap());
    subprocess
        .args([
            "--exact",
            "invocation::tests::signal_forwarder_subprocess_helper",
        ])
        .env("NO_MISTAKES_SIGNAL_FORWARDER_SUBPROCESS", "1");
    let mut subprocess = subprocess.spawn().unwrap();
    let Some(status) = subprocess.wait_timeout(Duration::from_secs(2)).unwrap() else {
        let _ = subprocess.kill();
        let _ = subprocess.wait();
        panic!("signal-forwarding subprocess did not terminate");
    };

    assert_eq!(status.signal(), Some(signal_hook::consts::SIGTERM));
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
            owner: Some(std::thread::current().id()),
            committed: false,
        })
    };
    let mut command = Command::new("definitely-not-a-real-no-mistakes-command");
    let error = command_output(&mut command).unwrap_err();
    *active_deadline().write().unwrap() = previous;
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
}

#[test]
fn production_test_planning_subprocesses_use_deadline_aware_output() {
    let sources = [
        include_str!("../tests/changed_files.rs"),
        include_str!("../tests/diff_parser.rs"),
        include_str!("../tests/lockfile_changes.rs"),
    ];

    assert!(sources.iter().all(|source| !source.contains(".output()")));
}
