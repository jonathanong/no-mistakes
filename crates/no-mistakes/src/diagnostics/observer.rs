mod timing_order;

use super::context::{measure_optional, with_timing_kind};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
    work: Option<Mutex<BTreeMap<&'static str, u64>>>,
    source_reads: Option<Mutex<BTreeMap<PathBuf, u64>>>,
}

impl InvocationObserver {
    pub fn new(verbose: bool) -> Arc<Self> {
        Arc::new(Self {
            verbose,
            started: Instant::now(),
            timings: Mutex::new(Vec::new()),
            work: verbose.then(|| Mutex::new(BTreeMap::new())),
            source_reads: verbose.then(|| Mutex::new(BTreeMap::new())),
        })
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }

    pub fn trace<T>(&self, label: &str, kind: TimingKind, operation: impl FnOnce() -> T) -> T {
        let started = Instant::now();
        let result = with_timing_kind(kind, operation);
        self.record_duration(label, started.elapsed(), kind);
        result
    }

    pub fn measure<T>(
        &self,
        label: &str,
        kind: TimingKind,
        operation: impl FnOnce() -> T,
    ) -> (T, Duration) {
        let (result, duration) = measure_optional(Some(self), Instant::now, || {
            with_timing_kind(kind, operation)
        });
        self.record_duration(label, duration, kind);
        (result, duration)
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
        *work
            .lock()
            .expect("diagnostics work mutex must not be poisoned")
            .entry(metric)
            .or_default() += amount;
    }

    pub(crate) fn record_source_read(&self, path: &Path) {
        let Some(source_reads) = &self.source_reads else {
            return;
        };
        *source_reads
            .lock()
            .expect("diagnostics source-read mutex must not be poisoned")
            .entry(path.to_path_buf())
            .or_default() += 1;
    }

    pub(crate) fn source_read_snapshot(&self) -> BTreeMap<PathBuf, u64> {
        self.source_reads
            .as_ref()
            .map(|source_reads| {
                source_reads
                    .lock()
                    .expect("diagnostics source-read mutex must not be poisoned")
                    .clone()
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
                .map(|work| {
                    work.lock()
                        .expect("diagnostics work mutex must not be poisoned")
                        .clone()
                })
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
        for entry in snapshot.timings {
            let suffix = match entry.kind {
                TimingKind::Serial => "",
                TimingKind::Parallel => " (parallel; non-additive)",
            };
            eprintln!(
                "[timing] {}: {:.3}ms{}",
                entry.label,
                entry.duration.as_secs_f64() * 1000.0,
                suffix
            );
        }
        if self.verbose {
            for (metric, count) in snapshot.work {
                eprintln!("[work] {metric}: {count}");
            }
        }
    }
}

pub(super) fn timing_order(label: &str) -> u16 {
    timing_order::rank(label)
}
