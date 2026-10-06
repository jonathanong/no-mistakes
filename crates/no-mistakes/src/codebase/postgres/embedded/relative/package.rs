use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::workspaces::IndexedWorkspaceMap;
use std::path::{Path, PathBuf};

/// Package portion of a bare specifier. Relative and absolute specifiers are `None`.
pub(crate) fn package_name(specifier: &str) -> Option<&str> {
    if specifier.is_empty() || specifier.starts_with('.') || specifier.starts_with('/') {
        return None;
    }
    if let Some(rest) = specifier.strip_prefix('@') {
        let (scope, after) = rest.split_once('/')?;
        if scope.is_empty() {
            return None;
        }
        let name = after.split('/').next().filter(|name| !name.is_empty())?;
        let end = 1 + scope.len() + 1 + name.len();
        return Some(&specifier[..end]);
    }
    let name = specifier
        .split('/')
        .next()
        .filter(|name| !name.is_empty())?;
    Some(name)
}

/// Directory of the package `specifier` names. `None` when that root cannot be determined.
pub(crate) fn package_root_for_specifier(
    specifier: &str,
    importing_file: &Path,
    workspace: &IndexedWorkspaceMap,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf>,
) -> Option<PathBuf> {
    let name = package_name(specifier)?;
    if let Some(package) = workspace.package_by_name(name) {
        return Some(package.dir.clone());
    }
    let resolved = resolve(specifier, importing_file)?;
    resolved.ancestors().skip(1).find_map(|dir| {
        workspace
            .package_by_dir(dir)
            .map(|package| package.dir.clone())
    })
}

pub(super) fn path_inside(file: &Path, root: &Path) -> bool {
    let file = normalize_path(file);
    let root = normalize_path(root);
    file.starts_with(&root) && file != root
}
