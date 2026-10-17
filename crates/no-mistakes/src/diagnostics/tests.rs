use super::context::measure_optional;
use super::observer::timing_order;
use super::*;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn disabled_measure_never_reads_the_clock() {
    let (value, duration) = measure_optional(
        None,
        "disabled",
        TimingKind::Serial,
        || panic!("clock was read"),
        || 42,
    );
    assert_eq!(value, 42);
    assert_eq!(duration, Duration::ZERO);
}

#[test]
fn snapshot_sorts_timings_and_work() {
    let observer = InvocationObserver::new(true);
    observer.record_duration("z", Duration::from_millis(2), TimingKind::Parallel);
    observer.record_duration("a", Duration::from_millis(1), TimingKind::Serial);
    observer.increment("source.reads", 2);
    observer.increment("source.reads", 1);
    let snapshot = observer.snapshot();
    assert_eq!(
        snapshot
            .timings
            .iter()
            .map(|entry| entry.label.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "z"]
    );
    assert_eq!(snapshot.work["source.reads"], 3);
}

#[test]
fn observer_modes_installation_and_legacy_adapter_are_covered() {
    assert!(DiagnosticsArgs::default().observer().is_none());
    assert!(DiagnosticsArgs::default().progress_interval().is_none());
    let progress = DiagnosticsArgs {
        progress: true,
        progress_interval: 5,
        ..DiagnosticsArgs::default()
    };
    assert_eq!(progress.progress_interval(), Some(Duration::from_millis(5)));
    assert!(progress.observer().unwrap().progress_enabled());
    assert!(DiagnosticsArgs {
        timings: true,
        verbose_timings: false,
        ..DiagnosticsArgs::default()
    }
    .observer()
    .is_some());
    let verbose = DiagnosticsArgs {
        timings: false,
        verbose_timings: true,
        ..DiagnosticsArgs::default()
    }
    .observer()
    .expect("verbose timings imply an observer");
    assert!(verbose.verbose());

    let guard = InvocationGuard::install(verbose.clone());
    assert!(Arc::ptr_eq(
        &current().expect("observer installed"),
        &verbose
    ));
    assert_eq!(verbose.trace("trace", TimingKind::Serial, || 7), 7);
    let (value, _) = verbose.measure("measure", TimingKind::Parallel, || 8);
    assert_eq!(value, 8);
    let (value, _) = measure_if_enabled("current", TimingKind::Serial, || 9);
    assert_eq!(value, 9);
    let nested = LegacyDiagnosticsGuard::new(true, true);
    assert!(nested.observer.is_none());
    drop(nested);
    verbose.render_stderr();
    drop(guard);

    let basic = InvocationObserver::new(false);
    basic.increment("ignored", 1);
    assert!(basic.snapshot().work.is_empty());
    basic.render_stderr();

    let disabled_legacy = LegacyDiagnosticsGuard::new(false, false);
    assert!(disabled_legacy.observer.is_none());
}

#[test]
fn stable_timing_order_covers_every_phase_family() {
    let labels = [
        "discovery",
        "discovery.files",
        "read",
        "read.source",
        "parse",
        "parse.source",
        "manifest",
        "manifest.package",
        "resolve",
        "resolve.import",
        "search",
        "ingest",
        "parse+analysis",
        "analysis",
        "prepare",
        "discover.dotnet",
        "discover.vitest",
        "discover.playwright",
        "discover.swift",
        "graph",
        "select.dotnet",
        "select.vitest",
        "select.playwright",
        "select.swift",
        "generic-checks",
        "analysis.react",
        "analysis.queues",
        "analysis.rules",
        "analysis.integration",
        "analysis.codebase",
        "analysis.filesystem_rules",
        "graph.imports",
        "graph.forward_adjacency_normalization",
        "graph.reverse_adjacency_normalization",
        "graph.canonical_flatten",
        "graph.ordinal_construction",
        "graph.playwright_selector_merge",
        "traversal.dependencies",
        "analysis.server",
        "rules.config",
        "playwright.routes",
        "other",
        "output",
    ];
    let observer = InvocationObserver::new(false);
    for label in labels.into_iter().rev() {
        observer.record_duration(label, Duration::ZERO, TimingKind::Serial);
    }
    let snapshot = observer.snapshot();
    assert_eq!(snapshot.timings.len(), labels.len());
    assert_eq!(snapshot.timings.first().unwrap().label, "discovery");
    assert_eq!(snapshot.timings.last().unwrap().label, "output");
}

