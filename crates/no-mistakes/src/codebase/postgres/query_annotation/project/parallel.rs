use super::super::evaluate::{
    module_cache::source_dependencies::SourceDependencies, ExecutionEvents,
};
use super::{evaluator, merge, prepare::Profile};
use crate::codebase::postgres::EmbeddedSqlOptions;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

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
        .map(|profile| {
            crate::diagnostics::with_observer(observer.clone(), || {
                let Profile {
                    options,
                    files,
                    roots,
                    untrusted_array_intrinsics,
                    failed_sources,
                } = profile;
                let mut per_file = root_tasks(roots)
                    .map(|path| {
                        let events = crate::diagnostics::with_observer(observer.clone(), || {
                            let mut evaluator =
                                evaluator::new(&files, resolve, untrusted_array_intrinsics);
                            evaluator.modules.sources =
                                SourceDependencies::new(std::sync::Arc::clone(&failed_sources));
                            super::super::progress::trace(
                                "postgres.annotation.root",
                                &path,
                                None,
                                || evaluator.run(&path),
                            );
                            (evaluator.events, evaluator.modules.sources.failures())
                        });
                        (path, events)
                    })
                    .collect::<Vec<_>>();
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

// One root can initialize a large import graph; keep its siblings stealable
// rather than folding several roots sequentially into one Rayon task.
pub(super) fn root_tasks(roots: Vec<PathBuf>) -> impl IndexedParallelIterator<Item = PathBuf> {
    roots.into_par_iter().with_max_len(1)
}
