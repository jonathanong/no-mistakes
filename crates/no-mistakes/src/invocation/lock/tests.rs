use super::{classify_try_lock, lock_file_path, TryLockOutcome};
use crate::invocation::{InvocationError, InvocationErrorKind};
use std::fs::TryLockError;
use std::path::Path;

#[test]
fn cargo_test_binaries_use_a_per_process_lock_file() {
    let path = lock_file_path(true).unwrap();
    assert_eq!(
        path.file_name().unwrap().to_string_lossy(),
        format!("{}.lock", std::process::id())
    );
    assert!(
        path.to_string_lossy().contains("no-mistakes-test-locks"),
        "{}",
        path.display()
    );
}

#[test]
fn lock_system_errors_surface_through_try_lock_classification() {
    let path = Path::new("synthetic-invocation.lock");
    let TryLockOutcome::Failed(error) = classify_try_lock(
        Err(TryLockError::Error(std::io::Error::other(
            "synthetic lock failure",
        ))),
        path,
        false,
    ) else {
        panic!("expected a classified lock failure");
    };
    assert!(error.to_string().contains(&path.display().to_string()));
    assert!(format!("{error:#}").contains("synthetic lock failure"));
}

#[test]
fn lock_busy_errors_surface_when_fail_on_lock_is_set() {
    let TryLockOutcome::Failed(error) =
        classify_try_lock(Err(TryLockError::WouldBlock), Path::new("busy.lock"), true)
    else {
        panic!("expected a busy lock failure");
    };
    assert_eq!(
        error.downcast_ref::<InvocationError>().unwrap().kind(),
        InvocationErrorKind::LockBusy
    );
}
