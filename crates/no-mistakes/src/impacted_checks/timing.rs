use super::ImpactedChecksTiming;
use anyhow::Result;
use std::time::{Duration, Instant};

/// Adapter shared by the root CLI observer and N-API timing collection.
/// Planner phases keep their stable labels and exclusive-duration behavior;
/// only N-API requests populate the structured timing array.
pub(crate) struct TimingTracker {
    emit_progress: bool,
    collect: bool,
    observer: Option<std::sync::Arc<crate::diagnostics::InvocationObserver>>,
    invocation_started: Option<Instant>,
    completed_phase_time: Duration,
    timings: Vec<ImpactedChecksTiming>,
}

pub(crate) struct PhaseTimer {
    started: Option<Instant>,
    completed_phase_time: Duration,
}

impl TimingTracker {
    pub(crate) fn new(emit_progress: bool, collect: bool) -> Self {
        let observer = crate::diagnostics::current();
        let enabled = emit_progress || collect || observer.is_some();
        Self {
            emit_progress,
            collect,
            observer,
            invocation_started: enabled.then(Instant::now),
            completed_phase_time: Duration::ZERO,
            timings: Vec::new(),
        }
    }

    /// Run one phase and record its duration after completion.
    pub(crate) fn run_phase<T>(
        &mut self,
        phase: &'static str,
        operation: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        let started = self.start_phase(phase);
        let result = operation();
        self.finish_phase_result(phase, started, result)
    }

    /// Finish a phase whose operation had to receive this tracker for nested
    /// phase timing and therefore could not be passed directly to
    /// [`Self::run_phase`].
    pub(crate) fn finish_phase_result<T>(
        &mut self,
        phase: &'static str,
        started: PhaseTimer,
        result: Result<T>,
    ) -> Result<T> {
        match result {
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
        let _ = phase;
        PhaseTimer {
            started: self.invocation_started.map(|_| Instant::now()),
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
        if let Some(observer) = &self.observer {
            observer.record_duration(phase, duration, crate::diagnostics::TimingKind::Serial);
        } else if self.emit_progress {
            eprintln!("[timing] {phase}: {:.3}ms", duration.as_secs_f64() * 1000.0);
        }
    }

    pub(crate) fn into_timings(self) -> Option<Vec<ImpactedChecksTiming>> {
        self.collect.then_some(self.timings)
    }

    /// Finish the invocation-level timer after all nested phases complete.
    pub(crate) fn finish_total(&mut self) {
        let duration = self
            .invocation_started
            .map(|started| started.elapsed())
            .unwrap_or_default();
        if self.collect {
            self.timings.push(ImpactedChecksTiming {
                phase: "total".to_string(),
                duration_ms: duration.as_secs_f64() * 1000.0,
            });
        }
        if self.observer.is_none() && !self.collect {
            self.record_success("total", duration);
        }
    }

    /// Report an invocation that failed after its active phase reported the
    /// more specific failure.
    pub(crate) fn fail_total(&self) {
        if self.emit_progress && self.observer.is_none() {
            let duration = self
                .invocation_started
                .map(|started| started.elapsed())
                .unwrap_or_default();
            eprintln!("[timing] total: {:.3}ms", duration.as_secs_f64() * 1000.0);
        }
    }

    fn record_success(&mut self, phase: &'static str, duration: Duration) {
        if let Some(observer) = &self.observer {
            observer.record_duration(phase, duration, crate::diagnostics::TimingKind::Serial);
        } else if self.emit_progress {
            eprintln!("[timing] {phase}: {:.3}ms", duration.as_secs_f64() * 1000.0);
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
        timer
            .started
            .map(|started| started.elapsed().saturating_sub(nested))
            .unwrap_or_default()
    }
}
