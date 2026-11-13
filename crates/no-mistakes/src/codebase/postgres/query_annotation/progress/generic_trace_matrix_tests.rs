use super::{fixture, trace};
use crate::diagnostics::{with_observer, InvocationObserver, TimingKind};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

static OPERATIONS: AtomicU64 = AtomicU64::new(0);

fn operation() -> u64 {
    OPERATIONS.fetch_add(1, Ordering::Relaxed);
    73
}

fn invoke(label: &'static str, offset: Option<u32>, observer: Option<Arc<InvocationObserver>>) {
    let before = OPERATIONS.load(Ordering::Relaxed);
    let value = with_observer(observer, || {
        trace(label, &fixture(), offset, operation as fn() -> u64)
    });
    assert_eq!(value, 73);
    assert_eq!(OPERATIONS.load(Ordering::Relaxed) - before, 1);
}

#[test]
fn one_operation_type_preserves_trace_results_labels_and_counts_in_every_observer_mode() {
    for label in [
        "postgres.annotation.root",
        "postgres.annotation.module",
        "postgres.annotation.matrix_activity",
    ] {
        for offset in [None, Some(42)] {
            invoke(label, offset, None);
            for (verbose, progress) in [(false, false), (true, false), (false, true), (true, true)]
            {
                let observer = InvocationObserver::with_progress(verbose, progress);
                invoke(label, offset, Some(observer.clone()));
                let snapshot = observer.snapshot();
                if verbose || progress {
                    assert_eq!(
                        snapshot.timings.len(),
                        1,
                        "{label}: verbose={verbose}, progress={progress}"
                    );
                    assert_eq!(snapshot.timings[0].label, label);
                    assert_eq!(snapshot.timings[0].kind, TimingKind::Parallel);
                } else {
                    assert!(snapshot.timings.is_empty());
                }
                if verbose {
                    assert_eq!(snapshot.work[label], 1);
                    assert_eq!(snapshot.work["postgres.annotation.completed_activities"], 1);
                    for (key, expected) in [
                        (
                            "postgres.annotation.roots_completed",
                            label == "postgres.annotation.root",
                        ),
                        (
                            "postgres.annotation.module_initializations",
                            label == "postgres.annotation.module",
                        ),
                        (
                            "postgres.annotation.module_initializations_completed",
                            label == "postgres.annotation.module",
                        ),
                    ] {
                        assert_eq!(
                            snapshot.work.get(key).copied(),
                            expected.then_some(1),
                            "{label}: {key}"
                        );
                    }
                } else {
                    assert!(snapshot.work.is_empty());
                }
                if progress {
                    assert!(observer.progress_snapshot().unwrap().active.is_empty());
                } else {
                    assert!(observer.progress_snapshot().is_none());
                }
            }
        }
    }
}
