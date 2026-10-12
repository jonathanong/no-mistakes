mod timing_order;
mod work;

use super::context::measure_optional;
use super::progress::ProgressState;
use dashmap::DashMap;
use rustc_hash::FxBuildHasher;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use work::WorkCounters;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimingKind {
    Serial,
    /// This phase may overlap sibling work and must not be added to enclosing
    /// wall-clock durations.
    Parallel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimingDiagnostic {
    pub label: String,
    pub duration: Duration,
    pub kind: TimingKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticsSnapshot {
    pub timings: Vec<TimingDiagnostic>,
    pub work: BTreeMap<&'static str, u64>,
}

pub struct InvocationObserver {
    verbose: bool,
    started: Instant,
    timings: Mutex<Vec<TimingDiagnostic>>,
    work: Option<Box<WorkCounters>>,
    source_reads: Option<DashMap<PathBuf, AtomicU64, FxBuildHasher>>,
    pub(super) progress: Option<Mutex<ProgressState>>,
}

impl InvocationObserver {
    pub fn new(verbose: bool) -> Arc<Self> {
        Self::with_progress(verbose, false)
    }

    pub fn with_progress(verbose: bool, progress: bool) -> Arc<Self> {
        Arc::new(Self {
            verbose,
            started: Instant::now(),
            timings: Mutex::new(Vec::new()),
            work: verbose.then(|| Box::new(WorkCounters::new())),
            source_reads: verbose.then(|| DashMap::with_hasher(FxBuildHasher)),
            progress: progress.then(|| Mutex::new(ProgressState::default())),
        })
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }

    pub fn progress_enabled(&self) -> bool {
        self.progress.is_some()
    }

    /// Fine-grained traces are needed for active progress without work ledgers.
    pub fn detailed(&self) -> bool {
        self.verbose || self.progress.is_some()
    }

    pub(super) fn snapshot_clock(&self) -> (Instant, Duration) {
        let now = Instant::now();
        (now, now.duration_since(self.started))
    }

    pub fn trace<T>(&self, label: &str, kind: TimingKind, operation: impl FnOnce() -> T) -> T {
        measure_optional(Some(self), label, kind, Instant::now, operation).0
    }

    pub fn measure<T>(
        &self,
        label: &str,
        kind: TimingKind,
        operation: impl FnOnce() -> T,
    ) -> (T, Duration) {
        measure_optional(Some(self), label, kind, Instant::now, operation)
    }

    pub fn record_duration(&self, label: &str, duration: Duration, kind: TimingKind) {
        self.timings
            .lock()
            .expect("diagnostics timing mutex must not be poisoned")
            .push(TimingDiagnostic {
                label: label.to_string(),
                duration,
                kind,
            });
    }

    pub fn increment(&self, metric: &'static str, amount: u64) {
        let Some(work) = &self.work else {
            return;
        };
        work.increment(metric, amount);
    }

    pub(crate) fn record_source_read(&self, path: &Path) {
        let Some(source_reads) = &self.source_reads else {
            return;
        };
        if let Some(counter) = source_reads.get(path) {
            counter.fetch_add(1, Ordering::Relaxed);
            return;
        }
        source_reads
            .entry(path.to_path_buf())
            .or_default()
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn source_read_snapshot(&self) -> BTreeMap<PathBuf, u64> {
        self.source_reads
            .as_ref()
            .map(|source_reads| {
                source_reads
                    .iter()
                    .map(|entry| (entry.key().clone(), entry.value().load(Ordering::Relaxed)))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn snapshot(&self) -> DiagnosticsSnapshot {
        let recorded = self
            .timings
            .lock()
            .expect("diagnostics timing mutex must not be poisoned")
            .clone();
        let mut grouped = BTreeMap::<(String, TimingKind), Duration>::new();
        for entry in recorded {
            *grouped.entry((entry.label, entry.kind)).or_default() += entry.duration;
        }
        let mut timings = grouped
            .into_iter()
            .map(|((label, kind), duration)| TimingDiagnostic {
                label,
                duration,
                kind,
            })
            .collect::<Vec<_>>();
        timings.sort_by(|a, b| {
            (timing_order(&a.label), a.label.as_str(), a.kind).cmp(&(
                timing_order(&b.label),
                b.label.as_str(),
                b.kind,
            ))
        });
        DiagnosticsSnapshot {
            timings,
            work: self
                .work
                .as_ref()
                .map(|work| work.snapshot())
                .unwrap_or_default(),
        }
    }

    /// Render a single deterministic diagnostics block. This is intentionally
    /// called by the CLI boundary after command dispatch, including failures.
    pub fn render_stderr(&self) {
        let mut snapshot = self.snapshot();
        snapshot.timings.push(TimingDiagnostic {
            label: "total".to_string(),
            duration: self.started.elapsed(),
            kind: TimingKind::Serial,
        });
        let mut output = self.progress_enabled().then(String::new);
        for entry in snapshot.timings {
            let suffix = match entry.kind {
                TimingKind::Serial => "",
                TimingKind::Parallel => " (parallel; non-additive)",
            };
            match &mut output {
                Some(output) => {
                    let _ = writeln!(
                        output,
                        "[timing] {}: {:.3}ms{}",
                        entry.label,
                        entry.duration.as_secs_f64() * 1000.0,
                        suffix
                    );
                }
                None => eprintln!(
                    "[timing] {}: {:.3}ms{}",
                    entry.label,
                    entry.duration.as_secs_f64() * 1000.0,
                    suffix
                ),
            }
        }
        if self.verbose {
            for (metric, count) in snapshot.work {
                match &mut output {
                    Some(output) => {
                        let _ = writeln!(output, "[work] {metric}: {count}");
                    }
                    None => eprintln!("[work] {metric}: {count}"),
                }
            }
        }
        if let Some(output) = output {
            self.emit_progress_output(output);
        }
    }
}

pub(super) fn timing_order(label: &str) -> u16 {
    timing_order::rank(label)
}
