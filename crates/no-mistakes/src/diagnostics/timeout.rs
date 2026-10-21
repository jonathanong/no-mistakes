//! Hard exits cannot unwind the ordinary final diagnostics renderer.
use super::progress::output::ProgressOutput;
use super::{InvocationObserver, TimingKind};
use std::fmt::Write;
use std::sync::{mpsc, Arc};
use std::time::Duration;

pub fn render_timeout_stderr(observer: Option<Arc<InvocationObserver>>, message: String) {
    // Created only after expiry, even with diagnostics disabled. A fresh writer
    // avoids both Rust's stderr lock and an occupied progress-output queue.
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
    let snapshot_output = Arc::clone(&output);
    // Snapshot locks/aggregation, as well as OS writes, must not delay exit.
    // Never join this worker: process::exit will reclaim a stalled snapshot.
    let _ = std::thread::Builder::new()
        .name("no-mistakes-timeout-snapshot".to_string())
        .spawn(move || {
            snapshot_output.emit(partial_snapshot(&observer));
            observer.render_progress_stderr();
            let _ = completed.send(());
        });
    if receiver.recv_timeout(Duration::from_millis(75)).is_err() {
        output.emit("[PARTIAL] timeout diagnostics incomplete: snapshot collection or delivery did not finish\n".to_string());
    }
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
