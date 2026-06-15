pub(crate) fn slash_node_name(node: &NodeId, root: &Path) -> String {
    match node {
        NodeId::File(p) => no_mistakes::codebase::ts_source::relative_slash_path(root, p),
        NodeId::Symbol { file, symbol } => {
            let rel = no_mistakes::codebase::ts_source::relative_slash_path(root, file);
            format!("{}#{}", rel, symbol)
        }
        NodeId::Module(specifier) => specifier.clone(),
        NodeId::QueueJob { queue_file, job } => {
            let rel = no_mistakes::codebase::ts_source::relative_slash_path(root, queue_file);
            format!("{}#{}", rel, job)
        }
    }
}

pub(crate) fn relative_path(root: &Path, absolute: &Path) -> String {
    no_mistakes::codebase::ts_source::relative_slash_path(root, absolute)
}

fn changed_start_nodes(graph: &DepGraph, changed: &Path, include_symbols: bool) -> Vec<NodeId> {
    symbol_aware_start_nodes(graph, changed, None, include_symbols)
}

pub(crate) fn symbol_aware_start_nodes(
    graph: &DepGraph,
    file: &Path,
    symbol: Option<&String>,
    include_symbols: bool,
) -> Vec<NodeId> {
    if let Some(symbol) = symbol.filter(|_| include_symbols) {
        return vec![NodeId::Symbol {
            file: file.to_path_buf(),
            symbol: symbol.clone(),
        }];
    }
    let file_node = NodeId::File(file.to_path_buf());
    let mut starts = vec![file_node.clone()];
    if include_symbols {
        if let Some(neighbors) = graph.dependencies_of_node(&file_node) {
            starts.extend(neighbors.iter().filter_map(|(node, _)| match node {
                NodeId::Symbol {
                    file: symbol_file, ..
                } if symbol_file == file => Some(node.clone()),
                _ => None,
            }));
        }
    }
    starts.sort();
    starts.dedup();
    starts
}

/// Custom BFS path finder in the reverse (dependents) direction.
/// Returns reachable test nodes, and a map of node -> (parent, edge_kind) for shortest paths.
#[allow(clippy::type_complexity)]
pub(crate) fn bfs_path_find(
    graph: &DepGraph,
    start: &NodeId,
    test_filter: &TestFileFilter,
    root: &Path,
) -> (
    Vec<(NodeId, Vec<EdgeKind>)>,
    HashMap<NodeId, (NodeId, EdgeKind)>,
) {
    let mut queue = VecDeque::new();
    let mut parents: HashMap<NodeId, (NodeId, EdgeKind)> = HashMap::new();
    let mut visited = HashSet::new();
    let mut owner_widened_files = HashSet::new();
    let mut reachable = Vec::new();

    queue.push_back(start.clone());
    visited.insert(start.clone());

    while let Some(current) = queue.pop_front() {
        if let NodeId::File(p) = &current {
            if current != *start && test_filter.is_match(root, p) {
                let mut edge_path = Vec::new();
                let mut curr_node = current.clone();
                while let Some((parent, kind)) = parents.get(&curr_node) {
                    edge_path.push(*kind);
                    curr_node = parent.clone();
                }
                edge_path.reverse();
                reachable.push((current.clone(), edge_path));
            }
        }

        if let Some(neighbors) = graph.dependents_of_node(&current) {
            for (neighbor, kind) in neighbors {
                if owner_widened_files.contains(&current)
                    && !owner_widened_neighbor_allowed(
                        root,
                        test_filter,
                        graph,
                        neighbor,
                        neighbors,
                    )
                {
                    continue;
                }
                if let (NodeId::Symbol { file, .. }, NodeId::File(neighbor_file)) =
                    (&current, neighbor)
                {
                    if current == *start
                        && file == neighbor_file
                        && !test_filter.is_match(root, neighbor_file)
                    {
                        continue;
                    }
                }
                if !visited.contains(neighbor) {
                    if let (NodeId::Symbol { file, .. }, NodeId::File(neighbor_file)) =
                        (&current, neighbor)
                    {
                        if file == neighbor_file {
                            owner_widened_files.insert(neighbor.clone());
                        }
                    }
                    visited.insert(neighbor.clone());
                    parents.insert(neighbor.clone(), (current.clone(), *kind));
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    (reachable, parents)
}

pub(crate) fn path_confidence(edges: &[EdgeKind]) -> Confidence {
    let mut conf = Confidence::High;
    for edge in edges {
        match edge {
            EdgeKind::HttpCall
            | EdgeKind::ProcessSpawn
            | EdgeKind::QueueEnqueue
            | EdgeKind::QueueWorker
            | EdgeKind::RouteRef
            | EdgeKind::Layout
            | EdgeKind::RouteTest
            | EdgeKind::Selector
            | EdgeKind::AssetImport
            | EdgeKind::ReactRender
            | EdgeKind::PackageDependency
            | EdgeKind::SwiftPackageDependency
            | EdgeKind::TerraformReference
            | EdgeKind::TerraformModuleRef
            | EdgeKind::TerraformOutputRef => return Confidence::Low,
            EdgeKind::DynamicImport => conf = Confidence::Medium,
            _ => {}
        }
    }
    conf
}

pub(crate) fn impact_reason_label(edge: EdgeKind) -> &'static str {
    match edge {
        EdgeKind::Import
        | EdgeKind::TypeImport
        | EdgeKind::DynamicImport
        | EdgeKind::Require
        | EdgeKind::WorkspaceImport => "dependency",
        EdgeKind::PackageDependency => "package-json dependency",
        EdgeKind::RouteRef | EdgeKind::RouteTest => "route",
        EdgeKind::Layout => "layout",
        EdgeKind::TestOf => "test",
        EdgeKind::QueueEnqueue | EdgeKind::QueueWorker => "queue",
        EdgeKind::MarkdownLink => "md",
        EdgeKind::CiInvocation => "ci",
        EdgeKind::HttpCall => "http",
        EdgeKind::ProcessSpawn => "process",
        EdgeKind::AssetImport => "asset",
        EdgeKind::ReactRender => "react-render",
        EdgeKind::Selector => "selector",
        EdgeKind::SwiftImport | EdgeKind::SwiftReference => "swift",
        EdgeKind::SwiftPackageDependency => "swift package dependency",
        EdgeKind::TerraformReference => "terraform-ref",
        EdgeKind::TerraformModuleRef => "terraform-module",
        EdgeKind::TerraformOutputRef => "terraform-output",
    }
}
