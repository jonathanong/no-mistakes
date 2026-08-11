use super::ast;
use super::checker::DynamicImportKey;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, GraphFiles};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_resolver::ImportResolution;
use crate::config::v2::NoMistakesConfig;
use anyhow::{Context, Result};
use dashmap::DashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod deferred;

pub(super) struct CachedFileFacts {
    pub(super) source: String,
    pub(super) dynamic_imports: Vec<ast::DynamicImport>,
}

pub(super) struct ReachableFinding {
    pub(super) key: DynamicImportKey,
    pub(super) finding: RuleFinding,
}

pub(super) struct ReachableResult {
    pub(super) findings: Vec<ReachableFinding>,
    pub(super) covered: HashSet<DynamicImportKey>,
}

pub(super) fn collect(
    ctx: ReachableContext<'_>,
    test_file: &Path,
    mocks: &HashSet<PathBuf>,
    dependency_cache: &DashMap<PathBuf, Arc<Vec<PathBuf>>>,
) -> Result<ReachableResult> {
    deferred::collect(ctx, test_file, mocks, dependency_cache, false)
}

pub(super) fn collect_with_deferred_suppression(
    ctx: ReachableContext<'_>,
    test_file: &Path,
    mocks: &HashSet<PathBuf>,
    dependency_cache: &DashMap<PathBuf, Arc<Vec<PathBuf>>>,
    defer_suppression: bool,
) -> Result<ReachableResult> {
    deferred::collect(ctx, test_file, mocks, dependency_cache, defer_suppression)
}

fn collect_outcome(result: &mut ReachableResult, outcome: super::checker::DynamicImportOutcome) {
    if outcome.covered {
        result.covered.insert(outcome.key);
        return;
    }
    result.findings.extend(
        outcome
            .findings
            .into_iter()
            .map(|finding| ReachableFinding {
                key: outcome.key.clone(),
                finding,
            }),
    );
}

pub(super) struct ReachableContext<'a> {
    pub(super) root: &'a Path,
    pub(super) config: &'a NoMistakesConfig,
    pub(super) resolver: &'a dyn ImportResolution,
    pub(super) graph: &'a DepGraph,
    pub(super) graph_files: Option<&'a GraphFiles>,
    pub(super) file_universe: Option<&'a HashSet<PathBuf>>,
    pub(super) shared: Option<&'a CheckFactMap>,
    pub(super) file_cache: Option<&'a DashMap<PathBuf, Arc<CachedFileFacts>>>,
}

fn get_or_cache_file(
    file: &PathBuf,
    cache: Option<&DashMap<PathBuf, Arc<CachedFileFacts>>>,
) -> Result<Arc<CachedFileFacts>> {
    if let Some(cache) = cache {
        if let Some(cached) = cache.get(file) {
            return Ok(cached.clone());
        }
        let source = std::fs::read_to_string(file)
            .context(format!("failed to read dependency file {}", file.display()))?;
        let facts = ast::extract(file, &source)?;
        let arc = Arc::new(CachedFileFacts {
            source,
            dynamic_imports: facts.dynamic_imports,
        });
        cache.insert(file.clone(), arc.clone());
        return Ok(arc);
    }
    let source = std::fs::read_to_string(file)
        .context(format!("failed to read dependency file {}", file.display()))?;
    let facts = ast::extract(file, &source)?;
    Ok(Arc::new(CachedFileFacts {
        source,
        dynamic_imports: facts.dynamic_imports,
    }))
}

fn is_under_skipped_dir(root: &Path, config: &NoMistakesConfig, file: &Path) -> bool {
    file.strip_prefix(root).ok().is_some_and(|rel| {
        if config
            .filesystem
            .skip_directories
            .iter()
            .map(Path::new)
            .any(|skip| rel == skip || rel.starts_with(skip))
        {
            return true;
        }
        rel.components().any(|component| {
            component
                .as_os_str()
                .to_str()
                .is_some_and(|name| crate::codebase::ts_source::SKIP_DIRS.contains(&name))
        })
    })
}
