use dashmap::DashSet;
use rayon::Scope;
use std::sync::Mutex;

/// Expand every reachable import once on the Rayon pool, without a BFS wave
/// barrier. `no-mistakes check` keeps cores busy with one `par_iter` over the
/// indexable universe; this does the same for the lazy reachable set by
/// spawning each newly discovered file immediately.
fn lazy_import_walk_parallel(
    input: LazyImportBuild<'_>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> LazyImportWalk {
    let LazyImportBuild {
        roots,
        tsconfig,
        tsconfig_catalog,
        graph_files,
        allowed,
        facts,
        workspace,
        import_resolution_cache,
        ..
    } = input;
    let resolver = crate::codebase::ts_resolver::ProjectImportResolver::new(
        tsconfig,
        tsconfig_catalog,
        graph_files,
        import_resolution_cache,
        session,
    );
    let fact_plan = facts.collect_plan;
    let visited: DashSet<NodeId> = DashSet::new();
    let edges = Mutex::new(Vec::new());
    let collected_facts = Mutex::new(Vec::new());
    let nodes = Mutex::new(Vec::new());
    let root_nodes: FxHashSet<NodeId> = roots.iter().cloned().collect();
    let ctx = ParallelExpand {
        resolver: &resolver,
        workspace,
        graph_files,
        allowed,
        facts,
        session,
        root_nodes: &root_nodes,
        visited: &visited,
        edges: &edges,
        collected_facts: &collected_facts,
        nodes: &nodes,
    };
    rayon::scope(|scope| {
        for root in roots {
            if !visited.insert(root.clone()) {
                continue;
            }
            scope.spawn(|scope| expand_reachable(scope, root.clone(), &ctx));
        }
    });
    let nodes = nodes
        .into_inner()
        .unwrap_or_else(|poison| poison.into_inner());
    let collected = collected_facts
        .into_inner()
        .unwrap_or_else(|poison| poison.into_inner());
    let edges = edges
        .into_inner()
        .unwrap_or_else(|poison| poison.into_inner());
    session.record_work("traversal.lazy_nodes", nodes.len() as u64);
    session.record_work("traversal.lazy_parallel_expand", 1);
    LazyImportWalk {
        entries: Vec::new(),
        facts: TsFactMap::from_iter_with_plan(collected, fact_plan)
            .into_iter()
            .collect(),
        edges,
        nodes,
    }
}

struct ParallelExpand<'a> {
    resolver: &'a crate::codebase::ts_resolver::ProjectImportResolver<'a>,
    workspace: &'a crate::codebase::workspaces::IndexedWorkspaceMap,
    graph_files: &'a GraphFiles,
    allowed: Option<&'a HashSet<EdgeKind>>,
    facts: LazyImportFacts<'a>,
    session: &'a crate::codebase::analysis_session::AnalysisSession,
    root_nodes: &'a FxHashSet<NodeId>,
    visited: &'a DashSet<NodeId>,
    edges: &'a Mutex<Vec<CanonicalEdge<NodeId, EdgeKind>>>,
    collected_facts: &'a Mutex<Vec<(PathBuf, TsFileFacts)>>,
    nodes: &'a Mutex<Vec<NodeId>>,
}

fn expand_reachable<'scope, 'a: 'scope>(
    scope: &Scope<'scope>,
    node: NodeId,
    ctx: &'scope ParallelExpand<'a>,
) {
    if crate::invocation::check_timeout().is_err() {
        return;
    }
    ctx.nodes
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .push(node.clone());
    let expanded = expand_import_node(
        &node,
        ctx.resolver,
        ctx.workspace,
        ctx.graph_files,
        ctx.allowed,
        ctx.facts,
        ctx.session,
    );
    if let Some(facts) = expanded.collected {
        ctx.collected_facts
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .push(facts);
    }
    let mut spawned = Vec::new();
    {
        let mut edges = ctx
            .edges
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        for (neighbor, kind) in expanded.neighbors {
            if is_symbol_owner_bridge(&node, &neighbor) && !ctx.root_nodes.contains(&node) {
                continue;
            }
            edges.push(CanonicalEdge::new(node.clone(), neighbor.clone(), kind));
            if ctx.visited.insert(neighbor.clone()) {
                spawned.push(neighbor);
            }
        }
    }
    for neighbor in spawned {
        scope.spawn(|scope| expand_reachable(scope, neighbor, ctx));
    }
}
