use anyhow::{Context, Result};
use no_mistakes::codebase::ts_source::VisiblePathSnapshot;
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::Path;
use std::sync::Arc;

pub(crate) struct PreparedCheckInputs {
    pub(crate) visible_paths: Arc<VisiblePathSnapshot>,
    pub(crate) inferred_roots: no_mistakes::codebase::config::InferredRoots,
    pub(crate) config: NoMistakesConfig,
    pub(crate) codebase_config: no_mistakes::codebase::config::Config,
    pub(crate) playwright: Option<no_mistakes::playwright::rules::PreparedPlaywrightRules>,
    pub(crate) react: no_mistakes::react_traits::PreparedReactCheck,
    pub(crate) tsconfig: no_mistakes::codebase::ts_resolver::TsConfig,
    pub(crate) tsconfig_catalog: Arc<no_mistakes::codebase::ts_resolver::TsConfigCatalog>,
    pub(crate) vitest_projects: Option<no_mistakes::codebase::rules::PreparedVitestProjectCatalog>,
}

pub(super) fn prepare_with_session(
    session: &no_mistakes::codebase::analysis_session::AnalysisSession,
    root: &Path,
    config_path: Option<&Path>,
    tsconfig_path: Option<&Path>,
) -> Result<PreparedCheckInputs> {
    let (visible_paths, _) = no_mistakes::diagnostics::measure_if_enabled(
        "discovery",
        no_mistakes::diagnostics::TimingKind::Serial,
        || session.visible_paths(root),
    );
    let config = session.config(root, config_path)?;
    let tsconfig = session.tsconfig(root, tsconfig_path)?;
    prepare_from_shared(
        root,
        config_path,
        tsconfig_path,
        visible_paths,
        config.as_ref().clone(),
        tsconfig.as_ref().clone(),
    )
}

pub(crate) fn prepare_from_shared(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig_path: Option<&Path>,
    visible_paths: Arc<VisiblePathSnapshot>,
    config: NoMistakesConfig,
    tsconfig: no_mistakes::codebase::ts_resolver::TsConfig,
) -> Result<PreparedCheckInputs> {
    let root_paths = visible_paths.paths_for(root);
    let inferred_roots =
        no_mistakes::codebase::config::InferredRoots::from_visible(root, root_paths.as_ref());
    let codebase_config =
        no_mistakes::codebase::config::config_from_loaded_v2(root, config_path, &config);
    let sources = visible_paths.source_store_for(root);
    let tsconfig_catalog = Arc::new(if let Some(path) = tsconfig_path {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        no_mistakes::codebase::ts_resolver::TsConfigCatalog::forced(
            root,
            tsconfig.clone(),
            Some(no_mistakes::codebase::ts_resolver::normalize_path(&path)),
        )
    } else {
        let mut candidate_roots = vec![root.to_path_buf()];
        candidate_roots
            .extend(no_mistakes::integration_tests::configured_runner_config_dirs(root, &config));
        candidate_roots.extend(
            no_mistakes::codebase::rules::require_storybook_stories::configured_project_roots(
                root, &config,
            ),
        );
        no_mistakes::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources(
            root,
            &candidate_roots,
            root_paths.as_ref(),
            &sources,
        )
    });
    let playwright = no_mistakes::playwright::rules::prepare_from_snapshot_with_catalog(
        root,
        config_path,
        &config,
        Arc::clone(&visible_paths),
        Arc::new(tsconfig.clone()),
        Arc::clone(&tsconfig_catalog),
    )
    .context("failed to prepare Playwright shared facts")?;
    let react = no_mistakes::react_traits::prepare_check_from_loaded_config(&config, false);
    let vitest_projects = (config
        .rule_configured(no_mistakes::codebase::rules::VITEST_PROJECT_MAPPING)
        || config.rule_configured(no_mistakes::codebase::rules::VITEST_CI_PATH_COVERAGE))
    .then(|| {
        no_mistakes::codebase::rules::prepare_vitest_project_catalog(
            root,
            &config,
            visible_paths.as_ref(),
            &tsconfig_catalog,
        )
    });
    Ok(PreparedCheckInputs {
        visible_paths,
        inferred_roots,
        config,
        codebase_config,
        playwright,
        react,
        tsconfig,
        tsconfig_catalog,
        vitest_projects,
    })
}
