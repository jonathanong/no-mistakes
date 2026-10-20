//! Live locations need not retain a completed timing for every module or binding.
use super::{ActiveSpanGuard, InvocationObserver, TimingKind};

impl InvocationObserver {
    pub(crate) fn activity<T>(
        &self,
        label: &str,
        kind: TimingKind,
        operation: impl FnOnce() -> T,
    ) -> T {
        if !self.progress_enabled() {
            return operation();
        }
        let _span = ActiveSpanGuard::start(self, label, kind, self.snapshot_clock().0);
        crate::diagnostics::with_timing_kind(kind, operation)
    }
}

#[cfg(test)]
mod tests;
