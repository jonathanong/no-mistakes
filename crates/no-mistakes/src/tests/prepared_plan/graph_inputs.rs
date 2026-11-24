use super::helpers::framework_graph_plan;
use super::*;

pub(super) struct GraphInputs {
    pub(super) tsconfig_catalog: Arc<no_mistakes::codebase::ts_resolver::TsConfigCatalog>,
    pub(super) graph_files: GraphFiles,
    pub(super) graph_plan: GraphBuildPlan,
    pub(super) prepared_graph_config: PreparedGraphConfig,
    pub(super) prepared_test_projects:
        Arc<no_mistakes::codebase::test_discovery::PreparedTestProjects>,
    pub(super) test_filter: no_mistakes::codebase::test_filter::TestFileFilter,
}

impl PreparedTestPlanInputs {
    pub(super) fn prepare_graph(
        &self,
        tsconfig: &TsConfig,
        sources: &Arc<SourceStore>,
        workspace_map: &WorkspaceMap,
    ) -> Result<GraphInputs> {
        let Self {
            args,
            root,
            config,
            visible_paths,
            root_visible_paths,
            collected,
            ..
        } = self;
        let mut tsconfig_candidate_roots = Vec::with_capacity(workspace_map.packages.len() + 1);
        tsconfig_candidate_roots.push(root.clone());
        tsconfig_candidate_roots.extend(
            workspace_map
                .packages
                .iter()
                .map(|package| package.dir.clone()),
        );
        tsconfig_candidate_roots.extend(
            no_mistakes::integration_tests::configured_runner_config_dirs_from_visible(
                root,
                config,
                root_visible_paths,
            ),
        );
        let preliminary_tsconfig_roots = tsconfig_candidate_roots.clone();
        let preliminary_tsconfig_catalog = Arc::new(if let Some(path) = args.tsconfig.as_deref() {
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            };
            no_mistakes::codebase::ts_resolver::TsConfigCatalog::forced(
                root,
                tsconfig.clone(),
                Some(no_mistakes::codebase::ts_resolver::normalize_path(&path)),
            )
        } else {
            no_mistakes::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources(
                root,
                &tsconfig_candidate_roots,
                root_visible_paths,
                sources,
            )
        });
        let framework_plan = args.framework.map_or_else(
            no_mistakes::codebase::test_discovery::FrameworkPreparationPlan::all,
            |framework| {
                no_mistakes::codebase::test_discovery::FrameworkPreparationPlan::for_runners([
                    test_runner(framework),
                ])
            },
        );
        let excluded_configs =
            framework_plan.excluded_config_paths(root, config, root_visible_paths);
        let graph_all_files = no_mistakes::codebase::ts_source::discover_files_from_visible(
            root,
            &[],
            root_visible_paths,
        );
        let discovery_files =
            no_mistakes::codebase::ts_source::filter_discovered_files_by_skip_directories(
                root,
                &config.filesystem.skip_directories,
                &graph_all_files,
            );
        let mut resource_candidates = visible_paths.tracked_paths_for(root).as_ref().clone();
        // Include deleted resources so reverse impact can follow their phantom nodes.
        resource_candidates.extend(collected.deleted.iter().cloned());
        let mut graph_files = GraphFiles::from_files_with_resource_candidates_excluding_indexable(
            graph_all_files.clone(),
            // Preserve tracked runtime inputs under source-skipped directories
            // such as `fixtures/`; they are resource targets, not parse roots.
            resource_candidates,
            &excluded_configs,
        );
        for path in &collected.authoritative_files {
            graph_files.add_explicit_root(path);
        }
        // Only generic plans opt into symbol edges.
        let mut graph_plan = args
            .framework
            .map_or_else(GraphBuildPlan::all, framework_graph_plan)
            .with_symbols(args.framework.is_none() && args.include_symbols);
        if no_mistakes::playwright::integration_routes::configured(config)
            && matches!(args.framework, None | Some(TestFramework::Vitest))
        {
            graph_plan.playwright_routes = true;
        }
        let codebase_config = no_mistakes::codebase::config::config_from_loaded_v2(
            root,
            args.config.as_deref(),
            config,
        );
        let preliminary_graph_config =
            no_mistakes::codebase::dependencies::graph::prepare_graph_config_with_test_filter(
                root,
                graph_plan,
                &codebase_config,
                config,
                visible_paths,
                no_mistakes::codebase::test_filter::TestFileFilter::fallback_only(),
            )?;
        let (runner_graph_plan, runner_graph_context) =
            no_mistakes::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
                root,
                graph_plan,
                &preliminary_graph_config,
            );
        let mut prepared_test_projects =
            no_mistakes::codebase::test_discovery::prepare_test_projects_from_visible_with_sources_and_plan(
                root,
                config,
                root_visible_paths,
                Arc::clone(&preliminary_tsconfig_catalog),
                no_mistakes::codebase::test_discovery::PreparedTestProjectRequest {
                    discovery_files: &discovery_files,
                    graph: (graph_files.indexable(), runner_graph_plan, runner_graph_context),
                    sources: Arc::clone(sources),
                    collect_graph_facts: true,
                    playwright: if no_mistakes::playwright::integration_routes::configured(config) {
                        preliminary_graph_config.playwright_fact_plan(root, tsconfig, visible_paths)?
                    } else { None },
                    preparation_plan: &framework_plan,
                },
            );
        tsconfig_candidate_roots.extend(prepared_test_projects.tsconfig_candidate_roots(root));
        let tsconfig_catalog =
            no_mistakes::codebase::test_discovery::reuse_or_rebuild_prepared_catalog(
                preliminary_tsconfig_catalog,
                &preliminary_tsconfig_roots,
                &mut tsconfig_candidate_roots,
                |candidate_roots| {
                    Arc::new(
                        no_mistakes::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources(
                            root,
                            candidate_roots,
                            root_visible_paths,
                            sources,
                        ),
                    )
                },
            );
        prepared_test_projects.reparse_vitest_with_final_catalog(
            root,
            config,
            root_visible_paths,
            Arc::clone(&tsconfig_catalog),
            Arc::clone(sources),
        );
        prepared_test_projects.reresolve_vitest_setups(root, &tsconfig_catalog, root_visible_paths);
        let prepared_test_projects = Arc::new(prepared_test_projects);
        let test_filter =
            no_mistakes::codebase::test_filter::TestFileFilter::from_prepared_projects(
                root,
                config,
                root_visible_paths,
                prepared_test_projects.project_filters(),
            );
        let prepared_graph_config =
            no_mistakes::codebase::dependencies::graph::prepare_graph_config_with_test_filter(
                root,
                graph_plan,
                &codebase_config,
                config,
                visible_paths,
                test_filter.clone(),
            )?;

        Ok(GraphInputs {
            tsconfig_catalog,
            graph_files,
            graph_plan,
            prepared_graph_config,
            prepared_test_projects,
            test_filter,
        })
    }
}
