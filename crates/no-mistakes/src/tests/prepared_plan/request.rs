use super::*;
use no_mistakes::codebase::dependencies::graph::PreparedGraphBuild;
use no_mistakes::codebase::test_discovery::TestRunner;
use std::path::Path;
use std::sync::atomic::Ordering;

impl PreparedTestPlanRequest {
    pub(crate) fn prepare(args: &PlanArgs) -> Result<Self> {
        PreparedTestPlanInputs::prepare(args)?.finish()
    }

    pub(crate) fn args(&self) -> &PlanArgs {
        &self.args
    }

    pub(crate) fn dotnet_facts(&self) -> Option<&no_mistakes::codebase::dotnet::DotnetFactMap> {
        self.prepared_test_projects.dotnet_facts()
    }

    pub(crate) fn root_visible_paths(&self) -> &[PathBuf] {
        &self.root_visible_paths
    }

    pub(crate) fn changed_file_inventory(&self) -> Vec<String> {
        let mut files: Vec<String> = self
            .collected
            .inventory_files
            .iter()
            .map(|path| no_mistakes::codebase::ts_source::relative_slash_path(&self.root, path))
            .collect();
        files.sort();
        files.dedup();
        files
    }

    pub(crate) fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    pub(crate) fn graph(&self) -> Result<&DepGraph> {
        self.graph
            .get_or_init(|| {
            self.graph_builds.fetch_add(1, Ordering::Relaxed);
            let (fact_plan, mut fact_context) =
                no_mistakes::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
                    &self.root,
                    self.graph_plan,
                    &self.prepared_graph_config,
                );
            fact_context.set_fixture_reads(
                self.changes_input.as_ref().and_then(super::super::plan::changes::ChangesInput::fixture_reads),
            );
            let graph_visible_paths = VisiblePathSnapshot::from_paths(
                &self.root,
                self.graph_files.all(),
            );
            let playwright = self
                .prepared_graph_config
                .playwright_fact_plan(&self.root, &self.tsconfig, &graph_visible_paths)
                .map_err(|error| format!("{error:#}"))?;
            let facts: Box<dyn no_mistakes::codebase::dependencies::graph::TsFactLookup> =
                if let Some(playwright) = playwright {
                    Box::new(no_mistakes::codebase::check_facts::collect_check_facts_with_precollected_graph_facts(
                        &self.root,
                        self.graph_files.iter_visible().cloned().collect(),
                        no_mistakes::codebase::check_facts::CheckFactPlan {
                            graph: fact_plan,
                            graph_context: fact_context,
                            ..Default::default()
                        },
                        playwright,
                        self.prepared_test_projects.graph_facts().clone(),
                        self.prepared_test_projects.graph_route_occurrences().clone(),
                    ))
                } else {
                    let mut facts = self.prepared_test_projects.graph_facts().clone();
                    let remaining = self
                        .graph_files
                        .indexable()
                        .iter()
                        .filter(|path| !facts.contains_key(path))
                        .cloned()
                        .collect::<Vec<_>>();
                    facts.extend(no_mistakes::codebase::ts_source::facts::collect_ts_facts_with_context_and_sources(
                        &remaining,
                        fact_plan,
                        &fact_context,
                        &self.sources,
                    ));
                    Box::new(facts)
                };
            let graph = DepGraph::build_with_plan_files_prepared_config_and_all_facts(
                PreparedGraphBuild {
                    root: &self.root,
                    tsconfig: &self.tsconfig,
                    tsconfig_catalog: Some(&self.tsconfig_catalog),
                    plan: self.graph_plan,
                    graph_files: &self.graph_files,
                    config_path: self.args.config.as_deref(),
                    prepared: &self.prepared_graph_config,
                    facts: Some(facts.as_ref()),
                    import_resolution_cache: None,
                    dotnet_facts: self.prepared_test_projects.dotnet_facts(),
                    swift_facts: self.prepared_test_projects.swift_facts(),
                    visible_paths: None,
                },
            )
            .map_err(|error| format!("{error:#}"))?;
            let graph = graph.with_vitest_setup_projects(
                self.prepared_test_projects.vitest_setup_projects(),
            );
            Ok(graph)
        })
            .as_ref()
            .map_err(|error| anyhow::Error::msg(error.clone()))
    }

    pub(crate) fn graph_is_initialized(&self) -> bool {
        self.graph.get().is_some()
    }

    pub(crate) fn graph_build_count(&self) -> usize {
        self.graph_builds.load(Ordering::Relaxed)
    }

    /// Parsed Vitest project metadata from the request's single runner-config
    /// pass. Callers use this for conservative setup diagnostics and must not
    /// reparse configuration when Vitest was not requested.
    pub(crate) fn vitest_projects(
        &self,
    ) -> Option<&[no_mistakes::integration_tests::types::ConfigProject]> {
        self.prepared_test_projects
            .prepared_projects(TestRunner::Vitest)
    }
}
