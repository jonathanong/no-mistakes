/// Runner projects parsed once for a request and reused by graph test filters
/// and every framework-specific discovery view.
#[doc(hidden)]
pub struct PreparedTestProjects {
    root: PathBuf,
    projects: BTreeMap<TestRunner, std::result::Result<Vec<ConfigProject>, String>>,
    discovery_files: Vec<PathBuf>,
    graph_facts: crate::codebase::ts_source::facts::TsFactMap,
    dotnet_facts: Option<crate::codebase::dotnet::DotnetFactMap>,
    swift_facts: Option<crate::codebase::swift::SwiftFactMap>,
}

/// Request-local graph inputs and framework demand used during test-project preparation.
#[doc(hidden)]
pub struct PreparedTestProjectRequest<'a> {
    /// The request's already-filtered candidate universe. Framework projections
    /// borrow this instead of rebuilding it from the visible-path inventory.
    pub discovery_files: &'a [PathBuf],
    pub graph: (
        &'a [PathBuf],
        crate::codebase::ts_source::facts::TsFactPlan,
        crate::codebase::ts_source::facts::TsFactContext,
    ),
    pub sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
    pub collect_graph_facts: bool,
    pub preparation_plan: &'a FrameworkPreparationPlan,
}

/// Prepares only the runner project state selected by `preparation_plan`, using
/// the request's canonical source store for runner config and helper reads.
#[doc(hidden)]
pub fn prepare_test_projects_from_visible_with_sources_and_plan(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
    tsconfig_catalog: std::sync::Arc<crate::codebase::ts_resolver::TsConfigCatalog>,
    request: PreparedTestProjectRequest<'_>,
) -> PreparedTestProjects {
    let PreparedTestProjectRequest {
        discovery_files,
        graph: (graph_indexable_files, graph_plan, graph_context),
        sources,
        collect_graph_facts,
        preparation_plan,
    } = request;
    let discovery_files = discovery_files.to_vec();
    let prepared_dotnet = preparation_plan
        .runners
        .contains(&TestRunner::Dotnet)
        .then(|| {
            let (projects, facts) = dotnet_projects::dotnet_projects_and_facts_from_visible_with_sources(
                root,
                config,
                visible_paths,
                Some(&sources),
            );
            (projects.map_err(|error| format!("{error:#}")), facts)
        });
    let prepared_swift = preparation_plan
        .runners
        .contains(&TestRunner::Swift)
        .then(|| {
            crate::codebase::swift::collect_swift_facts_with_sources(
                root,
                &discovery_files,
                &config.tests.swift.packages,
                Some(&sources),
            )
        });
    let runner_configs = crate::integration_tests::runner_config::prepare_with_catalog_and_sources(
        root,
        config,
        visible_paths,
        std::sync::Arc::clone(&tsconfig_catalog),
        sources,
    );
    let runner_fact_plan = crate::integration_tests::runner_config::RunnerConfigFactPlan {
        root: root.to_path_buf(),
        primary_files: std::collections::HashSet::new(),
        graph_files: graph_indexable_files.iter().cloned().collect(),
        primary_plan: crate::codebase::check_facts::CheckFactPlan::default(),
        graph_plan: crate::codebase::check_facts::CheckFactPlan {
            graph: graph_plan,
            graph_context,
            ..Default::default()
        },
        playwright: None,
    };
    let (projects, helper_facts) =
        runner_configs.with_request_cache(collect_graph_facts.then_some(runner_fact_plan), || {
            preparation_plan
                .runners()
                .map(|runner| {
                    let projects = if runner == TestRunner::Dotnet {
                        prepared_dotnet
                            .as_ref()
                            .expect("requested Dotnet projects are prepared")
                            .0
                            .clone()
                    } else if runner == TestRunner::Swift {
                        Ok(swift_projects::swift_projects_from_facts(
                            root,
                            config,
                            prepared_swift
                                .as_ref()
                                .expect("requested Swift projects are prepared"),
                        ))
                    } else {
                        projects::runner_projects_from_visible_with_catalog(
                            root,
                            config,
                            runner,
                            visible_paths,
                            &tsconfig_catalog,
                        )
                        .map_err(|error| format!("{error:#}"))
                    };
                    (runner, projects)
                })
                .collect()
        });
    PreparedTestProjects {
        root: root.to_path_buf(),
        projects,
        discovery_files,
        dotnet_facts: prepared_dotnet.map(|(_, facts)| facts),
        swift_facts: prepared_swift,
        graph_facts: crate::codebase::ts_source::facts::TsFactMap::from_shared_iter_with_plan(
            helper_facts
                .into_iter()
                .map(|(path, facts)| (path, facts.ts)),
            graph_plan,
        ),
    }
}
