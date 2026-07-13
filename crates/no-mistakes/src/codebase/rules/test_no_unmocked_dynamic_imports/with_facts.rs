use super::checker::{check_dynamic_import, DynamicCheckContext};
use super::{
    config, manual_mocks, matching_test_files_with_filter, reachable, resolve_mock_specifiers,
};
use super::{RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::rules::test_no_unmocked_dynamic_imports::runtime::runtime_deps;
use crate::codebase::ts_resolver::ImportResolver;
use crate::codebase::ts_resolver::TsConfig;
use crate::codebase::ts_source::{has_disable_comment, has_disable_file_comment};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use dashmap::DashMap;
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod setup_mocks;

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
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
        tsconfig_path,
        root,
        shared.files(),
    )?;
    check_with_prepared_facts(root, config, &tsconfig, shared)
}

#[doc(hidden)]
pub fn check_with_prepared_facts(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig: &TsConfig,
    shared: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let files = shared.files().to_vec();
    let graph_files = shared.graph_file_universe().to_vec();
    let visible_files = shared
        .graph_file_universe()
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let resolver = ImportResolver::new(tsconfig).with_visible(&visible_files);
    let graph = crate::perf_trace::trace("test_no_unmocked_dynamic_imports.graph_build", || {
        DepGraph::build_with_plan_file_list_config_and_complete_check_facts(
            root,
            tsconfig,
            GraphBuildPlan::imports_and_workspace(),
            graph_files,
            None,
            shared,
        )
    })?;
    let manual_mocks =
        crate::perf_trace::trace("test_no_unmocked_dynamic_imports.manual_mocks", || {
            manual_mocks::discover_from_files(root, &files)
        });
    let prepared =
        crate::perf_trace::trace("test_no_unmocked_dynamic_imports.prepare_config", || {
            config::prepare_from_visible(root, config, shared.graph_file_universe())
        })?;
    let test_files = matching_test_files_with_filter(root, &files, prepared.test_filter());
    let setup_data = prepared.setup_data();

    // Pre-populate the dependency cache for all test files in parallel so that
    // reachable source checks hit the cache instead of re-running BFS per test.
    let dependency_cache: DashMap<PathBuf, Arc<Vec<PathBuf>>> = DashMap::new();
    crate::perf_trace::trace(
        "test_no_unmocked_dynamic_imports.dependency_cache_prepopulate",
        || {
            test_files.par_iter().for_each(|file| {
                dependency_cache
                    .entry(file.clone())
                    .or_insert_with(|| Arc::new(runtime_deps(&graph, file.clone())));
            });
        },
    );

    let per_test_start = std::time::Instant::now();
    let per_test: Vec<PerTestResult> = test_files
        .into_par_iter()
        .map(|file| {
            let Some(file_facts) = shared.ts.get(&file) else {
                anyhow::bail!("missing shared facts for {}", file.display());
            };
            let Some(source) = file_facts.source.as_deref() else {
                anyhow::bail!("missing source facts for {}", file.display());
            };
            if has_disable_file_comment(source, RULE_ID) {
                return Ok(PerTestResult {
                    direct_findings: Vec::new(),
                    reachable_findings: Vec::new(),
                    covered_reachable_imports: HashSet::new(),
                });
            }
            if let Some(error) = &file_facts.parse_error {
                anyhow::bail!("failed to parse {}: {error}", file.display());
            }
            let Some(facts) = file_facts.dynamic_imports.as_ref() else {
                anyhow::bail!("missing dynamic import facts for {}", file.display());
            };
            let mut mocks = manual_mocks.clone();
            mocks.extend(setup_mocks::with_facts(
                root, setup_data, &file, &resolver, shared,
            )?);
            mocks.extend(resolve_mock_specifiers(
                &facts.mock_specifiers,
                &file,
                &resolver,
            ));
            let mut local_findings = Vec::new();
            {
                let mut check_context = DynamicCheckContext {
                    root,
                    file: &file,
                    resolver: &resolver,
                    graph: &graph,
                    mocks: &mocks,
                    dependency_cache: &dependency_cache,
                    findings: &mut local_findings,
                };
                for import in &facts.dynamic_imports {
                    if !has_disable_comment(source, import.line as u32, RULE_ID) {
                        check_dynamic_import(&mut check_context, import.clone());
                    }
                }
            }
            let reachable = reachable::collect(
                reachable::ReachableContext {
                    root,
                    config,
                    resolver: &resolver,
                    graph: &graph,
                    shared: Some(shared),
                    file_cache: None,
                },
                &file,
                &mocks,
                &dependency_cache,
            )?;
            Ok(PerTestResult {
                direct_findings: local_findings,
                reachable_findings: reachable.findings,
                covered_reachable_imports: reachable.covered,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    crate::perf_trace::record(
        "test_no_unmocked_dynamic_imports.per_test_analysis",
        per_test_start.elapsed(),
    );

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
