use super::prepared::{prepare_with_settings, PreparedPlaywrightRules};
use crate::codebase::check_facts::PlaywrightFactPlan;
use crate::config::v2::NoMistakesConfig;
use crate::playwright::config;
use crate::playwright::fsutil::VisiblePathSnapshot;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

pub fn prepare(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
) -> Result<Option<PreparedPlaywrightRules>> {
    let snapshot = Arc::new(VisiblePathSnapshot::new(root));
    let paths = snapshot.paths_for(root);
    let sources = snapshot.source_store_for(root);
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
        None, root, &paths, &sources,
    )?;
    let workspace = crate::codebase::workspaces::load_indexed_from_source_store(root, &sources)
        .unwrap_or_default();
    prepare_with_settings(
        root,
        config,
        snapshot,
        Arc::new(tsconfig),
        Arc::new(workspace),
        None,
        |project, snapshot| {
            config::load_settings_from_visible(root, config_path, &[], project, snapshot)
        },
    )
}

/// Prepare Playwright rule facts from the invocation's canonical candidates.
#[doc(hidden)]
pub fn prepare_from_snapshot(
    root: &Path,
    _config_path: Option<&Path>,
    config: &NoMistakesConfig,
    snapshot: Arc<VisiblePathSnapshot>,
    tsconfig: Arc<crate::codebase::ts_resolver::TsConfig>,
) -> Result<Option<PreparedPlaywrightRules>> {
    let workspace = Arc::new(
        crate::codebase::workspaces::load_indexed_from_source_store(
            root,
            &snapshot.source_store_for(root),
        )
        .unwrap_or_default(),
    );
    prepare_with_settings(
        root,
        config,
        snapshot,
        tsconfig,
        workspace,
        None,
        |project, snapshot| config::settings_from_loaded_v2(root, config, &[], project, snapshot),
    )
}

/// Prepare aggregate Playwright facts while resolving imported wrapper modules
/// with the tsconfig selected for each importing package.
#[doc(hidden)]
pub fn prepare_from_snapshot_with_catalog(
    root: &Path,
    _config_path: Option<&Path>,
    config: &NoMistakesConfig,
    snapshot: Arc<VisiblePathSnapshot>,
    tsconfig: Arc<crate::codebase::ts_resolver::TsConfig>,
    tsconfig_catalog: Arc<crate::codebase::ts_resolver::TsConfigCatalog>,
) -> Result<Option<PreparedPlaywrightRules>> {
    let workspace = Arc::new(
        crate::codebase::workspaces::load_indexed_from_source_store(
            root,
            &snapshot.source_store_for(root),
        )
        .unwrap_or_default(),
    );
    prepare_with_settings(
        root,
        config,
        snapshot,
        tsconfig,
        workspace,
        Some(tsconfig_catalog),
        |project, snapshot| config::settings_from_loaded_v2(root, config, &[], project, snapshot),
    )
}

pub fn fact_plan(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
) -> Result<Option<PlaywrightFactPlan>> {
    Ok(prepare(root, config_path, config)?.map(|prepared| prepared.fact_plan()))
}
