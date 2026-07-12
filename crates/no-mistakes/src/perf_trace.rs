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
//! `enabled()` is a single process-wide flag, set once (if at all) by the
//! CLI entrypoint before any work starts. It is never set by library/N-API
//! callers, so it has no effect on programmatic usage. This module is `pub`
//! only because `check.rs` is compiled as part of the separate `main.rs`
//! binary crate root and needs a real cross-crate reference to reach it —
//! it is not a stable public API and has no N-API binding.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

static ENABLED: AtomicBool = AtomicBool::new(false);

pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Print `label: <duration>` to stderr if verbose timing is enabled; a no-op
/// otherwise. Duration formatting matches `check.rs`'s existing `--timings`
/// output so both can be read the same way.
pub fn record(label: &str, elapsed: Duration) {
    if enabled() {
        eprintln!("[timing] {label}: {:.3}ms", elapsed.as_secs_f64() * 1000.0);
    }
}

/// Time `f` and report it under `label` via [`record`] regardless of whether
/// tracing is enabled (the `Instant::now()` call is cheap enough not to
/// bother skipping). Returns `f`'s result unchanged.
pub fn trace<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = f();
    record(label, start.elapsed());
    result
}

#[cfg(test)]
mod tests;
