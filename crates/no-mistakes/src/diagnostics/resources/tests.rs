use super::*;
use std::sync::{mpsc, Arc};

#[test]
fn resource_format_locks_cpu_util_and_high_water_delta() {
    let text = format_resources(aggregate(vec![PhaseResources {
        label: "parse".into(),
        wall: Duration::from_millis(10),
        start: Usage {
            user: Duration::from_millis(5),
            sys: Duration::from_millis(2),
            peak_rss_kib: 400,
        },
        end: Usage {
            user: Duration::from_millis(25),
            sys: Duration::from_millis(12),
            peak_rss_kib: 600,
        },
    }]));
    assert_eq!(text, "[cpu] parse: user=20.000ms sys=10.000ms util=3.000x\n[memory] parse: peak_rss_kib=600 delta_kib=200\n");
}

#[test]
fn zero_wall_and_decreasing_samples_are_saturating() {
    let start = Usage {
        user: Duration::from_secs(1),
        sys: Duration::from_secs(1),
        peak_rss_kib: 100,
    };
    let text = format_resources(aggregate(vec![PhaseResources {
        label: "empty".into(),
        wall: Duration::ZERO,
        start,
        end: Usage {
            user: Duration::ZERO,
            sys: Duration::ZERO,
            peak_rss_kib: 50,
        },
    }]));
    assert!(text.contains("user=0.000ms sys=0.000ms util=0.000x"));
    assert!(text.contains("delta_kib=0"));
}

#[test]
fn basic_and_disabled_diagnostics_add_no_resource_calls_or_state() {
    let count = usage::tests::sample_count();
    super::super::context::measure_optional(
        None,
        "disabled",
        TimingKind::Serial,
        || panic!("disabled clock"),
        || (),
    );
    let observer = InvocationObserver::new(false);
    observer.trace("basic", TimingKind::Serial, || ());
    assert!(observer.resources.is_none());
    assert!(observer.resource_block().is_empty());
    assert_eq!(usage::tests::sample_count(), count);
}

#[test]
#[cfg(unix)]
fn dispatch_is_transparent_and_nested_and_parallel_spans_do_not_sample() {
    let observer = InvocationObserver::new(true);
    let count = usage::tests::sample_count();
    observer.trace("invocation.dispatch", TimingKind::Serial, || {
        observer.trace("phase", TimingKind::Serial, || {
            observer.trace("nested", TimingKind::Serial, || ());
            observer.trace("worker", TimingKind::Parallel, || {
                observer.trace("worker.child", TimingKind::Serial, || ());
            });
        });
        observer.trace("parallel", TimingKind::Parallel, || {
            observer.trace("parallel.child", TimingKind::Serial, || ());
        });
    });
    assert_eq!(usage::tests::sample_count() - count, 2);
    let text = observer.resource_block();
    assert!(text.contains("[cpu] phase:"), "{text}");
    assert_eq!(text.lines().count(), 2, "{text}");
}

#[test]
fn unwind_removes_resource_scope() {
    let observer = InvocationObserver::new(true);
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        observer.trace("panic", TimingKind::Serial, || panic!("test"));
    }));
    assert_eq!(
        observer.resources.as_ref().unwrap().lock().unwrap().active,
        0
    );
    assert!(observer.resource_block().is_empty());
}

#[test]
fn resource_history_aggregation_does_not_hold_worker_locks() {
    let observer = InvocationObserver::new(true);
    let (captured, capture) = mpsc::channel();
    let (release, released) = mpsc::channel();
    std::thread::scope(|scope| {
        let worker_observer = Arc::clone(&observer);
        scope.spawn(move || {
            worker_observer.resource_snapshot_with(|completed| {
                captured.send(()).unwrap();
                released.recv().unwrap();
                aggregate(completed)
            })
        });
        capture.recv().unwrap();
        let (finished, finish) = mpsc::channel();
        let worker_observer = Arc::clone(&observer);
        scope.spawn(move || {
            rayon::ThreadPoolBuilder::new()
                .num_threads(2)
                .build()
                .unwrap()
                .install(|| {
                    rayon::join(
                        || worker_observer.trace("first", TimingKind::Parallel, || ()),
                        || worker_observer.trace("second", TimingKind::Parallel, || ()),
                    );
                });
            finished.send(()).unwrap();
        });
        let workers_finished = finish.recv_timeout(Duration::from_secs(5));
        release.send(()).unwrap();
        assert!(
            workers_finished.is_ok(),
            "aggregation blocked span entry or cleanup"
        );
    });
    assert_eq!(
        observer.resources.as_ref().unwrap().lock().unwrap().active,
        0
    );
}