#[test]
fn concurrent_observers_are_thread_isolated() {
    let first = InvocationObserver::new(true);
    let second = InvocationObserver::new(true);
    let first_thread = std::thread::spawn({
        let observer = Arc::clone(&first);
        move || {
            with_observer(Some(observer), || {
                measure_if_enabled("first", TimingKind::Serial, || ());
            });
        }
    });
    let second_thread = std::thread::spawn({
        let observer = Arc::clone(&second);
        move || {
            with_observer(Some(observer), || {
                measure_if_enabled("second", TimingKind::Serial, || ());
            });
        }
    });
    first_thread.join().unwrap();
    second_thread.join().unwrap();

    assert_eq!(first.snapshot().timings[0].label, "first");
    assert_eq!(second.snapshot().timings[0].label, "second");
}

#[test]
fn with_observer_none_masks_an_inherited_observer() {
    let observer = InvocationObserver::new(true);
    with_observer(Some(Arc::clone(&observer)), || {
        assert!(current().is_some());
        with_observer(None, || {
            assert!(current().is_none());
            measure_if_enabled("masked", TimingKind::Serial, || ());
            let inner = InvocationObserver::new(true);
            with_observer(Some(Arc::clone(&inner)), || {
                assert!(current().is_some());
                measure_if_enabled("inner", TimingKind::Serial, || ());
            });
            assert!(current().is_none());
            assert_eq!(inner.snapshot().timings[0].label, "inner");
        });
        assert!(current().is_some());
    });
    assert!(observer.snapshot().timings.is_empty());
}

#[test]
fn with_observer_resets_inherited_parallel_timing() {
    with_timing_kind(TimingKind::Parallel, || {
        assert_eq!(current_timing_kind(), TimingKind::Parallel);
        with_observer(None, || {
            assert_eq!(current_timing_kind(), TimingKind::Serial);
        });
        with_observer(Some(InvocationObserver::new(false)), || {
            assert_eq!(current_timing_kind(), TimingKind::Serial);
        });
        assert_eq!(current_timing_kind(), TimingKind::Parallel);
    });
}

#[test]
fn nested_timings_inherit_parallel_context() {
    let observer = InvocationObserver::new(true);
    with_observer(Some(Arc::clone(&observer)), || {
        measure_if_enabled("outer", TimingKind::Parallel, || {
            crate::perf_trace::trace("inner", || ());
        });
    });
    assert!(observer
        .snapshot()
        .timings
        .iter()
        .all(|timing| timing.kind == TimingKind::Parallel));
}
#[test]
fn timing_order_preserves_every_exact_rank() {
    let expected = [
        ("discovery", 10),
        ("read", 20),
        ("parse", 30),
        ("manifest", 40),
        ("resolve", 50),
        ("search", 60),
        ("ingest", 61),
        ("parse+analysis", 62),
        ("analysis", 63),
        ("prepare", 100),
        ("discover.dotnet", 110),
        ("discover.vitest", 111),
        ("discover.playwright", 112),
        ("discover.swift", 113),
        ("graph", 120),
        ("select.dotnet", 130),
        ("select.vitest", 131),
        ("select.playwright", 132),
        ("select.swift", 133),
        ("generic-checks", 140),
        ("analysis.react", 200),
        ("analysis.queues", 201),
        ("analysis.rules", 202),
        ("analysis.integration", 203),
        ("analysis.codebase", 204),
        ("analysis.filesystem_rules", 205),
        ("output", 900),
    ];

    for (label, rank) in expected {
        assert_eq!(timing_order(label), rank, "{label}");
    }
}

#[test]
fn timing_order_preserves_every_prefix_fallback_rank() {
    let expected = [
        ("discovery.files", 11),
        ("read.source", 21),
        ("parse.source", 31),
        ("manifest.package", 41),
        ("resolve.import", 51),
        ("graph.imports", 301),
        ("traversal.dependencies", 400),
        ("analysis.server", 500),
        ("rules.config", 600),
        ("playwright.routes", 700),
    ];

    for (label, rank) in expected {
        assert_eq!(timing_order(label), rank, "{label}");
    }
}

