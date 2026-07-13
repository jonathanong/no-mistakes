use super::ImpactedChecksTiming;
use anyhow::Result;
use std::time::{Duration, Instant};

/// Per-invocation timing state shared by the CLI and N-API entry points.
///
/// Planner phases call [`Self::run_phase`] with their stable phase label. The
/// CLI enables `emit_progress`, while the N-API enables `collect`; keeping the
/// two controls separate prevents diagnostics from leaking into either CLI
/// stdout or a Node process's stderr.
pub(crate) struct TimingTracker {
    emit_progress: bool,
    collect: bool,
    invocation_started: Instant,
    completed_phase_time: Duration,
    timings: Vec<ImpactedChecksTiming>,
}

pub(crate) struct PhaseTimer {
    started: Instant,
    completed_phase_time: Duration,
}

impl TimingTracker {
    pub(crate) fn new(emit_progress: bool, collect: bool) -> Self {
        if emit_progress {
            eprintln!("total: started");
        }
        Self {
            emit_progress,
            collect,
            invocation_started: Instant::now(),
            completed_phase_time: Duration::ZERO,
            timings: Vec::new(),
        }
    }

    /// Run one phase, emitting its start before expensive work and recording
    /// its duration only after successful completion.
    pub(crate) fn run_phase<T>(
        &mut self,
        phase: &'static str,
        operation: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        let started = self.start_phase(phase);
        match operation() {
            Ok(value) => {
                self.finish_phase(phase, started);
                Ok(value)
            }
            Err(error) => {
                self.fail_phase(phase, started);
                Err(error)
            }
        }
    }

    pub(crate) fn start_phase(&self, phase: &'static str) -> PhaseTimer {
        if self.emit_progress {
            eprintln!("{phase}: started");
        }
        PhaseTimer {
            started: Instant::now(),
            completed_phase_time: self.completed_phase_time,
        }
    }

    pub(crate) fn finish_phase(&mut self, phase: &'static str, timer: PhaseTimer) {
        let duration = self.exclusive_duration(&timer);
        self.completed_phase_time += duration;
        self.record_success(phase, duration);
    }

    pub(crate) fn fail_phase(&mut self, phase: &'static str, timer: PhaseTimer) {
        let duration = self.exclusive_duration(&timer);
        self.completed_phase_time += duration;
        if self.emit_progress {
            eprintln!(
                "{phase}: failed after {:.3}ms",
                duration.as_secs_f64() * 1000.0
            );
        }
    }

    pub(crate) fn into_timings(self) -> Option<Vec<ImpactedChecksTiming>> {
        self.collect.then_some(self.timings)
    }

    /// Finish the invocation-level timer after all nested phases complete.
    pub(crate) fn finish_total(&mut self) {
        self.record_success("total", self.invocation_started.elapsed());
    }

    /// Report an invocation that failed after its active phase reported the
    /// more specific failure.
    pub(crate) fn fail_total(&self) {
        if self.emit_progress {
            eprintln!(
                "total: failed after {:.3}ms",
                self.invocation_started.elapsed().as_secs_f64() * 1000.0
            );
        }
    }

    fn record_success(&mut self, phase: &'static str, duration: Duration) {
        if self.emit_progress {
            eprintln!("{phase}: {:.3}ms", duration.as_secs_f64() * 1000.0);
        }
        if self.collect {
            self.timings.push(ImpactedChecksTiming {
                phase: phase.to_string(),
                duration_ms: duration.as_secs_f64() * 1000.0,
            });
        }
    }

    fn exclusive_duration(&self, timer: &PhaseTimer) -> Duration {
        let nested = self
            .completed_phase_time
            .saturating_sub(timer.completed_phase_time);
        timer.started.elapsed().saturating_sub(nested)
    }
}
