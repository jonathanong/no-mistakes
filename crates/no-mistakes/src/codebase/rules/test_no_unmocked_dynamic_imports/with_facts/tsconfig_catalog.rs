use crate::codebase::ts_resolver::{TsConfig, TsConfigCatalog};
use anyhow::Result;
use std::path::Path;

pub(super) fn for_request(
    root: &Path,
    tsconfig_path: Option<&Path>,
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<(TsConfig, TsConfigCatalog)> {
    // Prepared facts define the complete request universe. Do not rebuild a
    // live snapshot here: it can discover ignored/generated configs that the
    // caller intentionally excluded from its graph facts.
    let visible = shared.files();
    let sources = crate::codebase::rules::source_store_for_files(visible);
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
        tsconfig_path,
        root,
        visible,
        &sources,
    )?;
    let catalog = crate::codebase::rules::run::prepared_tsconfig_catalog(
        root,
        tsconfig_path,
        &tsconfig,
        visible,
        &sources,
        None,
    );
    Ok((tsconfig, catalog))
}

pub(super) fn forced(root: &Path, tsconfig: &TsConfig) -> TsConfigCatalog {
    TsConfigCatalog::forced(root, tsconfig.clone(), None)
}
