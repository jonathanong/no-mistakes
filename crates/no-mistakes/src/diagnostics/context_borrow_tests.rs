use super::*;

#[test]
fn borrowed_observer_preserves_reference_count_disabled_scopes_and_unwinding() {
    assert!(with_current(|observer| observer.is_none()));
    let observer = InvocationObserver::new(true);
    let _guard = InvocationGuard::install(Arc::clone(&observer));
    let references = Arc::strong_count(&observer);
    with_current(|current| {
        assert!(std::ptr::eq(current.unwrap(), observer.as_ref()));
        // An owned current() lookup would add a reference inside this callback.
        assert_eq!(Arc::strong_count(&observer), references);
        current.unwrap().increment("borrowed.zero", 0);
    });
    with_observer(None, || {
        assert!(with_current(|current| current.is_none()));
        crate::diagnostics::increment("disabled", 1);
    });
    assert!(std::panic::catch_unwind(|| {
        with_current(|_| panic!("release the thread-local borrow on unwind"));
    })
    .is_err());
    with_observer(None, || assert!(current().is_none()));
    crate::diagnostics::increment("borrowed.zero", 2);
    assert_eq!(observer.snapshot().work["borrowed.zero"], 2);
    assert!(!observer.snapshot().work.contains_key("disabled"));
    assert_eq!(Arc::strong_count(&observer), references);
}

#[test]
fn borrowed_counters_follow_nested_request_ownership_and_timing_only_scopes() {
    let outer = InvocationObserver::new(true);
    let inner = InvocationObserver::new(true);
    with_observer(Some(Arc::clone(&outer)), || {
        crate::diagnostics::increment("nested", 2);
        with_observer(Some(Arc::clone(&inner)), || {
            crate::diagnostics::increment("nested", 3);
            crate::diagnostics::record_ast_walk();
        });
        crate::diagnostics::increment("nested", 5);
    });
    assert_eq!(outer.snapshot().work["nested"], 7);
    assert_eq!(inner.snapshot().work["nested"], 3);
    assert_eq!(inner.snapshot().work["ast.walks"], 1);
    let timings = InvocationObserver::new(false);
    with_observer(Some(Arc::clone(&timings)), || {
        crate::diagnostics::increment("postgres.annotation.reachability_walks", 1);
    });
    assert!(timings.snapshot().work.is_empty());
}

#[test]
fn borrowed_counters_are_isolated_across_parallel_requests() {
    let observers = [InvocationObserver::new(true), InvocationObserver::new(true)];
    std::thread::scope(|scope| {
        for (index, observer) in observers.iter().enumerate() {
            scope.spawn(move || {
                with_observer(Some(Arc::clone(observer)), || {
                    for _ in 0..1000 {
                        crate::diagnostics::increment(
                            "postgres.annotation.reachability_walks",
                            (index + 1) as u64,
                        );
                    }
                });
            });
        }
    });
    for (index, observer) in observers.iter().enumerate() {
        assert_eq!(
            observer.snapshot().work["postgres.annotation.reachability_walks"],
            ((index + 1) * 1000) as u64,
        );
    }
}
