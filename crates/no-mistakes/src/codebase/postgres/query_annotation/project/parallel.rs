use super::super::evaluate::ExecutionEvents;
use super::{evaluator, prepare::Profile};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub(super) fn evaluate(
    profiles: &[Profile<'_>],
    resolve: &(impl Fn(&str, &Path) -> Option<PathBuf> + Sync),
) -> Vec<(usize, PathBuf, ExecutionEvents)> {
    let work = profiles
        .iter()
        .enumerate()
        .flat_map(|(index, profile)| profile.roots.iter().map(move |path| (index, profile, path)))
        .collect::<Vec<_>>();
    let observer = crate::diagnostics::current();
    if let Some(observer) = &observer {
        observer.increment(
            "postgres.annotation.parallel_passes",
            u64::from(!work.is_empty()),
        );
    }
    work.par_iter()
        .map(|(index, profile, path)| {
            let events = crate::diagnostics::with_observer(observer.clone(), || {
                let mut evaluator =
                    evaluator::new(&profile.files, resolve, profile.untrusted_array_intrinsics);
                evaluator.run(path);
                evaluator.events
            });
            (*index, (*path).clone(), events)
        })
        .collect()
}
