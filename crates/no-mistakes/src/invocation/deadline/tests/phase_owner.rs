use super::*;

pub(super) fn expired_command_body() {
    let _serial = crate::invocation::deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let limit = Deadline {
        expires_at: clock::now() + Duration::from_secs(30),
        timeout: Duration::from_secs(30),
        owner: Some(std::thread::current().id()),
        committed: false,
    };
    let previous_phase = current_phase();
    set_timeout_phase("command");
    let outcome = DeadlineGuard::install_for_invocation_with_deadline(
        Some(Duration::ZERO),
        Some(std::thread::current().id()),
        Some(limit),
    )
    .map(|guard| {
        // Other tests stream subprocess output concurrently; this deliberately
        // expired test deadline must only affect its owning test thread.
        let foreign_timeout = std::thread::spawn(remaining_timeout).join().unwrap();
        assert_eq!(foreign_timeout.unwrap(), None);
        let result = check_timeout();
        drop(guard);
        result
    });
    set_timeout_phase(previous_phase);
    let error = outcome
        .expect("live invocation limit must allow the command guard")
        .expect_err("zero-duration native command must already be expired");
    assert_eq!(
        error.to_string(),
        "command timed out after 0 seconds during command"
    );
    let expired = Deadline {
        expires_at: clock::now(),
        ..limit
    };
    let error = DeadlineGuard::install_for_invocation_with_deadline(
        Some(Duration::from_secs(1)),
        None,
        Some(expired),
    )
    .err()
    .expect("already-expired invocation must fail before publication");
    assert_eq!(
        error.to_string(),
        "invocation deadline elapsed during lock acquisition"
    );
}