#[test]
fn timing_order_uses_unknown_fallback_rank() {
    for label in ["", "other", "discover.unknown"] {
        assert_eq!(timing_order(label), 800, "{label}");
    }
}

#[test]
fn timing_order_prefers_exact_rank_over_matching_prefix() {
    assert_eq!(timing_order("analysis.react"), 200);
    assert_eq!(timing_order("analysis.react.child"), 500);
}

#[test]
fn record_ast_walk_increments_only_when_an_observer_is_installed() {
    record_ast_walk();
    let observer = InvocationObserver::new(true);
    let guard = InvocationGuard::install(observer.clone());
    record_ast_walk();
    record_ast_walk();
    drop(guard);
    assert_eq!(observer.snapshot().work["ast.walks"], 2);
}

#[test]
fn concurrent_work_and_source_counts_preserve_exact_totals() {
    let observer = InvocationObserver::new(true);
    let other = InvocationObserver::new(true);
    std::thread::scope(|scope| {
        for worker in 0..8 {
            let observer = &observer;
            let other = &other;
            scope.spawn(move || {
                let own_path = std::path::PathBuf::from(format!("worker-{worker}.ts"));
                with_observer(Some(Arc::clone(observer)), || {
                    for _ in 0..1_000 {
                        // Start with missing keys to exercise concurrent insertion
                        // as well as the established counter fast path.
                        current().unwrap().increment("shared", 3);
                        observer.increment("parse.requests", 1);
                        observer.increment("resolver.computations", 2);
                        observer.increment("source.bytes", 0);
                        observer.increment("extension.zero", 0);
                        observer.record_source_read(std::path::Path::new("shared.ts"));
                        observer.record_source_read(&own_path);
                        with_observer(Some(Arc::clone(other)), || {
                            current().unwrap().increment("nested", 2);
                        });
                    }
                });
            });
        }
    });
    assert_eq!(
        observer.snapshot().work,
        std::collections::BTreeMap::from([
            ("extension.zero", 0),
            ("parse.requests", 8_000),
            ("resolver.computations", 16_000),
            ("shared", 24_000),
            ("source.bytes", 0),
        ])
    );
    assert_eq!(
        other.snapshot().work,
        std::collections::BTreeMap::from([("nested", 16_000)])
    );
    let reads = observer.source_read_snapshot();
    assert_eq!(reads.len(), 9);
    assert_eq!(reads[std::path::Path::new("shared.ts")], 8_000);
    for worker in 0..8 {
        assert_eq!(
            reads[&std::path::PathBuf::from(format!("worker-{worker}.ts"))],
            1_000
        );
    }
    let basic = InvocationObserver::new(false);
    basic.record_source_read(std::path::Path::new("ignored.ts"));
    assert!(basic.source_read_snapshot().is_empty());
}

#[test]
fn borrowed_observer_keeps_reference_count_and_disabled_scope() {
    assert!(with_current(|observer| observer.is_none()));
    let observer = InvocationObserver::new(true);
    with_observer(Some(Arc::clone(&observer)), || {
        let installed_count = Arc::strong_count(&observer);
        with_current(|borrowed| {
            // An owned accessor adds a shared atomic reference on every call.
            assert_eq!(Arc::strong_count(&observer), installed_count);
            assert!(std::ptr::eq(borrowed.unwrap(), observer.as_ref()));
            borrowed
                .unwrap()
                .increment("postgres.annotation.reachability_walks", 1);
            assert_eq!(
                observer.snapshot().work["postgres.annotation.reachability_walks"],
                1
            );
        });
        with_observer(None, || {
            assert!(with_current(|observer| observer.is_none()))
        });
        assert!(with_current(|borrowed| borrowed.is_some()));
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_current::<()>(|_| panic!("borrow ends"))
        }));
        assert!(panic.is_err());
        // Unwinding the callback must release the thread-local borrow.
        with_observer(None, || {
            assert!(with_current(|observer| observer.is_none()))
        });
    });
    assert_eq!(Arc::strong_count(&observer), 1);
    assert!(with_current(|observer| observer.is_none()));
}
