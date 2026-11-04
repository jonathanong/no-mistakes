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
fn rule_timing_is_enabled_for_basic_observers() {
    assert_eq!(rule("analysis.rules.example", || 1), 1);
    assert_eq!(filesystem_rule("example", || 2), 2);
    let observer = crate::diagnostics::InvocationObserver::new(false);
    let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    assert_eq!(rule("analysis.rules.example", || 3), 3);
    assert_eq!(filesystem_rule("example", || 4), 4);
    let labels: Vec<_> = observer
        .snapshot()
        .timings
        .into_iter()
        .map(|entry| entry.label)
        .collect();
    assert_eq!(
        labels,
        [
            "analysis.rules.example",
            "analysis.filesystem_rules.example"
        ]
    );
}
