use super::evaluate::{File, Value};
use super::modules::Module;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::FileIdMap;
use crate::fx::FxHashMap;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod demand;
mod evaluator;
mod merge;
mod output;
mod parallel;
mod prepare;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectedSql {
    pub text: String,
    pub complete: bool,
    pub only_value_placeholders: bool,
}

pub(crate) type ProjectedCalls =
    BTreeMap<(PathBuf, (u32, u32)), Result<Vec<ProjectedSql>, &'static str>>;

/// Resolve owned syntax against the request's canonical module facts and
/// resolver. No source reads or parsing occur in this projection.
pub(crate) fn project(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf> + Sync,
) {
    let _ = project_impl(files, resolve, false);
}

pub(crate) fn project_executed(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf> + Sync,
) -> ProjectedCalls {
    project_impl(files, resolve, true)
}

fn project_impl(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf> + Sync,
    collect_executed: bool,
) -> ProjectedCalls {
    let mut projected = ProjectedCalls::new();
    let originals = files
        .into_iter()
        .map(|(path, _)| (normalize_path(path), path.clone()))
        .collect::<FxHashMap<_, _>>();
    let (profiles, mut per_file) = {
        let profiles = prepare::profiles(files, &resolve);
        let per_file = parallel::evaluate(&profiles, &resolve);
        let options = profiles
            .into_iter()
            .map(|profile| profile.options)
            .collect::<Vec<_>>();
        (options, per_file)
    };
    // Keep the historical profile/root merge order after the shared parallel pass.
    per_file.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let mut per_file = per_file.into_iter().peekable();
    for (index, options) in profiles.into_iter().enumerate() {
        let mut events = BTreeMap::<_, Vec<_>>::new();
        while let Some((_, _, file_events)) = per_file.next_if(|(profile, _, _)| *profile == index)
        {
            for (call, values) in file_events {
                events.entry(call).or_default().extend(values);
            }
        }
        merge::project(
            files,
            &originals,
            &options,
            events,
            collect_executed,
            &mut projected,
        );
    }
    projected
}

#[cfg(test)]
mod tests;
