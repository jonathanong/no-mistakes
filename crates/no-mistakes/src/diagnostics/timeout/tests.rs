use super::*;

#[test]
fn partial_snapshot_preserves_published_counts_and_completed_timings() {
    let observer = InvocationObserver::new(true);
    observer.increment("source.reads", 7);
    observer.increment("postgres.annotation.prefix_nodes", 42);
    observer.record_duration("prepared", Duration::from_millis(3), TimingKind::Serial);
    observer.record_duration("parallel", Duration::from_millis(2), TimingKind::Parallel);
    let text = partial_snapshot(&observer);
    assert!(text.contains("[PARTIAL] timeout snapshot:"));
    assert!(text.contains("lower bounds"));
    assert!(text.contains("[PARTIAL work] source.reads: 7\n"));
    assert!(text.contains("[PARTIAL work] postgres.annotation.prefix_nodes: 42\n"));
    assert!(text.contains("[PARTIAL timing] prepared: 3.000ms\n"));
    assert!(text.contains("[PARTIAL timing] parallel: 2.000ms (parallel; non-additive)\n"));
    assert!(text.contains("[PARTIAL timing] elapsed:"));
    assert!(!observer.progress_enabled());
}

#[test]
fn basic_timings_do_not_create_work_counts() {
    let observer = InvocationObserver::new(false);
    observer.increment("source.reads", 7);
    let text = partial_snapshot(&observer);
    assert!(!text.contains("[PARTIAL work]"));
    assert!(!observer.progress_enabled());
}

#[test]
fn stalled_progress_snapshot_does_not_hold_timeout_exit() {
    let observer = InvocationObserver::with_progress(true, true);
    let held = observer.progress.as_ref().unwrap().lock().unwrap();
    let started = std::time::Instant::now();
    render_timeout_stderr(
        Some(Arc::clone(&observer)),
        "forced test deadline".to_string(),
    );
    assert!(started.elapsed() < Duration::from_secs(1));
    drop(held);
}
