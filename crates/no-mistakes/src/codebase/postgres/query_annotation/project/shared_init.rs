use super::super::evaluate::module_cache::shared::SharedInit;
use super::super::module_ids::ModuleId;
use super::File;
use crate::codebase::check_facts::sql_source_prerequisites::FailedSqlSources;
use crate::diagnostics::{InvocationObserver, TimingKind};
use crate::fx::{FxHashMap, FxHashSet};
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod candidates;
mod diagnostics;
mod eager_deps;
pub(in crate::codebase::postgres::query_annotation) mod executor_scan;

pub(super) use diagnostics::record;

pub(super) enum SharedInitStatus {
    Enabled {
        shared: Arc<SharedInit>,
        candidates: usize,
        root_reachable: FxHashMap<PathBuf, Vec<ModuleId>>,
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

    pub(super) fn next_seed_root(&self, remaining: &[PathBuf]) -> Option<usize> {
        let Self::Enabled {
            shared,
            root_reachable,
            ..
        } = self
        else {
            return None;
        };
        if shared.entries.len() >= shared.eligible.len() {
            return None;
        }
        remaining
            .iter()
            .enumerate()
            .map(|(index, path)| {
                let count = root_reachable.get(path).map_or(0, |ids| {
                    ids.iter()
                        .filter(|id| !shared.entries.contains_key(id))
                        .count()
                });
                (index, count)
            })
            .filter(|(_, count)| *count > 0)
            .max_by(|(i1, c1), (i2, c2)| c1.cmp(c2).then_with(|| i2.cmp(i1)))
            .map(|(index, _)| index)
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
        let root_reachable = if eligible.is_empty() {
            FxHashMap::default()
        } else {
            roots
                .iter()
                .zip(graph.root_reachable)
                .filter_map(|(path, reachable)| {
                    let covered: Vec<ModuleId> = reachable
                        .into_iter()
                        .filter(|id| eligible.contains(id))
                        .collect();
                    (!covered.is_empty()).then(|| (path.clone(), covered))
                })
                .collect()
        };
        let direct_deps = eligible
            .iter()
            .map(|&id| {
                let direct = graph.direct_deps.get(&id).map_or(&[][..], Vec::as_slice);
                (id, eager_deps::for_module(files, &ids, id, direct, resolve))
            })
            .collect();
        let shared = Arc::new(SharedInit {
            untrusted_array_intrinsics,
            eligible,
            direct_deps,
            entries: DashMap::new(),
        });
        (shared, root_reachable)
    };
    let (shared, root_reachable) = match observer {
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
        root_reachable,
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod seed_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
