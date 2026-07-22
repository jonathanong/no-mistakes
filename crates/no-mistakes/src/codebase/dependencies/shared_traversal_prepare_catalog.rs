/// Builds the automatic catalog in the same frozen request universe used to
/// prepare runner projects. Keeping this shared prevents standalone reports
/// from missing aliases owned by configured runner or framework project roots.
pub(crate) struct FrameworkCatalogPreparation<'a> {
    pub(crate) root: &'a Path,
    pub(crate) tsconfig_path: Option<&'a Path>,
    pub(crate) tsconfig: &'a TsConfig,
    pub(crate) config: &'a crate::config::v2::NoMistakesConfig,
    pub(crate) codebase_config: &'a crate::codebase::config::Config,
    pub(crate) workspace: &'a std::sync::Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
    pub(crate) root_visible_paths: &'a [PathBuf],
    pub(crate) visible_paths: &'a crate::codebase::ts_source::VisiblePathSnapshot,
    pub(crate) sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
    pub(crate) build_plan: graph::GraphBuildPlan,
    pub(crate) graph_files: &'a graph::GraphFiles,
    pub(crate) collect_graph_facts: bool,
    pub(crate) framework_plan: &'a crate::codebase::test_discovery::FrameworkPreparationPlan,
}

pub(crate) fn prepare_tsconfig_catalog_with_framework_projects(
    input: FrameworkCatalogPreparation<'_>,
) -> Result<(
    std::sync::Arc<crate::codebase::ts_resolver::TsConfigCatalog>,
    crate::codebase::test_discovery::PreparedTestProjects,
)> {
    let FrameworkCatalogPreparation {
        root,
        tsconfig_path,
        tsconfig,
        config,
        codebase_config,
        workspace,
        root_visible_paths,
        visible_paths,
        sources,
        build_plan,
        graph_files,
        collect_graph_facts,
        framework_plan,
    } = input;
    let mut candidate_roots = Vec::with_capacity(workspace.packages.len() + 1);
    candidate_roots.push(root.to_path_buf());
    candidate_roots.extend(workspace.packages.iter().map(|package| package.dir.clone()));
    candidate_roots.extend(crate::integration_tests::configured_runner_config_dirs(root, config));
    let preliminary_catalog = std::sync::Arc::new(catalog_for_request(
        root,
        tsconfig_path,
        tsconfig,
        &candidate_roots,
        root_visible_paths,
        &sources,
    ));
    let preliminary_graph = graph::prepare_graph_config_with_test_filter_and_workspace(
        root,
        build_plan,
        codebase_config,
        config,
        visible_paths,
        crate::codebase::test_filter::TestFileFilter::fallback_only(),
        std::sync::Arc::clone(workspace),
    )?;
    let (fact_plan, fact_context) = graph::ts_fact_plan_and_context_for_plan_with_prepared(
        root,
        build_plan,
        &preliminary_graph,
    );
    let graph_files = if collect_graph_facts {
        graph_files.indexable()
    } else {
        &[]
    };
    let projects = crate::codebase::test_discovery::prepare_test_projects_from_visible_with_sources_and_plan(
        root,
        config,
        root_visible_paths,
        std::sync::Arc::clone(&preliminary_catalog),
        crate::codebase::test_discovery::PreparedTestProjectRequest {
            graph: (graph_files, fact_plan, fact_context),
            sources: std::sync::Arc::clone(&sources),
            collect_graph_facts,
            preparation_plan: framework_plan,
        },
    );
    candidate_roots.extend(projects.tsconfig_candidate_roots(root));
    candidate_roots.sort();
    candidate_roots.dedup();
    Ok((
        std::sync::Arc::new(catalog_for_request(
            root,
            tsconfig_path,
            tsconfig,
            &candidate_roots,
            root_visible_paths,
            &sources,
        )),
        projects,
    ))
}

fn catalog_for_request(
    root: &Path,
    tsconfig_path: Option<&Path>,
    tsconfig: &TsConfig,
    candidate_roots: &[PathBuf],
    visible_paths: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> crate::codebase::ts_resolver::TsConfigCatalog {
    if let Some(path) = tsconfig_path {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        crate::codebase::ts_resolver::TsConfigCatalog::forced(
            root,
            tsconfig.clone(),
            Some(crate::codebase::ts_resolver::normalize_path(&path)),
        )
    } else {
        crate::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources(
            root,
            candidate_roots,
            visible_paths,
            sources,
        )
    }
}
