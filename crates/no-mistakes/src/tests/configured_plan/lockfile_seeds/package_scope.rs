use super::*;
use no_mistakes::codebase::dependencies::graph::EdgeKind;

pub(super) struct ScopedImporterStart {
    pub(super) node: NodeId,
    prefix: Option<(NodeId, EdgeKind)>,
}

pub(super) fn scoped_importer_start_nodes(
    graph: &DepGraph,
    package: &NodeId,
    manifest_scopes: &[PathBuf],
    workspace_map: &WorkspaceMap,
) -> Vec<ScopedImporterStart> {
    if manifest_scopes.is_empty() {
        return vec![ScopedImporterStart {
            node: package.clone(),
            prefix: None,
        }];
    }
    graph
        .dependents_of_node(package)
        .into_iter()
        .flatten()
        .filter_map(|(node, kind)| {
            node.as_file()
                .filter(|path| {
                    manifest_scopes
                        .iter()
                        .any(|manifest| importer_belongs_to_manifest(path, manifest, workspace_map))
                })
                .map(|_| ScopedImporterStart {
                    node: node.clone(),
                    prefix: Some((package.clone(), *kind)),
                })
        })
        .collect()
}

pub(super) fn prefix_package_path(
    root: &Path,
    start: &ScopedImporterStart,
    mut path: Vec<String>,
    mut edges: Vec<EdgeKind>,
) -> (Vec<String>, Vec<EdgeKind>) {
    if path.is_empty() {
        path.push(slash_node_name(&start.node, root));
    }
    if let Some((package, kind)) = &start.prefix {
        path.insert(0, slash_node_name(package, root));
        edges.insert(0, *kind);
    }
    (path, edges)
}

fn importer_belongs_to_manifest(
    importer: &Path,
    manifest: &Path,
    workspace_map: &WorkspaceMap,
) -> bool {
    let owner = manifest
        .parent()
        .expect("package manifest has a parent directory");
    let nearest_workspace = workspace_map
        .packages
        .iter()
        .filter(|package| importer.starts_with(&package.dir))
        .max_by_key(|package| package.dir.components().count());
    nearest_workspace.map_or_else(
        || importer.starts_with(owner),
        |package| package.dir == owner,
    )
}
