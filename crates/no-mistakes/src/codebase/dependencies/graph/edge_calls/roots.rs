impl DepGraph {
    pub fn resolved_call_sites(&self) -> &[ResolvedCallSite] {
        &self.resolved_call_sites
    }
}

/// Root selector primitives consumed by call-policy checks. Test catalogs turn
/// into file roots in their prepared owner; this graph layer deliberately does
/// not discover runner configuration a second time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallRoot {
    File(std::path::PathBuf),
    Module(std::path::PathBuf),
    Function {
        file: std::path::PathBuf,
        symbol: String,
    },
    /// Files selected by the prepared Vitest project catalog for the requested
    /// project set. Catalog parsing remains owned by the request preparation.
    Vitest {
        files: Vec<std::path::PathBuf>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallTraversal {
    Direct,
    File,
    Transitive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallTrace {
    pub root: NodeId,
    pub target: NodeId,
    pub nodes: Vec<NodeId>,
}

impl DepGraph {
    /// Expand stable file/module/function roots against this prepared graph.
    /// Unknown files or symbols simply produce no roots; configuration
    /// validation belongs to the rule layer that owns diagnostics.
    pub fn expand_call_roots(&self, roots: &[CallRoot]) -> Vec<NodeId> {
        let mut expanded = roots
            .iter()
            .flat_map(|root| match root {
                CallRoot::Module(file) => {
                    let node = NodeId::file(crate::codebase::ts_resolver::normalize_path(file));
                    self.has_call_site_in_file(node.as_file().expect("file node"))
                        .then_some(node)
                        .into_iter()
                        .collect::<Vec<_>>()
                }
                CallRoot::File(file) => self.callable_file_roots(file),
                CallRoot::Function { file, symbol } => {
                    let node = NodeId::symbol(
                        crate::codebase::ts_resolver::normalize_path(file),
                        symbol.as_str(),
                    );
                    self.resolve_display_callable(&node).into_iter().collect()
                }
                CallRoot::Vitest { files } => files
                    .iter()
                    .flat_map(|file| self.callable_file_roots(file))
                    .collect::<Vec<_>>(),
            })
            .collect::<Vec<_>>();
        expanded.sort();
        expanded.dedup();
        expanded
    }

    fn callable_file_roots(&self, file: &std::path::Path) -> Vec<NodeId> {
        let file = crate::codebase::ts_resolver::normalize_path(file);
        let mut nodes = self
            .callable_nodes_by_file
            .get(&file)
            .cloned()
            .unwrap_or_default();
        let module = NodeId::file(&file);
        if self.has_call_site_in_file(&file) {
            nodes.push(module);
        }
        nodes.sort();
        nodes.dedup();
        nodes
    }

    fn has_call_site_in_file(&self, file: &std::path::Path) -> bool {
        self.resolved_call_sites.iter().any(|site| site.file == file)
    }

    /// Follow canonical call edges with deterministic shortest traces. `File`
    /// never crosses a source-file boundary; `Direct` is one call edge; and
    /// `Transitive` follows resolved calls until `max_depth` when supplied.
    pub fn call_traces(
        &self,
        roots: &[NodeId],
        traversal: CallTraversal,
        max_depth: Option<usize>,
    ) -> Vec<CallTrace> {
        let roots = normalize_nodes(
            &roots
                .iter()
                .flat_map(|root| self.resolve_display_callable(root))
                .collect::<Vec<_>>(),
        );
        let edges = self.traversal_edges().forward();
        let mut out = Vec::new();
        for root in roots {
            let root_file = root.as_file().map(std::path::Path::to_path_buf);
            let cap = match traversal {
                CallTraversal::Direct => Some(1),
                _ => max_depth,
            };
            let mut seen = std::collections::HashSet::new();
            let mut queue = std::collections::VecDeque::new();
            seen.insert(root.clone());
            queue.push_back((root.clone(), vec![root.clone()], 0usize));
            while let Some((node, path, depth)) = queue.pop_front() {
                if cap.is_some_and(|limit| depth >= limit) {
                    continue;
                }
                let Some(neighbors) = edges.get(&node) else {
                    continue;
                };
                for (neighbor, kind) in &neighbors.neighbors {
                    if *kind != EdgeKind::Call {
                        continue;
                    }
                    if traversal == CallTraversal::File
                        && neighbor.as_file() != root_file.as_deref()
                    {
                        continue;
                    }
                    if !seen.insert(neighbor.clone()) {
                        continue;
                    }
                    let mut next = path.clone();
                    next.push(neighbor.clone());
                    out.push(CallTrace {
                        root: root.clone(),
                        target: neighbor.clone(),
                        nodes: next.clone(),
                    });
                    queue.push_back((neighbor.clone(), next, depth + 1));
                }
            }
        }
        out.sort_by(|left, right| {
            (&left.root, &left.target, &left.nodes).cmp(&(&right.root, &right.target, &right.nodes))
        });
        out
    }

    /// A public/query symbol has no opaque identity. Resolve it only when its
    /// stable display name selects exactly one internal callable; ambiguous
    /// sibling declarations stay distinct and require a file root instead.
    fn resolve_display_callable(&self, node: &NodeId) -> Vec<NodeId> {
        let NodeId::Symbol {
            file,
            symbol,
            callable_id: None,
        } = node
        else {
            return vec![node.clone()];
        };
        let matches = self
            .callable_nodes_by_file
            .get(file.as_ref())
            .into_iter()
            .flatten()
            .filter(|candidate| {
                matches!(candidate, NodeId::Symbol { symbol: candidate_symbol, .. } if candidate_symbol == symbol)
            })
            .cloned()
            .collect::<Vec<_>>();
        if matches.len() == 1 {
            matches
        } else {
            Vec::new()
        }
    }
}
