use super::checker::{check_dynamic_import, DynamicCheckContext};
use super::{ast, config, reachable, RuleFinding, RULE_ID};
use crate::codebase::dependencies::graph::{DepGraph, GraphFiles};
use crate::codebase::ts_resolver::{normalize_path, ImportResolution, ImportResolver, TsConfig};
use crate::codebase::ts_source::{has_disable_comment, has_disable_file_comment};
use crate::config::v2::NoMistakesConfig;
use anyhow::{Context, Result};
use dashmap::DashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod imported_helpers;
mod setup_mocks;
use setup_mocks::{precompute_setup_mock_map, setup_mocks, SetupMockRequest};

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
    let setup_data = config::precompute_setup_data(root, config)?;
    let test_files = matching_test_files(root, files, config)?;
    let setup_mock_map =
        precompute_setup_mock_map(root, &test_files, &setup_data, &resolver, None)?;
    for file in test_files {
        let source = std::fs::read_to_string(&file)
            .context(format!("failed to read test file {}", file.display()))?;
        if has_disable_file_comment(&source, RULE_ID) {
            continue;
        }
        let facts = ast::extract(&file, &source)?;
        let rel_path = crate::codebase::ts_source::relative_slash_path(root, &file);
        for setup_files in config::setup_contexts_for_test_precomputed(&rel_path, &setup_data) {
            let mut mocks = manual_mocks.clone();
            mocks.extend(setup_mocks(SetupMockRequest {
                setup_files: &setup_files,
                mock_map: &setup_mock_map,
                graph,
                visible_files: &visible_files,
                resolver: &resolver,
                file_cache: &file_cache,
            })?);
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
            let reachable = reachable::collect(
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

pub(crate) fn resolve_mock_specifiers(
    specifiers: &[String],
    file: &Path,
    resolver: &dyn ImportResolution,
    graph_files: Option<&GraphFiles>,
) -> HashSet<PathBuf> {
    specifiers
        .iter()
        .map(|specifier| {
            resolver
                .resolve(specifier, file)
                .map(|path| remap_resolved_path(graph_files, path))
                .unwrap_or_else(|| PathBuf::from(specifier))
        })
        .collect()
}

/// Keep resolver results in the lexical namespace owned by the dependency
/// graph. A scoped resolver may return a canonical target through a symlinked
/// package root, while graph nodes and discovered manual mocks retain the
/// caller-visible lexical path.
pub(crate) fn remap_resolved_path(graph_files: Option<&GraphFiles>, path: PathBuf) -> PathBuf {
    graph_files
        .and_then(|files| files.visible_path(&path).map(Path::to_path_buf))
        .unwrap_or(path)
}

fn matching_test_files(
    root: &Path,
    files: &[PathBuf],
    config: &NoMistakesConfig,
) -> Result<Vec<PathBuf>> {
    let filter = config::test_filter(root, config)?;
    Ok(matching_test_files_with_filter(root, files, &filter))
}

pub(crate) fn matching_test_files_with_filter(
    root: &Path,
    files: &[PathBuf],
    filter: &config::TestFilter,
) -> Vec<PathBuf> {
    files
        .iter()
        .filter(|file| crate::codebase::dependencies::extract::is_indexable(file))
        .filter(|file| {
            filter.is_match(&crate::codebase::ts_source::relative_slash_path(root, file))
        })
        .map(|file| normalize_path(file))
        .collect()
}
