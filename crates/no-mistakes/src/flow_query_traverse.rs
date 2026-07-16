#[derive(Clone, Copy)]
enum TraverseDirection {
    Deps,
    Dependents,
}

struct Traversal<'a> {
    graph: &'a DepGraph,
    root: &'a Path,
    max_depth: usize,
    allowed: Option<&'a HashSet<EdgeKind>>,
    nodes: &'a mut BTreeMap<String, FlowNode>,
    edges: &'a mut BTreeSet<FlowEdge>,
}

impl Traversal<'_> {
    fn traverse(&mut self, target: &NodeId, direction: TraverseDirection) {
        let mut queue = VecDeque::from([(target.clone(), 0usize)]);
        let mut seen = BTreeMap::from([(node_key(target, self.root), 0usize)]);
        while let Some((node, depth)) = queue.pop_front() {
            if crate::invocation::check_timeout().is_err() {
                break;
            }
            if depth >= self.max_depth {
                continue;
            }
            let neighbors = match direction {
                TraverseDirection::Deps => self.graph.dependencies_of_node(&node),
                TraverseDirection::Dependents => self.graph.dependents_of_node(&node),
            };
            let Some(neighbors) = neighbors else {
                continue;
            };
            let skip_symbol_owner_bridge = matches!(&node, NodeId::Symbol { .. }) && depth == 0;
            for (neighbor, kind) in neighbors {
                if matches!(direction, TraverseDirection::Dependents) && skip_symbol_owner_bridge {
                    if let (NodeId::Symbol { file: owner, .. }, NodeId::File(neighbor_file)) =
                        (&node, neighbor)
                    {
                        if neighbor_file == owner {
                            continue;
                        }
                    }
                }
                if self.allowed.is_some_and(|allowed| !allowed.contains(kind)) {
                    continue;
                }
                let next_depth = depth + 1;
                insert_node(self.nodes, neighbor, self.root, next_depth);
                let (from, to) = match direction {
                    TraverseDirection::Deps => {
                        (node_key(&node, self.root), node_key(neighbor, self.root))
                    }
                    TraverseDirection::Dependents => {
                        (node_key(neighbor, self.root), node_key(&node, self.root))
                    }
                };
                self.edges.insert(FlowEdge {
                    from,
                    to,
                    kind: kind.as_str(),
                });
                let key = node_key(neighbor, self.root);
                let should_visit = match seen.get(&key) {
                    Some(existing) => next_depth < *existing,
                    None => true,
                };
                if should_visit {
                    seen.insert(key, next_depth);
                    queue.push_back((neighbor.clone(), next_depth));
                }
            }
        }
    }
}

fn insert_node(nodes: &mut BTreeMap<String, FlowNode>, node: &NodeId, root: &Path, depth: usize) {
    let key = node_key(node, root);
    nodes
        .entry(key)
        .and_modify(|existing| existing.depth = existing.depth.min(depth))
        .or_insert_with(|| flow_node(node, root, depth));
}

fn flow_node(node: &NodeId, root: &Path, depth: usize) -> FlowNode {
    let id = node_key(node, root);
    match node {
        NodeId::File(path) => FlowNode {
            id,
            kind: "file",
            depth,
            file: Some(relative(root, path)),
            symbol: None,
            module: None,
            queue_file: None,
            job: None,
        },
        NodeId::Symbol { file, symbol } => FlowNode {
            id,
            kind: "symbol",
            depth,
            file: Some(relative(root, file)),
            symbol: Some(symbol.clone()),
            module: None,
            queue_file: None,
            job: None,
        },
        NodeId::Module(module) => FlowNode {
            id,
            kind: "module",
            depth,
            file: None,
            symbol: None,
            module: Some(module.clone()),
            queue_file: None,
            job: None,
        },
        NodeId::QueueJob { queue_file, job } => FlowNode {
            id,
            kind: "queue-job",
            depth,
            file: None,
            symbol: None,
            module: None,
            queue_file: Some(relative(root, queue_file)),
            job: Some(job.clone()),
        },
    }
}

fn resolve_target(root: &Path, raw: &str) -> NodeId {
    let (file, symbol) = parse_entrypoint(raw);
    let path = if file.is_absolute() {
        file
    } else {
        root.join(file)
    };
    let path = normalize_path(&path);
    match symbol {
        Some(symbol) => NodeId::Symbol { file: path, symbol },
        None => NodeId::File(path),
    }
}

fn node_key(node: &NodeId, root: &Path) -> String {
    node.display_name(root).replace('\\', "/")
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
