use super::CheckFileFacts;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::postgres::{
    package_name, package_root_for_specifier, project_relative_scoped_facts,
};
use crate::codebase::ts_resolver::{
    resolve_tsconfig_from_visible_and_sources, ImportResolver, TsConfig,
};
use crate::codebase::ts_source::{FileIdMap, SourceStore};
use crate::codebase::workspaces::{load_indexed_from_source_store, IndexedWorkspaceMap};
use crate::fx::PathSet;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Promote relative scoped-executor candidates using the request resolver cache.
///
/// Files with no candidates are left untouched. A missing package root drops them.
pub(super) fn project_relative_executor_scopes(
    session: &AnalysisSession,
    root: &Path,
    sources: &SourceStore,
    files: &mut FileIdMap<CheckFileFacts>,
) {
    let annotation = files
        .into_iter()
        .any(|(_, file)| !file.query_annotation.is_empty());
    if !has_relative_candidates(files) && !annotation {
        return;
    }
    let tsconfig = tsconfig_for(session, root, sources);
    let workspace = workspace_for(session, root, sources);
    let mut visible = PathSet::default();
    for path in sources.inventory().paths().iter() {
        visible.insert(path.clone());
    }
    let resolver = ImportResolver::new_in_session(tsconfig.as_ref(), Some(&visible), session);
    let mut package_roots = HashMap::<String, PathBuf>::new();
    for (path, facts) in &mut *files {
        for (options, embedded) in &mut facts.embedded_sql {
            if embedded.pending_relative.candidates.is_empty() {
                continue;
            }
            let package_root = cached_package_root(
                &mut package_roots,
                &options.import_specifier,
                path,
                &workspace,
                &resolver,
            );
            project_relative_scoped_facts(embedded, package_root.as_deref(), |specifier| {
                resolver.resolve(specifier, path)
            });
        }
    }
    if annotation {
        crate::codebase::postgres::query_annotation::project::project(files, |specifier, from| {
            resolver.resolve(specifier, from)
        });
    }
}

fn has_relative_candidates(files: &FileIdMap<CheckFileFacts>) -> bool {
    files.into_iter().any(|(_, facts)| {
        facts
            .embedded_sql
            .iter()
            .any(|(_, embedded)| !embedded.pending_relative.candidates.is_empty())
    })
}

fn cached_package_root(
    cache: &mut HashMap<String, PathBuf>,
    specifier: &str,
    file: &Path,
    workspace: &IndexedWorkspaceMap,
    resolver: &ImportResolver<'_>,
) -> Option<PathBuf> {
    if let Some(cached) = cache.get(specifier) {
        return Some(cached.clone());
    }
    let resolved = package_root_for_specifier(specifier, file, workspace, |spec, from| {
        resolver.resolve(spec, from)
    });
    remember_package_root(cache, specifier, workspace, resolved)
}

/// Cache a root only when `package_by_name` already named it. A fallback that
/// walked up from one file is not stable for the next importer.
pub(super) fn remember_package_root(
    cache: &mut HashMap<String, PathBuf>,
    specifier: &str,
    workspace: &IndexedWorkspaceMap,
    resolved: Option<PathBuf>,
) -> Option<PathBuf> {
    if let Some(root) = resolved.clone() {
        if package_name(specifier).is_some_and(|name| workspace.package_by_name(name).is_some()) {
            cache.insert(specifier.to_string(), root);
        }
    }
    resolved
}

fn tsconfig_for(session: &AnalysisSession, root: &Path, sources: &SourceStore) -> Arc<TsConfig> {
    let loaded = if session.existing_sources_for(root).is_some() {
        session.tsconfig(root, None).ok()
    } else {
        resolve_tsconfig_from_visible_and_sources(
            None,
            root,
            sources.inventory().paths().as_ref(),
            sources,
        )
        .ok()
        .map(Arc::new)
    };
    or_empty_tsconfig(root, loaded)
}

pub(super) fn or_empty_tsconfig(root: &Path, config: Option<Arc<TsConfig>>) -> Arc<TsConfig> {
    config.unwrap_or_else(|| {
        Arc::new(TsConfig {
            dir: root.to_path_buf(),
            paths_dir: root.to_path_buf(),
            ..TsConfig::default()
        })
    })
}

fn workspace_for(
    session: &AnalysisSession,
    root: &Path,
    sources: &SourceStore,
) -> Arc<IndexedWorkspaceMap> {
    if session.existing_sources_for(root).is_some() {
        return session.workspace(root);
    }
    workspace_or_empty(load_indexed_from_source_store(root, sources))
}

pub(super) fn workspace_or_empty(
    loaded: anyhow::Result<IndexedWorkspaceMap>,
) -> Arc<IndexedWorkspaceMap> {
    Arc::new(loaded.unwrap_or_default())
}

#[cfg(test)]
mod tests;
