use crate::codebase::ci_workflows::{ParsedWorkflowSet, PreparedWorkflowTopologyInputs};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn from_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
) -> PreparedWorkflowTopologyInputs {
    let directories: Vec<_> = config
        .ci
        .workflow_dirs
        .iter()
        .map(|dir| crate::codebase::ts_resolver::normalize_path(&root.join(dir)))
        .collect();
    let workflows = files.iter().flat_map(|path| {
        directories.iter().filter_map(move |directory| {
            let child = path.strip_prefix(directory).ok()?.components().next()?;
            let path = directory.join(child.as_os_str());
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml")
                })
                .then_some(path)
        })
    });
    let actions_requested = config.rule_applications(super::RULE_ID).iter().any(|rule| {
        rule.options
            .get("secretInventory")
            .is_some_and(|value| !value.is_null())
    });
    let actions = files
        .iter()
        .filter(|path| {
            actions_requested
                && path
                    .file_name()
                    .is_some_and(|name| name == "action.yml" || name == "action.yaml")
        })
        .cloned();
    PreparedWorkflowTopologyInputs::new(
        root,
        &config.ci,
        Arc::new(ParsedWorkflowSet::from_paths_and_sources(
            root, workflows, sources,
        )),
        ParsedWorkflowSet::from_paths_and_sources(root, actions, sources),
    )
}

pub(super) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    _all_files: &[PathBuf],
) -> anyhow::Result<Vec<super::RuleFinding>> {
    // Compatibility tests own a repository request. Production runners borrow
    // their already-prepared inputs below.
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let sources = snapshot.source_store_for(root);
    let files = snapshot.paths_for(root);
    check_with_files_and_sources(root, config, &files, &sources)
}

pub(super) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> anyhow::Result<Vec<super::RuleFinding>> {
    let inputs = from_files(root, config, all_files, sources);
    let config_path =
        crate::config::v2::effective_v2_config_path_from_visible(root, None, all_files)?;
    super::check_prepared(
        root,
        config,
        all_files,
        sources,
        &inputs,
        config_path.as_deref(),
    )
}
