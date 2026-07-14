use super::context::{measure_optional, with_timing_kind};
use std::collections::BTreeMap;
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
    work: Mutex<BTreeMap<&'static str, u64>>,
}

impl InvocationObserver {
    pub fn new(verbose: bool) -> Arc<Self> {
        Arc::new(Self {
            verbose,
            started: Instant::now(),
            timings: Mutex::new(Vec::new()),
            work: Mutex::new(BTreeMap::new()),
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
        if !self.verbose {
            return;
        }
        *self
            .work
            .lock()
            .expect("diagnostics work mutex must not be poisoned")
            .entry(metric)
            .or_default() += amount;
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
                .lock()
                .expect("diagnostics work mutex must not be poisoned")
                .clone(),
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

fn timing_order(label: &str) -> u16 {
    match label {
        "discovery" => 10,
        "read" => 20,
        "parse" => 30,
        "manifest" => 40,
        "resolve" => 50,
        "search" => 60,
        "ingest" => 61,
        "parse+analysis" => 62,
        "analysis" => 63,
        "prepare" => 100,
        "discover.dotnet" => 110,
        "discover.vitest" => 111,
        "discover.playwright" => 112,
        "discover.swift" => 113,
        "graph" => 120,
        "select.dotnet" => 130,
        "select.vitest" => 131,
        "select.playwright" => 132,
        "select.swift" => 133,
        "generic-checks" => 140,
        "analysis.react" => 200,
        "analysis.queues" => 201,
        "analysis.rules" => 202,
        "analysis.integration" => 203,
        "analysis.codebase" => 204,
        "analysis.filesystem_rules" => 205,
        "output" => 900,
        label if label.starts_with("discovery.") => 11,
        label if label.starts_with("read.") => 21,
        label if label.starts_with("parse.") => 31,
        label if label.starts_with("manifest.") => 41,
        label if label.starts_with("resolve.") => 51,
        label if label.starts_with("graph.") => 301,
        label if label.starts_with("traversal.") => 400,
        label if label.starts_with("analysis.") => 500,
        label if label.starts_with("rules.") => 600,
        label if label.starts_with("playwright.") => 700,
        _ => 800,
    }
}
