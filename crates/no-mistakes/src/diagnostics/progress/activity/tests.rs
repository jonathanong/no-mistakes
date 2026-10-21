use super::*;

#[test]
fn live_locations_are_visible_without_retaining_completed_timings() {
    let observer = InvocationObserver::with_progress(true, true);
    for _ in 0..1024 {
        observer.activity("annotation.root", TimingKind::Parallel, || {
            let snapshot = observer.progress_snapshot().unwrap();
            assert_eq!(snapshot.active.len(), 1);
            assert_eq!(snapshot.active[0].phase, ["annotation.root"]);
            assert!(snapshot.active[0].parallel);
        });
    }
    let snapshot = observer.progress_snapshot().unwrap();
    assert!(snapshot.active.is_empty());
    assert_eq!(snapshot.started_spans, 1024);
    assert_eq!(snapshot.completed_spans, 1024);
    assert!(snapshot.completed.is_empty());
    assert!(observer.snapshot().timings.is_empty());
}

#[test]
fn nested_locations_leave_no_active_span_after_unwinding() {
    let observer = InvocationObserver::with_progress(false, true);
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        observer.activity("root", TimingKind::Parallel, || {
            observer.activity("binding", TimingKind::Serial, || {
                let snapshot = observer.progress_snapshot().unwrap();
                assert_eq!(snapshot.active[1].phase, ["root", "binding"]);
                panic!("fixture failure");
            });
        });
    }));
    assert!(panic.is_err());
    let snapshot = observer.progress_snapshot().unwrap();
    assert!(snapshot.active.is_empty());
    assert_eq!(snapshot.started_spans, snapshot.completed_spans);
}

#[test]
fn disabled_activity_preserves_return_value_without_recording() {
    let observer = InvocationObserver::new(false);
    assert_eq!(observer.activity("ignored", TimingKind::Serial, || 42), 42);
    assert!(observer.progress_snapshot().is_none());
    assert!(observer.snapshot().timings.is_empty());
    assert!(observer.snapshot().work.is_empty());
}
