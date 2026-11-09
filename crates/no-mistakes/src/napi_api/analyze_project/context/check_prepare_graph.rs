fn configure_scopes(
    context: (&crate::codebase::analysis_session::AnalysisSession, &Path),
    prepared: &crate::check_runner::prepared::PreparedCheckInputs,
    config: &crate::config::v2::NoMistakesConfig,
    plan: &mut crate::codebase::check_facts::CheckFactPlan,
) -> Result<()> {
    let (session, root) = context;
    if plan.integration {
        session.prepare_integration_fact_discovery(
            root,
            prepared.visible_paths.paths_for(root).as_ref(),
            config,
        );
    }
    crate::codebase::check_facts::configure_family_path_scopes(
        root,
        prepared.visible_paths.paths_for(root).as_ref(),
        config,
        prepared.visible_paths.source_store_for(root).as_ref(),
        plan,
    )?;
    crate::codebase::check_facts::configure_nextjs_caching_scope(
        root,
        prepared.visible_paths.paths_for(root).as_ref(),
        config,
        &prepared.inferred_roots,
        plan,
    )
}

fn prepare_disable_directives(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    prepared: &crate::check_runner::prepared::PreparedCheckInputs,
) -> Result<crate::codebase::rules::disable_directive_audit::PreparedDisableDirectives> {
    crate::codebase::rules::disable_directive_audit::PreparedDisableDirectives::prepare(
        root,
        config,
        files,
        &prepared.visible_paths.source_store_for(root),
    )
}
