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

        let edge_inputs = GraphEdgeBuildInputs {
            root,
            tsconfig,
            plan,
            graph_files,
            config_options: config_options.as_ref(),
            config_path,
        };
        collect_and_merge_all_edges(
            &edge_inputs,
            facts,
            &resolver,
            &parsed_imports,
            &workspace,
            &mut forward,
            &mut reverse,
        );

        sort_adjacency_lists(&mut forward, &mut reverse);

        Self {
            root: root.to_path_buf(),
            forward,
            reverse,
        }
    }

    pub(crate) fn build_with_plan_file_list_config_and_check_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        files: Vec<PathBuf>,
        config_path: Option<&Path>,
        facts: &crate::codebase::check_facts::CheckFactMap,
    ) -> Self {
        let graph_files = GraphFiles::from_files(files);
        Self::build_with_plan_files_config_and_facts(
            root,
            tsconfig,
            plan,
            &graph_files,
            config_path,
            Some(facts as &dyn TsFactLookup),
        )
    }

    pub(crate) fn build_with_plan_file_list_and_check_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        files: Vec<PathBuf>,
        facts: &crate::codebase::check_facts::CheckFactMap,
    ) -> Self {
        Self::build_with_plan_file_list_config_and_check_facts(
            root, tsconfig, plan, files, None, facts,
        )
    }
}
