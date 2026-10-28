//! Invocation-scoped performance diagnostics.
//!
//! An observer is only constructed for an explicitly instrumented invocation.
//! Callers must keep the disabled state as `None`, so adding instrumentation to
//! a hot path does not even read the clock during an ordinary invocation.

pub(crate) mod context;
mod memory;
mod observer;
mod progress;
mod request;
mod timeout;

use std::sync::Arc;

pub use context::{
    current, current_timing_kind, measure_if_enabled, with_observer, with_timing_kind,
    InvocationGuard, LegacyDiagnosticsGuard,
};
pub use observer::{DiagnosticsSnapshot, InvocationObserver, TimingDiagnostic, TimingKind};
pub use progress::{ActiveSpanSnapshot, CompletedSpanSnapshot, ProgressGuard, ProgressSnapshot};
pub use request::{progress_for_request, DiagnosticRequestGuard};
// The native invocation harness compiles watchdog source across this crate boundary.
#[doc(hidden)]
pub use timeout::render_timeout_stderr;

/// Count one AST visitor walk when verbose timings are enabled.
pub fn record_ast_walk() {
    if let Some(observer) = current() {
        observer.increment("ast.walks", 1);
    }
}

#[derive(clap::Args, Debug, Clone, Copy)]
pub struct DiagnosticsArgs {
    /// Print invocation phase timings to stderr.
    #[arg(long, global = true)]
    timings: bool,
    /// Print fine-grained timings and deterministic work counters to stderr.
    /// This implies `--timings`.
    #[arg(long, global = true)]
    verbose_timings: bool,
    /// Flush active phase snapshots to stderr periodically. Implies `--timings`.
    #[arg(long, global = true)]
    progress: bool,
    /// Active phase snapshot interval in milliseconds. Requires `--progress`.
    #[arg(long, global = true, value_name = "MILLISECONDS", default_value_t = 1000, value_parser = clap::value_parser!(u64).range(1..), requires = "progress")]
    progress_interval: u64,
}

impl DiagnosticsArgs {
    pub fn observer(self) -> Option<Arc<InvocationObserver>> {
        (self.timings || self.verbose_timings || self.progress)
            .then(|| InvocationObserver::with_progress(self.verbose_timings, self.progress))
    }
    pub fn progress_interval(self) -> Option<std::time::Duration> {
        self.progress
            .then(|| std::time::Duration::from_millis(self.progress_interval))
    }
}

impl Default for DiagnosticsArgs {
    fn default() -> Self {
        Self {
            timings: false,
            verbose_timings: false,
            progress: false,
            progress_interval: 1000,
        }
    }
}

#[cfg(test)]
mod tests;

/// Preserve ordinary stderr messages while CLI progress uses bounded output.
pub fn print_stderr(message: std::fmt::Arguments<'_>) {
    if !current().is_some_and(|observer| {
        observer.progress_enabled() && observer.emit_progress_output(format!("{message}\n"))
    }) {
        eprintln!("{message}");
    }
}
