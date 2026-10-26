mod activity;
pub(super) mod output;
mod spans;
use super::{InvocationObserver, TimingDiagnostic, TimingKind};
use output::ProgressOutput;
use serde::Serialize;
pub(super) use spans::ActiveSpanGuard;
use std::collections::BTreeMap;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveSpanSnapshot {
    pub id: u64,
    pub phase: Vec<String>,
    pub started_ms: f64,
    pub elapsed_ms: f64,
    pub parallel: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletedSpanSnapshot {
    pub label: String,
    pub duration_ms: f64,
    pub parallel: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressSnapshot {
    pub elapsed_ms: f64,
    pub started_spans: u64,
    pub completed_spans: u64,
    pub active: Vec<ActiveSpanSnapshot>,
    pub completed: Vec<CompletedSpanSnapshot>,
}

struct ActiveSpan {
    phase: Vec<String>,
    started: Instant,
    parallel: bool,
}

#[derive(Default)]
pub(super) struct ProgressState {
    started: u64,
    completed: u64,
    active: BTreeMap<u64, ActiveSpan>,
    output: Option<Arc<ProgressOutput>>,
}

impl InvocationObserver {
    pub fn progress_snapshot(&self) -> Option<ProgressSnapshot> {
        self.progress_snapshot_with_timings(|| self.snapshot().timings)
    }

    fn progress_snapshot_with_timings(
        &self,
        timings: impl FnOnce() -> Vec<TimingDiagnostic>,
    ) -> Option<ProgressSnapshot> {
        let mut snapshot = {
            let progress = self.progress.as_ref()?;
            let state = progress
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let (now, elapsed) = self.snapshot_clock();
            ProgressSnapshot {
                elapsed_ms: elapsed.as_secs_f64() * 1000.0,
                completed: Vec::new(),
                started_spans: state.started,
                completed_spans: state.completed,
                active: state
                    .active
                    .iter()
                    .map(|(id, span)| ActiveSpanSnapshot {
                        id: *id,
                        phase: span.phase.clone(),
                        started_ms: elapsed
                            .saturating_sub(now.duration_since(span.started))
                            .as_secs_f64()
                            * 1000.0,
                        elapsed_ms: now.duration_since(span.started).as_secs_f64() * 1000.0,
                        parallel: span.parallel,
                    })
                    .collect(),
            }
        };
        snapshot.completed = timings()
            .into_iter()
            .map(|timing| CompletedSpanSnapshot {
                label: timing.label,
                duration_ms: timing.duration.as_secs_f64() * 1000.0,
                parallel: timing.kind == TimingKind::Parallel,
            })
            .collect();
        Some(snapshot)
    }

    /// Returns whether an installed CLI progress writer handles this message.
    /// This does not report queue acceptance or successful delivery: a full or
    /// disconnected writer remains handled. Deadline and warning callers must
    /// not fall back to potentially blocking stderr when diagnostics are dropped.
    pub fn emit_progress_output(&self, text: String) -> bool {
        if let Some(output) = self.progress_output() {
            output.emit(text);
            true
        } else {
            false
        }
    }

    /// Called on ordinary listener threads, never from a raw signal handler.
    pub fn render_progress_stderr(&self) {
        if let Some(line) = self.render_progress_text() {
            self.emit_progress_output(line);
        }
    }

    #[doc(hidden)]
    pub fn render_progress_signal_stderr(&self) {
        if let (Some(line), Some(output)) = (self.render_progress_text(), self.progress_output()) {
            output.emit_signal(line);
        }
    }

    fn progress_output(&self) -> Option<Arc<ProgressOutput>> {
        self.progress.as_ref().and_then(|progress| {
            progress
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .output
                .clone()
        })
    }

    /// One `[progress]` line, without touching the heartbeat stderr writer.
    /// The timeout path emits this on its own writer so the two cannot interleave.
    pub(in crate::diagnostics) fn render_progress_text(&self) -> Option<String> {
        let snapshot = self.progress_snapshot()?;
        let mut line = None;
        render_snapshot(&snapshot, |text| line = Some(text));
        line
    }
}

fn render_snapshot(snapshot: &impl Serialize, emit: impl FnOnce(String)) {
    if let Ok(json) = serde_json::to_string(snapshot) {
        emit(format!("[progress] {json}\n"));
    }
}

/// A bounded heartbeat. Dropping it wakes and joins the worker immediately,
/// so neither fast invocations nor cleanup wait for the configured interval.
pub struct ProgressGuard {
    stop: mpsc::Sender<()>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl ProgressGuard {
    pub fn install(observer: Arc<InvocationObserver>, interval: Duration) -> Self {
        if let Some(progress) = &observer.progress {
            progress
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .output = Some(ProgressOutput::stderr());
        }
        Self::start(interval, move || observer.render_progress_stderr())
    }

    fn start(interval: Duration, snapshot: impl Fn() + Send + 'static) -> Self {
        let (stop, receiver) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            while let Err(mpsc::RecvTimeoutError::Timeout) = receiver.recv_timeout(interval) {
                snapshot();
            }
        });
        Self {
            stop,
            worker: Some(worker),
        }
    }
}

impl Drop for ProgressGuard {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        let worker = self.worker.take().expect("heartbeat guard owns its worker");
        let _ = worker.join();
    }
}

#[cfg(test)]
mod tests;
