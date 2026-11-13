use super::*;
use std::fs::File;
use std::time::Duration;

#[test]
fn cli_path_resolution_error_releases_early_invocation_deadline() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let result = InvocationGuard::acquire_resolving_path_with_json(
        InvocationOptions {
            invocation_timeout: Some(Duration::from_secs(5)),
            ..InvocationOptions::default()
        },
        true,
        || Err(anyhow::anyhow!("fixture path resolver failed")),
        true,
    );
    let error = result.err().expect("resolver failure must be returned");
    assert!(error.to_string().contains("fixture path resolver failed"));
    assert!(active_deadline().read().unwrap().is_none());
}

#[test]
fn lock_classifier_preserves_successful_open_result() {
    let file = File::open(fixture_path("lock-holder/invocation.lock")).unwrap();
    let acquired = super::super::acquisition::classify_lock_result(Ok(file), None, false).unwrap();
    assert!(acquired.metadata().unwrap().is_file());
}

#[test]
fn unrepresentable_invocation_deadline_is_rejected_before_resolving_a_lock() {
    for forward_signals in [false, true] {
        for json in [false, true] {
            let called = std::cell::Cell::new(false);
            let result = InvocationGuard::acquire_resolving_path_with_json(
                InvocationOptions {
                    invocation_timeout: Some(Duration::MAX),
                    ..InvocationOptions::default()
                },
                forward_signals,
                || {
                    called.set(true);
                    Ok(fixture_path("lock-holder/invocation.lock"))
                },
                json,
            );
            let error = result.err().expect("an overflowing deadline must fail");
            assert_eq!(error.to_string(), "invocation timeout is too large");
            assert!(
                !called.get(),
                "the lock resolver must not run on invalid options"
            );
        }
    }
}

#[test]
fn lock_classifier_preserves_existing_command_errors_under_shared_deadlines() {
    for expired in [false, true] {
        for invocation_limited in [false, true] {
            let now = Instant::now();
            let deadline = Deadline {
                expires_at: if expired {
                    now - Duration::from_secs(1)
                } else {
                    now + Duration::from_secs(30)
                },
                timeout: Duration::from_secs(30),
                owner: None,
                committed: false,
            };
            let error = InvocationError::new(
                InvocationErrorKind::CommandTimeout,
                "existing child command deadline",
            )
            .with_phase("command.child");
            let error = super::super::acquisition::classify_lock_result(
                Err(error.into()),
                Some(deadline),
                invocation_limited,
            )
            .unwrap_err();
            let retained = error.downcast_ref::<InvocationError>().unwrap();
            assert_eq!(retained.kind(), InvocationErrorKind::CommandTimeout);
            assert_eq!(retained.phase(), Some("command.child"));
            assert_eq!(error.to_string(), "existing child command deadline");
        }
    }
}

#[test]
fn cli_and_napi_acquisition_release_locks_and_each_deadline_policy() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let fixture = crate::test_support::materialize_saved_fixture(&fixture_path("lock-holder"));
    let path = fixture.path().join("invocation.lock");
    for forward_signals in [false, true] {
        for json in [false, true] {
            for (timeout, invocation_timeout) in [
                (None, None),
                (Some(Duration::from_secs(30)), None),
                (None, Some(Duration::from_secs(60))),
                (Some(Duration::from_secs(30)), Some(Duration::from_secs(60))),
            ] {
                let guard = InvocationGuard::acquire_resolving_path_with_json(
                    InvocationOptions {
                        timeout,
                        invocation_timeout,
                        fail_on_lock: true,
                        ..InvocationOptions::default()
                    },
                    forward_signals,
                    || Ok(path.clone()),
                    json,
                )
                .unwrap();
                let active = *active_deadline().read().unwrap();
                assert_eq!(active.is_some(), timeout.or(invocation_timeout).is_some());
                if let Some(active) = active {
                    assert_eq!(active.timeout, timeout.or(invocation_timeout).unwrap());
                }
                drop(guard);
                assert!(active_deadline().read().unwrap().is_none());
                let released = acquire_lock(&path, None, true).unwrap();
                drop(released);
            }
        }
    }
}

#[test]
fn lock_open_io_errors_keep_their_cause_and_clear_early_cli_deadlines() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let fixture = crate::test_support::materialize_saved_fixture(&fixture_path("lock-holder"));
    for forward_signals in [false, true] {
        for json in [false, true] {
            let error = InvocationGuard::acquire_resolving_path_with_json(
                InvocationOptions {
                    invocation_timeout: Some(Duration::from_secs(30)),
                    fail_on_lock: true,
                    ..InvocationOptions::default()
                },
                forward_signals,
                || Ok(fixture.path().to_path_buf()),
                json,
            )
            .err()
            .expect("a directory cannot be opened as an invocation lock file");
            assert!(error.to_string().contains("opening invocation lock"));
            assert!(error
                .to_string()
                .contains(&fixture.path().display().to_string()));
            assert!(error.downcast_ref::<std::io::Error>().is_some());
            assert!(error.downcast_ref::<InvocationError>().is_none());
            assert_eq!(exit_code(&error), 2);
            assert!(active_deadline().read().unwrap().is_none());
        }
    }
}
