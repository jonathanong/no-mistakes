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
    let profiles = {
        let profiles = crate::perf_trace::trace("postgres.annotation.prepare", || {
            prepare::profiles(files, &resolve)
        });
        crate::perf_trace::trace("postgres.annotation.evaluate", || {
            parallel::evaluate(profiles, &resolve, collect_executed)
        })
    };
    // Indexed profile collection preserves the historical overwrite order.
    for profile in profiles {
        merge::apply(
            files,
            &originals,
            &profile.options,
            profile.projection,
            &mut projected,
        );
    }
    projected
}

#[cfg(test)]
mod tests;
