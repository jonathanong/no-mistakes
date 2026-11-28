use super::analysis_session::{AnalysisSession, PathInterner};
use super::dependencies::graph::{Edge, GraphFiles};
use super::ts_source::SourceStore;
use super::workspace_resolution::{IndexedWorkspaceMap, WorkspaceMap};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

type LoadWorkspace = fn(&Path, &[PathBuf]) -> Result<IndexedWorkspaceMap>;
type LoadWorkspaceWithSources = fn(&Path, &[PathBuf], &SourceStore) -> Result<IndexedWorkspaceMap>;
type ManifestEdges = fn(&[PathBuf], &IndexedWorkspaceMap, &GraphFiles, &PathInterner) -> Vec<Edge>;
type ProductionImportFiles =
    fn(&Path, &NoMistakesConfig, &[PathBuf]) -> Result<Arc<BTreeSet<PathBuf>>>;

pub(crate) struct Hooks {
    pub(crate) load_workspace_from_files: LoadWorkspace,
    pub(crate) load_workspace_with_sources: LoadWorkspaceWithSources,
    pub(crate) manifest_edges: ManifestEdges,
    pub(crate) production_import_files: ProductionImportFiles,
}

static HOOKS: OnceLock<&'static Hooks> = OnceLock::new();

pub(crate) fn register(hooks: &'static Hooks) {
    HOOKS.get_or_init(|| hooks);
}

pub(crate) fn hooks() -> &'static Hooks {
    HOOKS
        .get()
        .expect("package analysis hooks registered by the facade")
}

pub(crate) fn load_indexed_from_files(
    root: &Path,
    files: &[PathBuf],
) -> Result<IndexedWorkspaceMap> {
    (hooks().load_workspace_from_files)(root, files)
}

pub(crate) fn load_indexed_from_source_store(
    root: &Path,
    sources: &SourceStore,
) -> Result<IndexedWorkspaceMap> {
    let root = super::ts_resolver::normalize_path(root);
    let files = sources.inventory().paths();
    (hooks().load_workspace_with_sources)(&root, &files, sources)
}

pub(crate) fn load_from_files(root: &Path, files: &[PathBuf]) -> Result<WorkspaceMap> {
    load_indexed_from_files(root, files).map(|indexed| WorkspaceMap::clone(&indexed))
}

pub(crate) fn load_from_files_with_session(
    root: &Path,
    files: &[PathBuf],
    session: Option<&AnalysisSession>,
) -> Result<WorkspaceMap> {
    let Some(session) = session else {
        return load_from_files(root, files);
    };
    let snapshot = session.visible_paths(root);
    let sources = snapshot.source_store_for(root);
    (hooks().load_workspace_with_sources)(root, files, &sources)
        .map(|indexed| WorkspaceMap::clone(&indexed))
}
