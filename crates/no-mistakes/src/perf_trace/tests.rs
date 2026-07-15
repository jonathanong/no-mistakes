use super::*;
use std::time::Duration;

#[test]
fn trace_uses_the_installed_verbose_observer_and_returns_its_result() {
    assert_eq!(trace("disabled", || 1 + 1), 2);

    let observer = crate::diagnostics::InvocationObserver::new(true);
    let guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    assert!(enabled());
    assert_eq!(trace("enabled", || "value"), "value");
    record("direct-call", Duration::from_millis(1));
    assert_eq!(observer.snapshot().timings.len(), 2);

    drop(guard);
}
