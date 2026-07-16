use super::*;

#[test]
fn committed_output_disables_later_deadline_checks() {
    let _serial = super::super::deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let guard = DeadlineGuard::install_with_owner(
        Some(Duration::from_secs(30)),
        Some(std::thread::current().id()),
    )
    .unwrap();

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

    *active_deadline().write().unwrap() = previous;
}
