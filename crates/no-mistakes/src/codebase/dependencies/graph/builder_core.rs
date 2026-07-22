impl DepGraph {
    fn build_with_plan_files_options_and_facts(
        edge_inputs: GraphEdgeBuildInputs<'_>,
        facts: Option<&dyn TsFactLookup>,
        supplied_fact_policy: SuppliedFactPolicy,
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    ) -> Result<Self> {
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
        fact_context.set_visible_files(graph_files.visible().iter().cloned());
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
                    .is_some_and(|files| !same_graph_universe(files, graph_files.visible()));
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
                    graph_files.visible(),
                )
            });
        let facts: Option<&dyn TsFactLookup> = fallback_lookup
            .as_ref()
            .map(|lookup| lookup as &dyn TsFactLookup)
            .or(facts)
            .or_else(|| owned_facts.as_ref().map(|facts| facts as &dyn TsFactLookup));

        let mut forward: EdgeMap = HashMap::new();
        let mut reverse: EdgeMap = HashMap::new();
        let files = &graph_files.indexable;

        for file in files {
            forward.entry(NodeId::File(file.clone())).or_default();
        }

        let parsed_imports = parsed_imports_for_plan(plan, files, facts)?;
        crate::invocation::check_timeout()?;
        let needs_workspace = plan.imports || plan.workspace || plan.package || plan.symbols;
        let owned_workspace = (needs_workspace && supplied_workspace.is_none()).then(|| {
            crate::codebase::workspaces::load_indexed_from_files(root, &graph_files.all)
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
            },
        )?;
        crate::invocation::check_timeout()?;
        let mut graph = Self {
            root: root.to_path_buf(),
            edges: edge_index_from_maps(forward, reverse),
            parse_errors,
        };
        if plan.playwright_selectors {
            let snapshot = playwright_snapshot
                .as_ref()
                .expect("Playwright selector plan prepares a visible-path snapshot");
            let selector_edges = crate::perf_trace::trace("graph.playwright_selectors", || {
                collect_playwright_selector_edges_with_graph(
                    root,
                    config_path,
                    PlaywrightSelectorEdgeInputs {
                        all_files: &graph_files.all,
                        facts,
                        partial_graph: plan.route_imports.then_some(&graph),
                        graph_tsconfig: plan.route_imports.then_some(tsconfig),
                        snapshot,
                        prepared_settings: edge_inputs.playwright_settings,
                    },
                )
            })?;
            graph.merge_canonical_edges(selector_edges);
        }
        record_graph_observability(&graph, &session);
        Ok(graph)
    }
}
