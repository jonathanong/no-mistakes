#![cfg_attr(not(target_os = "linux"), allow(unused_imports))]
use super::{active_deadline, DeadlineGuard};

// Linux Instant documentation explicitly demonstrates this representable millennium.
#[cfg(target_os = "linux")]
#[test]
fn public_timeout_keeps_a_representable_linux_deadline_and_duration() {
    let _serial = super::super::deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (_, options) = crate::invocation::extract_napi_options_value(r#"{"timeout":31556952000}"#)
        .expect("public seconds input accepts a non-negative u64, also a JS safe integer");
    let timeout = std::time::Duration::from_secs(31_556_952_000);
    assert_eq!(options.timeout, Some(timeout));
    let _guard = DeadlineGuard::install_for_invocation(options.timeout, None)
        .expect("Linux supports the documented millennium Instant addition");
    let original = active_deadline()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .copied()
        .expect("installed original deadline");
    let observed = super::publish::published_deadline().expect("published deadline");
    // Both components must survive exactly; checking only 'still future' hides truncation.
    assert_eq!(
        (observed.timeout, observed.expires_at),
        (original.timeout, original.expires_at),
    );
}
