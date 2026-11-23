use super::{load_package_with_sources, normalize_path, WorkspaceMap, WorkspaceSources};
use crate::codebase::ts_source::SourceStore;
use crate::fx::PathSet;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(crate) struct ManualWorkspace {
    pub(crate) workspace: WorkspaceMap,
    pub(crate) visible: PathSet,
}

/// Load manifests selected by ordered roots, independent of workspace globs.
pub(crate) fn load_manual_workspace_from_source_store<'a>(
    roots: impl IntoIterator<Item = &'a Path>,
    files: &[PathBuf],
    sources: &SourceStore,
) -> anyhow::Result<ManualWorkspace> {
    let mut ordered_roots = Vec::new();
    for root in roots {
        if !ordered_roots.contains(&root) {
            ordered_roots.push(root);
        }
    }
    let manifests = files
        .iter()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("package.json"))
        .collect::<Vec<_>>();
    let mut selected = ordered_roots
        .iter()
        .flat_map(|root| manifests.iter().filter(move |path| path.starts_with(root)))
        .filter_map(|path| path.parent())
        .peekable();
    if selected.peek().is_none() {
        return Ok(ManualWorkspace {
            workspace: WorkspaceMap::from_packages(Vec::new()),
            visible: PathSet::default(),
        });
    }
    if let Some(observer) = sources.observer() {
        observer.increment("workspace.manual_visibility_constructions", 1);
    }
    let visible: PathSet = files.iter().map(|path| normalize_path(path)).collect();
    let mut packages = BTreeMap::new();
    for dir in selected {
        if let Some(package) =
            load_package_with_sources(dir, Some(&visible), WorkspaceSources::Store(sources))?
        {
            packages.insert(package.name.clone(), package);
        }
    }
    Ok(ManualWorkspace {
        workspace: WorkspaceMap::from_packages(packages.into_values().collect()),
        visible,
    })
}
