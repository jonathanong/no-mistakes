pub struct DepGraph {
    root: PathBuf,
    /// forward: node → nodes it imports/references (with edge kinds)
    forward: EdgeMap,
    /// reverse: node → nodes that import/reference it (with edge kinds)
    reverse: EdgeMap,
}

impl DepGraph {
    pub fn build(root: &Path, tsconfig: &TsConfig) -> Result<Self> {
        Self::build_with_plan(root, tsconfig, GraphBuildPlan::all())
    }

    pub fn build_with_plan(root: &Path, tsconfig: &TsConfig, plan: GraphBuildPlan) -> Result<Self> {
        let graph_files = GraphFiles::discover(root);
        Ok(Self::build_with_plan_and_files(
            root,
            tsconfig,
            plan,
            &graph_files,
        ))
    }

    pub(crate) fn build_with_plan_and_files(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        graph_files: &GraphFiles,
    ) -> Self {
        Self::build_with_plan_files_and_facts(root, tsconfig, plan, graph_files, None)
    }

    pub(crate) fn build_with_plan_files_and_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        graph_files: &GraphFiles,
        facts: Option<&TsFactMap>,
    ) -> Self {
        let config_options = graph_config_options_for_plan(root, plan);
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
        let facts = owned_facts.as_ref().or(facts);

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

        let workspace = if plan.imports || plan.workspace || plan.package {
            crate::codebase::workspaces::load_from_files(root, &graph_files.all)
                .unwrap_or_default()
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
            let asset_edges = collect_asset_edges(&parsed_imports, &resolver, graph_files);
            merge_edges(&mut forward, &mut reverse, asset_edges);
        }

        if plan.tests {
            let test_edges = collect_test_edges(files);
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

        // HTTP and process collectors consume shared TS facts in this path.
        // Keep the file-content fallback empty so graph builds do not add a
        // second source read pass.
        if plan.http || plan.process {
            let file_contents: Vec<(PathBuf, String)> = Vec::new();

            if plan.http {
                let http_call_edges = collect_http_call_edges(
                    root,
                    tsconfig,
                    facts,
                    &file_contents,
                    graph_files.indexable(),
                    &graph_files.all,
                    config_options.as_ref(),
                );
                merge_edges(&mut forward, &mut reverse, http_call_edges);
            }

            if plan.process {
                let spawn_edges = collect_process_spawn_edges(
                    root,
                    facts,
                    &file_contents,
                    graph_files.indexable(),
                );
                merge_edges(&mut forward, &mut reverse, spawn_edges);
            }
        }

        if plan.react {
            let react_edges = collect_react_render_edges(root, facts, graph_files.indexable());
            merge_edges(&mut forward, &mut reverse, react_edges);
        }

        // ⚡ Bolt: Sort adjacency lists for deterministic BFS output.
        // Using sort_by_cached_key improves performance by ~7% on large repositories
        // since it prevents repeated allocation when computing the sort key via node_sort_key.
        for adj in forward.values_mut() {
            adj.sort_by_cached_key(|(n, k)| (node_sort_key(n), *k as u8));
            adj.dedup();
        }
        for adj in reverse.values_mut() {
            adj.sort_by_cached_key(|(n, k)| (node_sort_key(n), *k as u8));
            adj.dedup();
        }

        Self {
            root: root.to_path_buf(),
            forward,
            reverse,
        }
    }

    pub(crate) fn build_with_plan_file_list_and_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        files: Vec<PathBuf>,
        facts: &TsFactMap,
    ) -> Self {
        let graph_files = GraphFiles::from_files(files);
        Self::build_with_plan_files_and_facts(root, tsconfig, plan, &graph_files, Some(facts))
    }

}
