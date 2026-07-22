use super::RuleFinding;
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

mod colocated_tests;
mod config;
mod coverage;
mod coverage_graph;
mod findings;
mod prepared;
mod runner;
mod selection;
mod types;

use colocated_tests::covered_components as colocated_test_covered_components;
use config::effective_story_patterns;
use coverage::{all_react_component_keys, directly_covered_components, reachable_story_files};
use coverage_graph::{dynamic_or_mock_boundary_files, transitive_covered_components};
use findings::{namespace_import_findings, stale_or_blank_allow_findings};
pub(crate) use prepared::{
    check_with_prepared_facts_and_inferred_and_session, check_with_prepared_facts_and_session,
};
use selection::{component_disabled, file_disabled, selected_components};
use types::{GlobMatcher, Options};

pub const RULE_ID: &str = "require-storybook-stories";

/// Catalog roots explicitly selected by Storybook rule applications.
///
/// These projects may be standalone directories rather than workspace
/// packages, so they cannot rely on workspace discovery to surface their
/// package-local tsconfig aliases.
#[doc(hidden)]
pub fn configured_project_roots(root: &Path, config: &NoMistakesConfig) -> Vec<std::path::PathBuf> {
    let mut roots = config
        .rule_applications(RULE_ID)
        .iter()
        .flat_map(|rule| rule.projects.iter())
        .filter_map(|name| config.projects.get(name))
        .map(|project| {
            project
                .root
                .as_deref()
                .map(|path| root.join(path))
                .unwrap_or_else(|| root.to_path_buf())
        })
        .collect::<Vec<_>>();
    roots.sort();
    roots.dedup();
    roots
}

pub fn check(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig_path: Option<&Path>,
) -> Result<Vec<RuleFinding>> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible_paths = snapshot.paths_for(root);
    let files = crate::codebase::ts_source::discover_files_from_visible(
        root,
        &config.filesystem.skip_directories,
        &visible_paths,
    );
    let facts = crate::codebase::check_facts::collect_check_facts(
        root,
        files,
        CheckFactPlan {
            react: true,
            symbols: true,
            storybook: true,
            dynamic_imports: true,
            source: true,
            ..Default::default()
        },
    );
    let sources = snapshot.source_store_for(root);
    let catalog = tsconfig_catalog(root, config, tsconfig_path, &visible_paths, &sources)?;
    check_with_facts_and_catalog(root, config, &facts, &catalog, None)
}

fn check_with_facts_and_catalog(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &CheckFactMap,
    catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
) -> Result<Vec<RuleFinding>> {
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let visible_files = shared
        .files()
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<HashSet<_>>();
    let resolver = crate::codebase::ts_resolver::ScopedImportResolver::new_in_session(
        catalog,
        &visible_files,
        &session,
    );
    runner::check_with_resolver(root, config, shared, &resolver, inferred_roots)
}

fn tsconfig_catalog(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig_path: Option<&Path>,
    visible_paths: &[std::path::PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<crate::codebase::ts_resolver::TsConfigCatalog> {
    if let Some(path) = tsconfig_path {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
            Some(&path),
            root,
            visible_paths,
        )?;
        return Ok(crate::codebase::ts_resolver::TsConfigCatalog::forced(
            root,
            tsconfig,
            Some(crate::codebase::ts_resolver::normalize_path(&path)),
        ));
    }
    let mut candidate_roots = vec![root.to_path_buf()];
    candidate_roots.extend(configured_project_roots(root, config));
    Ok(
        crate::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources(
            root,
            &candidate_roots,
            visible_paths,
            sources,
        ),
    )
}

#[cfg(test)]
mod tests;
