//! Manual `package.json` discovery and file-to-package ownership, kept
//! separate from `scan.rs`'s orchestration for the 200-line source cap.

use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::SourceStore;
use crate::codebase::workspaces::{self, WorkspaceMap};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Discover every `package.json` under `workspace_roots`, bypassing the root
/// manifest's `workspaces` glob membership so a package not (yet) listed
/// there is still checked. `workspace_roots` is the sole discovery scope —
/// the check root is never implicitly included (see the `workspaceRoots`
/// config validation message in the parent module).
pub(super) fn load_workspace(
    workspace_roots: &[PathBuf],
    files: &[PathBuf],
    sources: &SourceStore,
) -> anyhow::Result<WorkspaceMap> {
    let mut roots: Vec<&Path> = Vec::new();
    for workspace_root in workspace_roots {
        if !roots.contains(&workspace_root.as_path()) {
            roots.push(workspace_root);
        }
    }
    let mut packages = std::collections::BTreeMap::new();
    for workspace_root in roots {
        let manifests = files.iter().filter(|path| {
            path.starts_with(workspace_root)
                && path.file_name().and_then(|name| name.to_str()) == Some("package.json")
        });
        for manifest in manifests {
            let Some(package_root) = manifest.parent() else {
                continue;
            };
            let Some(package) =
                workspaces::load_root_package_from_source_store(package_root, files, sources)?
            else {
                continue;
            };
            packages.insert(package.name.clone(), package);
        }
    }
    Ok(WorkspaceMap::from_packages(
        packages.into_values().collect(),
    ))
}

/// The nearest workspace package (by ancestor directory) owning each file,
/// for files that have one.
pub(super) fn compute_owners(
    workspace: &WorkspaceMap,
    files: &[PathBuf],
) -> HashMap<PathBuf, PathBuf> {
    files
        .iter()
        .filter_map(|file| {
            let file = normalize_path(file);
            nearest_package_dir(workspace, &file).map(|dir| (file, dir))
        })
        .collect()
}

fn nearest_package_dir(workspace: &WorkspaceMap, file: &Path) -> Option<PathBuf> {
    file.ancestors().find_map(|dir| {
        workspace
            .package_by_dir(dir)
            .map(|package| normalize_path(&package.dir))
    })
}

/// Invert `owners` into package dir -> its owned files, for reachability
/// closure membership checks.
pub(super) fn group_by_package(
    owners: &HashMap<PathBuf, PathBuf>,
) -> HashMap<PathBuf, HashSet<PathBuf>> {
    let mut grouped: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();
    for (file, dir) in owners {
        grouped.entry(dir.clone()).or_default().insert(file.clone());
    }
    grouped
}

#[cfg(test)]
mod tests;
