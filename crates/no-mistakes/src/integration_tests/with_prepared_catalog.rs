/// Request-scoped aggregate entry point with per-importer TypeScript resolution.
#[doc(hidden)]
pub fn check_with_prepared_facts_catalog_and_session(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    tsconfig_catalog: std::sync::Arc<crate::codebase::ts_resolver::TsConfigCatalog>,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Result<Vec<IntegrationFinding>> {
    config::validate_config(config)?;

    let root_paths = visible_paths.paths_for(root);
    let runner_configs = runner_config::prepare_with_catalog_and_sources(
        root,
        config,
        &root_paths,
        std::sync::Arc::clone(&tsconfig_catalog),
        visible_paths.source_store_for(root),
    );
    let prepared_runner_configs =
        runner_config::ParsedRunnerConfigs::with_files(shared.integration_runner_configs.clone());
    if !prepared_runner_configs.covers(&runner_configs) {
        anyhow::bail!(
            "prepared integration runner facts are incomplete; collect shared facts with \
             CheckFactPlan.integration_runner_configs from prepare_runner_configs()"
        );
    }
    let suites = config::configured_suites_from_runner_configs(
        root,
        config,
        &runner_configs,
        &prepared_runner_configs,
    )?;
    if suites.is_empty() {
        return Ok(Vec::new());
    }

    let analyses: std::collections::BTreeMap<std::path::PathBuf, types::FileAnalysis> = shared
        .ts
        .iter()
        .filter_map(|(path, facts)| {
            facts
                .integration
                .as_ref()
                .map(|analysis| (path.clone(), analysis.clone()))
        })
        .collect();
    let visible_files: crate::fx::PathSet = analyses
        .keys()
        .cloned()
        .chain(
            shared
                .ts
                .iter()
                .filter_map(|(path, facts)| facts.parse_error.as_ref().map(|_| path.clone())),
        )
        .collect();
    let import_resolver = crate::codebase::ts_resolver::ScopedImportResolver::new_in_session(
        &tsconfig_catalog,
        &visible_files,
        session,
    );
    check_suites_with_resolver_and_session(
        root,
        &suites,
        &analyses,
        &import_resolver,
        Some(shared),
        Some(session),
    )
}
