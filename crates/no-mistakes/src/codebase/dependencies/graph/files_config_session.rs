pub(crate) fn ts_fact_plan_and_context_for_plan_with_config_and_session(
    root: &Path,
    plan: GraphBuildPlan,
    config_path: Option<&Path>,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
    visible_paths: Option<&[PathBuf]>,
) -> (TsFactPlan, TsFactContext) {
    let options = graph_config_options_for_plan_with_config_and_session(
        root,
        plan,
        config_path,
        session,
        visible_paths,
    );
    (
        effective_ts_fact_plan(plan, options.as_ref()),
        ts_fact_context_from_options(root, plan, options.as_ref()),
    )
}

fn graph_config_options_with_config_and_session(
    root: &Path,
    config_path: Option<&Path>,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
    visible_paths: Option<&[PathBuf]>,
) -> Option<GraphConfigOptions> {
    let config = match config_path {
        Some(path) => crate::codebase::config::load_config_with_path(root, Some(path)),
        None => crate::codebase::config::load_config(root),
    }
    .ok()?;
    let v2_config = load_v2_config(root, config_path).ok()?;
    let snapshot;
    let snapshot_paths;
    let discovered;
    let visible_paths = if let Some(paths) = visible_paths {
        paths
    } else if let Some(session) = session {
        snapshot = session.visible_paths(root);
        snapshot_paths = snapshot.paths_for(root);
        snapshot_paths.as_ref()
    } else {
        discovered = crate::codebase::ts_source::discover_visible_paths(root);
        &discovered
    };
    Some(match session {
        Some(session) => graph_config_options_from_loaded_with_test_filter(
            root,
            &config,
            &v2_config,
            visible_paths,
            Some(
                (*session.test_file_filter_with_visible(root, &v2_config, Some(visible_paths)))
                    .clone(),
            ),
        ),
        None => graph_config_options_from_loaded(root, &config, &v2_config, visible_paths),
    })
}

fn graph_config_options_for_plan_with_config_and_session(
    root: &Path,
    plan: GraphBuildPlan,
    config_path: Option<&Path>,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
    visible_paths: Option<&[PathBuf]>,
) -> Option<GraphConfigOptions> {
    if !graph_plan_needs_config(plan) {
        return None;
    }
    graph_config_options_with_config_and_session(root, config_path, session, visible_paths)
}
