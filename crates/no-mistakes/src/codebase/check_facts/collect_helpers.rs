use super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use crate::codebase::dependencies::extract::is_indexable;
use rayon::prelude::*;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn request_sources(
    files: &[PathBuf],
    graph_files: &[PathBuf],
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
) -> Arc<crate::codebase::ts_source::SourceStore> {
    let mut paths = files.to_vec();
    paths.extend(graph_files.iter().cloned());
    paths.extend(
        plan.graph_context
            .visible_files
            .iter()
            .flat_map(|files| files.iter().cloned()),
    );
    if let Some(configs) = &plan.integration_runner_configs {
        paths.extend(configs.paths().cloned());
    }
    if let Some(playwright) = playwright {
        paths.extend(playwright.paths().cloned());
        paths.extend(playwright.source_files().iter().cloned());
        paths.extend(playwright.config_files().iter().cloned());
    }
    Arc::new(crate::codebase::ts_source::SourceStore::new(Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&paths),
    )))
}

pub(super) fn uncollected_files(
    files: &[PathBuf],
    facts: &HashMap<PathBuf, CheckFileFacts>,
    helper_paths: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    files
        .iter()
        .filter(|path| {
            !facts.contains_key(*path)
                && !helper_paths.contains(&crate::codebase::ts_resolver::normalize_path(path))
        })
        .cloned()
        .collect()
}

pub(crate) fn collect_fact_map_with_sources(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    files: &[PathBuf],
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    sources: &crate::codebase::ts_source::SourceStore,
) -> HashMap<PathBuf, CheckFileFacts> {
    let files = crate::codebase::ts_source::deduplicate_analysis_paths(
        files
            .iter()
            .filter(|path| is_indexable(path) || (plan.storybook && super::is_mdx_file(path))),
    );
    files
        .par_iter()
        .map(|path| {
            crate::invocation::check_timeout().ok().map(|()| {
                super::collect_file_facts_with_session_and_sources(
                    session, root, path, plan, playwright, sources,
                )
                .map(|facts| (path.clone(), facts))
            })
        })
        .while_some()
        .flatten()
        .collect()
}

pub(super) fn collect_fact_map_sequential_with_sources(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    files: &[PathBuf],
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    sources: &crate::codebase::ts_source::SourceStore,
) -> HashMap<PathBuf, CheckFileFacts> {
    let files = crate::codebase::ts_source::deduplicate_analysis_paths(
        files
            .iter()
            .filter(|path| is_indexable(path) || (plan.storybook && super::is_mdx_file(path))),
    );
    files
        .iter()
        .take_while(|_| crate::invocation::check_timeout().is_ok())
        .filter_map(|path| {
            super::collect_file_facts_with_session_and_sources(
                session, root, path, plan, playwright, sources,
            )
            .map(|facts| (path.clone(), facts))
        })
        .collect()
}

pub(crate) fn graph_only_files(files: &[PathBuf], graph_files: &[PathBuf]) -> Vec<PathBuf> {
    if graph_files.is_empty() {
        return Vec::new();
    }
    let scoped: BTreeSet<&PathBuf> = files.iter().collect();
    graph_files
        .iter()
        .filter(|path| !scoped.contains(path))
        .cloned()
        .collect()
}
