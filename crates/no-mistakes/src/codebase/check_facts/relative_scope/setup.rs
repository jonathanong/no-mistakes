use crate::codebase::{
    analysis_session::AnalysisSession,
    postgres::{package_name, package_root_for_specifier},
    ts_resolver::{
        resolve_tsconfig_from_visible_and_sources, ImportResolver, ScopedImportResolver, TsConfig,
    },
    ts_source::SourceStore,
    workspaces::{load_indexed_from_source_store, IndexedWorkspaceMap},
};
use crate::fx::PathSet;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
pub(super) fn tsconfig_for(
    session: &AnalysisSession,
    root: &Path,
    sources: &SourceStore,
) -> Arc<TsConfig> {
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

pub(super) fn workspace_for(
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

pub(super) fn cached_package_root(
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

pub(super) fn resolve_exact(
    resolver: &ScopedImportResolver<'_>,
    workspace: &IndexedWorkspaceMap,
    visible: &PathSet,
    specifier: &str,
    from: &Path,
) -> Option<PathBuf> {
    if let Some(target) = resolver.resolve(specifier, from) {
        return visible.contains(&target).then_some(target);
    }
    workspace.resolve_specifier_from_file_visible(specifier, from, visible)
}
