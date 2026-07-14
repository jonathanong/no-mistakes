use super::{preserved, FILESYSTEM_RULE_IDS};
use crate::codebase::rules::{rule_enabled, RuleFinding};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Run filesystem rules using a pre-discovered file list so the caller's single
/// `git ls-files` result is reused. Rules run in parallel.
pub fn run_filesystem_rules_with_files(
    root: &Path,
    config_path: Option<&Path>,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let config = crate::config::v2::load_v2_config(root, config_path)?;
    run_filesystem_rules_with_config(root, &config, files)
}

/// Standalone entry point: discover files once, then reuse the with-files
/// dispatcher for every enabled filesystem rule.
pub fn run_filesystem_rules(root: &Path, config_path: Option<&Path>) -> Result<Vec<RuleFinding>> {
    let config = crate::config::v2::load_v2_config(root, config_path)?;
    if !FILESYSTEM_RULE_IDS
        .iter()
        .any(|rule_id| rule_enabled(&config, rule_id))
    {
        return Ok(Vec::new());
    }
    let preserved_roots =
        preserved::filesystem_rule_target_roots(root, &config, FILESYSTEM_RULE_IDS);
    let files = crate::codebase::ts_source::discover_files_preserving_roots(
        root,
        &config.filesystem.skip_directories,
        &preserved_roots,
    );
    run_filesystem_rules_with_config(root, &config, &files)
}

#[doc(hidden)]
pub fn run_filesystem_rules_with_config(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(root, files);
    run_filesystem_rules_with_config_and_snapshot(root, config, files, &snapshot)
}

#[doc(hidden)]
pub fn run_filesystem_rules_with_config_and_snapshot(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    run_filesystem_rules_with_config_snapshot_and_vitest_catalog(
        &root, config, files, snapshot, None,
    )
}

#[doc(hidden)]
pub fn run_filesystem_rules_with_config_snapshot_and_vitest_catalog(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    vitest_catalog: Option<&crate::codebase::rules::PreparedVitestProjectCatalog>,
) -> Result<Vec<RuleFinding>> {
    super::run_filesystem_rules_with_config_snapshot_catalog_and_sources(
        root,
        config,
        files,
        snapshot,
        vitest_catalog,
        snapshot.source_store_for(root),
    )
}
