use super::CliErrorOutput;
use crate::invocation::{InvocationError, InvocationErrorKind};

fn deadline() -> anyhow::Error {
    InvocationError::new(InvocationErrorKind::CommandTimeout, "deadline elapsed")
        .with_phase("command")
        .into()
}

#[test]
fn concurrent_boundary_and_watchdog_claims_have_exactly_one_winner() {
    let output = CliErrorOutput::new(true);
    let barrier = std::sync::Barrier::new(16);
    let winners = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..16)
            .map(|_| {
                let output = output.clone();
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    usize::from(output.report(&deadline()).is_some())
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .sum::<usize>()
    });
    assert_eq!(winners, 1);
    assert!(output.report(&deadline()).is_none());
}

#[test]
fn requests_are_independent_and_unrelated_errors_do_not_consume_the_claim() {
    let first = CliErrorOutput::new(true);
    let second = CliErrorOutput::new(true);
    assert!(first
        .report(&anyhow::anyhow!("ordinary command error"))
        .is_none());
    for output in [first, second] {
        let report = output.report(&deadline()).unwrap();
        assert_eq!(report["error"]["kind"], "deadline");
        assert_eq!(report["error"]["phase"], "command");
        assert!(output.report(&deadline()).is_none());
    }
    assert!(CliErrorOutput::new(false).report(&deadline()).is_none());
}

#[test]
fn boundary_waits_for_claimed_watchdog_output_completion() {
    use std::sync::mpsc;
    use std::time::Duration;
    let output = CliErrorOutput::new(true);
    assert!(output.report(&deadline()).is_some());
    // Keep the boundary from spending its grace period before the parent is
    // scheduled to observe it. Production never holds this mutex during I/O.
    let mut pending = output.0.as_ref().unwrap().completed.lock().unwrap();
    let (started, starting) = mpsc::channel();
    let (finished, finishing) = mpsc::channel();
    std::thread::scope(|scope| {
        let output = &output;
        scope.spawn(move || {
            started.send(()).unwrap();
            assert!(output.report_or_wait(&deadline()).is_none());
            finished.send(()).unwrap();
        });
        starting.recv().unwrap();
        // Hold acknowledgement until after the boundary has had a chance to
        // lose the claim. The boundary must not immediately terminate the run.
        assert!(finishing.recv_timeout(Duration::from_millis(5)).is_err());
        *pending = true;
        drop(pending);
        output.complete();
        finishing.recv_timeout(Duration::from_secs(1)).unwrap();
    });
    assert!(output.report_or_wait(&deadline()).is_none());
}

#[test]
fn missing_writer_completion_only_receives_a_bounded_grace_period() {
    let output = CliErrorOutput::new(true);
    assert!(output.report(&deadline()).is_some());
    let started = std::time::Instant::now();
    assert!(output.report_or_wait(&deadline()).is_none());
    let elapsed = started.elapsed();
    assert!(
        elapsed >= std::time::Duration::from_millis(20),
        "{elapsed:?}"
    );
    assert!(elapsed < std::time::Duration::from_secs(1), "{elapsed:?}");
}

#[test]
fn unrelated_errors_and_disabled_output_do_not_wait_on_completion() {
    let output = CliErrorOutput::new(true);
    assert!(output.report_or_wait(&deadline()).is_some());
    let claim = output.0.as_ref().unwrap();
    let _pending = claim.completed.lock().unwrap();
    // Non-invocation errors and disabled JSON must never wait for a writer.
    assert!(output
        .report_or_wait(&anyhow::anyhow!("unrelated error"))
        .is_none());
    assert!(CliErrorOutput::new(false)
        .report_or_wait(&deadline())
        .is_none());
}
