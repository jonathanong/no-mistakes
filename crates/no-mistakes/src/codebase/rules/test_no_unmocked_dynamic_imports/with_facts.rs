use super::RuleFinding;
use super::{config, manual_mocks, matching_test_files_with_filter, reachable};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, GraphFiles};
use crate::codebase::rules::test_no_unmocked_dynamic_imports::runtime::runtime_deps;
use crate::codebase::ts_resolver::{ScopedImportResolver, TsConfig};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use dashmap::DashMap;
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod graph;
mod per_test;
mod setup_mocks;
mod tsconfig_catalog;

pub(crate) use graph::check_with_prepared_facts_and_session;

#[derive(Default)]
struct PerTestResult {
    direct_findings: Vec<RuleFinding>,
    reachable_findings: Vec<reachable::ReachableFinding>,
    covered_reachable_imports: HashSet<super::checker::DynamicImportKey>,
}

pub fn check_with_facts(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig_path: Option<&Path>,
    shared: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    session.insert_visible_paths(
        root,
        std::sync::Arc::new(crate::codebase::ts_source::VisiblePathSnapshot::from_paths(
            root,
            shared.files(),
        )),
    );
    let sources = session.visible_paths(root).source_store_for(root);
    let (tsconfig, catalog) = tsconfig_catalog::for_request(root, tsconfig_path, shared, &sources)?;
    check_with_prepared_facts_and_session(
        root, config, &tsconfig, &catalog, shared, &session, &sources,
    )
}

#[doc(hidden)]
pub fn check_with_prepared_facts(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig: &TsConfig,
    shared: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let catalog = tsconfig_catalog::forced(root, tsconfig);
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    session.insert_visible_paths(
        root,
        std::sync::Arc::new(crate::codebase::ts_source::VisiblePathSnapshot::from_paths(
            root,
            shared.files(),
        )),
    );
    let sources = session.visible_paths(root).source_store_for(root);
    check_with_prepared_facts_and_session(
        root, config, tsconfig, &catalog, shared, &session, &sources,
    )
}

pub(crate) struct PreparedFactsGraphRequest<'a> {
    pub(crate) root: &'a Path,
    pub(crate) config: &'a NoMistakesConfig,
    pub(crate) tsconfig_catalog: &'a crate::codebase::ts_resolver::TsConfigCatalog,
    pub(crate) shared: &'a CheckFactMap,
    pub(crate) graph: &'a DepGraph,
    pub(crate) session: &'a std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    pub(crate) sources: &'a crate::codebase::ts_source::SourceStore,
    pub(crate) defer_suppression: bool,
}

pub(crate) fn check_with_prepared_facts_graph_and_session(
    request: PreparedFactsGraphRequest<'_>,
) -> Result<Vec<RuleFinding>> {
    let PreparedFactsGraphRequest {
        root,
        config,
        tsconfig_catalog,
        shared,
        graph,
        session,
        sources,
        defer_suppression,
    } = request;
    let files = shared.files().to_vec();
    let visible_files = files.iter().cloned().collect::<HashSet<_>>();
    // Dynamic-import policy is filesystem-scoped even when another consumer
    // shares a graph built from the complete repository universe.
    let graph_files = GraphFiles::from_files(files.clone());
    let resolver = ScopedImportResolver::new_in_session(tsconfig_catalog, &visible_files, session);
    let manual_mocks =
        crate::perf_trace::trace("test_no_unmocked_dynamic_imports.manual_mocks", || {
            manual_mocks::discover_from_files(root, &files)
        });
    let prepared =
        crate::perf_trace::trace("test_no_unmocked_dynamic_imports.prepare_config", || {
            config::prepare_from_visible(root, config, &files, sources)
        })?;
    let test_files = matching_test_files_with_filter(root, &files, prepared.test_filter());
    let setup_data = prepared.setup_data();

    let dependency_cache: DashMap<PathBuf, Arc<Vec<PathBuf>>> = DashMap::new();
    crate::perf_trace::trace(
        "test_no_unmocked_dynamic_imports.dependency_cache_prepopulate",
        || {
            test_files.par_iter().for_each(|file| {
                dependency_cache.entry(file.clone()).or_insert_with(|| {
                    Arc::new(runtime_deps(graph, file.clone(), Some(&visible_files)))
                });
            });
        },
    );

    let per_test =
        crate::perf_trace::trace("test_no_unmocked_dynamic_imports.per_test_analysis", || {
            test_files
                .into_par_iter()
                .map(|file| {
                    per_test::analyze(
                        per_test::Request {
                            root,
                            config,
                            resolver: &resolver,
                            graph,
                            graph_files: &graph_files,
                            visible_files: &visible_files,
                            manual_mocks: &manual_mocks,
                            setup_data,
                            shared,
                            sources,
                            dependency_cache: &dependency_cache,
                            defer_suppression,
                        },
                        file,
                    )
                })
                .collect::<Result<Vec<_>>>()
        })?;

    let mut covered_reachable_imports = HashSet::new();
    for result in &per_test {
        covered_reachable_imports.extend(result.covered_reachable_imports.iter().cloned());
    }
    let mut findings: Vec<RuleFinding> = per_test
        .into_iter()
        .flat_map(|result| {
            result.direct_findings.into_iter().chain(
                result
                    .reachable_findings
                    .into_iter()
                    .filter(|entry| !covered_reachable_imports.contains(&entry.key))
                    .map(|entry| entry.finding),
            )
        })
        .collect();
    findings.sort_by(|a, b| (&a.file, a.line, &a.target).cmp(&(&b.file, b.line, &b.target)));
    Ok(findings)
}
