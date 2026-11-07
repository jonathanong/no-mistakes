use super::*;
use std::sync::{mpsc, Arc};

#[test]
fn resource_format_locks_cpu_util_and_high_water_delta() {
    let text = format_resources(vec![PhaseResources {
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
    }]);
    assert_eq!(text, "[cpu] parse: user=20.000ms sys=10.000ms util=3.000x\n[memory] parse: peak_rss_kib=600 delta_kib=200\n");
}

#[test]
fn zero_wall_and_decreasing_samples_are_saturating() {
    let start = Usage {
        user: Duration::from_secs(1),
        sys: Duration::from_secs(1),
        peak_rss_kib: 100,
    };
    let text = format_resources(vec![PhaseResources {
        label: "empty".into(),
        wall: Duration::ZERO,
        start,
        end: Usage {
            peak_rss_kib: 50,
            ..start
        },
    }]);
    assert!(text.contains("util=0.000x"));
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
fn resource_history_formatting_does_not_hold_worker_locks() {
    let observer = InvocationObserver::new(true);
    let (captured, capture) = mpsc::channel();
    let (release, released) = mpsc::channel();
    std::thread::scope(|scope| {
        let worker_observer = Arc::clone(&observer);
        scope.spawn(move || {
            worker_observer.resource_block_with(|completed| {
                captured.send(()).unwrap();
                released.recv().unwrap();
                format_resources(completed)
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
