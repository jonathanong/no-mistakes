fn has_callable_relationship(allowed: Option<&std::collections::HashSet<EdgeKind>>) -> bool {
    allowed.is_some_and(|allowed| {
        allowed.contains(&EdgeKind::Call) || allowed.contains(&EdgeKind::AssertionOwner)
    })
}

fn call_roots(entrypoints: &[Entrypoint]) -> Vec<graph::CallRoot> {
    entrypoints
        .iter()
        .filter_map(|entrypoint| {
            let file = entrypoint.node.as_file()?.to_path_buf();
            Some(match &entrypoint.symbol {
                Some(symbol) => graph::CallRoot::Function {
                    file,
                    symbol: symbol.clone(),
                },
                None => graph::CallRoot::File(file),
            })
        })
        .collect()
}

/// Mixed relationship traversals retain file roots for file-level edges and
/// expanded roots for callable edges. Callable-only symbol queries omit the
/// file node so projection does not also walk unrelated top-level calls.
fn roots_with_call_roots(
    roots: &[NodeId],
    call_roots: Vec<NodeId>,
    entrypoints: &[Entrypoint],
    allowed: Option<&std::collections::HashSet<EdgeKind>>,
) -> Vec<NodeId> {
    let callable_only = allowed.is_some_and(|kinds| {
        !kinds.is_empty()
            && kinds
                .iter()
                .all(|kind| matches!(kind, EdgeKind::Call | EdgeKind::AssertionOwner))
    });
    let mut combined = Vec::with_capacity(roots.len() + call_roots.len());
    combined.extend(
        roots
            .iter()
            .filter(|root| keep_callable_projection_file_root(root, entrypoints, callable_only))
            .cloned(),
    );
    combined.extend(call_roots);
    combined.sort();
    combined.dedup();
    combined
}

fn keep_callable_projection_file_root(
    root: &NodeId,
    entrypoints: &[Entrypoint],
    callable_only: bool,
) -> bool {
    if !callable_only {
        return true;
    }
    let NodeId::File(file) = root else {
        return true;
    };
    !entrypoints.iter().any(|entrypoint| {
        entrypoint.symbol.is_some() && entrypoint.node.as_file() == Some(file.as_ref())
    })
}

fn report_dependents(
    graph: &graph::DepGraph,
    roots: &[NodeId],
    depth: Option<usize>,
    allowed: Option<&std::collections::HashSet<EdgeKind>>,
    excluded: &std::collections::HashSet<PathBuf>,
) -> Vec<graph::NodeEntry> {
    if excluded.is_empty() {
        graph.dependents_of(roots, depth, allowed)
    } else {
        graph.dependents_of_excluding_files(roots, depth, allowed, excluded)
    }
}

fn report_symbol_nodes(
    graph: &graph::DepGraph,
    roots: &[NodeId],
    depth: Option<usize>,
    allowed: Option<&std::collections::HashSet<EdgeKind>>,
    excluded: &std::collections::HashSet<PathBuf>,
) -> Vec<graph::NodeEntry> {
    if excluded.is_empty() {
        graph.dependents_of_symbol_nodes(roots, depth, allowed)
    } else {
        graph.dependents_of_symbol_nodes_excluding_files(roots, depth, allowed, excluded)
    }
}

fn ordinary_exclusions<'a>(
    shared: &'a SharedTraversalContext,
    entrypoints: &[Entrypoint],
) -> std::borrow::Cow<'a, std::collections::HashSet<PathBuf>> {
    let excluded = &shared.ordinary_excluded_files;
    let retained = &shared.ordinary_retained_roots;
    if !retained.iter().any(|file| excluded.contains(file))
        && !entrypoints
            .iter()
            .any(|entry| excluded.contains(&entry.file))
    {
        return std::borrow::Cow::Borrowed(excluded);
    }
    let mut selected = excluded.clone();
    selected.retain(|file| !retained.contains(file));
    for entrypoint in entrypoints {
        selected.remove(&entrypoint.file);
    }
    std::borrow::Cow::Owned(selected)
}
