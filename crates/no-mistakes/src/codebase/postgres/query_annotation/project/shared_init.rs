use super::super::evaluate::module_cache::shared::{CachedModule, SharedInit};
use super::{evaluator, File};
use crate::codebase::check_facts::sql_source_prerequisites::FailedSqlSources;
use crate::diagnostics::{InvocationObserver, TimingKind};
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

mod candidates;
mod executor_scan;

pub(super) enum SharedInitStatus {
    Enabled {
        shared: Arc<SharedInit>,
        candidates: usize,
        cached: usize,
        misses: usize,
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
        let mut shared = Arc::new(SharedInit::default());
        let mut uncacheable = graph.cyclic;
        for id in graph.order {
            if uncacheable.contains(&id) {
                continue;
            }
            let target = ids.path(id);
            let Some(file) = files.get(target) else {
                uncacheable.insert(id);
                continue;
            };
            if executor_scan::has_uncacheable_top_level(file) {
                uncacheable.insert(id);
                continue;
            }
            let deps = graph.direct_deps.get(&id).cloned().unwrap_or_default();
            if deps.iter().any(|dep| !shared.modules.contains_key(dep)) {
                uncacheable.insert(id);
                continue;
            }
            let mut eval = evaluator::new(files, resolve, untrusted_array_intrinsics);
            eval.modules.shared = Some(Arc::clone(&shared));
            let mut dep_refs = Vec::new();
            for &dep in &deps {
                let dep_path = ids.path(dep);
                let dep_cached = Arc::clone(&shared.modules[&dep]);
                eval.install_cached_module(dep_path, dep, &dep_cached, Some(&mut dep_refs));
            }
            let prefix = eval.scopes.len();
            let dep_module_count = eval.modules.len();
            let prefix_scopes = eval.scopes.clone();
            eval.module_environment(target);
            eval.prune_snapshot_state();
            if eval.is_pure_shared_module(&prefix_scopes, dep_module_count) {
                let own_scopes = eval.scopes.split_off(prefix);
                drop(eval);
                Arc::make_mut(&mut shared).modules.insert(
                    id,
                    Arc::new(CachedModule {
                        deps,
                        dep_refs,
                        own_scopes,
                    }),
                );
            } else {
                uncacheable.insert(id);
            }
        }
        shared
    };
    let shared = match observer {
        Some(obs) => obs.trace(
            "postgres.annotation.shared_init.build",
            TimingKind::Parallel,
            run_build,
        ),
        None => run_build(),
    };
    let cached = shared.modules.len();
    let misses = total_candidates.saturating_sub(cached);
    SharedInitStatus::Enabled {
        shared,
        candidates: total_candidates,
        cached,
        misses,
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
            candidates,
            cached,
            misses,
            ..
        } => {
            observer.increment("postgres.annotation.shared_init.disabled", 0);
            observer.increment("postgres.annotation.shared_init.cache_hits", 0);
            observer.increment(
                "postgres.annotation.shared_init.cache_misses",
                *misses as u64,
            );
            observer.increment("postgres.annotation.shared_init.builds", *cached as u64);
            observer.record_note(format!(
                "postgres.annotation.shared_init.profile[{profile_index}]: enabled (candidates={candidates}, cached={cached}, roots={roots_len})"
            ));
        }
    }
}
