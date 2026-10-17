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

#[test]
fn traced_operations_can_change_observer_scopes() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        // trace runs arbitrary caller code, so it must retain an owned observer
        // rather than borrowing the thread-local stack through this closure.
        trace("outer", || {
            crate::diagnostics::with_observer(None, || {
                assert!(!enabled());
                record("disabled-inner", Duration::from_millis(1));
            });
            assert!(enabled());
        });
    });
    let snapshot = observer.snapshot();
    assert_eq!(snapshot.timings.len(), 1);
}
