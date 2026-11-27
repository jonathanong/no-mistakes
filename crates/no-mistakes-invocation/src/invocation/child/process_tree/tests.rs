use super::*;

// Regression for a review finding on #587's coverage gate: `terminate()`'s
// `killpg` call can't be made to fail in a portable test (any process we
// spawn ourselves is one we're permitted to signal), so the mapping from a
// signal-delivery failure to an `io::Error` was previously untested. Split
// into `classify_killpg_result` so the mapping is exercised directly with a
// synthetic error, independent of the real syscall.
#[test]
fn classify_killpg_result_maps_a_genuine_signal_failure_to_an_io_error() {
    let error = classify_killpg_result(Err(nix::errno::Errno::EPERM)).unwrap_err();
    assert_eq!(error.raw_os_error(), Some(nix::errno::Errno::EPERM as i32));
}

#[test]
fn classify_killpg_result_treats_already_gone_as_success() {
    assert!(classify_killpg_result(Err(nix::errno::Errno::ESRCH)).is_ok());
}

#[test]
fn classify_killpg_result_treats_success_as_success() {
    assert!(classify_killpg_result(Ok(())).is_ok());
}
