pub(crate) struct SharedTraversalContext {
    root: PathBuf,
    tsconfig: TsConfig,
    graph_files: graph::GraphFiles,
    build_plan: graph::GraphBuildPlan,
    graph: Option<graph::DepGraph>,
    pub(crate) graph_builds: usize,
}

impl SharedTraversalContext {
    pub(crate) fn new(root: PathBuf, tsconfig: TsConfig, graph_files: graph::GraphFiles) -> Self {
        Self {
            root,
            tsconfig,
            graph_files,
            build_plan: graph::GraphBuildPlan::default(),
            graph: None,
            graph_builds: 0,
        }
    }

    pub(crate) fn include_plan(&mut self, plan: graph::GraphBuildPlan) {
        self.build_plan.include(plan);
    }

    fn graph(&mut self) -> &graph::DepGraph {
        if self.graph.is_none() {
            self.graph = Some(graph::DepGraph::build_with_plan_and_files(
                &self.root,
                &self.tsconfig,
                self.build_plan,
                &self.graph_files,
            ));
            self.graph_builds += 1;
        }
        self.graph.as_ref().expect("graph is initialized")
    }

    fn request_graph_without_symbols(
        &self,
        allowed: Option<&std::collections::HashSet<EdgeKind>>,
    ) -> graph::DepGraph {
        graph::DepGraph::build_with_plan_and_files(
            &self.root,
            &self.tsconfig,
            graph::GraphBuildPlan::from_allowed(allowed),
            &self.graph_files,
        )
    }
}

pub(crate) fn collect_and_filter_entries_shared(
    args: &TraverseArgs,
    direction: Direction,
    cwd_early: &Path,
    shared: &mut SharedTraversalContext,
) -> Result<TraversalResult> {
    let entrypoints = resolve_entrypoints_with_files(
        &args.files,
        &args.file_symbols,
        &args.file_entrypoints_are_structured,
        &shared.root,
        cwd_early,
        &shared.graph_files,
        args.include_symbols,
    );

    validate_direction(&direction, &entrypoints)?;

    let allowed = relationship_filter(&args.relationships);
    let roots: Vec<NodeId> = entrypoints.iter().map(|e| e.node.clone()).collect();
    let import_only = !args.include_symbols && relationships_are_import_only(&args.relationships);
    let any_symbol = entrypoints
        .iter()
        .any(|entrypoint| entrypoint.symbol.is_some());
    let symbol_index = if matches!(direction, Direction::Dependents) && any_symbol && !args.include_symbols
    {
        Some(graph::SymbolIndex::build_from_files(
            &shared.tsconfig,
            &shared.graph_files,
        ))
    } else {
        None
    };
    let root = shared.root.clone();
    let entries = match direction {
        Direction::Deps if import_only => graph::lazy_import_deps_of_with_files(
            &roots,
            &shared.root,
            &shared.tsconfig,
            args.depth,
            &shared.graph_files,
            allowed.as_ref(),
        ),
        Direction::Deps if shared.build_plan.symbols && !args.include_symbols => shared
            .request_graph_without_symbols(allowed.as_ref())
            .deps_of(&roots, args.depth, allowed.as_ref()),
        Direction::Deps => shared.graph().deps_of(&roots, args.depth, allowed.as_ref()),
        Direction::Dependents if args.include_symbols => {
            let graph = shared.graph();
            let roots = roots_with_existing_queue_jobs(&roots, &entrypoints, graph);
            let roots = roots_with_exported_symbol_roots(&roots, graph);
            graph.dependents_of_symbol_nodes(&roots, args.depth, allowed.as_ref())
        }
        Direction::Dependents if any_symbol && shared.build_plan.symbols => {
            let graph = shared.request_graph_without_symbols(allowed.as_ref());
            resolve_symbol_dependents(
                &root,
                &entrypoints,
                args.depth,
                allowed.as_ref(),
                &graph,
                symbol_index
                    .as_ref()
                    .expect("symbol index is built for symbol dependents"),
            )
        }
        Direction::Dependents if any_symbol => resolve_symbol_dependents(
            &root,
            &entrypoints,
            args.depth,
            allowed.as_ref(),
            shared.graph(),
            symbol_index
                .as_ref()
                .expect("symbol index is built for symbol dependents"),
        ),
        Direction::Dependents if shared.build_plan.symbols && !args.include_symbols => shared
            .request_graph_without_symbols(allowed.as_ref())
            .dependents_of(&roots, args.depth, allowed.as_ref()),
        Direction::Dependents => shared.graph().dependents_of(&roots, args.depth, allowed.as_ref()),
    };
    let entries = apply_filters(entries, args, &shared.root)?;

    Ok(TraversalResult {
        entries,
        root: shared.root.clone(),
    })
}
