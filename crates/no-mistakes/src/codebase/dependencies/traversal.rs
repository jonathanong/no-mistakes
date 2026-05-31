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

fn resolve_entrypoints_with_files(
    raw_entrypoints: &[PathBuf],
    root: &Path,
    cwd: &Path,
    graph_files: &graph::GraphFiles,
    include_symbols: bool,
) -> Vec<Entrypoint> {
    let workspace =
        crate::codebase::workspaces::load_from_files(root, graph_files.all()).unwrap_or_default();
    let root_dependencies = root_dependency_names(root);
    raw_entrypoints
        .iter()
        .map(|raw| {
            let raw_str = raw.to_string_lossy();
            let (raw_file, symbol) = parse_entrypoint(&raw_str);
            let raw_for_node = raw_file.to_string_lossy().to_string();
            let file = if raw_file.is_absolute() {
                raw_file
            } else {
                let from_root = root.join(&raw_file);
                if from_root.exists() {
                    from_root
                } else {
                    cwd.join(&raw_file)
                }
            };
            let normalized = crate::codebase::ts_resolver::normalize_path(&file);
            let mut node =
                resolve_entrypoint_node(&raw_for_node, &normalized, &workspace, &root_dependencies);
            let file = match &node {
                NodeId::File(path) => path.clone(),
                NodeId::Symbol { file, .. } => file.clone(),
                _ => normalized,
            };
            if include_symbols {
                if let (NodeId::File(file), Some(symbol)) = (&node, &symbol) {
                    node = NodeId::Symbol {
                        file: file.clone(),
                        symbol: symbol.clone(),
                    };
                }
            }
            Entrypoint { file, node, symbol }
        })
        .collect()
}

fn resolve_entrypoint_node(
    raw: &str,
    path: &Path,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
    root_dependencies: &std::collections::HashSet<String>,
) -> NodeId {
    if path.is_dir() {
        if let Some(entry) = package_dir_entry(path, workspace) {
            return NodeId::File(entry);
        }
    }
    if workspace.resolve_specifier(raw).is_none()
        && raw_package_name(raw).is_some_and(|name| root_dependencies.contains(&name))
    {
        return NodeId::Module(raw.to_string());
    }
    if path.exists() || raw.starts_with('.') || Path::new(raw).is_absolute() {
        return NodeId::File(path.to_path_buf());
    }
    if let Some(entry) = workspace.resolve_specifier(raw) {
        return NodeId::File(entry);
    }
    if raw_looks_like_source_file(raw, path, root_dependencies) {
        return NodeId::File(path.to_path_buf());
    }
    NodeId::Module(raw.to_string())
}

fn validate_direction(direction: &Direction, entrypoints: &[Entrypoint]) -> Result<()> {
    if matches!(direction, Direction::Deps) {
        for ep in entrypoints {
            if ep.symbol.is_some() && !matches!(ep.node, NodeId::Symbol { .. }) {
                bail!(
                    "#symbol targeting (e.g. `file.mts#exportName`) is only supported \
                     in the `dependents` direction unless --symbols is enabled."
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
    if ctx.build_plan.symbols {
        return graph::DepGraph::build_with_plan_and_files(
            ctx.root,
            ctx.tsconfig,
            ctx.build_plan,
            ctx.graph_files,
        )
        .dependents_of_symbol_nodes(roots, depth, ctx.allowed);
    }
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
