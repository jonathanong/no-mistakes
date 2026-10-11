use super::*;

/// Pre-compute per-config filter and setup files once, so the per-test loop can skip
/// re-reading and re-parsing config files on every iteration.
pub(in super::super) fn prepare_standalone(
    root: &Path,
    config: &NoMistakesConfig,
    visible_files: &crate::fx::PathSet,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> Result<super::prepared::PreparedConfig> {
    let mut files = visible_files.iter().cloned().collect::<Vec<_>>();
    files.sort();
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
    super::prepare_with_runner_facts(root, config, &files, &sources, parsed.file_facts())
}
