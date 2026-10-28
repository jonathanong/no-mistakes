//! Opt-in locations for long-running annotation evaluation.
use crate::diagnostics::{InvocationObserver, TimingKind};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

/// Bracket lexical state saves/restores without moving their local bindings.
pub(super) fn timing(label: &'static str) -> Option<Timing> {
    start(crate::diagnostics::current(), label, Instant::now)
}

/// Count a timed walk through the same observer that owns its duration.
/// Inline the disabled path so this wrapper adds no call per reachability walk.
#[inline(always)]
pub(super) fn timing_and_count(label: &'static str, counter: &'static str) -> Option<Timing> {
    let timing = timing(label);
    if let Some(timing) = &timing {
        timing.observer.increment(counter, 1);
    }
    timing
}

pub(super) struct Timing {
    observer: Arc<InvocationObserver>,
    label: &'static str,
    started: Instant,
}

fn start(
    observer: Option<Arc<InvocationObserver>>,
    label: &'static str,
    now: impl FnOnce() -> Instant,
) -> Option<Timing> {
    let observer = observer.filter(|observer| observer.detailed())?;
    Some(Timing {
        observer,
        label,
        started: now(),
    })
}

impl Drop for Timing {
    fn drop(&mut self) {
        self.observer
            .record_duration(self.label, self.started.elapsed(), TimingKind::Parallel);
    }
}

pub(super) fn trace<T>(
    label: &'static str,
    path: &Path,
    offset: Option<u32>,
    operation: impl FnOnce() -> T,
) -> T {
    located(label, path, offset, operation)
}

fn located<T>(
    label: &'static str,
    path: &Path,
    location: Option<u32>,
    operation: impl FnOnce() -> T,
) -> T {
    let Some(observer) = crate::diagnostics::current().filter(|observer| observer.detailed())
    else {
        return operation();
    };
    let _timing = start(Some(observer.clone()), label, Instant::now);
    observer.increment(label, 1);
    let result = if observer.progress_enabled() {
        let location = match location {
            Some(offset) => format!("{label} {}:{offset}", path.display()),
            None => format!("{label} {}", path.display()),
        };
        observer.activity(
            &location,
            crate::diagnostics::TimingKind::Parallel,
            operation,
        )
    } else {
        operation()
    };
    observer.increment("postgres.annotation.completed_activities", 1);
    match label {
        "postgres.annotation.root" => observer.increment("postgres.annotation.roots_completed", 1),
        "postgres.annotation.module" => {
            observer.increment("postgres.annotation.module_initializations_completed", 1)
        }
        _ => {}
    }
    result
}

#[cfg(test)]
mod tests;
