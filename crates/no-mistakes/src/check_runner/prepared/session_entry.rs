pub(super) fn prepare_with_session(
    session: &no_mistakes::codebase::analysis_session::AnalysisSession,
    root: &Path,
    config_path: Option<&Path>,
    tsconfig_path: Option<&Path>,
    effort: Option<no_mistakes::config::v2::Effort>,
    deadlines: bool,
) -> Result<PreparedCheckInputs> {
    let (visible_paths, _) = no_mistakes::diagnostics::measure_if_enabled(
        "discovery",
        no_mistakes::diagnostics::TimingKind::Serial,
        || session.visible_paths(root),
    );
    let (config, effective_config_path) = session.config_with_path(root, config_path)?;
    let tsconfig = session.tsconfig(root, tsconfig_path)?;
    let workspace = SharedWorkspace {
        map: consumers::needs_workspace(&config, tsconfig_path, deadlines)
            .then(|| session.workspace(root)),
        deadlines,
    };
    let mut config = config.as_ref().clone();
    config.effort_override = effort;
    let prepared = prepare_from_shared(
        root,
        effective_config_path.as_deref(),
        tsconfig_path,
        visible_paths,
        config,
        tsconfig.as_ref().clone(),
        workspace,
    )?;
    failures::record(session, &prepared);
    Ok(prepared)
}

