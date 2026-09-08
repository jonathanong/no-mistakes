fn roots_with_existing_queue_jobs(
    roots: &[NodeId],
    entrypoints: &[Entrypoint],
    graph: &graph::DepGraph,
    interner: &PathInterner,
) -> Vec<NodeId> {
    roots_with_existing_queue_jobs_by(
        roots,
        entrypoints,
        |node| graph.has_reverse_node(node),
        interner,
    )
}

fn roots_with_exported_symbol_roots(roots: &[NodeId], graph: &graph::DepGraph) -> Vec<NodeId> {
    roots_with_exported_symbol_roots_by(roots, |node| graph.dependencies_of_node(node))
}

fn roots_with_call_caller_symbols(
    roots: &[NodeId],
    facts: &dyn graph::TsFactLookup,
    interner: &PathInterner,
) -> Vec<NodeId> {
    let mut seen = std::collections::HashSet::new();
    let mut expanded = Vec::new();
    for root in roots {
        push_unique_root(&mut seen, &mut expanded, root.clone());
        let NodeId::File(file) = root else {
            continue;
        };
        let Some(file_facts) = facts.get_ts_facts(file) else {
            continue;
        };
        let callers: std::collections::BTreeSet<_> = file_facts
            .call_reachability
            .iter()
            .filter_map(|call| call.caller.as_deref())
            .collect();
        for caller in callers {
            push_unique_root(
                &mut seen,
                &mut expanded,
                NodeId::symbol_in(interner, file.clone(), caller),
            );
        }
    }
    expanded
}

fn roots_with_exported_callable_symbols(
    roots: &[NodeId],
    graph: &graph::DepGraph,
) -> Vec<NodeId> {
    let mut seen = std::collections::HashSet::new();
    let mut expanded = Vec::new();
    for root in roots {
        push_unique_root(&mut seen, &mut expanded, root.clone());
        let NodeId::File(file) = root else {
            continue;
        };
        for callable in graph.call_target_symbols_in_file(file) {
            push_unique_root(&mut seen, &mut expanded, callable);
        }
    }
    expanded
}

fn roots_with_exported_symbol_roots_by<'a, F>(roots: &[NodeId], dependencies_of: F) -> Vec<NodeId>
where
    F: Fn(&NodeId) -> Option<&'a Vec<(NodeId, EdgeKind)>>,
{
    let mut seen = std::collections::HashSet::new();
    let mut expanded = Vec::new();
    for root in roots {
        push_unique_root(&mut seen, &mut expanded, root.clone());
        let NodeId::File(root_file) = root else {
            continue;
        };
        let Some(dependencies) = dependencies_of(root) else {
            continue;
        };
        for (node, _) in dependencies {
            if matches!(node, NodeId::Symbol { file, .. } if file == root_file) {
                push_unique_root(&mut seen, &mut expanded, node.clone());
            }
        }
    }
    expanded
}

fn push_unique_root(
    seen: &mut std::collections::HashSet<NodeId>,
    expanded: &mut Vec<NodeId>,
    node: NodeId,
) {
    if seen.insert(node.clone()) {
        expanded.push(node);
    }
}

fn roots_with_existing_queue_jobs_by<F>(
    roots: &[NodeId],
    entrypoints: &[Entrypoint],
    has_reverse_node: F,
    interner: &PathInterner,
) -> Vec<NodeId>
where
    F: Fn(&NodeId) -> bool,
{
    let mut roots = roots.to_vec();
    for entrypoint in entrypoints {
        let Some(symbol) = &entrypoint.symbol else {
            continue;
        };
        if matches!(entrypoint.node, NodeId::Module(_)) {
            continue;
        }
        let queue_job = NodeId::queue_job_in(interner, entrypoint.file.clone(), symbol.clone());
        if has_reverse_node(&queue_job) {
            roots.push(queue_job);
        }
    }
    roots
}
