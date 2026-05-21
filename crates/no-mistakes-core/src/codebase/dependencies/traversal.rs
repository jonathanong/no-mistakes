struct TraversalCtx<'a> {
    root: &'a Path,
    tsconfig: &'a TsConfig,
    graph_files: &'a graph::GraphFiles,
    build_plan: graph::GraphBuildPlan,
    allowed: Option<&'a std::collections::HashSet<EdgeKind>>,
}

fn resolve_tsconfig(args: &TraverseArgs, root: &Path) -> Result<TsConfig> {
    match args.tsconfig {
        Some(ref path) => crate::codebase::ts_resolver::load_tsconfig(path),
        None => match crate::codebase::ts_resolver::find_tsconfig(root) {
            Some(path) => crate::codebase::ts_resolver::load_tsconfig(&path),
            None => Ok(crate::codebase::ts_resolver::TsConfig {
                dir: root.to_path_buf(),
                paths: vec![],
                paths_dir: root.to_path_buf(),
                base_url: None,
            }),
        },
    }
}

fn resolve_root(args: &TraverseArgs, cwd: &Path) -> PathBuf {
    match &args.root {
        Some(p) => {
            if p.is_absolute() {
                p.clone()
            } else {
                cwd.join(p)
            }
        }
        None => cwd.to_path_buf(),
    }
}

fn resolve_entrypoints(raw_entrypoints: &[PathBuf], root: &Path, cwd: &Path) -> Vec<Entrypoint> {
    raw_entrypoints
        .iter()
        .map(|raw| {
            let raw_str = raw.to_string_lossy();
            let ep = parse_entrypoint(&raw_str);
            let file = if ep.file.is_absolute() {
                ep.file
            } else {
                let from_root = root.join(&ep.file);
                if from_root.exists() {
                    from_root
                } else {
                    cwd.join(ep.file)
                }
            };
            Entrypoint {
                file,
                symbol: ep.symbol,
            }
        })
        .collect()
}

fn validate_direction(direction: &Direction, entrypoints: &[Entrypoint]) -> Result<()> {
    if matches!(direction, Direction::Deps) {
        for ep in entrypoints {
            if ep.symbol.is_some() {
                bail!(
                    "#symbol targeting (e.g. `file.mts#exportName`) is only supported \
                     in the `dependents` direction. For `dependencies`, use a plain file path."
                );
            }
        }
    }
    Ok(())
}

fn deps_entries(
    depth: Option<usize>,
    import_only: bool,
    roots: &[NodeId],
    ctx: &TraversalCtx<'_>,
) -> Vec<graph::NodeEntry> {
    if import_only {
        graph::lazy_import_deps_of_with_files(roots, ctx.root, ctx.tsconfig, depth, ctx.graph_files, ctx.allowed)
    } else {
        graph::DepGraph::build_with_plan_and_files(
            ctx.root,
            ctx.tsconfig,
            ctx.build_plan,
            ctx.graph_files,
        )
        .deps_of(roots, depth, ctx.allowed)
    }
}

fn get_entries(
    direction: Direction,
    roots: &[NodeId],
    entrypoints: &[Entrypoint],
    depth: Option<usize>,
    import_only: bool,
    ctx: &TraversalCtx<'_>,
) -> Vec<graph::NodeEntry> {
    match direction {
        Direction::Deps => deps_entries(depth, import_only, roots, ctx),
        Direction::Dependents => dependents_entries(entrypoints, roots, depth, ctx),
    }
}

fn dependents_entries(
    entrypoints: &[Entrypoint],
    roots: &[NodeId],
    depth: Option<usize>,
    ctx: &TraversalCtx<'_>,
) -> Vec<graph::NodeEntry> {
    let any_symbol = entrypoints.iter().any(|e| e.symbol.is_some());
    let symbol_facts = any_symbol.then(|| {
        let mut fact_plan = ctx.build_plan.ts_fact_plan();
        fact_plan.imports = true;
        fact_plan.symbols = true;
        let fact_context = graph::ts_fact_context_for_plan(ctx.root, ctx.build_plan);
        crate::codebase::ts_source::facts::collect_ts_facts_with_context(
            ctx.graph_files.indexable(),
            fact_plan,
            &fact_context,
        )
    });
    let graph = build_dependents_graph(ctx, symbol_facts.as_ref());
    if any_symbol {
        let facts = symbol_facts
            .as_ref()
            .expect("symbol facts are collected for symbol queries");
        let symbol_index =
            graph::SymbolIndex::build_from_facts(ctx.tsconfig, ctx.graph_files, facts);
        resolve_symbol_dependents(
            ctx.root,
            entrypoints,
            depth,
            ctx.allowed,
            &graph,
            &symbol_index,
        )
    } else {
        graph.dependents_of(roots, depth, ctx.allowed)
    }
}
