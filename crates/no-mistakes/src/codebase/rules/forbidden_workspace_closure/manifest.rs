use super::{alias, Dependency, PackageNode};
use crate::codebase::{package_deps, workspaces};
use std::collections::BTreeMap;

pub(super) fn manifest_nodes(
    workspace: &workspaces::WorkspaceMap,
    dependency_types: &[&str],
) -> BTreeMap<String, PackageNode> {
    workspace
        .packages
        .iter()
        .map(|package| {
            let manifest = package.dir.join("package.json");
            let deps = package_deps::dependency_entries(&manifest, dependency_types)
                .into_iter()
                .map(|entry| Dependency {
                    name: entry.name,
                    resolved_name: alias::resolved_dependency_name(
                        &entry.specifier,
                        &package.dir,
                        workspace,
                    ),
                    field: entry.field,
                })
                .collect();
            (package.name.clone(), PackageNode { manifest, deps })
        })
        .collect()
}
