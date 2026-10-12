use super::*;

#[test]
fn live_parallel_spans_coexist_and_returned_spans_close() {
    let observer = InvocationObserver::with_progress(false, true);
    let (ready, started) = mpsc::channel();
    let (release_a, wait_a) = mpsc::channel();
    let (release_b, wait_b) = mpsc::channel();
    let first = std::thread::spawn({
        let observer = Arc::clone(&observer);
        let ready = ready.clone();
        move || {
            observer.trace("analysis.a", TimingKind::Parallel, || {
                observer.measure("analysis.a.inner", TimingKind::Parallel, || {
                    ready.send(()).unwrap();
                    wait_a.recv().unwrap();
                    Ok::<_, &'static str>(42)
                })
            })
        }
    });
    let second = std::thread::spawn({
        let observer = Arc::clone(&observer);
        move || {
            observer.trace("analysis.b", TimingKind::Parallel, || {
                ready.send(()).unwrap();
                wait_b.recv().unwrap();
                Err::<(), _>("analysis failed")
            })
        }
    });
    for _ in 0..2 {
        started.recv_timeout(Duration::from_secs(2)).unwrap();
    }
    let snapshot = observer.progress_snapshot().unwrap();
    assert_eq!(snapshot.started_spans, 3);
    assert_eq!(snapshot.completed_spans, 0);
    assert_eq!(snapshot.active.len(), 3);
    assert!(snapshot
        .active
        .iter()
        .all(|span| span.parallel && span.elapsed_ms >= 0.0 && span.started_ms >= 0.0));
    assert!(snapshot
        .active
        .iter()
        .any(|span| span.phase == ["analysis.a", "analysis.a.inner"]));
    assert_eq!(
        snapshot
            .active
            .iter()
            .map(|span| span.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
    release_a.send(()).unwrap();
    assert_eq!(first.join().unwrap().0, Ok(42));
    let snapshot = observer.progress_snapshot().unwrap();
    assert_eq!(snapshot.completed_spans, 2);
    assert_eq!(snapshot.active[0].phase, ["analysis.b"]);
    release_b.send(()).unwrap();
    assert_eq!(second.join().unwrap(), Err("analysis failed"));
    let snapshot = observer.progress_snapshot().unwrap();
    assert!(snapshot.active.is_empty());
    assert_eq!(snapshot.completed_spans, snapshot.started_spans);
    assert_eq!(observer.snapshot().timings.len(), 3);
    assert_eq!(snapshot.completed.len(), 3);
    assert!(snapshot.completed.iter().all(|span| span.parallel));
}

#[test]
fn panic_unwinds_active_spans_and_unrelated_observers_do_not_inherit_phases() {
    let observer = InvocationObserver::with_progress(false, true);
    let other = InvocationObserver::with_progress(false, true);
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        observer.trace("outer", TimingKind::Serial, || {
            other.trace("other", TimingKind::Serial, || {
                assert_eq!(
                    other.progress_snapshot().unwrap().active[0].phase,
                    ["other"]
                );
            });
            observer.trace("panic", TimingKind::Serial, || panic!("unwind"));
        });
    }));
    assert!(observer.progress_snapshot().unwrap().active.is_empty());
    observer.trace("next", TimingKind::Serial, || {
        assert_eq!(
            observer.progress_snapshot().unwrap().active[0].phase,
            ["next"]
        );
    });
}

#[test]
fn ordinary_timing_observers_have_no_progress_state() {
    let observer = InvocationObserver::new(true);
    observer.trace("closed", TimingKind::Serial, || ());
    assert!(observer.progress.is_none());
    assert!(observer.progress_snapshot().is_none());
    observer.render_progress_stderr();
    // Installing a heartbeat does not turn an ordinary observer into progress.
    drop(ProgressGuard::install(observer, Duration::from_secs(3600)));
}

#[test]
fn heartbeat_is_periodic_and_stops_promptly_without_waiting_for_interval() {
    let (sender, receiver) = mpsc::channel();
    let guard = ProgressGuard::start(Duration::from_millis(10), move || {
        sender.send(()).unwrap();
    });
    for _ in 0..3 {
        receiver.recv_timeout(Duration::from_secs(2)).unwrap();
    }
    drop(guard);
    // Joining ensures no callback can run after cleanup; drain in-flight messages.
    while receiver.try_recv().is_ok() {}
    assert_eq!(
        receiver.recv_timeout(Duration::from_millis(30)),
        Err(mpsc::RecvTimeoutError::Disconnected)
    );
    let guard = ProgressGuard::start(Duration::from_secs(3600), || panic!("early heartbeat"));
    let started = Instant::now();
    drop(guard);
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn parallel_workers_share_the_live_dispatch_ancestor() {
    let observer = InvocationObserver::with_progress(false, true);
    observer.trace("invocation.dispatch", TimingKind::Serial, || {
        std::thread::spawn({
            let observer = Arc::clone(&observer);
            move || {
                observer.trace("worker", TimingKind::Parallel, || {
                    let snapshot = observer.progress_snapshot().unwrap();
                    assert_eq!(snapshot.active.len(), 2);
                    assert!(snapshot.active.iter().any(|span| {
                        span.phase == ["invocation.dispatch", "worker"] && span.parallel
                    }));
                })
            }
        })
        .join()
        .unwrap();
    });
    let snapshot = observer.progress_snapshot().unwrap();
    assert!(snapshot.active.is_empty());
    assert_eq!(snapshot.completed_spans, 2);
    assert!(!observer.emit_progress_output("no CLI writer".to_string()));
}

#[test]
fn failed_snapshot_serialization_does_not_emit_a_partial_progress_line() {
    struct Unsupported;
    impl serde::Serialize for Unsupported {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("unsupported diagnostic value"))
        }
    }
    render_snapshot(&Unsupported, |_| panic!("failed snapshot must not emit"));
}
