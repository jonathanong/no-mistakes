use super::{
    note_names, note_rebuild, note_slots, note_walk, RootCounters, NAMES, REBUILDS, SLOTS, WALKS,
};
use crate::diagnostics::{work_enabled, InvocationGuard, InvocationObserver};
use std::sync::Arc;

#[test]
fn root_batch_sums_inner_notes_without_cloning_the_observer() {
    let observer = InvocationObserver::new(true);
    let _guard = InvocationGuard::install(Arc::clone(&observer));
    let installed = Arc::strong_count(&observer);
    {
        let _batch = RootCounters::begin();
        for _ in 0..32 {
            note_walk();
            note_slots(0);
        }
        note_slots(5);
        note_rebuild();
        note_names(2);
        note_names(0);
        assert_eq!(Arc::strong_count(&observer), installed);
        assert!(
            observer.snapshot().work.is_empty(),
            "inner iterations must not publish until the root flushes"
        );
    }
    let work = observer.snapshot().work;
    assert_eq!(work[WALKS], 32);
    assert_eq!(work[SLOTS], 5);
    assert_eq!(work[REBUILDS], 1);
    assert_eq!(work[NAMES], 2);
    assert_eq!(Arc::strong_count(&observer), installed);
}

#[test]
fn nested_roots_flush_independently() {
    let observer = InvocationObserver::new(true);
    let _guard = InvocationGuard::install(Arc::clone(&observer));
    {
        let _outer = RootCounters::begin();
        note_walk();
        {
            let _inner = RootCounters::begin();
            note_walk();
            note_slots(4);
            assert!(observer.snapshot().work.is_empty());
        }
        assert_eq!(observer.snapshot().work[WALKS], 1);
        assert_eq!(observer.snapshot().work[SLOTS], 4);
        note_rebuild();
    }
    let work = observer.snapshot().work;
    assert_eq!(work[WALKS], 2);
    assert_eq!(work[SLOTS], 4);
    assert_eq!(work[REBUILDS], 1);
}

#[test]
fn unbatched_notes_publish_immediately_including_explicit_zeros() {
    let observer = InvocationObserver::new(true);
    let _guard = InvocationGuard::install(Arc::clone(&observer));
    note_walk();
    note_slots(0);
    note_rebuild();
    note_names(0);
    let work = observer.snapshot().work;
    assert_eq!(work[WALKS], 1);
    assert_eq!(work[SLOTS], 0);
    assert_eq!(work[REBUILDS], 1);
    assert_eq!(work[NAMES], 0);
}

#[test]
fn disabled_and_non_verbose_observers_do_not_record() {
    assert!(!work_enabled());
    {
        let _batch = RootCounters::begin();
        note_walk();
        note_slots(9);
        note_rebuild();
        note_names(3);
    }
    let quiet = InvocationObserver::new(false);
    let _guard = InvocationGuard::install(Arc::clone(&quiet));
    assert!(!work_enabled());
    {
        let _batch = RootCounters::begin();
        note_walk();
        note_slots(9);
        note_rebuild();
        note_names(3);
    }
    note_walk();
    assert!(quiet.snapshot().work.is_empty());
}
