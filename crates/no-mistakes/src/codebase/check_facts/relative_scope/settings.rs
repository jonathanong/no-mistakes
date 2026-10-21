use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::ts_resolver::{resolve_tsconfig_from_visible_and_sources, TsConfig};
use crate::codebase::ts_source::SourceStore;
use crate::codebase::workspaces::{load_indexed_from_source_store, IndexedWorkspaceMap};
use std::{path::Path, sync::Arc};

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
