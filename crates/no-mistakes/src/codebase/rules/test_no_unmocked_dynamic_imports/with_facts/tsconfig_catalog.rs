use crate::codebase::ts_resolver::{TsConfig, TsConfigCatalog};
use anyhow::Result;
use std::path::Path;

pub(super) fn for_request(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    tsconfig_path: Option<&Path>,
    shared: &crate::codebase::check_facts::CheckFactMap,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<(TsConfig, TsConfigCatalog)> {
    for_files(root, config, tsconfig_path, shared.files(), sources)
}

pub(super) fn for_files(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    tsconfig_path: Option<&Path>,
    visible: &[std::path::PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<(TsConfig, TsConfigCatalog)> {
    // Prepared facts define the complete request universe. Do not rebuild a
    // live snapshot here: it can discover ignored/generated configs that the
    // caller intentionally excluded from its graph facts.
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
        tsconfig_path,
        root,
        visible,
        sources,
    );
    let tsconfig = tsconfig?;
    let catalog = if tsconfig_path.is_none() {
        let mut roots = vec![root.to_path_buf()];
        roots.extend(
            crate::integration_tests::configured_runner_config_dirs_from_visible(
                root, config, visible,
            ),
        );
        TsConfigCatalog::from_visible_and_sources(root, &roots, visible, sources)
    } else {
        crate::codebase::rules::run::prepared_tsconfig_catalog(
            root,
            tsconfig_path,
            &tsconfig,
            visible,
            sources,
            Some(config),
        )
    };
    Ok((tsconfig, catalog))
}

pub(super) fn forced(root: &Path, tsconfig: &TsConfig) -> TsConfigCatalog {
    TsConfigCatalog::forced(root, tsconfig.clone(), None)
}
