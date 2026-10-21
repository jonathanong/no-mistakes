use crate::diagnostics::{InvocationObserver, TimingKind};

pub(super) fn trace_rule<T>(
    observer: Option<&InvocationObserver>,
    label: impl FnOnce() -> String,
    run: impl FnOnce() -> T,
) -> T {
    match observer.filter(|observer| observer.detailed()) {
        Some(observer) => observer.trace(&label(), TimingKind::Parallel, run),
        None => run(),
    }
}

#[cfg(test)]
#[path = "timing/tests.rs"]
mod tests;
