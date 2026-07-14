//! Invocation-scoped performance diagnostics.
//!
//! An observer is only constructed for an explicitly instrumented invocation.
//! Callers must keep the disabled state as `None`, so adding instrumentation to
//! a hot path does not even read the clock during an ordinary invocation.

mod context;
mod observer;

use std::sync::Arc;

pub use context::{
    current, current_timing_kind, measure_if_enabled, with_observer, InvocationGuard,
    LegacyDiagnosticsGuard,
};
pub use observer::{DiagnosticsSnapshot, InvocationObserver, TimingDiagnostic, TimingKind};

#[derive(clap::Args, Debug, Clone, Copy, Default)]
pub struct DiagnosticsArgs {
    /// Print invocation phase timings to stderr.
    #[arg(long, global = true)]
    timings: bool,
    /// Print fine-grained timings and deterministic work counters to stderr.
    /// This implies `--timings`.
    #[arg(long, global = true)]
    verbose_timings: bool,
}

impl DiagnosticsArgs {
    pub fn observer(self) -> Option<Arc<InvocationObserver>> {
        (self.timings || self.verbose_timings)
            .then(|| InvocationObserver::new(self.verbose_timings))
    }
}

#[cfg(test)]
mod tests;
