use super::{
    acquire_lock, classify_try_lock, create_lock_directory, lock_file_path, TryLockOutcome,
};
use crate::invocation::{InvocationError, InvocationErrorKind};
use std::fs::TryLockError;
use std::path::Path;
use std::time::Duration;

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

#[test]
fn user_lock_files_live_under_the_project_dirs_cache() {
    let path = lock_file_path(false).unwrap();
    assert_eq!(
        path.file_name().unwrap().to_string_lossy(),
        "invocation.lock"
    );
}

#[test]
fn create_lock_directory_reports_when_the_path_is_a_file() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("not-a-directory");
    std::fs::write(&file, b"lock").unwrap();
    let error = create_lock_directory(&file).unwrap_err();
    assert!(error
        .to_string()
        .contains("creating no-mistakes invocation lock directory"));
}

#[test]
fn acquire_lock_reports_when_the_path_is_a_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let error = acquire_lock(tmp.path(), Some(Duration::from_millis(1)), false).unwrap_err();
    assert!(error.to_string().contains("opening invocation lock"));
}
