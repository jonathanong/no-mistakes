pub struct DepGraph {
    root: PathBuf,
    /// forward: node → nodes it imports/references (with edge kinds)
    forward: EdgeMap,
    /// reverse: node → nodes that import/reference it (with edge kinds)
    reverse: EdgeMap,
    parse_errors: HashMap<PathBuf, String>,
}

impl DepGraph {
    pub(crate) fn build_with_plan_files_config_and_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        graph_files: &GraphFiles,
        config_path: Option<&Path>,
        facts: Option<&dyn TsFactLookup>,
    ) -> Result<Self> {
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
        let fallback_facts = (!fact_plan.is_empty()).then_some(facts).flatten().and_then(|facts| {
            let facts_cover_plan = facts.covers_ts_fact_plan(fact_plan);
            let missing = graph_files
                .indexable()
                .iter()
                .filter(|path| !facts_cover_plan || facts.get_ts_facts(path).is_none())
                .cloned()
                .collect::<Vec<_>>();
            (!missing.is_empty()).then(|| {
                collect_ts_facts_with_context(&missing, fact_plan, &fact_context)
            })
        });
        let fallback_lookup = facts.zip(fallback_facts.as_ref()).map(|(primary, fallback)| {
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
            .or_else(|| {
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
        let parse_errors = if fact_plan.is_empty() {
            HashMap::new()
        } else {
            facts
                .map(|facts| {
                files
                    .iter()
                    .filter_map(|path| {
                        facts
                            .get_ts_facts(path)
                            .and_then(|file_facts| file_facts.parse_error.as_ref())
                            .map(|error| (path.clone(), error.clone()))
                    })
                    .collect()
                })
                .unwrap_or_default()
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

        let mut graph = Self {
            root: root.to_path_buf(),
            forward,
            reverse,
            parse_errors,
        };
        if plan.playwright_selectors {
            let selector_edges = crate::perf_trace::trace("graph.playwright_selectors", || {
                collect_playwright_selector_edges_with_graph(
                    root,
                    config_path,
                    &graph_files.all,
                    facts,
                    plan.route_imports.then_some(&graph),
                    plan.route_imports.then_some(tsconfig),
                )
            })?;
            merge_edges(&mut graph.forward, &mut graph.reverse, selector_edges);
            sort_adjacency_lists(&mut graph.forward, &mut graph.reverse);
        }
        Ok(graph)
    }

    pub(crate) fn build_with_plan_file_list_config_and_check_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        files: Vec<PathBuf>,
        config_path: Option<&Path>,
        facts: &crate::codebase::check_facts::CheckFactMap,
    ) -> Result<Self> {
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
    ) -> Result<Self> {
        Self::build_with_plan_file_list_config_and_check_facts(
            root, tsconfig, plan, files, None, facts,
        )
    }
}
