use super::{alias, Dependency, PackageNode};
use crate::codebase::{package_deps, workspaces};
use std::collections::BTreeMap;

pub(super) fn manifest_nodes_with_sources(
    workspace: &workspaces::WorkspaceMap,
    dependency_types: &[&str],
    sources: &crate::codebase::ts_source::SourceStore,
) -> BTreeMap<String, PackageNode> {
    workspace
        .packages
        .iter()
        .map(|package| {
            let manifest = package.dir.join("package.json");
            let deps = package_deps::dependency_entries_from_source_store(
                &manifest,
                dependency_types,
                sources,
            )
            .into_iter()
            .map(|entry| Dependency {
                workspace_name: alias::workspace_dependency_name_with_sources(
                    &entry.name,
                    &entry.specifier,
                    &package.dir,
                    workspace,
                    sources,
                ),
                resolved_name: alias::resolved_dependency_name_with_sources(
                    &entry.specifier,
                    &package.dir,
                    workspace,
                    sources,
                ),
                name: entry.name,
                field: entry.field,
            })
            .collect();
            (package.name.clone(), PackageNode { manifest, deps })
        })
        .collect()
}
