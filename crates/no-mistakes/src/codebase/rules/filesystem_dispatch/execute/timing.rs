use crate::diagnostics::{InvocationObserver, TimingKind};

pub(super) fn trace_rule<T>(
    observer: Option<&InvocationObserver>,
    label: impl FnOnce() -> String,
    run: impl FnOnce() -> T,
) -> T {
    match observer {
        Some(observer) => observer.trace(&label(), TimingKind::Parallel, run),
        None => run(),
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "timing/tests.rs"]
mod tests;
