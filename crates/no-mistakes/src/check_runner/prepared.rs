use anyhow::{Context, Result};
use no_mistakes::codebase::ts_source::VisiblePathSnapshot;
use no_mistakes::config::v2::{load_v2_config_from_visible, NoMistakesConfig};
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
    let root_paths = visible_paths.paths_for(root);
    let config_key = config_path
        .map(|path| no_mistakes::codebase::ts_resolver::normalize_path(&root.join(path)))
        .unwrap_or_else(|| root.join(".no-mistakes.auto"));
    let config = session.load_document("config", &config_key, || {
        load_v2_config_from_visible(root, config_path, &root_paths)
    })?;
    let tsconfig_key = tsconfig_path
        .map(|path| no_mistakes::codebase::ts_resolver::normalize_path(&root.join(path)))
        .unwrap_or_else(|| root.join("tsconfig.auto.json"));
    let tsconfig = session.load_document("tsconfig", &tsconfig_key, || {
        no_mistakes::codebase::ts_resolver::resolve_tsconfig_from_visible(
            tsconfig_path,
            root,
            &root_paths,
        )
    })?;
    prepare_from_shared(
        root,
        config_path,
        visible_paths,
        config.as_ref().clone(),
        tsconfig.as_ref().clone(),
    )
}

pub(crate) fn prepare_from_shared(
    root: &Path,
    config_path: Option<&Path>,
    visible_paths: Arc<VisiblePathSnapshot>,
    config: NoMistakesConfig,
    tsconfig: no_mistakes::codebase::ts_resolver::TsConfig,
) -> Result<PreparedCheckInputs> {
    let root_paths = visible_paths.paths_for(root);
    let inferred_roots =
        no_mistakes::codebase::config::InferredRoots::from_visible(root, root_paths.as_ref());
    let codebase_config =
        no_mistakes::codebase::config::config_from_loaded_v2(root, config_path, &config);
    let playwright = no_mistakes::playwright::rules::prepare_from_snapshot(
        root,
        config_path,
        &config,
        Arc::clone(&visible_paths),
        Arc::new(tsconfig.clone()),
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
            &tsconfig,
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
        vitest_projects,
    })
}
