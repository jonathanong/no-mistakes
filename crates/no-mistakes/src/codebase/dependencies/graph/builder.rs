pub struct DepGraph {
    root: PathBuf,
    /// forward: node → nodes it imports/references (with edge kinds)
    forward: EdgeMap,
    /// reverse: node → nodes that import/reference it (with edge kinds)
    reverse: EdgeMap,
}

impl DepGraph {
    pub(crate) fn build_with_plan_files_config_and_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        graph_files: &GraphFiles,
        config_path: Option<&Path>,
        facts: Option<&dyn TsFactLookup>,
    ) -> Self {
        let config_options = graph_config_options_for_plan_with_config(root, plan, config_path);
        let resolver = ImportResolver::new(tsconfig).with_visible(graph_files.visible());
        let fact_plan = effective_ts_fact_plan(plan, config_options.as_ref());
        let fact_context = ts_fact_context_from_options(root, plan, config_options.as_ref());
        let owned_facts = if !fact_plan.is_empty() && facts.is_none() {
            Some(collect_ts_facts_with_context(
                graph_files.indexable(),
                fact_plan,
                &fact_context,
            ))
        } else {
            None
        };
        let facts: Option<&dyn TsFactLookup> = facts.or_else(|| {
            owned_facts
                .as_ref()
                .map(|facts| facts as &dyn TsFactLookup)
        });

        let mut forward: EdgeMap = HashMap::new();
        let mut reverse: EdgeMap = HashMap::new();

        let files = &graph_files.indexable;

        // Pre-populate all known file nodes.
        for f in files {
            forward.entry(NodeId::File(f.clone())).or_default();
        }

        let parsed_imports = if plan.imports || plan.workspace || plan.assets {
            let facts = facts.expect(
                "TS import facts are collected when import, workspace, or asset edges are requested",
            );
            collect_parsed_imports_from_facts(files, facts)
        } else {
            Vec::new()
        };

        let workspace = if plan.imports || plan.workspace || plan.package || plan.symbols {
            crate::codebase::workspaces::load_from_files(root, &graph_files.all).unwrap_or_default()
        } else {
            Default::default()
        };

        if plan.imports {
            let import_edges =
                collect_import_edges(&parsed_imports, &resolver, &workspace, graph_files);
            merge_edges(&mut forward, &mut reverse, import_edges);
        }

        if plan.workspace {
            let workspace_edges =
                collect_workspace_edges(&parsed_imports, &resolver, &workspace, graph_files);
            merge_edges(&mut forward, &mut reverse, workspace_edges);
        }

        if plan.package {
            let workspace_manifest_edges =
                collect_workspace_manifest_edges(&graph_files.all, &workspace, graph_files);
            merge_edges(&mut forward, &mut reverse, workspace_manifest_edges);
        }

        if plan.assets {
            merge_edges(
                &mut forward,
                &mut reverse,
                collect_asset_edges(&parsed_imports, &resolver, graph_files),
            );
        }

        if plan.symbols {
            let symbol_edges = collect_symbol_edges(
                root,
                files,
                &graph_files.all,
                facts.expect("TS symbol facts are collected when symbol edges are requested"),
                &resolver,
                &workspace,
                config_options.as_ref(),
            );
            merge_edges(&mut forward, &mut reverse, symbol_edges);
        }

        if plan.tests {
            let test_edges = collect_test_edges(
                root,
                files,
                config_options
                    .as_ref()
                    .and_then(|options| options.test_filter.as_ref()),
            );
            merge_edges(&mut forward, &mut reverse, test_edges);
        }

        if plan.markdown {
            let md_edges = collect_md_edges(&graph_files.all, graph_files);
            merge_edges(&mut forward, &mut reverse, md_edges);
        }

        if plan.ci {
            add_ci_edges(root, &graph_files.all, &mut forward, &mut reverse);
        }

        if plan.routes {
            let route_edges = collect_route_edges(
                root,
                tsconfig,
                &graph_files.all,
                facts,
                config_options.as_ref(),
            );
            merge_edges(&mut forward, &mut reverse, route_edges);
        }

        if plan.queues {
            add_queue_edges(
                root,
                &resolver,
                files,
                facts,
                config_options.as_ref(),
                &mut forward,
                &mut reverse,
            );
        }

        if plan.playwright_routes {
            let playwright_edges = collect_playwright_route_edges(root, &graph_files.all);
            merge_edges(&mut forward, &mut reverse, playwright_edges);
        }

        if plan.playwright_selectors {
            let selector_edges = collect_playwright_selector_edges(root, &graph_files.all);
            merge_edges(&mut forward, &mut reverse, selector_edges);
        }

        let edge_inputs = GraphEdgeBuildInputs {
            root,
            tsconfig,
            plan,
            graph_files,
            config_options: config_options.as_ref(),
        };
        merge_http_process_edges(&edge_inputs, facts, &mut forward, &mut reverse);

        if plan.react {
            let react_edges = collect_react_render_edges(root, facts, graph_files.indexable());
            merge_edges(&mut forward, &mut reverse, react_edges);
        }

        merge_swift_edges(&edge_inputs, &mut forward, &mut reverse);
        merge_terraform_edges(&edge_inputs, &mut forward, &mut reverse);

        sort_adjacency_lists(&mut forward, &mut reverse);

        Self {
            root: root.to_path_buf(),
            forward,
            reverse,
        }
    }

    pub(crate) fn build_with_plan_file_list_and_check_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        files: Vec<PathBuf>,
        facts: &crate::codebase::check_facts::CheckFactMap,
    ) -> Self {
        let graph_files = GraphFiles::from_files(files);
        Self::build_with_plan_files_and_facts(
            root,
            tsconfig,
            plan,
            &graph_files,
            Some(facts as &dyn TsFactLookup),
        )
    }
}
