use no_mistakes::codebase::ts_resolver::{TsConfig, TsConfigCatalog};
use no_mistakes::codebase::ts_source::SourceStore;
use no_mistakes::codebase::workspaces::IndexedWorkspaceMap;
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn prepare(
    root: &Path,
    tsconfig_path: Option<&Path>,
    tsconfig: &TsConfig,
    config: &NoMistakesConfig,
    root_paths: &[PathBuf],
    sources: &SourceStore,
    workspace: Option<&IndexedWorkspaceMap>,
) -> Arc<TsConfigCatalog> {
    let catalog = if let Some(path) = tsconfig_path {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        TsConfigCatalog::forced(
            root,
            tsconfig.clone(),
            Some(no_mistakes::codebase::ts_resolver::normalize_path(&path)),
        )
    } else if !super::consumers::needs_tsconfig_catalog(config) {
        TsConfigCatalog::forced(root, tsconfig.clone(), None)
    } else {
        let mut candidate_roots = vec![root.to_path_buf()];
        candidate_roots.extend(
            no_mistakes::integration_tests::configured_runner_config_dirs_from_visible(
                root, config, root_paths,
            ),
        );
        candidate_roots.extend(
            no_mistakes::codebase::rules::require_storybook_stories::configured_project_roots(
                root, config,
            ),
        );
        TsConfigCatalog::from_visible_and_sources_with_workspace(
            root,
            &candidate_roots,
            root_paths,
            sources,
            workspace.expect("automatic tsconfig catalog requires a workspace projection"),
        )
    };
    Arc::new(catalog)
}
