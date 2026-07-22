pub(crate) fn collect_and_filter_entries_shared(
    args: &TraverseArgs,
    direction: Direction,
    cwd_early: &Path,
    shared: &mut SharedTraversalContext,
) -> Result<TraversalResult> {
    shared.session.record_work("traversal.requests", 1);
    shared.tsconfig_catalog.clear_runtime_diagnostics();
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
    let mut allowed_key = allowed
        .iter()
        .flat_map(|allowed| allowed.iter().copied())
        .collect::<Vec<_>>();
    allowed_key.sort();
    let traversal_key = TraversalCacheKey {
        generation: shared.analysis_generation,
        dependents: matches!(direction, Direction::Dependents),
        entrypoints: entrypoints
            .iter()
            .map(|entrypoint| {
                (
                    entrypoint.file.clone(),
                    entrypoint.node.clone(),
                    entrypoint.symbol.clone(),
                )
            })
            .collect(),
        depth: args.depth,
        allowed: allowed_key,
        include_symbols: args.include_symbols,
        import_only,
    };
    let cached_entries = shared
        .traversal_results
        .iter()
        .find(|(cached_key, _)| cached_key == &traversal_key)
        .map(|(_, traversal)| traversal.clone());
    let cache_hit = cached_entries.is_some();
    let entries = if let Some(cached) = cached_entries {
        shared.session.record_work("traversal.reuses", 1);
        shared
            .tsconfig_catalog
            .replay_runtime_diagnostics(&cached.runtime_diagnostics);
        cached.entries
    } else {
        let symbol_index = if matches!(direction, Direction::Dependents)
            && any_symbol
            && !args.include_symbols
        {
            Some(shared.symbol_index()?)
        } else {
            None
        };
        let entries = collect_uncached_entries(
            UncachedTraversalRequest {
                args,
                direction,
                entrypoints: &entrypoints,
                roots: &roots,
                allowed: allowed.as_ref(),
                import_only,
                any_symbol,
                symbol_index: symbol_index.as_deref(),
            },
            shared,
        )?;
        shared.session.record_work("traversal.computations", 1);
        entries
    };
    crate::invocation::check_timeout()?;
    let tsconfig_provenance = entrypoints
        .iter()
        .filter_map(|entrypoint| entrypoint.node.as_file())
        .map(|file| shared.tsconfig_catalog.provenance_for(file))
        .map(|mut provenance| {
            provenance.importer = provenance
                .importer
                .strip_prefix(&shared.root)
                .unwrap_or(&provenance.importer)
                .to_path_buf();
            provenance.config = provenance.config.map(|config| visible_provenance_path(shared, config));
            provenance
        })
        .collect();
    let runtime_diagnostics = shared.tsconfig_catalog.runtime_diagnostics();
    if !cache_hit {
        shared.traversal_results.push((
            traversal_key,
            CachedTraversal {
                entries: entries.clone(),
                runtime_diagnostics: runtime_diagnostics.clone(),
            },
        ));
    }
    let entries = apply_filters(
        entries,
        args,
        &shared.root,
        &shared.config,
        &shared.tsconfig,
        shared.dataset.visible_paths(),
        shared.prepared_test_projects.as_ref(),
    )?;
    shared
        .session
        .record_work("traversal.nodes", entries.len() as u64);
    let diagnostics = shared
        .tsconfig_build_diagnostics
        .iter()
        .cloned()
        .chain(runtime_diagnostics)
        .map(|mut diagnostic| {
            let root_text = shared.root.to_string_lossy();
            diagnostic.detail = diagnostic
                .detail
                .replace(&format!("{root_text}/"), "");
            diagnostic.config = diagnostic.config.map(|config| {
                config
                    .strip_prefix(&shared.root)
                    .unwrap_or(&config)
                    .to_path_buf()
            });
            diagnostic.file = diagnostic.file.map(|file| {
                file.strip_prefix(&shared.root)
                    .unwrap_or(&file)
                    .to_path_buf()
            });
            diagnostic.candidates = diagnostic
                .candidates
                .into_iter()
                .map(|candidate| {
                    candidate
                        .strip_prefix(&shared.root)
                        .unwrap_or(&candidate)
                        .to_path_buf()
                })
                .collect();
            diagnostic
        })
        .collect();
    Ok(TraversalResult {
        entries,
        root: shared.root.clone(),
        diagnostics,
        tsconfig_provenance,
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
