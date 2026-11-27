use crate::codebase::postgres::{package_name, package_root_for_specifier};
use crate::codebase::ts_resolver::ImportResolver;
use crate::codebase::workspace_resolution::IndexedWorkspaceMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
