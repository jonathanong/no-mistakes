use super::super::evaluate::module_cache::shared::SharedInit;
use super::File;
use crate::codebase::check_facts::sql_source_prerequisites::FailedSqlSources;
use crate::diagnostics::{InvocationObserver, TimingKind};
use crate::fx::{FxHashMap, FxHashSet};
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

mod candidates;
pub(in crate::codebase::postgres::query_annotation) mod executor_scan;

pub(super) enum SharedInitStatus {
    Enabled {
        shared: Arc<SharedInit>,
        candidates: usize,
        seed_root: Option<usize>,
    },
    Disabled {
        reason: &'static str,
    },
}

impl SharedInitStatus {
    pub(super) fn shared(&self) -> Option<Arc<SharedInit>> {
        match self {
            Self::Enabled { shared, .. } => Some(Arc::clone(shared)),
            Self::Disabled { .. } => None,
        }
    }

    pub(super) fn seed_root(&self) -> Option<usize> {
        match self {
            Self::Enabled { seed_root, .. } => *seed_root,
            Self::Disabled { .. } => None,
        }
    }
}

pub(super) fn build(
    files: &FxHashMap<PathBuf, File<'_>>,
    roots: &[PathBuf],
    resolve: &(impl Fn(&str, &Path) -> Option<PathBuf> + Sync),
    untrusted_array_intrinsics: bool,
    failed_sources: &FailedSqlSources,
    observer: Option<&InvocationObserver>,
) -> SharedInitStatus {
    if roots.len() < 2 {
        return SharedInitStatus::Disabled {
            reason: "fewer_than_two_roots",
        };
    }
    if !failed_sources.is_empty() {
        return SharedInitStatus::Disabled {
            reason: "failed_sql_sources",
        };
    }
    let Some(ids) = files
        .values()
        .find_map(|file| file.module.module_ids.as_ref())
        .map(Arc::clone)
    else {
        return SharedInitStatus::Disabled {
            reason: "no_shared_imports",
        };
    };
    let graph = candidates::discover(files, roots, &ids, resolve);
    if graph.order.is_empty() {
        return SharedInitStatus::Disabled {
            reason: "no_shared_imports",
        };
    }
    let total_candidates = graph.order.len();
    let run_build = || {
        let mut eligible = FxHashSet::default();
        let mut uncacheable = graph.cyclic;
        for id in &graph.order {
            if uncacheable.contains(id) {
                continue;
            }
            let target = ids.path(*id);
            let file = &files[target];
            if !executor_scan::is_statically_pure_top_level(file, target, resolve) {
                uncacheable.insert(*id);
                continue;
            }
            let deps_ok = graph
                .direct_deps
                .get(id)
                .is_none_or(|deps| deps.iter().all(|dep| eligible.contains(dep)));
            if deps_ok {
                eligible.insert(*id);
            } else {
                uncacheable.insert(*id);
            }
        }
        let seed_root = if eligible.is_empty() {
            None
        } else {
            graph
                .root_reachable
                .iter()
                .enumerate()
                .map(|(index, reachable)| {
                    (
                        index,
                        reachable.iter().filter(|id| eligible.contains(id)).count(),
                    )
                })
                .filter(|(_, count)| *count > 0)
                .max_by(|(i1, c1), (i2, c2)| c1.cmp(c2).then_with(|| i2.cmp(i1)))
                .map(|(index, _)| index)
        };
        let shared = Arc::new(SharedInit {
            untrusted_array_intrinsics,
            eligible,
            direct_deps: graph.direct_deps,
            entries: DashMap::new(),
        });
        (shared, seed_root)
    };
    let (shared, seed_root) = match observer {
        Some(obs) => obs.trace(
            "postgres.annotation.shared_init.build",
            TimingKind::Parallel,
            run_build,
        ),
        None => run_build(),
    };
    SharedInitStatus::Enabled {
        shared,
        candidates: total_candidates,
        seed_root,
    }
}

pub(super) fn record(
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

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
