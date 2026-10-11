use super::super::checker::{check_dynamic_import, DynamicCheckContext};
use super::super::{ast, config, reachable, RuleFinding, RULE_ID};
use super::{
    imported_helpers, matching_test_files_with_filter, resolve_mock_specifiers, setup_mocks,
};
use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::ts_resolver::{ImportResolver, TsConfig};
use crate::codebase::ts_source::{has_disable_comment, has_disable_file_comment};
use crate::config::v2::NoMistakesConfig;
use anyhow::{Context, Result};
use dashmap::DashMap;
use setup_mocks::{precompute_setup_mock_map, setup_mocks, SetupMockRequest};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) fn check_inner(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    tsconfig: &TsConfig,
    graph: &DepGraph,
    manual_mocks: &HashSet<PathBuf>,
) -> Result<Vec<RuleFinding>> {
    let visible_files = files.iter().cloned().collect::<crate::fx::PathSet>();
    let resolver = ImportResolver::new(tsconfig).with_visible(&visible_files);
    let dependency_cache: DashMap<PathBuf, Arc<Vec<PathBuf>>> = DashMap::new();
    let file_cache: DashMap<PathBuf, Arc<reachable::CachedFileFacts>> = DashMap::new();
    let mut findings = Vec::new();
    let prepared = config::standalone::prepare_standalone(root, config, &visible_files, tsconfig)?;
    let setup_data = prepared.setup_data();
    let test_files = matching_test_files_with_filter(root, files, prepared.test_filter());
    let selected_runners = config::SelectedRunners::for_config(config);
    let setup_mock_map = precompute_setup_mock_map(
        root,
        &test_files,
        setup_data,
        selected_runners,
        &resolver,
        None,
    )?;
    let mut tests = Vec::new();
    for file in test_files {
        let source = std::fs::read_to_string(&file)
            .context(format!("failed to read test file {}", file.display()))?;
        if has_disable_file_comment(&source, RULE_ID) {
            continue;
        }
        let facts = ast::extract(&file, &source)?;
        let rel_path = crate::codebase::ts_source::relative_slash_path(root, &file);
        let contexts =
            config::setup_contexts_for_test_precomputed(&rel_path, setup_data, selected_runners);
        tests.push((file, source, facts, contexts));
    }
    let context_mocks = setup_mocks::precompute_contexts(
        tests
            .iter()
            .flat_map(|(_, _, _, contexts)| contexts.iter().cloned()),
        |setup_files| {
            setup_mocks(SetupMockRequest {
                setup_files,
                mock_map: &setup_mock_map,
                graph,
                visible_files: &visible_files,
                resolver: &resolver,
                file_cache: &file_cache,
            })
        },
    )?;
    for (file, source, facts, contexts) in tests {
        for setup_files in contexts {
            let mut mocks = manual_mocks.clone();
            mocks.extend(context_mocks[&setup_files].iter().cloned());
            mocks.extend(resolve_mock_specifiers(
                &facts.mock_specifiers,
                &file,
                &resolver,
                None,
            ));
            mocks.extend(imported_helpers::collect(
                graph,
                &file,
                &visible_files,
                &resolver,
                &file_cache,
                &mocks,
            )?);
            let mut check_context = DynamicCheckContext {
                root,
                file: &file,
                resolver: &resolver,
                graph,
                graph_files: None,
                file_universe: Some(&visible_files),
                mocks: &mocks,
                dependency_cache: &dependency_cache,
                findings: &mut findings,
            };
            for import in facts.dynamic_imports.iter().cloned() {
                if has_disable_comment(&source, import.line as u32, RULE_ID) {
                    continue;
                }
                check_dynamic_import(&mut check_context, import);
            }
            let reachable = reachable::test_api::collect(
                reachable::ReachableContext {
                    root,
                    config,
                    resolver: &resolver,
                    graph,
                    graph_files: None,
                    file_universe: Some(&visible_files),
                    shared: None,
                    file_cache: Some(&file_cache),
                },
                &file,
                &mocks,
                &dependency_cache,
            );
            let reachable = reachable?;
            findings.extend(
                reachable
                    .findings
                    .into_iter()
                    .filter(|entry| !reachable.covered.contains(&entry.key))
                    .map(|entry| entry.finding),
            );
        }
    }
    findings.sort_by(|a, b| {
        (&a.file, a.line, &a.target)
            .cmp(&(&b.file, b.line, &b.target))
            .then_with(|| a.cmp(b))
    });
    findings.dedup();
    Ok(findings)
}
