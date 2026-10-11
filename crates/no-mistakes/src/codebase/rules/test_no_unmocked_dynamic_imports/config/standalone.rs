use super::*;

/// Pre-compute per-config filter and setup files once, so the per-test loop can skip
/// re-reading and re-parsing config files on every iteration.
pub fn precompute_setup_data(
    root: &Path,
    config: &NoMistakesConfig,
    visible_files: &crate::fx::PathSet,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> Result<Vec<ConfigSetupData>> {
    let mut files = visible_files.iter().cloned().collect::<Vec<_>>();
    files.sort();
    let config_files = discovery::config_files_from_visible(root, config, &files);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let runner = crate::integration_tests::runner_config::prepare_dynamic_configs(
        root,
        config,
        &files,
        std::sync::Arc::new(crate::codebase::ts_resolver::TsConfigCatalog::forced(
            root,
            tsconfig.clone(),
            None,
        )),
        std::sync::Arc::clone(&sources),
    );
    let parsed = runner.parse_all()?;
    let runner_facts = parsed.file_facts();
    let mut data = precompute_setup_data_from_config_files_inner(
        root,
        &config_files,
        Some(visible_files),
        Some(&sources),
        Some((config, runner_facts)),
    )?;
    data.extend(explicit_project_setup_data(root, config, visible_files)?);
    data.extend(rule_targets::playwright_setup_data(root, config)?);
    Ok(data)
}
