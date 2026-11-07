use super::{demand, File};
use crate::diagnostics::{InvocationObserver, TimingKind};
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

pub(super) fn roots(
    observer: &InvocationObserver,
    files: &FxHashMap<PathBuf, File<'_>>,
    after: &[&Path],
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) {
    if !observer.verbose() {
        return;
    }
    let before = observer.trace(
        "postgres.annotation.prepare.roots_before_scoping",
        TimingKind::Serial,
        || demand::roots_before_scoping(files, resolve),
    );
    observer.increment(
        "postgres.annotation.roots_before_scoping",
        before.len() as u64,
    );
    observer.increment(
        "postgres.annotation.roots_after_scoping",
        after.len() as u64,
    );
    observer.increment(
        "postgres.annotation.excluded_roots_retained",
        after
            .iter()
            .filter(|path| files[**path].facts.in_scope == Some(false))
            .count() as u64,
    );
    observer.increment(
        "postgres.annotation.excluded_roots_pruned",
        before.len().saturating_sub(after.len()) as u64,
    );
    observer.increment("postgres.annotation.shared_init.disabled", 1);
    observer.increment("postgres.annotation.shared_init.cache_hits", 0);
    observer.increment("postgres.annotation.shared_init.cache_misses", 0);
    observer.increment("postgres.annotation.shared_init.builds", 0);
    observer.record_duration(
        "postgres.annotation.shared_init.build",
        std::time::Duration::ZERO,
        TimingKind::Parallel,
    );
}
