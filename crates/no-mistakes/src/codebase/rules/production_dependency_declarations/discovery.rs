//! Manual `package.json` discovery and file-to-package ownership, kept
//! separate from `scan.rs`'s orchestration for the 200-line source cap.

use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::workspaces::WorkspaceMap;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

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

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
