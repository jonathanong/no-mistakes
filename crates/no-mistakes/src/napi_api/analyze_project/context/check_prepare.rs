include!("check_prepare_graph.rs");
impl SharedCheckContext {
    fn prepare(
        session: &crate::codebase::analysis_session::AnalysisSession,
        root: &Path,
        options: SharedCheckPreparationOptions<'_>,
        visible_paths: std::sync::Arc<crate::codebase::ts_source::VisiblePathSnapshot>,
        config: &crate::config::v2::NoMistakesConfig,
        tsconfig: &crate::codebase::ts_resolver::TsConfig,
        workspace: std::sync::Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
    ) -> Result<Self> {
        use crate::check_runner::enabled::{
            fact_plan, integration_configured, ConfiguredChecks, EnabledChecks,
        };
        use crate::check_tasks::{
            filesystem_rules_configured, queues_configured, unique_exports_configured,
        };
        let SharedCheckPreparationOptions {
            config_path,
            tsconfig_path,
            deadlines,
            effort,
        } = options;
        let session = session.scoped_checks();
        session.enable_partial_checks();
        let root = root.to_path_buf();
        let mut config = config.clone();
        config.effort_override = effort;
        let prepared = crate::check_runner::prepared::prepare_from_shared(
            &root,
            config_path,
            tsconfig_path,
            visible_paths,
            config,
            tsconfig.clone(),
            crate::check_runner::prepared::SharedWorkspace {
                map: Some(workspace),
                deadlines,
            },
        )?;
        crate::check_runner::prepared::failures::record(&session, &prepared);
        let config = &prepared.config;
        let queues_enabled = queues_configured(config);
        let unique_exports_enabled = unique_exports_configured(config);
        let enabled = ConfiguredChecks::from_config(config);
        let filesystem_rules_enabled = filesystem_rules_configured(config);
        let playwright_rules_enabled = crate::playwright::rules::configured(config);
        let graph_plan = crate::codebase::rules::try_canonical_graph_plan(config)?;
        let graph_requires_full_file_universe =
            crate::codebase::rules::canonical_graph_requires_full_file_universe(config);
        let graph_rules_enabled = graph_plan.is_some();
        let mut playwright_fact_plan = prepared
            .playwright
            .as_ref()
            .map(crate::playwright::rules::PreparedPlaywrightRules::fact_plan);
        let integration_enabled = integration_configured(config);
        let react_enabled = prepared.react.enabled();
        let mut plan = fact_plan(EnabledChecks {
            react: react_enabled,
            queue: queues_enabled,
            queue_factory_names: config.queues.factories.queue_names(),
            queue_factory_roles: config
                .queues
                .factories
                .clone()
                .with_project_wait_scopes(config),
            dynamic_import_rules: enabled.dynamic_import_rules,
            boundary_rules: enabled.boundary_rules,
            nextjs_api_routes: enabled.nextjs_api_routes,
            nextjs_caching: enabled.nextjs_caching,
            storybook_stories: enabled.storybook_stories,
            integration: integration_enabled,
            unique_exports: unique_exports_enabled,
            embedded_sql: enabled.embedded_sql,
        });
        crate::codebase::rules::nextjs_no_caching::configure_effort_plan(config, &mut plan)?;
        crate::codebase::rules::require_storybook_stories::configure_effort_plan(
            config, &mut plan,
        )?;
        crate::codebase::postgres::assign_prepared_embedded_sql_options(config, &mut plan)?;
        plan.postgres_schema_catalog_paths =
            crate::codebase::postgres::configured_schema_catalog_paths(
                config,
                crate::codebase::postgres::SCHEMA_CATALOG_RULE_IDS,
            )?;
        plan.query_annotation_catalog = Some(std::sync::Arc::clone(&prepared.tsconfig_catalog));
        plan.production_import_paths =
            crate::codebase::rules::production_dependency_declarations::required_import_fact_files(
                &root,
                config,
                prepared.visible_paths.paths_for(&root).as_ref(),
            )?;
        plan.graph.imports |= !plan.production_import_paths.is_empty();
        configure_shared_family_scopes(&root, &prepared, config, &mut plan)?;
        crate::codebase::postgres::configure_prepared_postgres_plan(&root, config, &mut plan)?;
        let prepare_runner_configs = if deadlines {
            crate::integration_tests::prepare_runner_configs_with_deadline_evidence
        } else {
            crate::integration_tests::prepare_runner_configs_with_catalog
        };
        if integration_enabled || enabled.dynamic_import_rules || deadlines {
            plan.integration_runner_configs = Some(std::sync::Arc::new(prepare_runner_configs(
                &root,
                config,
                prepared.visible_paths.paths_for(&root).as_ref(),
                std::sync::Arc::clone(&prepared.tsconfig_catalog),
                prepared.visible_paths.source_store_for(&root),
            )));
        }
        let prepared_graph = crate::check_runner::graph_plan::prepare(
            &root,
            config,
            crate::check_runner::graph_plan::PreparedInputs {
                session: &session,
                codebase_config: &prepared.codebase_config,
                tsconfig: &prepared.tsconfig,
                visible_paths: prepared.visible_paths.as_ref(),
                workflow_documents: prepared.workflow_documents.as_ref(),
            },
            graph_plan,
            &mut playwright_fact_plan,
            &mut plan,
        )?;
        plan.graph.per_item_calls |=
            config.rule_configured(crate::codebase::rules::QUERY_REACHED_PER_ITEM);
        crate::codebase::rules::forbidden_calls::configure_argument_facts(&root, config, &mut plan);
        let mut fact_demand = crate::check_runner::finite_set_plan::prepare(
            &root,
            config,
            &mut plan,
            graph_rules_enabled,
            playwright_fact_plan.is_some(),
        )?;
        let skip_directories = config.filesystem.skip_directories.clone();
        let views = crate::check_discovery::discover_check_file_views_from_snapshot(
            &root,
            config,
            &skip_directories,
            unique_exports_enabled,
            prepared.visible_paths.as_ref(),
        );
        crate::codebase::rules::required_companion_imports::configure_fact_plan(
            &root,
            config,
            &views.filesystem,
            &prepared.visible_paths.source_store_for(&root),
            &mut plan,
        )?;
        fact_demand.include_companion_plan(&plan);
        let needs_shared_facts = fact_demand.needs_shared_facts();
        let (discovered, graph_files) = crate::check_discovery::select_graph_files(
            views,
            needs_shared_facts,
            graph_requires_full_file_universe,
            playwright_fact_plan.is_some(),
            enabled.dynamic_import_rules,
        );
        let fact_files = if needs_shared_facts {
            fact_demand.primary_files(discovered.clone())
        } else {
            Default::default()
        };
        let supplemental_call_site_files =
            fact_demand.supplemental_call_site_files(&fact_files, &graph_files);
        let fs_files = if filesystem_rules_enabled || config.audits_disable_directives() {
            discovered
        } else {
            Default::default()
        };
        Ok(Self {
            session,
            disable_directives: prepare_disable_directives(&root, config, &fs_files, &prepared)?,
            root,
            config_path: config_path.map(Path::to_path_buf),
            tsconfig_path: tsconfig_path.map(Path::to_path_buf),
            prepared,
            plan,
            playwright_fact_plan,
            fact_files,
            supplemental_call_site_files,
            graph_files,
            fs_files,
            prepared_graph,
            react_enabled,
            queues_enabled,
            unique_exports_enabled,
            filesystem_rules_enabled,
            graph_rules_enabled,
            playwright_rules_enabled,
            graph_plan,
        })
    }
}
