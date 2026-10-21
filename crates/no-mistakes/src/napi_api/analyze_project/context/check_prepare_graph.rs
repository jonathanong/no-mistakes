fn configure_shared_family_scopes(
    root: &Path,
    prepared: &crate::check_runner::prepared::PreparedCheckInputs,
    config: &crate::config::v2::NoMistakesConfig,
    plan: &mut crate::codebase::check_facts::CheckFactPlan,
) -> Result<()> {
    crate::codebase::check_facts::configure_family_path_scopes(
        root,
        prepared.visible_paths.paths_for(root).as_ref(),
        config,
        prepared.visible_paths.source_store_for(root).as_ref(),
        plan,
    )
}

fn prepare_shared_graph(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    prepared: &crate::check_runner::prepared::PreparedCheckInputs,
    graph_plan: Option<GraphBuildPlan>,
) -> Result<Option<crate::codebase::dependencies::graph::PreparedGraphConfig>> {
    let mut prepared_graph = graph_plan
        .map(|graph_plan| {
            let test_filter = crate::codebase::test_filter::TestFileFilter::new(root, config);
            crate::codebase::dependencies::graph::prepare_graph_config_with_test_filter(
                root,
                graph_plan,
                &prepared.codebase_config,
                config,
                prepared.visible_paths.as_ref(),
                test_filter,
            )
        })
        .transpose()?;
    if let Some(graph) = prepared_graph.as_mut() {
        graph.set_workflow_inputs(prepared.workflow_documents.clone());
    }
    Ok(prepared_graph)
}
