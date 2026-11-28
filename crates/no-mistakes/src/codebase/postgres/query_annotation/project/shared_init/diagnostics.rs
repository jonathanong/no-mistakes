use super::SharedInitStatus;
use crate::diagnostics::{InvocationObserver, TimingKind};
use std::time::Duration;

pub(in super::super) fn record(
    observer: &InvocationObserver,
    profile_index: usize,
    roots_len: usize,
    status: &SharedInitStatus,
) {
    if !observer.verbose() {
        return;
    }
    match status {
        SharedInitStatus::Disabled { reason } => {
            observer.increment("postgres.annotation.shared_init.disabled", 1);
            observer.increment("postgres.annotation.shared_init.cache_hits", 0);
            observer.increment("postgres.annotation.shared_init.cache_misses", 0);
            observer.increment("postgres.annotation.shared_init.builds", 0);
            observer.record_duration(
                "postgres.annotation.shared_init.build",
                Duration::ZERO,
                TimingKind::Parallel,
            );
            observer.record_note(format!(
                "postgres.annotation.shared_init.profile[{profile_index}]: disabled (reason={reason}, roots={roots_len})"
            ));
        }
        SharedInitStatus::Enabled {
            shared, candidates, ..
        } => {
            let cached = shared.cached_count();
            let misses = candidates.saturating_sub(cached);
            observer.increment("postgres.annotation.shared_init.disabled", 0);
            observer.increment("postgres.annotation.shared_init.cache_hits", 0);
            observer.increment(
                "postgres.annotation.shared_init.cache_misses",
                misses as u64,
            );
            observer.increment("postgres.annotation.shared_init.builds", cached as u64);
            observer.record_note(format!(
                "postgres.annotation.shared_init.profile[{profile_index}]: enabled (candidates={candidates}, cached={cached}, roots={roots_len})"
            ));
        }
    }
}
