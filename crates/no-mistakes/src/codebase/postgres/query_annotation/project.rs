use super::evaluate::{File, Value};
use super::modules::Module;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::FileIdMap;
use crate::fx::FxHashMap;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod demand;
pub(super) mod evaluator;
mod merge;
pub(in crate::codebase::postgres::query_annotation) mod output;
mod parallel;
mod prepare;
pub(crate) use prepare::Catalog;
mod work;

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
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
pub(crate) fn project(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf> + Sync,
) {
    project_with_catalog(files, resolve, None);
}

pub(crate) fn project_executed(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf> + Sync,
) -> ProjectedCalls {
    project_impl(files, resolve, true, None)
}

pub(crate) type SyntaxDemand = FxHashMap<PathBuf, BTreeSet<EmbeddedSqlOptions>>;
pub(crate) type SyntaxOverlay<'a> =
    dyn FnMut(&mut FileIdMap<CheckFileFacts>, SyntaxDemand) -> bool + 'a;

pub(crate) fn prepare_syntax(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf> + Sync,
    overlay: &mut SyntaxOverlay<'_>,
) -> Catalog {
    let mut catalog = prepare::catalog(files);
    let mut demand = SyntaxDemand::default();
    for profile in prepare::profiles_with_catalog(files, &resolve, &catalog, true) {
        for path in demand::reachable(&profile.files, &profile.roots, &resolve) {
            demand
                .entry(path.to_path_buf())
                .or_default()
                .insert(profile.options.clone());
        }
        catalog
            .syntax_roots
            .insert(profile.options, profile.roots.into_iter().collect());
    }
    if !overlay(files, demand) {
        catalog.options.retain(|options| {
            files.values().any(|file| {
                file.query_annotation
                    .iter()
                    .any(|(profile, _)| profile == options)
            })
        });
    }
    catalog
}

pub(crate) fn project_with_catalog(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf> + Sync,
    catalog: Option<Catalog>,
) {
    let _ = project_impl(files, resolve, false, catalog);
}

fn project_impl(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf> + Sync,
    collect_executed: bool,
    catalog: Option<Catalog>,
) -> ProjectedCalls {
    let mut projected = ProjectedCalls::new();
    let originals = files
        .into_iter()
        .map(|(path, _)| (normalize_path(path), path.clone()))
        .collect::<FxHashMap<_, _>>();
    let profiles = {
        let profiles = crate::perf_trace::trace("postgres.annotation.prepare", || {
            let catalog = catalog.unwrap_or_else(|| prepare::catalog(files));
            prepare::profiles_with_catalog(files, &resolve, &catalog, false)
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

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
