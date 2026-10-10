use super::*;

#[test]
fn committed_output_disables_later_deadline_checks() {
    let _serial = super::super::deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let guard = DeadlineGuard::install_for_invocation(
        Some(Duration::from_secs(30)),
        Some(std::thread::current().id()),
    )
    .unwrap();

    commit_timeout().unwrap();
    commit_timeout().unwrap();
    active_deadline()
        .write()
        .unwrap()
        .as_mut()
        .unwrap()
        .expires_at = Instant::now();

    check_timeout().unwrap();
    assert_eq!(remaining_timeout().unwrap(), None);
    drop(guard);
}

#[test]
fn expired_deadline_cannot_be_committed() {
    let _serial = super::super::deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let previous = active_deadline().write().unwrap().replace(Deadline {
        expires_at: Instant::now(),
        timeout: Duration::from_secs(1),
        owner: Some(std::thread::current().id()),
        committed: false,
    });

    assert!(commit_timeout().is_err());
    assert_eq!(
        super::timeout_diagnostic(1, "check.parse"),
        "command timed out after 1 seconds during check.parse"
    );

    *active_deadline().write().unwrap() = previous;
}

#[test]
fn disabled_cli_timeout_does_not_start_a_watchdog() {
    let _serial = super::super::deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let guard = DeadlineGuard::install_for_cli(None).unwrap();
    assert!(guard.cancel_watch.is_none());
}

#[test]
fn replaced_deadline_lets_the_watchdog_return() {
    let _serial = super::super::deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let expires_at = Instant::now()
        .checked_sub(Duration::from_secs(1))
        .expect("past instant");
    // The expired instant is not the active deadline, so the watcher must return
    // instead of exiting the process.
    super::watch_deadline(
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        expires_at,
        Duration::from_secs(1),
    );
}
