{
        session.record_work("graph.builds", 1);
        let root = edge_inputs.root;
        let tsconfig = edge_inputs.tsconfig;
        let plan = edge_inputs.plan;
        let graph_files = edge_inputs.graph_files;
        let config_options = edge_inputs.config_options;
        let config_path = edge_inputs.config_path;
        let supplied_workspace = edge_inputs.workspace;
        let resolver = graph_import_resolver(&edge_inputs, &session);
        let fact_plan = effective_ts_fact_plan(plan, config_options);
        let mut fact_context = ts_fact_context_from_options(root, plan, config_options);
        fact_context.set_visible_file_set(graph_files.visible_path_set());
        let owned_facts = if !fact_plan.is_empty() && facts.is_none() {
            Some(collect_ts_facts_with_session_and_context(
                &session,
                graph_files.indexable(),
                fact_plan,
                &fact_context,
            ))
        } else {
            None
        };
        crate::invocation::check_timeout()?;
        let fallback_facts = match facts {
            Some(primary) => {
                let covers_plan = primary.covers_ts_fact_plan(fact_plan);
                let universe_mismatch = primary
                    .graph_files()
                    .is_some_and(|files| !same_graph_universe(files, graph_files));
                let missing = if fact_plan.is_empty() {
                    Vec::new()
                } else {
                    graph_files
                        .indexable()
                        .iter()
                        .filter(|path| primary.get_ts_facts(path).is_none())
                        .cloned()
                        .collect::<Vec<_>>()
                };

                match supplied_fact_policy {
                    SuppliedFactPolicy::RequireComplete if !covers_plan => {
                        anyhow::bail!(
                            "prepared graph facts do not cover the required TS fact plan"
                        );
                    }
                    SuppliedFactPolicy::RequireComplete if !missing.is_empty() => {
                        anyhow::bail!(
                            "prepared graph facts are missing {} indexable file(s)",
                            missing.len()
                        );
                    }
                    SuppliedFactPolicy::RequireComplete if universe_mismatch => {
                        anyhow::bail!(
                            "prepared graph fact universe does not match the requested graph files"
                        );
                    }
                    SuppliedFactPolicy::RequireComplete => None,
                    SuppliedFactPolicy::FillSparse => {
                        let fallback_paths = if !covers_plan || universe_mismatch {
                            graph_files.indexable().to_vec()
                        } else {
                            missing
                        };
                        (!fallback_paths.is_empty() || universe_mismatch).then(|| {
                            collect_ts_facts_with_session_and_context(
                                &session,
                                &fallback_paths,
                                fact_plan,
                                &fact_context,
                            )
                        })
                    }
                }
            }
            None => None,
        };
        let fallback_lookup = facts
            .zip(fallback_facts.as_ref())
            .map(|(primary, fallback)| {
                FallbackTsFactLookup::new(
                    primary,
                    fallback,
                    !primary.covers_ts_fact_plan(fact_plan),
                    graph_files.all(),
                    graph_files,
                )
            });
        let facts: Option<&dyn TsFactLookup> = fallback_lookup
            .as_ref()
            .map(|lookup| lookup as &dyn TsFactLookup)
            .or(facts)
            .or_else(|| owned_facts.as_ref().map(|facts| facts as &dyn TsFactLookup));

        let mut forward: EdgeMap = EdgeMap::default();
        let mut reverse: EdgeMap = EdgeMap::default();
        let mut resource_edge_details: ResourceEdgeDetails = fx_map();
        let mut resource_diagnostics = Vec::new();
        let mut callable_export_resolutions: FxHashMap<
            (PathBuf, String),
            ExportedCallableResolution,
        > = fx_map();
        let mut resolved_call_sites = Vec::new();
        let files = graph_files.indexable();

        for file in files {
            forward
                .entry(NodeId::file_in(&edge_inputs.interner, file))
                .or_default();
        }

        let parsed_imports = parsed_imports_for_plan(plan, files, facts)?;
        crate::invocation::check_timeout()?;
        let needs_workspace = plan.imports || plan.workspace || plan.package || plan.symbols;
        let owned_workspace = (needs_workspace && supplied_workspace.is_none()).then(|| {
            crate::codebase::workspaces::load_indexed_from_files(root, graph_files.all())
                .unwrap_or_default()
        });
        let empty_workspace = crate::codebase::workspaces::IndexedWorkspaceMap::default();
        let workspace = supplied_workspace
            .or(owned_workspace.as_ref())
            .unwrap_or(&empty_workspace);
        let parse_errors = graph_parse_errors(fact_plan, files, facts);

        let owned_playwright_snapshot = (plan.playwright_routes || plan.playwright_selectors)
            .then(|| {
                edge_inputs.visible_paths.is_none().then(|| {
                    crate::playwright::fsutil::VisiblePathSnapshot::from_paths(
                        root,
                        graph_files.all(),
                    )
                })
            })
            .flatten();
        let playwright_snapshot = edge_inputs
            .visible_paths
            .or(owned_playwright_snapshot.as_ref());

        collect_and_merge_all_edges(
            &edge_inputs,
            playwright_snapshot,
            facts,
            EdgeResolutionContext {
                resolver: &resolver,
                session: &session,
            },
            &parsed_imports,
            workspace,
            EdgeMaps {
                forward: &mut forward,
                reverse: &mut reverse,
                resource_edge_details: &mut resource_edge_details,
                resource_diagnostics: &mut resource_diagnostics,
                callable_export_resolutions: &mut callable_export_resolutions,
                resolved_call_sites: &mut resolved_call_sites,
            },
        )?;

        include!("finish.rs")
}
