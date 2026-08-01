use super::{preserved, FILESYSTEM_RULE_IDS};
use crate::codebase::rules::{rule_enabled, RuleFinding};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Run filesystem rules using an authoritative tracked-file list. Rules run in
/// parallel. Callers that have a visible list containing untracked files must
/// use [`run_filesystem_rules_with_visible_and_snapshot`] instead.
pub fn run_filesystem_rules_with_files(
    root: &Path,
    config_path: Option<&Path>,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let (config, effective_path) = crate::config::v2::load_v2_config_with_path(root, config_path)?;
    run_filesystem_rules_with_config_and_path(root, &config, effective_path.as_deref(), files)
}

/// Run filesystem rules with a caller-supplied visible work list and the
/// request's existing discovery snapshot. This preserves tracked-only rules
/// without a second Git discovery.
pub fn run_filesystem_rules_with_visible_and_snapshot(
    root: &Path,
    config_path: Option<&Path>,
    visible_files: &[PathBuf],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
) -> Result<Vec<RuleFinding>> {
    let (config, effective_path) = crate::config::v2::load_v2_config_with_path_from_visible(
        root,
        config_path,
        &snapshot.paths_for(root),
    )?;
    run_filesystem_rules_with_config_snapshot_and_path(
        root,
        &config,
        effective_path.as_deref(),
        visible_files,
        snapshot,
    )
}

/// Standalone entry point: discover files once, then reuse the with-files
/// dispatcher for every enabled filesystem rule.
pub fn run_filesystem_rules(root: &Path, config_path: Option<&Path>) -> Result<Vec<RuleFinding>> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible_paths = snapshot.paths_for(root);
    let (config, effective_path) = crate::config::v2::load_v2_config_with_path_from_visible(
        root,
        config_path,
        &visible_paths,
    )?;
    if !FILESYSTEM_RULE_IDS
        .iter()
        .any(|rule_id| rule_enabled(&config, rule_id))
    {
        return Ok(Vec::new());
    }
    let preserved_roots =
        preserved::filesystem_rule_target_roots(root, &config, FILESYSTEM_RULE_IDS);
    let files = crate::codebase::ts_source::discover_files_preserving_roots_from_visible(
        root,
        &config.filesystem.skip_directories,
        &preserved_roots,
        &visible_paths,
    );
    run_filesystem_rules_with_config_snapshot_and_path(
        root,
        &config,
        effective_path.as_deref(),
        &files,
        &snapshot,
    )
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

fn run_filesystem_rules_with_config_and_path(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    config_path: Option<&Path>,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(root, files);
    run_filesystem_rules_with_config_snapshot_and_path(root, config, config_path, files, &snapshot)
}

#[doc(hidden)]
pub fn run_filesystem_rules_with_config_and_snapshot(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
) -> Result<Vec<RuleFinding>> {
    run_filesystem_rules_with_config_snapshot_and_path(root, config, None, files, snapshot)
}

fn run_filesystem_rules_with_config_snapshot_and_path(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    config_path: Option<&Path>,
    files: &[PathBuf],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
) -> Result<Vec<RuleFinding>> {
    run_filesystem_rules_with_config_snapshot_path_and_catalog(
        root,
        config,
        config_path,
        files,
        snapshot,
        None,
    )
}

fn run_filesystem_rules_with_config_snapshot_path_and_catalog(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    config_path: Option<&Path>,
    files: &[PathBuf],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    vitest_catalog: Option<&crate::codebase::rules::PreparedVitestProjectCatalog>,
) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let sources = snapshot.source_store_for(&root);
    let workflows =
        rule_enabled(config, crate::codebase::rules::TSCONFIG_GATE_COVERAGE).then(|| {
            crate::codebase::ci_workflows::ParsedWorkflowSet::load_from_snapshot_and_sources(
                &root, &config.ci, snapshot, &sources,
            )
        });
    let project_inputs = rule_enabled(config, crate::codebase::rules::TSCONFIG_GATE_COVERAGE)
        .then(|| {
            let workspace =
                crate::codebase::workspaces::load_indexed_from_source_store(&root, &sources)?;
            Ok::<_, anyhow::Error>(
                crate::codebase::rules::tsconfig_gate_coverage::prepare_project_source_inputs(
                    &root,
                    snapshot.paths_for(&root).as_ref(),
                    &sources,
                    &workspace,
                ),
            )
        })
        .transpose()?;
    super::run_filesystem_rules_with_config_snapshot_catalog_and_sources(
        &root,
        config,
        files,
        super::PreparedFilesystemRuleInputs {
            snapshot,
            vitest_catalog,
            sources,
            workflow_documents: workflows.as_ref(),
            tsconfig_gate_project_inputs: project_inputs.as_ref(),
            config_path,
        },
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
    run_filesystem_rules_with_config_snapshot_path_and_catalog(
        root,
        config,
        None,
        files,
        snapshot,
        vitest_catalog,
    )
}
