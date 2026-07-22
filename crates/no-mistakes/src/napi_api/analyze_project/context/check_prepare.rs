struct SharedCheckContext {
    root: PathBuf,
    config_path: Option<PathBuf>,
    tsconfig_path: Option<PathBuf>,
    prepared: crate::check_runner::prepared::PreparedCheckInputs,
    plan: crate::codebase::check_facts::CheckFactPlan,
    playwright_fact_plan: Option<crate::codebase::check_facts::PlaywrightFactPlan>,
    fact_files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    fs_files: Vec<PathBuf>,
    prepared_graph: Option<crate::codebase::dependencies::graph::PreparedGraphConfig>,
    react_enabled: bool,
    queues_enabled: bool,
    unique_exports_enabled: bool,
    filesystem_rules_enabled: bool,
    forbidden_deps_enabled: bool,
    playwright_rules_enabled: bool,
    graph_plan: Option<crate::codebase::dependencies::graph::GraphBuildPlan>,
}

impl SharedCheckContext {
    fn prepare(
        root: &Path,
        config_path: Option<&Path>,
        tsconfig_path: Option<&Path>,
        visible_paths: std::sync::Arc<crate::codebase::ts_source::VisiblePathSnapshot>,
        config: &crate::config::v2::NoMistakesConfig,
        tsconfig: &crate::codebase::ts_resolver::TsConfig,
    ) -> Result<Self> {
        use crate::check_runner::enabled::{
            fact_plan, integration_configured, plan_requests_facts, ConfiguredChecks, EnabledChecks,
        };
        use crate::check_tasks::{
            filesystem_rules_configured, forbidden_dependencies_configured, queues_configured,
            unique_exports_configured,
        };

        // The aggregate request establishes one normalized root for discovery, graph nodes,
        // and shared fact keys. Canonicalizing only the check sub-context (for example,
        // `/var` to `/private/var` on macOS) would split those identities and make otherwise
        // complete shared symbol facts unreachable from the graph.
        let root = root.to_path_buf();
        let prepared = crate::check_runner::prepared::prepare_from_shared(
            &root,
            config_path,
            tsconfig_path,
            visible_paths,
            config.clone(),
            tsconfig.clone(),
        )?;
        let config = &prepared.config;
        let queues_enabled = queues_configured(config);
        let unique_exports_enabled = unique_exports_configured(config);
        let enabled = ConfiguredChecks::from_config(config);
        let filesystem_rules_enabled = filesystem_rules_configured(config);
        let forbidden_deps_enabled = forbidden_dependencies_configured(config);
        let playwright_rules_enabled = crate::playwright::rules::configured(config);
        let forbidden_graph_plan = forbidden_deps_enabled
            .then(|| crate::codebase::rules::forbidden_dependencies::graph_plan(config))
            .flatten();
        let graph_plan = crate::codebase::rules::canonical_graph_plan(config);
        let mut playwright_fact_plan = prepared
            .playwright
            .as_ref()
            .map(crate::playwright::rules::PreparedPlaywrightRules::fact_plan);
        let playwright_facts_enabled = playwright_fact_plan.is_some();
        let integration_enabled = integration_configured(config);
        let react_enabled = prepared.react.enabled();
        let mut plan = fact_plan(EnabledChecks {
            react: react_enabled,
            queue: queues_enabled,
            queue_factory_names: config.queues.factories.clone(),
            dynamic_import_rules: enabled.dynamic_import_rules,
            boundary_rules: enabled.boundary_rules,
            nextjs_api_routes: enabled.nextjs_api_routes,
            nextjs_caching: enabled.nextjs_caching,
            storybook_stories: enabled.storybook_stories,
            integration: integration_enabled,
            unique_exports: unique_exports_enabled,
        });
        if integration_enabled {
            plan.integration_runner_configs = Some(std::sync::Arc::new(
                crate::integration_tests::prepare_runner_configs_with_catalog(
                    &root,
                    config,
                    prepared.visible_paths.paths_for(&root).as_ref(),
                    std::sync::Arc::clone(&prepared.tsconfig_catalog),
                    prepared.visible_paths.source_store_for(&root),
                ),
            ));
        }
        let prepared_graph = forbidden_graph_plan
            .map(|graph_plan| {
                crate::codebase::dependencies::graph::prepare_graph_config(
                    &root,
                    graph_plan,
                    &prepared.codebase_config,
                    config,
                    prepared.visible_paths.as_ref(),
                )
            })
            .transpose()?;
        if let Some(graph_playwright) = prepared_graph
            .as_ref()
            .map(|graph| {
                graph.playwright_fact_plan(
                    &root,
                    &prepared.tsconfig,
                    prepared.visible_paths.as_ref(),
                )
            })
            .transpose()?
            .flatten()
        {
            match playwright_fact_plan.as_mut() {
                Some(plan) => plan.include(graph_playwright),
                None => playwright_fact_plan = Some(graph_playwright),
            }
        }
        if let (Some(graph_plan), Some(prepared_graph)) =
            (forbidden_graph_plan, prepared_graph.as_ref())
        {
            let (fact_plan, fact_context) =
                crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
                    &root,
                    graph_plan,
                    prepared_graph,
                );
            plan.graph.include(fact_plan);
            plan.graph_context = fact_context;
        }
        let skip_directories = config.filesystem.skip_directories.clone();
        let views = crate::check_discovery::discover_check_file_views_from_snapshot(
            &root,
            config,
            &skip_directories,
            unique_exports_enabled,
            prepared.visible_paths.as_ref(),
        );
        let needs_shared_facts =
            plan_requests_facts(&plan) || playwright_fact_plan.is_some() || forbidden_deps_enabled;
        let needs_full_graph_files = forbidden_graph_plan.is_some() || playwright_facts_enabled;
        let needs_graph_files =
            needs_shared_facts && (needs_full_graph_files || enabled.dynamic_import_rules);
        let (discovered, graph_files) = if needs_full_graph_files {
            (views.filesystem, views.graph)
        } else if needs_graph_files {
            // The dynamic-import rule traverses the same filesystem-scoped
            // visible universe it analyzes. Supplying that universe explicitly
            // keeps prepared graph construction strict without a fallback parse.
            let graph_files = views.filesystem.clone();
            (views.filesystem, graph_files)
        } else {
            (views.filesystem, Vec::new())
        };
        let fact_files = if needs_shared_facts {
            discovered.clone()
        } else {
            Default::default()
        };
        let fs_files = if filesystem_rules_enabled {
            discovered
        } else {
            Default::default()
        };
        Ok(Self {
            root,
            config_path: config_path.map(Path::to_path_buf),
            tsconfig_path: tsconfig_path.map(Path::to_path_buf),
            prepared,
            plan,
            playwright_fact_plan,
            fact_files,
            graph_files,
            fs_files,
            prepared_graph,
            react_enabled,
            queues_enabled,
            unique_exports_enabled,
            filesystem_rules_enabled,
            forbidden_deps_enabled,
            playwright_rules_enabled,
            graph_plan,
        })
    }
}
