use super::acquire_lock;
use crate::invocation::holder::LockHolder;
use crate::invocation::lock_status::{status_at, LockState};
use crate::invocation::{
    error_report, exit_code, timeout_exit_code, InvocationError, InvocationErrorKind,
    EXIT_TEMPFAIL, EXIT_TIMEOUT,
};
use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/invocation")
        .join(name)
        .join("invocation.lock");
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn holder_records_parse_from_json_and_legacy_pid_files() {
    let holder = LockHolder::parse(&fixture("lock-holder")).unwrap();
    assert_eq!(holder.pid, 4242);
    assert_eq!(holder.cwd.as_deref(), Some("/work/other-repo"));
    assert_eq!(holder.command, "no-mistakes check --format json");
    let legacy = LockHolder::parse(&fixture("lock-holder-legacy")).unwrap();
    assert_eq!(
        (legacy.pid, legacy.cwd.clone(), legacy.started_at_ms),
        (264, None, 0)
    );
    assert_eq!(legacy.describe(), "pid 264");
    assert!(LockHolder::parse("not a holder").is_none());
    assert!(LockHolder::parse("").is_none());
}

#[test]
fn describe_names_pid_cwd_command_and_age() {
    let holder = LockHolder::parse(&fixture("lock-holder"))
        .unwrap()
        .with_age();
    let text = holder.describe();
    assert!(text.starts_with("pid 4242 (cwd /work/other-repo, command `no-mistakes check"));
    assert!(text.contains("held for "), "{text}");
}

#[test]
fn acquisition_records_this_process_as_the_holder() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("invocation.lock");
    let _held = acquire_lock(&path, None, false).unwrap();
    let holder = LockHolder::read(&path).unwrap();
    assert_eq!(holder.pid, std::process::id());
    assert_eq!(
        holder.cwd.as_deref(),
        std::env::current_dir().unwrap().to_str()
    );
    assert!(holder.started_at_ms > 0);
    assert!(holder.age_seconds.is_some());
}

#[test]
fn busy_and_timeout_errors_name_the_holder_and_exit_75() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("invocation.lock");
    let _held = acquire_lock(&path, None, false).unwrap();
    let pid = format!("pid {}", std::process::id());
    let busy = acquire_lock(&path, None, true).unwrap_err();
    assert!(busy.to_string().contains(&pid), "{busy}");
    assert_eq!(exit_code(&busy), EXIT_TEMPFAIL);
    assert_eq!(timeout_exit_code(&busy), None);
    let timeout = acquire_lock(&path, Some(Duration::from_millis(1)), false).unwrap_err();
    assert!(timeout.to_string().contains(&pid), "{timeout}");
    assert_eq!(exit_code(&timeout), EXIT_TEMPFAIL);
    let report = error_report(&timeout).unwrap();
    assert_eq!(report["error"]["kind"], "lock_timeout");
    assert_eq!(report["error"]["phase"], "invocation.lock");
    assert_eq!(report["error"]["exitCode"], 75);
    assert_eq!(report["error"]["holder"]["pid"], std::process::id());
}

#[test]
fn deadline_errors_keep_exit_124_and_report_their_phase() {
    let deadline: anyhow::Error = InvocationError::new(
        InvocationErrorKind::CommandTimeout,
        "command timed out after 3 seconds during check.prepare",
    )
    .with_phase("check.prepare")
    .into();
    assert_eq!(exit_code(&deadline), EXIT_TIMEOUT);
    let report = error_report(&deadline).unwrap();
    assert_eq!(report["error"]["kind"], "deadline");
    assert_eq!(report["error"]["phase"], "check.prepare");
    assert!(report["error"].get("holder").is_none());
    let io: anyhow::Error = std::io::Error::from(std::io::ErrorKind::TimedOut).into();
    assert_eq!(error_report(&io).unwrap()["error"]["phase"], json!(null));
    let other = anyhow::anyhow!("unrelated");
    assert_eq!(exit_code(&other), 2);
    assert!(error_report(&other).is_none());
}

#[test]
fn status_reports_held_free_and_missing_lock_files() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("invocation.lock");
    let missing = status_at(&path).unwrap();
    assert_eq!((missing.state, missing.holder), (LockState::Free, None));
    let held = acquire_lock(&path, None, false).unwrap();
    let status = status_at(&path).unwrap();
    assert_eq!(status.state, LockState::Held);
    assert_eq!(status.holder.unwrap().pid, std::process::id());
    drop(held);
    // The record outlives the lock, but a free lock never reports a holder.
    let status = status_at(&path).unwrap();
    assert_eq!((status.state, status.holder), (LockState::Free, None));
}

#[test]
fn status_surfaces_unopenable_lock_paths() {
    let directory = tempfile::tempdir().unwrap();
    let blocked = directory.path().join("file.txt").join("invocation.lock");
    std::fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/invocation/lock-holder/invocation.lock"),
        directory.path().join("file.txt"),
    )
    .unwrap();
    assert!(status_at(&blocked).is_err());
}
