mod output;
use super::{InvocationObserver, TimingKind};
use output::ProgressOutput;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

thread_local! {
    // The observer identity prevents a nested unrelated request inheriting a phase.
    static SPANS: RefCell<Vec<(usize, u64, String)>> = const { RefCell::new(Vec::new()) };
}

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

pub(super) struct ActiveSpanGuard<'a> {
    observer: &'a InvocationObserver,
    id: Option<u64>,
}

impl<'a> ActiveSpanGuard<'a> {
    pub(super) fn start(
        observer: &'a InvocationObserver,
        label: &str,
        kind: TimingKind,
        started: Instant,
    ) -> Self {
        let id = observer.progress.as_ref().map(|progress| {
            let identity = std::ptr::from_ref(observer) as usize;
            let mut state = progress
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.started += 1;
            let id = state.started;
            let mut phase = SPANS.with(|spans| {
                let mut spans = spans.borrow_mut();
                let mut phase = spans
                    .iter()
                    .filter(|(owner, _, _)| *owner == identity)
                    .map(|(_, _, label)| label.clone())
                    .collect::<Vec<_>>();
                phase.push(label.to_string());
                spans.push((identity, id, label.to_string()));
                phase
            });
            // Rayon tasks carry the request observer, not another thread's TLS
            // stack. The explicit dispatch span remains their common ancestor.
            if phase
                .first()
                .is_none_or(|label| label != "invocation.dispatch")
                && state
                    .active
                    .values()
                    .any(|span| span.phase == ["invocation.dispatch"])
            {
                phase.insert(0, "invocation.dispatch".to_string());
            }
            state.active.insert(
                id,
                ActiveSpan {
                    phase,
                    started,
                    parallel: kind == TimingKind::Parallel,
                },
            );
            id
        });
        Self { observer, id }
    }
}

impl Drop for ActiveSpanGuard<'_> {
    fn drop(&mut self) {
        let Some(id) = self.id else {
            return;
        };
        let mut state = self
            .observer
            .progress
            .as_ref()
            .unwrap()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.active.remove(&id);
        state.completed += 1;
        SPANS.with(|spans| {
            spans.borrow_mut().pop();
        });
    }
}

impl InvocationObserver {
    pub fn progress_snapshot(&self) -> Option<ProgressSnapshot> {
        let progress = self.progress.as_ref()?;
        let state = progress
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = Instant::now();
        let elapsed = self.elapsed();
        Some(ProgressSnapshot {
            elapsed_ms: elapsed.as_secs_f64() * 1000.0,
            completed: self
                .snapshot()
                .timings
                .into_iter()
                .map(|timing| CompletedSpanSnapshot {
                    label: timing.label,
                    duration_ms: timing.duration.as_secs_f64() * 1000.0,
                    parallel: timing.kind == TimingKind::Parallel,
                })
                .collect(),
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
        })
    }

    pub fn emit_progress_output(&self, text: String) -> bool {
        let output = self.progress.as_ref().and_then(|progress| {
            progress
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .output
                .clone()
        });
        if let Some(output) = output {
            output.emit(text);
            true
        } else {
            false
        }
    }

    /// Called on ordinary listener threads, never from a raw signal handler.
    pub fn render_progress_stderr(&self) {
        let Some(snapshot) = self.progress_snapshot() else {
            return;
        };
        if let Ok(json) = serde_json::to_string(&snapshot) {
            self.emit_progress_output(format!("[progress] {json}\n"));
        }
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
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests;
