pub(crate) fn collect_and_filter_entries_shared(
    args: &TraverseArgs,
    direction: Direction,
    cwd_early: &Path,
    shared: &mut SharedTraversalContext,
) -> Result<TraversalResult> {
    let explicit_roots = explicit_existing_entry_files(args, &shared.root, cwd_early);
    shared.add_explicit_roots(&explicit_roots);
    let workspace = shared.dataset.workspace();
    let entrypoints = resolve_entrypoints_with_files_and_workspace(EntrypointResolution {
        raw_entrypoints: &args.files,
        symbol_entrypoints: &args.file_symbols,
        structured_entrypoints: &args.file_entrypoints_are_structured,
        root: &shared.root,
        cwd: cwd_early,
        graph_files: &shared.graph_files,
        include_symbols: args.include_symbols,
        workspace: &workspace,
    });
    validate_direction(&direction, &entrypoints)?;

    let allowed = relationship_filter(&args.relationships);
    let roots: Vec<NodeId> = entrypoints
        .iter()
        .map(|entrypoint| entrypoint.node.clone())
        .collect();
    let import_only = !args.include_symbols && relationships_are_import_only(&args.relationships);
    let any_symbol = entrypoints
        .iter()
        .any(|entrypoint| entrypoint.symbol.is_some());
    let symbol_index =
        if matches!(direction, Direction::Dependents) && any_symbol && !args.include_symbols {
            Some(shared.symbol_index()?)
        } else {
            None
        };
    let root = shared.root.clone();
    let entries = match direction {
        Direction::Deps if import_only => {
            let sources = shared.dataset.sources_for(&shared.root);

            let (entries, collected) =
                graph::lazy_import_deps_of_with_files_facts_workspace_and_resolution_cache(
                    graph::LazyImportBuild {
                        roots: &roots,
                        tsconfig: &shared.tsconfig,
                        max_depth: args.depth,
                        graph_files: &shared.graph_files,
                        allowed: allowed.as_ref(),
                        facts: graph::LazyImportFacts::new(
                            shared
                                .facts
                                .as_ref()
                                .map(|facts| facts as &dyn graph::TsFactLookup),
                            shared.fact_plan,
                            &shared.fact_context,
                        )
                        .with_source_store(&sources)
                        .retain_collected(),
                        workspace: &workspace,
                        import_resolution_cache: Some(&shared.import_resolution_cache),
                    },
                );
            if !collected.is_empty() {
                if let Some(facts) = &mut shared.facts {
                    facts.extend(collected);
                } else {
                    shared.facts = Some(
                        crate::codebase::ts_source::facts::TsFactMap::from_iter_with_plan(
                            collected,
                            shared.fact_plan,
                        ),
                    );
                }
                shared.invalidate_analysis_caches();
            }
            entries
        }
        Direction::Deps if shared.build_plan.symbols && !args.include_symbols => shared
            .request_graph_without_symbols(allowed.as_ref())?
            .deps_of(&roots, args.depth, allowed.as_ref()),
        Direction::Deps => shared
            .graph()?
            .deps_of(&roots, args.depth, allowed.as_ref()),
        Direction::Dependents if args.include_symbols => {
            let graph = shared.graph()?;
            let roots = roots_with_existing_queue_jobs(&roots, &entrypoints, graph);
            let roots = roots_with_exported_symbol_roots(&roots, graph);
            graph.dependents_of_symbol_nodes(&roots, args.depth, allowed.as_ref())
        }
        Direction::Dependents if any_symbol && shared.build_plan.symbols => {
            let graph = shared.request_graph_without_symbols(allowed.as_ref())?;
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
            shared.graph()?,
            symbol_index
                .as_ref()
                .expect("symbol index is built for symbol dependents"),
        ),
        Direction::Dependents if shared.build_plan.symbols && !args.include_symbols => shared
            .request_graph_without_symbols(allowed.as_ref())?
            .dependents_of(&roots, args.depth, allowed.as_ref()),
        Direction::Dependents => {
            shared
                .graph()?
                .dependents_of(&roots, args.depth, allowed.as_ref())
        }
    };
    let entries = apply_filters(
        entries,
        args,
        &shared.root,
        &shared.config,
        &shared.tsconfig,
        shared.dataset.visible_paths(),
        shared.prepared_test_projects.as_ref(),
    )?;

    Ok(TraversalResult {
        entries,
        root: shared.root.clone(),
    })
}

fn explicit_existing_entry_files(args: &TraverseArgs, root: &Path, cwd: &Path) -> Vec<PathBuf> {
    args.files
        .iter()
        .enumerate()
        .filter_map(|(index, raw)| {
            let structured = args
                .file_entrypoints_are_structured
                .get(index)
                .copied()
                .unwrap_or(false);
            let raw_file = if structured {
                raw.clone()
            } else {
                parse_entrypoint(&raw.to_string_lossy()).0
            };
            let path = if raw_file.is_absolute() {
                raw_file
            } else {
                let from_root = root.join(&raw_file);
                if from_root.exists() {
                    from_root
                } else {
                    cwd.join(raw_file)
                }
            };
            path.is_file()
                .then(|| crate::codebase::ts_resolver::normalize_path(&path))
        })
        .collect()
}
