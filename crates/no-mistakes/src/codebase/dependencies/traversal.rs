use super::*;

pub(super) struct TraversalCtx<'a> {
    pub(super) root: &'a Path,
    pub(super) tsconfig: &'a TsConfig,
    pub(super) graph_files: &'a graph::GraphFiles,
    pub(super) build_plan: graph::GraphBuildPlan,
    pub(super) allowed: Option<&'a std::collections::HashSet<EdgeKind>>,
    pub(super) symbols: bool,
}

pub(super) fn deps_entries(
    depth: Option<usize>,
    import_only: bool,
    roots: &[NodeId],
    entrypoints: &[Entrypoint],
    ctx: &TraversalCtx<'_>,
) -> Result<Vec<graph::NodeEntry>> {
    if has_call_relationship(ctx.allowed) {
        let graph = graph::DepGraph::build_with_plan_and_files(
            ctx.root,
            ctx.tsconfig,
            ctx.build_plan,
            ctx.graph_files,
        )?;
        let call_roots = graph.expand_call_roots(&call_roots(entrypoints));
        let roots = roots_with_call_roots(roots, call_roots);
        return Ok(graph.deps_of(&roots, depth, ctx.allowed));
    }
    if import_only {
        Ok(graph::lazy_import_deps_of_with_files(
            roots,
            ctx.root,
            ctx.tsconfig,
            depth,
            ctx.graph_files,
            ctx.allowed,
        ))
    } else {
        Ok(graph::DepGraph::build_with_plan_and_files(
            ctx.root,
            ctx.tsconfig,
            ctx.build_plan,
            ctx.graph_files,
        )?
        .deps_of(roots, depth, ctx.allowed))
    }
}

pub(super) fn get_entries(
    direction: Direction,
    roots: &[NodeId],
    entrypoints: &[Entrypoint],
    depth: Option<usize>,
    import_only: bool,
    ctx: &TraversalCtx<'_>,
) -> Result<Vec<graph::NodeEntry>> {
    match direction {
        Direction::Deps => deps_entries(depth, import_only, roots, entrypoints, ctx),
        Direction::Dependents => dependents_entries(entrypoints, roots, depth, ctx),
    }
}

pub(super) fn dependents_entries(
    entrypoints: &[Entrypoint],
    roots: &[NodeId],
    depth: Option<usize>,
    ctx: &TraversalCtx<'_>,
) -> Result<Vec<graph::NodeEntry>> {
    if has_call_relationship(ctx.allowed) {
        let graph = graph::DepGraph::build_with_plan_and_files(
            ctx.root,
            ctx.tsconfig,
            ctx.build_plan,
            ctx.graph_files,
        )?;
        let callable_roots = graph.expand_call_roots(&call_roots(entrypoints));
        let roots = roots_with_call_roots(roots, callable_roots);
        return Ok(graph.dependents_of(&roots, depth, ctx.allowed));
    }
    let any_symbol = entrypoints.iter().any(|e| e.symbol.is_some());
    if ctx.symbols {
        let graph = graph::DepGraph::build_with_plan_and_files(
            ctx.root,
            ctx.tsconfig,
            ctx.build_plan,
            ctx.graph_files,
        );
        let graph = graph?;
        let roots =
            roots_with_existing_queue_jobs(roots, entrypoints, &graph, &PathInterner::new());
        let roots = roots_with_exported_symbol_roots(&roots, &graph);
        return Ok(graph.dependents_of_symbol_nodes(&roots, depth, ctx.allowed));
    }
    let symbol_facts = any_symbol.then(|| {
        let mut fact_plan = ctx.build_plan.ts_fact_plan();
        fact_plan.imports = true;
        fact_plan.symbols = true;
        let mut fact_context =
            graph::test_support::ts_fact_context_for_plan(ctx.root, ctx.build_plan);
        fact_context.set_visible_file_set(ctx.graph_files.visible_path_set());
        crate::codebase::ts_source::facts::collect_ts_facts_with_context(
            ctx.graph_files.indexable(),
            fact_plan,
            &fact_context,
        )
    });
    crate::invocation::check_timeout()?;
    let graph = build_dependents_graph(ctx, symbol_facts.as_ref())?;
    if any_symbol {
        let facts = symbol_facts
            .as_ref()
            .expect("symbol facts are collected for symbol queries");
        let symbol_index =
            graph::SymbolIndex::build_from_facts(ctx.root, ctx.tsconfig, ctx.graph_files, facts);
        Ok(resolve_symbol_dependents(
            ctx.root,
            entrypoints,
            depth,
            ctx.allowed,
            &graph,
            &symbol_index,
        ))
    } else {
        Ok(graph.dependents_of(roots, depth, ctx.allowed))
    }
}

pub(super) fn has_call_relationship(allowed: Option<&std::collections::HashSet<EdgeKind>>) -> bool {
    allowed.is_some_and(|allowed| allowed.contains(&EdgeKind::Call))
}

pub(super) fn call_roots(entrypoints: &[Entrypoint]) -> Vec<graph::CallRoot> {
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

include!("traversal_mixed_relationships_tests.rs");

fn build_dependents_graph(
    ctx: &TraversalCtx<'_>,
    symbol_facts: Option<&crate::codebase::ts_source::facts::TsFactMap>,
) -> Result<graph::DepGraph> {
    match symbol_facts {
        Some(facts) => graph::DepGraph::build_with_plan_files_and_facts(
            ctx.root,
            ctx.tsconfig,
            ctx.build_plan,
            ctx.graph_files,
            Some(facts as &dyn graph::TsFactLookup),
        ),
        None => graph::DepGraph::build_with_plan_and_files(
            ctx.root,
            ctx.tsconfig,
            ctx.build_plan,
            ctx.graph_files,
        ),
    }
}
