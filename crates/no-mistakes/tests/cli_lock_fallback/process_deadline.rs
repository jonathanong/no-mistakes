use super::{isolated_command, ManagedChild};
use crate::saved_fixture;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

const NAPI_WORKER: &str =
    "cli_lock_fallback::concurrency::process_deadline::unavailable_napi_lock_worker";
const NAPI_WORKER_ENV: &str = "NO_MISTAKES_UNAVAILABLE_NAPI_LOCK";

#[test]
fn unavailable_napi_lock_keeps_process_deadline_ownership() {
    let fixture = saved_fixture::materialize("invocation", "read-only-runtime");
    let root = fixture.path();
    let mut worker = ManagedChild(
        isolated_command(Command::new(std::env::current_exe().unwrap()), root)
            .args(["--exact", NAPI_WORKER, "--quiet", "--nocapture"])
            .env(NAPI_WORKER_ENV, "1")
            .env("CARGO_BIN_EXE_no-mistakes", "1")
            .env("TMPDIR", root.join("repo/message.ts"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let status = worker
        .0
        .wait_timeout(Duration::from_secs(10))
        .unwrap()
        .unwrap();
    let mut stdout = String::new();
    worker
        .0
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut stdout)
        .unwrap();
    let mut stderr = String::new();
    worker
        .0
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    assert!(status.success(), "{stdout}\n{stderr}");
    assert!(stdout.contains("protected deadline ownership"), "{stdout}");
    assert!(stderr.contains("\"lockFile\":null"), "{stderr}");
}

#[test]
fn unavailable_napi_lock_worker() {
    if std::env::var_os(NAPI_WORKER_ENV).is_none() {
        return;
    }
    use no_mistakes::invocation::{
        InvocationError, InvocationErrorKind, InvocationGuard, InvocationOptions,
    };

    let first = InvocationGuard::acquire(InvocationOptions {
        timeout: Some(Duration::from_secs(5)),
        ..InvocationOptions::default()
    })
    .unwrap();
    for (options, kind) in [
        (
            InvocationOptions {
                fail_on_lock: true,
                ..Default::default()
            },
            InvocationErrorKind::LockBusy,
        ),
        (
            InvocationOptions {
                lock_timeout: Some(Duration::ZERO),
                ..Default::default()
            },
            InvocationErrorKind::LockTimeout,
        ),
        (
            InvocationOptions {
                lock_timeout: Some(Duration::from_millis(10)),
                ..Default::default()
            },
            InvocationErrorKind::LockTimeout,
        ),
        (
            InvocationOptions {
                invocation_timeout: Some(Duration::from_millis(10)),
                ..Default::default()
            },
            InvocationErrorKind::CommandTimeout,
        ),
    ] {
        std::thread::spawn(move || {
            let error = InvocationGuard::acquire(options)
                .err()
                .expect("the active request must retain ownership");
            assert_eq!(
                error.downcast_ref::<InvocationError>().unwrap().kind(),
                kind
            );
        })
        .join()
        .unwrap();
    }
    drop(first);
    let _next = InvocationGuard::acquire(InvocationOptions {
        lock_timeout: Some(Duration::ZERO),
        ..Default::default()
    })
    .unwrap();
    println!("protected deadline ownership");
}
