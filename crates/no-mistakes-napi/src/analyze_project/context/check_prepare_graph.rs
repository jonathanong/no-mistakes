fn configure_scopes(
    context: (&no_mistakes::codebase::analysis_session::AnalysisSession, &Path),
    prepared: &no_mistakes::check_runner::prepared::PreparedCheckInputs,
    config: &no_mistakes::config::v2::NoMistakesConfig,
    plan: &mut no_mistakes::codebase::check_facts::CheckFactPlan,
) -> Result<()> {
    let (session, root) = context;
    if plan.integration {
        session.prepare_integration_fact_discovery(
            root,
            prepared.visible_paths.paths_for(root).as_ref(),
            config,
        );
    }
    no_mistakes::codebase::check_facts::configure_family_path_scopes(
        root,
        prepared.visible_paths.paths_for(root).as_ref(),
        config,
        prepared.visible_paths.source_store_for(root).as_ref(),
        plan,
    )?;
    (no_mistakes::codebase::postgres_analysis::hooks().configure_helper_scope)(
        root,
        config,
        &prepared.visible_paths,
        plan,
    )?;
    no_mistakes::codebase::check_facts::configure_nextjs_caching_scope(
        root,
        prepared.visible_paths.paths_for(root).as_ref(),
        config,
        &prepared.inferred_roots,
        plan,
    )
}

fn prepare_disable_directives(
    root: &Path,
    config: &no_mistakes::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    prepared: &no_mistakes::check_runner::prepared::PreparedCheckInputs,
) -> Result<no_mistakes::codebase::rules::disable_directive_audit::PreparedDisableDirectives> {
    no_mistakes::codebase::rules::disable_directive_audit::PreparedDisableDirectives::prepare(
        root,
        config,
        files,
        &prepared.visible_paths.source_store_for(root),
    )
}
