use super::*;
use crate::diagnostics::{with_observer, InvocationObserver};

fn fixture() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../fixtures/performance/annotation-shared-record-dag/shared-object-dag-12/root.mts",
    )
}

#[test]
fn annotation_locations_expose_active_file_offset_and_arm() {
    let path = fixture();
    assert!(path.is_file());
    let observer = InvocationObserver::with_progress(true, true);
    with_observer(Some(observer.clone()), || {
        trace("postgres.annotation.root", &path, None, || {
            trace("postgres.annotation.entrypoint", &path, Some(42), || {
                trace(
                    "postgres.annotation.prefix_compare_arm",
                    &path,
                    None,
                    || {
                        let snapshot = observer.progress_snapshot().unwrap();
                        let phase = &snapshot.active.last().unwrap().phase;
                        assert!(phase[0].ends_with("root.mts"));
                        assert!(phase[1].ends_with("root.mts:42"));
                        assert!(phase[2].ends_with("root.mts"));
                        assert!(phase[2].starts_with("postgres.annotation.prefix_compare_arm"));
                    },
                );
            });
        });
    });
    let snapshot = observer.snapshot();
    assert_eq!(snapshot.work["postgres.annotation.completed_activities"], 3);
    assert!(snapshot.timings.is_empty());
    assert!(observer.progress_snapshot().unwrap().active.is_empty());
}

#[test]
fn verbose_only_keeps_counters_without_timing_history_and_disabled_records_nothing() {
    let path = fixture();
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        assert_eq!(trace("annotation.root", &path, None, || 7), 7);
    });
    assert!(observer.snapshot().timings.is_empty());
    assert_eq!(observer.snapshot().work["annotation.root"], 1);
    with_observer(None, || {
        assert_eq!(trace("disabled", &path, None, || 9), 9);
    });
}
