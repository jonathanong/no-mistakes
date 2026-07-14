fn load_package(
    dir: &Path,
    visible_files: Option<&std::collections::HashSet<PathBuf>>,
) -> Result<Option<WorkspacePackage>> {
    load_package_with_sources(dir, visible_files, WorkspaceSources::Filesystem)
}

fn load_package_with_sources(
    dir: &Path,
    visible_files: Option<&std::collections::HashSet<PathBuf>>,
    sources: WorkspaceSources<'_>,
) -> Result<Option<WorkspacePackage>> {
    let pkg_path = dir.join("package.json");
    if !workspace_path_is_file(&pkg_path, visible_files) {
        return Ok(None);
    }

    let package = sources
        .parse_json(&pkg_path)
        .ok()
        .and_then(|value| serde_json::from_value((*value).clone()).ok())
        .unwrap_or_default();
    Ok(workspace_package_from_json(dir, package, visible_files))
}

fn workspace_package_from_json(
    dir: &Path,
    package: PackageJson,
    visible_files: Option<&std::collections::HashSet<PathBuf>>,
) -> Option<WorkspacePackage> {
    let name = package
        .name
        .as_ref()
        .filter(|name| !name.is_empty())?
        .clone();

    // Resolve the entry file in priority order: exports > module > main > types.
    let entry = resolve_entry_with_visibility(dir, &package, visible_files);

    Some(WorkspacePackage {
        name,
        dir: dir.to_path_buf(),
        entry,
        exports: package.exports,
        imports: package.imports,
    })
}

pub fn load_root_package(dir: &Path) -> Result<Option<WorkspacePackage>> {
    let files = crate::codebase::ts_source::discover_visible_paths(dir);
    load_root_package_from_files(dir, &files)
}

#[doc(hidden)]
pub fn load_root_package_from_files(
    dir: &Path,
    files: &[PathBuf],
) -> Result<Option<WorkspacePackage>> {
    let manifest = normalize_path(&dir.join("package.json"));
    if !files.iter().any(|path| normalize_path(path) == manifest) {
        return Ok(None);
    }
    let visible: std::collections::HashSet<PathBuf> =
        files.iter().map(|path| normalize_path(path)).collect();
    load_package(dir, Some(&visible))
}

#[doc(hidden)]
pub fn load_root_package_from_source_store(
    dir: &Path,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Option<WorkspacePackage>> {
    let manifest = normalize_path(&dir.join("package.json"));
    if !files.iter().any(|path| normalize_path(path) == manifest) {
        return Ok(None);
    }
    let visible: std::collections::HashSet<PathBuf> =
        files.iter().map(|path| normalize_path(path)).collect();
    load_package_with_sources(dir, Some(&visible), WorkspaceSources::Store(sources))
}

fn resolve_entry_with_visibility(
    dir: &Path,
    pkg: &PackageJson,
    visible_files: Option<&std::collections::HashSet<PathBuf>>,
) -> Option<PathBuf> {
    // Check `exports` first (supports both string and `{".": ...}` forms).
    if let Some(exports) = &pkg.exports {
        if let Some(entry_str) = exports_to_entry_path(exports) {
            let candidate = normalize_path(&dir.join(entry_str));
            if let Some(resolved) = resolve_workspace_path(&candidate, visible_files) {
                return Some(resolved);
            }
        }
    }

    // module field (ESM)
    if let Some(module) = &pkg.module {
        let candidate = normalize_path(&dir.join(module));
        if let Some(resolved) = resolve_workspace_path(&candidate, visible_files) {
            return Some(resolved);
        }
    }

    // main field (CJS/default)
    if let Some(main) = &pkg.main {
        let candidate = normalize_path(&dir.join(main));
        if let Some(resolved) = resolve_workspace_path(&candidate, visible_files) {
            return Some(resolved);
        }
    }

    // types field
    if let Some(types) = &pkg.types {
        let candidate = normalize_path(&dir.join(types));
        if workspace_path_is_file(&candidate, visible_files) {
            return Some(candidate);
        }
    }

    // Fallback: try common entry file names.
    for name in &[
        "src/index.mts",
        "src/index.ts",
        "src/index.tsx",
        "index.mts",
        "index.ts",
    ] {
        let p = normalize_path(&dir.join(name));
        if workspace_path_is_file(&p, visible_files) {
            return Some(p);
        }
    }

    None
}

fn resolve_workspace_path(
    path: &Path,
    visible_files: Option<&std::collections::HashSet<PathBuf>>,
) -> Option<PathBuf> {
    match visible_files {
        Some(visible) => try_resolve_from_visible(path, visible),
        None => try_resolve(path),
    }
}
