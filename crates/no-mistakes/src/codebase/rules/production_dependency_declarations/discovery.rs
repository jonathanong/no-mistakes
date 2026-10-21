//! Manual `package.json` discovery and file-to-package ownership, kept
//! separate from `scan.rs`'s orchestration for the 200-line source cap.

use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::SourceStore;
use crate::codebase::workspaces::{self, WorkspaceMap};
use crate::fx::PathSet;
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
    visible: &std::sync::OnceLock<HashSet<PathBuf>>,
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
            let Some(package) = workspaces::load_root_package_from_visible_sources(
                package_root,
                visible.get_or_init(|| normalized_visible_files(files)),
                sources,
            )?
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

/// Keep the application's exact scoped visibility shared across package loading
/// and reachability, retaining the manifest loader's standard hash semantics.
fn normalized_visible_files(files: &[PathBuf]) -> HashSet<PathBuf> {
    files
        .iter()
        .map(|path| {
            workspaces::record_preparation!(visible_path);
            normalize_path(path)
        })
        .collect()
}

/// The nearest workspace package (by ancestor directory) owning each file,
/// for files that have one.
pub(super) fn compute_owners(
    workspace: &WorkspaceMap,
    files: &[PathBuf],
) -> HashMap<PathBuf, PathBuf> {
    let directories: PathSet = workspace
        .packages
        .iter()
        .map(|package| {
            workspaces::record_preparation!(package_dir);
            normalize_path(&package.dir)
        })
        .collect();
    files
        .iter()
        .filter_map(|file| {
            let file = normalize_path(file);
            nearest_package_dir(&directories, &file).map(|dir| (file, dir))
        })
        .collect()
}

fn nearest_package_dir(directories: &PathSet, file: &Path) -> Option<PathBuf> {
    file.ancestors().find_map(|dir| {
        workspaces::record_preparation!(ancestor_lookup);
        directories.get(dir).cloned()
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
