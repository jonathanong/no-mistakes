use super::super::evaluate::ExecutionEvents;
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
                } = profile;
                let mut per_file = roots
                    .into_par_iter()
                    .map(|path| {
                        let events = crate::diagnostics::with_observer(observer.clone(), || {
                            let mut evaluator =
                                evaluator::new(&files, resolve, untrusted_array_intrinsics);
                            evaluator.run(&path);
                            evaluator.events
                        });
                        (path, events)
                    })
                    .collect::<Vec<_>>();
                // Values retain their own data; rendering needs no module catalog.
                drop(files);
                per_file.sort_by(|left, right| left.0.cmp(&right.0));
                let mut events = ExecutionEvents::new();
                for (_, file_events) in per_file {
                    for (call, values) in file_events {
                        events.entry(call).or_default().extend(values);
                    }
                }
                ProfileProjection {
                    options,
                    projection: merge::reduce(events, collect_executed),
                }
            })
        })
        .collect()
}
