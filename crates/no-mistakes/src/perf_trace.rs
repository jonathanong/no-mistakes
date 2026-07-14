//! Opt-in, fine-grained timing diagnostics for internal hot paths.
//!
//! `--timings` (see `check.rs`) already reports the coarse, top-level phase
//! breakdown (discover/parse_extract/react/queues/rules/...). This module
//! backs a second, deeper level of detail — e.g. which sub-check inside
//! `rules` dominates, or which `DepGraph` edge kind is expensive — without
//! needing a special instrumented build. It exists because diagnosing every
//! performance regression in this codebase has so far required hand-editing
//! `eprintln!` calls into hot paths, rebuilding, and reverting before commit;
//! this makes that available permanently behind a flag.
//!
//! The root CLI installs one invocation observer. Library and N-API callers do
//! not install that compatibility bridge, so this has no effect on ordinary
//! programmatic usage. New code should carry the observer in `AnalysisSession`;
//! this module remains as an adapter for existing deep trace call sites.

use std::time::Duration;

pub fn enabled() -> bool {
    crate::diagnostics::current().is_some_and(|observer| observer.verbose())
}

/// Record a duration if verbose timing is enabled; a no-op otherwise. The CLI
/// boundary later renders all records in deterministic order.
pub fn record(label: &str, elapsed: Duration) {
    if let Some(observer) = crate::diagnostics::current() {
        if observer.verbose() {
            observer.record_duration(label, elapsed, crate::diagnostics::current_timing_kind());
        }
    }
}

/// Time `f` and report it under `label` via [`record`] when tracing is
/// enabled; otherwise just calls `f`, skipping the `Instant::now()` calls so
/// a `trace`-wrapped call added inside a future hot inner loop stays free
/// when the flag is off. Returns `f`'s result unchanged.
pub fn trace<T>(label: &str, f: impl FnOnce() -> T) -> T {
    match crate::diagnostics::current() {
        Some(observer) if observer.verbose() => {
            observer.trace(label, crate::diagnostics::current_timing_kind(), f)
        }
        _ => f(),
    }
}

#[cfg(test)]
mod tests;
