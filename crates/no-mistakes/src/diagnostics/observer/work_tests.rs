use super::*;

#[test]
fn core_indices_and_lazy_extension_allocation_are_preserved() {
    let counters = WorkCounters::new();
    assert!(counters.extensions.get().is_none());
    assert!(counters.snapshot().is_empty());
    for (index, name) in NAMES.iter().enumerate() {
        assert_eq!(core_index(name), Some(index));
        counters.increment(name, 0);
    }
    let snapshot = counters.snapshot();
    assert_eq!(snapshot.len(), NAMES.len());
    assert!(snapshot.values().all(|value| *value == 0));
    assert!(counters.extensions.get().is_none());
    counters.increment("extension.zero", 0);
    assert_eq!(counters.snapshot()["extension.zero"], 0);
    assert!(counters.extensions.get().is_some());
}

#[test]
fn timing_only_observers_allocate_no_counter_storage() {
    let observer = super::super::InvocationObserver::new(false);
    assert!(observer.work.is_none());
    assert!(observer.source_reads.is_none());
    observer.increment("parse.requests", 1);
    observer.increment("extension.zero", 0);
    assert!(observer.snapshot().work.is_empty());
}

#[test]
fn unknown_metrics_in_every_length_bucket_use_extensions() {
    let counters = WorkCounters::new();
    for metric in [
        "",
        "unknown.short",
        "unknown.middle.xx",
        "unknown.longer.xxxxxx",
        "unknown.longest.xxxxxxxxxxx",
    ] {
        assert_eq!(core_index(metric), None);
        counters.increment(metric, 1);
        assert_eq!(counters.snapshot()[metric], 1);
    }
}

#[test]
fn annotation_counters_are_live_core_slots_in_parallel() {
    let counters = WorkCounters::new();
    let metrics = [
        "postgres.annotation.modules",
        "postgres.annotation.target_resolutions",
        "postgres.annotation.reachability_walks",
        "postgres.annotation.incoming_dense_slots_copied",
        "postgres.annotation.captured_reader_rebuilds",
        "postgres.annotation.captured_reader_names_retained",
        "postgres.annotation.roots",
    ];
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| {
                for metric in metrics {
                    counters.increment(metric, 0);
                    for _ in 0..100 {
                        counters.increment(metric, 1);
                    }
                }
            });
        }
    });
    assert!(counters.extensions.get().is_none());
    assert_eq!(
        counters.snapshot(),
        metrics.into_iter().map(|metric| (metric, 400)).collect()
    );
}
