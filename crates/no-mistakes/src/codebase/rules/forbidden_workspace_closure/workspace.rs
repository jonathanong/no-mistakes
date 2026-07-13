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
        let manifests = files.iter().filter(|path| {
            path.starts_with(target_root)
                && path.file_name().and_then(|name| name.to_str()) == Some("package.json")
        });
        for manifest in manifests {
            let Some(package_root) = manifest.parent() else {
                continue;
            };
            let Some(package) = workspaces::load_root_package_from_files(package_root, files)?
            else {
                continue;
            };
            packages.insert(package.name.clone(), package);
        }
    }
    Ok(workspaces::WorkspaceMap {
        packages: packages.into_values().collect(),
    })
}
