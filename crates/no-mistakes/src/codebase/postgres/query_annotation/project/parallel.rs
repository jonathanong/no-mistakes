use super::super::evaluate::{
    module_cache::{shared::SharedInit, source_dependencies::SourceDependencies},
    ExecutionEvents, File,
};
use super::{evaluator, merge, prepare::Profile, shared_init};
use crate::codebase::check_facts::sql_source_prerequisites::FailedSqlSources;
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::diagnostics::InvocationObserver;
use crate::fx::FxHashMap;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) struct ProfileProjection {
    pub options: EmbeddedSqlOptions,
    pub projection: merge::Projection,
}

pub(super) fn evaluate(
    profiles: Vec<Profile<'_>>,
    resolve: &(impl Fn(&str, &Path) -> Option<PathBuf> + Sync),
    collect_executed: bool,
) -> Vec<ProfileProjection> {
    let observer = crate::diagnostics::current();
    if let Some(observer) = &observer {
        observer.increment(
            "postgres.annotation.parallel_passes",
            u64::from(profiles.iter().any(|profile| !profile.roots.is_empty())),
        );
    }
    // Nested iterators use this same Rayon pool: roots remain stealable across
    // profiles without keeping every profile's state until the pass finishes.
    profiles
        .into_par_iter()
        .enumerate()
        .map(|(profile_index, profile)| {
            crate::diagnostics::with_observer(observer.clone(), || {
                let Profile {
                    options,
                    files,
                    mut roots,
                    untrusted_array_intrinsics,
                    failed_sources,
                } = profile;
                let roots_len = roots.len();
                let shared_status = shared_init::build(
                    &files,
                    &roots,
                    resolve,
                    untrusted_array_intrinsics,
                    &failed_sources,
                    observer.as_deref(),
                );
                let shared = shared_status.shared();
                let mut per_file = Vec::with_capacity(roots_len);
                if let Some(seed_index) = shared_status.seed_root() {
                    let seed_path = roots.remove(seed_index);
                    let events = run_root(
                        &files,
                        resolve,
                        untrusted_array_intrinsics,
                        &shared,
                        &failed_sources,
                        &observer,
                        &seed_path,
                    );
                    per_file.push((seed_path, events));
                    if let Some(shared_cache) = &shared {
                        if !roots.is_empty()
                            && shared_cache.entries.len() < shared_cache.eligible.len()
                        {
                            let next_seed = roots.remove(0);
                            let events = run_root(
                                &files,
                                resolve,
                                untrusted_array_intrinsics,
                                &shared,
                                &failed_sources,
                                &observer,
                                &next_seed,
                            );
                            per_file.push((next_seed, events));
                        }
                    }
                }
                per_file.par_extend(root_tasks(roots).map(|path| {
                    let events = run_root(
                        &files,
                        resolve,
                        untrusted_array_intrinsics,
                        &shared,
                        &failed_sources,
                        &observer,
                        &path,
                    );
                    (path, events)
                }));
                if let Some(observer) = &observer {
                    shared_init::record(observer, profile_index, roots_len, &shared_status);
                }
                // Values retain their own data; rendering needs no module catalog.
                drop(files);
                per_file.sort_by(|left, right| left.0.cmp(&right.0));
                let mut events = ExecutionEvents::new();
                let mut failures = std::collections::BTreeMap::new();
                let mut source_events = merge::SourceEvents::new();
                for (path, (file_events, consulted)) in per_file {
                    if !consulted.is_empty() {
                        failures.insert(path, consulted.clone());
                    }
                    for (call, values) in file_events {
                        if !consulted.is_empty() {
                            source_events.entry(call.clone()).or_default().extend(
                                values
                                    .iter()
                                    .map(|(generic, _)| (*generic, consulted.clone())),
                            );
                        }
                        events.entry(call).or_default().extend(values);
                    }
                }
                let mut projection = if source_events.is_empty() {
                    merge::reduce(events, collect_executed)
                } else {
                    merge::reduce_with_sources(events, collect_executed, source_events)
                };
                projection.failures = failures;
                ProfileProjection {
                    options,
                    projection,
                }
            })
        })
        .collect()
}

fn run_root(
    files: &FxHashMap<PathBuf, File<'_>>,
    resolve: &(impl Fn(&str, &Path) -> Option<PathBuf> + Sync),
    untrusted_array_intrinsics: bool,
    shared: &Option<Arc<SharedInit>>,
    failed_sources: &Arc<FailedSqlSources>,
    observer: &Option<Arc<InvocationObserver>>,
    path: &Path,
) -> (ExecutionEvents, FailedSqlSources) {
    crate::diagnostics::with_observer(observer.clone(), || {
        let mut evaluator = evaluator::new(files, resolve, untrusted_array_intrinsics);
        evaluator.modules.shared = shared.clone();
        evaluator.modules.sources = SourceDependencies::new(Arc::clone(failed_sources));
        super::super::progress::trace("postgres.annotation.root", path, None, || {
            evaluator.run(path)
        });
        (evaluator.events, evaluator.modules.sources.failures())
    })
}

// One root can initialize a large import graph; keep its siblings stealable
// rather than folding several roots sequentially into one Rayon task.
pub(super) fn root_tasks(roots: Vec<PathBuf>) -> impl IndexedParallelIterator<Item = PathBuf> {
    roots.into_par_iter().with_max_len(1)
}
