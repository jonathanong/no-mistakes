use crate::codebase::workspaces;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) fn load_workspace(
    root: &Path,
    target_roots: &[PathBuf],
    files: &[PathBuf],
) -> Result<workspaces::WorkspaceMap> {
    let mut roots: Vec<&Path> = Vec::new();
    roots.push(root);
    for target_root in target_roots {
        if !roots.contains(&target_root.as_path()) {
            roots.push(target_root);
        }
    }
    let mut packages = BTreeMap::new();
    for target_root in roots {
        if files.contains(&target_root.join("package.json")) {
            if let Some(package) = workspaces::load_root_package(target_root)? {
                packages.insert(package.name.clone(), package);
            }
        }
        for package in workspaces::load_from_files(target_root, files)?.packages {
            packages.insert(package.name.clone(), package);
        }
    }
    Ok(workspaces::WorkspaceMap {
        packages: packages.into_values().collect(),
    })
}
