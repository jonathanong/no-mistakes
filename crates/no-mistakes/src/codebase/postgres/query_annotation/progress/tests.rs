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
    assert_eq!(snapshot.timings.len(), 3);
    assert!(snapshot
        .timings
        .iter()
        .all(|timing| !timing.label.contains("root.mts")));
    assert!(observer.progress_snapshot().unwrap().active.is_empty());
}

#[test]
fn verbose_only_aggregates_fixed_timing_labels_and_disabled_records_nothing() {
    let path = fixture();
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        assert_eq!(trace("annotation.root", &path, None, || 7), 7);
        assert_eq!(trace("annotation.root", &path, Some(42), || 8), 8);
    });
    let snapshot = observer.snapshot();
    assert_eq!(snapshot.timings.len(), 1);
    assert_eq!(snapshot.timings[0].label, "annotation.root");
    assert_eq!(snapshot.timings[0].kind, TimingKind::Parallel);
    assert_eq!(snapshot.work["annotation.root"], 2);
    with_observer(None, || {
        assert_eq!(trace("disabled", &path, None, || 9), 9);
    });
}

#[test]
fn disabled_timing_does_not_read_the_clock() {
    assert!(start(None, "disabled", || panic!("disabled clock read")).is_none());
    let observer = InvocationObserver::new(false);
    assert!(start(Some(observer.clone()), "disabled", || panic!(
        "disabled clock read"
    ))
    .is_none());
    assert!(observer.snapshot().timings.is_empty());
    with_observer(Some(observer.clone()), || {
        assert_eq!(trace("disabled", &fixture(), None, || 9), 9);
    });
    assert!(observer.snapshot().timings.is_empty());
    assert!(observer.snapshot().work.is_empty());
}

#[test]
fn progress_only_records_fixed_timings_without_work_counters() {
    let observer = InvocationObserver::with_progress(false, true);
    with_observer(Some(observer.clone()), || {
        assert_eq!(
            trace("postgres.annotation.root", &fixture(), None, || 11),
            11
        );
    });
    let snapshot = observer.snapshot();
    assert!(snapshot.work.is_empty());
    assert_eq!(snapshot.timings.len(), 1);
    assert_eq!(snapshot.timings[0].label, "postgres.annotation.root");
    assert!(observer.progress_snapshot().unwrap().active.is_empty());
}
