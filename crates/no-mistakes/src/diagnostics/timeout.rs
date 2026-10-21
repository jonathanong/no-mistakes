//! Hard exits cannot unwind the ordinary final diagnostics renderer.
use super::progress::output::ProgressOutput;
use super::{InvocationObserver, TimingKind};
use std::fmt::Write;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

const INCOMPLETE_LINE: &str =
    "[PARTIAL] timeout diagnostics incomplete: snapshot collection or delivery did not finish\n";

#[derive(Clone, Copy, PartialEq, Eq)]
enum SnapshotPhase {
    Open,
    Published,
    Finished,
    Abandoned,
}

pub fn render_timeout_stderr(observer: Option<Arc<InvocationObserver>>, message: String) {
    // Created only after expiry, even with diagnostics disabled. Snapshot text,
    // the optional progress line, and the incomplete marker share this writer.
    let output = ProgressOutput::stderr();
    let mut initial = format!("error: {message}\n");
    if observer.is_some() {
        initial.push_str(
            "[PARTIAL] timeout snapshot pending; collection and delivery are best-effort\n",
        );
    }
    output.emit(initial);
    let Some(observer) = observer else {
        return;
    };
    let (completed, receiver) = mpsc::channel();
    let gate = Arc::new(Mutex::new(SnapshotPhase::Open));
    let snapshot_output = Arc::clone(&output);
    let snapshot_gate = Arc::clone(&gate);
    // Collection and OS writes must not delay exit. Never join this worker:
    // process::exit reclaims a stalled snapshot. The gate is held only around
    // a non-blocking enqueue, never during collection or the OS write.
    let _ = std::thread::Builder::new()
        .name("no-mistakes-timeout-snapshot".to_string())
        .spawn(move || {
            if publish_timeout_snapshot(&observer, &snapshot_output, &snapshot_gate) {
                let _ = completed.send(());
            }
        });
    if receiver.recv_timeout(Duration::from_millis(75)).is_err() {
        abandon_timeout_snapshot(&gate, &output);
    }
}

fn publish_timeout_snapshot(
    observer: &InvocationObserver,
    output: &ProgressOutput,
    gate: &Mutex<SnapshotPhase>,
) -> bool {
    // Queue the partial snapshot before progress collection. Progress may block
    // on its own mutex; the exit path must still be able to see this chunk.
    let queued = claim_enqueue(
        gate,
        SnapshotPhase::Open,
        SnapshotPhase::Published,
        output,
        partial_snapshot(observer),
    );
    if !queued {
        return false;
    }
    match observer.render_progress_text() {
        Some(progress) => {
            let _ = claim_enqueue(
                gate,
                SnapshotPhase::Published,
                SnapshotPhase::Finished,
                output,
                progress,
            );
        }
        None => {
            let _ = claim_transition(gate, SnapshotPhase::Published, SnapshotPhase::Finished);
        }
    }
    true
}

fn abandon_timeout_snapshot(gate: &Mutex<SnapshotPhase>, output: &ProgressOutput) {
    // Winning Open means the snapshot was not queued, so there is no late line
    // for this marker to precede. Winning Published means the snapshot is
    // already queued and a later progress line must be skipped.
    if claim_enqueue(
        gate,
        SnapshotPhase::Open,
        SnapshotPhase::Abandoned,
        output,
        INCOMPLETE_LINE.to_string(),
    ) {
        return;
    }
    let _ = claim_enqueue(
        gate,
        SnapshotPhase::Published,
        SnapshotPhase::Abandoned,
        output,
        INCOMPLETE_LINE.to_string(),
    );
}

fn claim_enqueue(
    gate: &Mutex<SnapshotPhase>,
    from: SnapshotPhase,
    to: SnapshotPhase,
    output: &ProgressOutput,
    text: String,
) -> bool {
    let mut phase = gate
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if *phase != from {
        return false;
    }
    let Some(receiver) = output.try_enqueue(text) else {
        return false;
    };
    *phase = to;
    drop(phase);
    output.flush_enqueued(receiver);
    true
}

fn claim_transition(gate: &Mutex<SnapshotPhase>, from: SnapshotPhase, to: SnapshotPhase) -> bool {
    let mut phase = gate
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if *phase != from {
        return false;
    }
    *phase = to;
    true
}

fn partial_snapshot(observer: &InvocationObserver) -> String {
    let (snapshot, timings_available) = observer.partial_snapshot();
    let mut text = String::from(
        "[PARTIAL] timeout snapshot: published work counts are lower bounds; unfinished work and timings may be absent\n",
    );
    let (_, elapsed) = observer.snapshot_clock();
    if !timings_available {
        text.push_str("[PARTIAL] completed timings unavailable: timing ledger busy or poisoned\n");
    }
    let _ = writeln!(
        text,
        "[PARTIAL timing] elapsed: {:.3}ms",
        elapsed.as_secs_f64() * 1000.0
    );
    for timing in snapshot.timings {
        let suffix = if timing.kind == TimingKind::Parallel {
            " (parallel; non-additive)"
        } else {
            ""
        };
        let _ = writeln!(
            text,
            "[PARTIAL timing] {}: {:.3}ms{}",
            timing.label,
            timing.duration.as_secs_f64() * 1000.0,
            suffix
        );
    }
    for (metric, count) in snapshot.work {
        let _ = writeln!(text, "[PARTIAL work] {metric}: {count}");
    }
    text
}

#[cfg(test)]
mod tests;
