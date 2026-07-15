/// Load the workspace map from `root/package.json` or `root/pnpm-workspace.yaml`.
///
/// Returns an empty map if neither file declares workspaces.
///
/// Derives package directories from the shared ignore-aware candidate list.
/// Callers that already have a discovered file list on hand should use
/// [`load_from_files`] directly to avoid repeating discovery.
pub fn load(root: &Path) -> Result<WorkspaceMap> {
    let files = crate::codebase::ts_source::discover_visible_paths(root);
    load_from_files(root, &files)
}

pub fn load_from_files(root: &Path, files: &[PathBuf]) -> Result<WorkspaceMap> {
    load_indexed_from_files(root, files).map(|indexed| indexed.workspace.as_ref().clone())
}

pub(crate) fn load_from_files_with_session(
    root: &Path,
    files: &[PathBuf],
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
) -> Result<WorkspaceMap> {
    let Some(session) = session else {
        return load_from_files(root, files);
    };
    let snapshot = session.visible_paths(root);
    let sources = snapshot.source_store_for(root);
    load_from_files_with_sources(root, files, WorkspaceSources::Store(&sources))
        .map(|indexed| indexed.workspace.as_ref().clone())
}

pub(crate) fn load_indexed_from_files(
    root: &Path,
    files: &[PathBuf],
) -> Result<IndexedWorkspaceMap> {
    load_from_files_with_sources(root, files, WorkspaceSources::Filesystem)
}

#[doc(hidden)]
pub fn load_from_source_store(
    root: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<WorkspaceMap> {
    load_indexed_from_source_store(root, sources).map(|indexed| indexed.workspace.as_ref().clone())
}

pub(crate) fn load_indexed_from_source_store(
    root: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<IndexedWorkspaceMap> {
    let root = normalize_path(root);
    let files = sources.inventory().paths();
    load_from_files_with_sources(&root, &files, WorkspaceSources::Store(sources))
}

fn load_from_files_with_sources(
    root: &Path,
    files: &[PathBuf],
    sources: WorkspaceSources<'_>,
) -> Result<IndexedWorkspaceMap> {
    let metadata = load_workspace_metadata_from_files(root, files, sources)?;
    let workspace_dirs = expand_workspace_globs_from_files(root, &metadata.globs, files)
        .into_iter()
        .map(|path| normalize_path(&path))
        .collect::<std::collections::HashSet<_>>();
    let visible = files
        .iter()
        .map(|path| normalize_path(path))
        .collect::<std::collections::HashSet<_>>();
    let root_manifest = normalize_path(&root.join("package.json"));
    let mut manifest_paths = visible
        .iter()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("package.json"))
        .cloned()
        .collect::<Vec<_>>();
    manifest_paths.sort();

    let mut packages = Vec::new();
    let mut manifest_dependency_names = std::collections::BTreeMap::new();
    if visible.contains(&root_manifest) {
        manifest_dependency_names.insert(
            root_manifest.clone(),
            sorted_dependency_names(&metadata.root_dependency_names),
        );
    }
    for manifest in manifest_paths {
        if manifest == root_manifest {
            continue;
        }
        let dir = manifest
            .parent()
            .map(normalize_path)
            .expect("package manifest has a parent directory");
        let is_workspace = workspace_dirs.contains(&dir);
        let value = match sources.parse_json(&manifest) {
            Ok(value) => value,
            // Preserve tolerant workspace discovery: one malformed or unreadable
            // package manifest must not hide valid sibling packages.
            Err(_) => continue,
        };
        let package = serde_json::from_value::<PackageJson>((*value).clone()).unwrap_or_default();
        manifest_dependency_names.insert(
            manifest,
            sorted_dependency_names(&package.dependency_names()),
        );
        if is_workspace {
            if let Some(package) = workspace_package_from_json(&dir, package, Some(&visible)) {
                packages.push(package);
            }
        }
    }

    Ok(IndexedWorkspaceMap::new(
        WorkspaceMap { packages },
        metadata.root_dependency_names,
        manifest_dependency_names,
    ))
}

fn sorted_dependency_names(names: &std::collections::HashSet<String>) -> Vec<String> {
    let mut names = names.iter().cloned().collect::<Vec<_>>();
    names.sort();
    names
}