#[test]
#[cfg(unix)]
fn phase_wall_interval_includes_both_resource_samples() {
    let observer = InvocationObserver::new(true);
    let before = usage::tests::sample_count();
    let samples_at_clock = std::cell::RefCell::new(Vec::new());
    super::super::context::measure_optional(
        Some(&observer),
        "phase",
        TimingKind::Serial,
        || {
            samples_at_clock
                .borrow_mut()
                .push(usage::tests::sample_count());
            std::time::Instant::now()
        },
        || (),
    );
    assert_eq!(*samples_at_clock.borrow(), [before, before + 2]);
}

#[test]
fn repeated_labels_share_cli_and_structured_totals_in_first_seen_order() {
    let observer = InvocationObserver::new(true);
    let completed = vec![
        resource_phase("discovery.classify", [5, 2, 3], [600, 700]),
        resource_phase("discovery", [10, 20, 10], [400, 600]),
        resource_phase("discovery", [30, 3, 7], [700, 750]),
        resource_phase("discovery.classify", [15, 9, 1], [750, 690]),
    ];
    let state = observer.resources.as_ref().unwrap();
    state.lock().unwrap().completed = completed;
    let expected = vec![
        PhaseResourceDiagnostic {
            label: "discovery.classify".into(),
            wall_ms: 20.0,
            user_ms: 11.0,
            system_ms: 4.0,
            utilization: 0.75,
            peak_rss_kib: 700,
            peak_rss_delta_kib: 100,
        },
        PhaseResourceDiagnostic {
            label: "discovery".into(),
            wall_ms: 40.0,
            user_ms: 23.0,
            system_ms: 17.0,
            utilization: 1.0,
            peak_rss_kib: 750,
            peak_rss_delta_kib: 250,
        },
    ];
    assert_eq!(observer.resource_snapshot(), expected);
    let json = serde_json::to_value(observer.verbose_snapshot().unwrap()).unwrap();
    assert_eq!(json["phases"], serde_json::to_value(expected).unwrap());
    assert_eq!(
        observer.resource_block(),
        concat!(
            "[cpu] discovery.classify: user=11.000ms sys=4.000ms util=0.750x\n",
            "[memory] discovery.classify: peak_rss_kib=700 delta_kib=100\n",
            "[cpu] discovery: user=23.000ms sys=17.000ms util=1.000x\n",
            "[memory] discovery: peak_rss_kib=750 delta_kib=250\n",
        )
    );
}

fn resource_phase(label: &str, [wall, user, sys]: [u64; 3], rss: [u64; 2]) -> PhaseResources {
    PhaseResources {
        label: label.into(),
        wall: Duration::from_millis(wall),
        start: Usage {
            user: Duration::from_millis(5),
            sys: Duration::from_millis(2),
            peak_rss_kib: rss[0],
        },
        end: Usage {
            user: Duration::from_millis(5 + user),
            sys: Duration::from_millis(2 + sys),
            peak_rss_kib: rss[1],
        },
    }
}

#[test]
#[cfg(unix)]
fn facts_preparation_is_transparent_to_syntax_and_projection_resources() {
    let observer = InvocationObserver::new(true);
    let count = usage::tests::sample_count();
    observer.trace("facts.prepare", TimingKind::Serial, || {
        for _ in 0..2 {
            observer.trace("parse", TimingKind::Serial, || ());
            observer.trace("postgres.annotation.project", TimingKind::Serial, || ());
        }
    });
    assert_eq!(usage::tests::sample_count() - count, 8);
    let phases = observer.resource_snapshot();
    assert_eq!(phases.len(), 2);
    assert_eq!(phases[0].label, "parse");
    assert_eq!(phases[1].label, "postgres.annotation.project");
}
