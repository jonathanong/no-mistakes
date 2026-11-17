#![expect(
    clippy::disallowed_methods,
    reason = "Deadline and instrumentation tests require a real monotonic clock."
)]
use super::*;
use crate::diagnostics::progress::output::ProgressOutput;
use std::sync::Mutex;

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
fn abandoned_gate_skips_snapshot_publish() {
    let observer = InvocationObserver::new(false);
    let output = ProgressOutput::stderr();
    let gate = Mutex::new(SnapshotPhase::Abandoned);
    assert!(!publish_timeout_snapshot(&observer, &output, &gate));
    assert_eq!(*gate.lock().unwrap(), SnapshotPhase::Abandoned);
}

#[test]
fn unstarted_snapshot_takes_the_incomplete_line() {
    let (output, captured) = captured_output();
    let gate = Mutex::new(SnapshotPhase::Open);
    abandon_timeout_snapshot(&gate, &output);
    assert_eq!(*gate.lock().unwrap(), SnapshotPhase::Abandoned);
    assert_eq!(
        captured.recv_timeout(Duration::from_secs(2)).unwrap(),
        INCOMPLETE_LINE
    );
}

#[test]
fn transition_loses_when_the_phase_already_changed() {
    let gate = Mutex::new(SnapshotPhase::Abandoned);
    assert!(!claim_transition(
        &gate,
        SnapshotPhase::Published,
        SnapshotPhase::Finished
    ));
    assert_eq!(*gate.lock().unwrap(), SnapshotPhase::Abandoned);
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

fn captured_output() -> (Arc<ProgressOutput>, mpsc::Receiver<String>) {
    struct Capture(mpsc::Sender<String>);
    impl std::io::Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0
                .send(String::from_utf8(bytes.to_vec()).unwrap())
                .unwrap();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let (sender, receiver) = mpsc::channel();
    let output = ProgressOutput::start(Ok(Capture(sender)));
    output.emit("ready".into());
    assert_eq!(
        receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
        "ready"
    );
    (output, receiver)
}

#[test]
fn timeout_payload_includes_the_active_progress_phase() {
    let observer = InvocationObserver::with_progress(false, true);
    observer.trace("analysis.waiting", TimingKind::Parallel, || {
        let partial = partial_snapshot(&observer);
        assert!(
            partial.starts_with("[PARTIAL] timeout snapshot:"),
            "{partial}"
        );
        let progress = observer.render_progress_text().unwrap();
        let snapshot: serde_json::Value =
            serde_json::from_str(progress.strip_prefix("[progress] ").unwrap()).unwrap();
        assert!(snapshot["active"]
            .as_array()
            .unwrap()
            .iter()
            .any(|span| span["phase"] == serde_json::json!(["analysis.waiting"])));
    });
}

#[test]
fn abandoned_published_snapshot_reports_incomplete_without_late_progress() {
    let observer = InvocationObserver::with_progress(false, true);
    let (output, captured) = captured_output();
    let gate = Mutex::new(SnapshotPhase::Published);
    abandon_timeout_snapshot(&gate, &output);
    assert_eq!(*gate.lock().unwrap(), SnapshotPhase::Abandoned);
    assert_eq!(
        captured.recv_timeout(Duration::from_secs(2)).unwrap(),
        INCOMPLETE_LINE
    );
    assert!(!publish_timeout_snapshot(&observer, &output, &gate));
    assert!(captured.try_recv().is_err());
}
