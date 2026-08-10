use super::super::checker::{check_dynamic_import, DynamicCheckContext};
use super::super::RULE_ID;
use super::super::{config, reachable, resolve_mock_specifiers};
use super::PerTestResult;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, GraphFiles};
use crate::codebase::ts_resolver::ScopedImportResolver;
use crate::codebase::ts_source::{has_disable_comment, has_disable_file_comment};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use dashmap::DashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) struct Request<'a> {
    pub(super) root: &'a Path,
    pub(super) config: &'a NoMistakesConfig,
    pub(super) resolver: &'a ScopedImportResolver<'a>,
    pub(super) graph: &'a DepGraph,
    pub(super) graph_files: &'a GraphFiles,
    pub(super) visible_files: &'a HashSet<PathBuf>,
    pub(super) manual_mocks: &'a HashSet<PathBuf>,
    pub(super) setup_data: &'a [config::ConfigSetupData],
    pub(super) shared: &'a CheckFactMap,
    pub(super) dependency_cache: &'a DashMap<PathBuf, Arc<Vec<PathBuf>>>,
    pub(super) defer_suppression: bool,
}

pub(super) fn analyze(request: Request<'_>, file: PathBuf) -> Result<PerTestResult> {
    let Request {
        root,
        config,
        resolver,
        graph,
        graph_files,
        visible_files,
        manual_mocks,
        setup_data,
        shared,
        dependency_cache,
        defer_suppression,
    } = request;
    let Some(file_facts) = shared.ts.get(&file) else {
        anyhow::bail!("missing shared facts for {}", file.display());
    };
    let Some(source) = file_facts.source.as_deref() else {
        anyhow::bail!("missing source facts for {}", file.display());
    };
    let file_disabled = has_disable_file_comment(source, RULE_ID);
    // A file-disabled parse error cannot yield findings for audit mode, but
    // must not abort unrelated test files.
    if file_disabled && (file_facts.parse_error.is_some() || !defer_suppression) {
        return Ok(PerTestResult::default());
    }
    if let Some(error) = &file_facts.parse_error {
        anyhow::bail!("failed to parse {}: {error}", file.display());
    }
    let Some(facts) = file_facts.dynamic_imports.as_ref() else {
        anyhow::bail!("missing dynamic import facts for {}", file.display());
    };
    let mut mocks = manual_mocks.clone();
    mocks.extend(super::setup_mocks::with_facts(
        root,
        setup_data,
        &file,
        resolver,
        graph_files,
        shared,
    )?);
    mocks.extend(resolve_mock_specifiers(
        &facts.mock_specifiers,
        &file,
        resolver,
        Some(graph_files),
    ));
    let mut direct_findings = Vec::new();
    {
        let mut check_context = DynamicCheckContext {
            root,
            file: &file,
            resolver,
            graph,
            graph_files: Some(graph_files),
            file_universe: Some(visible_files),
            mocks: &mocks,
            dependency_cache,
            findings: &mut direct_findings,
        };
        for import in &facts.dynamic_imports {
            if defer_suppression || !has_disable_comment(source, import.line as u32, RULE_ID) {
                check_dynamic_import(&mut check_context, import.clone());
            }
        }
    }
    let reachable = reachable::collect_with_deferred_suppression(
        reachable::ReachableContext {
            root,
            config,
            resolver,
            graph,
            graph_files: Some(graph_files),
            file_universe: Some(visible_files),
            shared: Some(shared),
            file_cache: None,
        },
        &file,
        &mocks,
        dependency_cache,
        defer_suppression,
    )?;
    Ok(PerTestResult {
        direct_findings,
        reachable_findings: reachable.findings,
        reachable_suppression_file: file_disabled
            .then(|| crate::codebase::ts_source::relative_slash_path(root, &file)),
        covered_reachable_imports: if file_disabled {
            HashSet::new()
        } else {
            reachable.covered
        },
    })
}
