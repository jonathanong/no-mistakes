use anyhow::{Context, Result};
use no_mistakes::codebase::ts_source::VisiblePathSnapshot;
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::Path;
use std::sync::Arc;

mod catalog;
mod independent;
mod parallel;

pub(crate) struct PreparedCheckInputs {
    pub(crate) visible_paths: Arc<VisiblePathSnapshot>,
    pub(crate) inferred_roots: no_mistakes::codebase::config::InferredRoots,
    pub(crate) config: NoMistakesConfig,
    pub(crate) config_path: Option<std::path::PathBuf>,
    pub(crate) codebase_config: no_mistakes::codebase::config::Config,
    pub(crate) playwright: Option<no_mistakes::playwright::rules::PreparedPlaywrightRules>,
    pub(crate) react: no_mistakes::react_traits::PreparedReactCheck,
    pub(crate) tsconfig: no_mistakes::codebase::ts_resolver::TsConfig,
    pub(crate) tsconfig_catalog: Arc<no_mistakes::codebase::ts_resolver::TsConfigCatalog>,
    pub(crate) vitest_projects: Option<no_mistakes::codebase::rules::PreparedVitestProjectCatalog>,
    pub(crate) playwright_projects:
        Option<no_mistakes::codebase::rules::PreparedPlaywrightProjectCatalog>,
    pub(crate) workflow_documents:
        Option<Arc<no_mistakes::codebase::ci_workflows::PreparedWorkflowTopologyInputs>>,
    pub(crate) tsconfig_gate_project_inputs:
        Option<no_mistakes::codebase::rules::tsconfig_gate_coverage::ProjectSourceInputs>,
}

pub(super) fn prepare_with_session(
    session: &no_mistakes::codebase::analysis_session::AnalysisSession,
    root: &Path,
    config_path: Option<&Path>,
    tsconfig_path: Option<&Path>,
    effort: Option<no_mistakes::config::v2::Effort>,
) -> Result<PreparedCheckInputs> {
    let (visible_paths, _) = no_mistakes::diagnostics::measure_if_enabled(
        "discovery",
        no_mistakes::diagnostics::TimingKind::Serial,
        || session.visible_paths(root),
    );
    let (config, effective_config_path) = session.config_with_path(root, config_path)?;
    let tsconfig = session.tsconfig(root, tsconfig_path)?;
    let workspace = (tsconfig_path.is_none()
        || no_mistakes::playwright::rules::configured(&config)
        || config.rule_configured(no_mistakes::codebase::rules::TSCONFIG_GATE_COVERAGE))
    .then(|| session.workspace(root));
    let mut config = config.as_ref().clone();
    config.effort_override = effort;
    prepare_from_shared(
        root,
        effective_config_path.as_deref(),
        tsconfig_path,
        visible_paths,
        config,
        tsconfig.as_ref().clone(),
        workspace,
    )
}

pub(crate) fn prepare_from_shared(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig_path: Option<&Path>,
    visible_paths: Arc<VisiblePathSnapshot>,
    mut config: NoMistakesConfig,
    tsconfig: no_mistakes::codebase::ts_resolver::TsConfig,
    workspace: Option<Arc<no_mistakes::codebase::workspaces::IndexedWorkspaceMap>>,
) -> Result<PreparedCheckInputs> {
    let root_paths = visible_paths.paths_for(root);
    let sources = visible_paths.source_store_for(root);
    no_mistakes::codebase::rules::analysis_policy::prepare_rule_analysis(
        root,
        &mut config,
        &root_paths,
        &sources,
    )?;
    if config.rule_configured(no_mistakes::codebase::rules::REQUIRE_STORYBOOK_STORIES) {
        no_mistakes::codebase::rules::require_storybook_stories::authorize_configured_sources(
            root, &config, &sources,
        );
    }
    let observer = no_mistakes::diagnostics::current();
    // Branches must stay AST-free: REQUEST_PARSE_CACHES, runner REQUEST_CACHES, and the SPANS stacks are not propagated.
    let (
        tsconfig_catalog,
        (inferred_roots, codebase_config, react, workflow_documents, tsconfig_gate_project_inputs),
    ) = no_mistakes::diagnostics::measure_if_enabled(
        "check.prepare",
        no_mistakes::diagnostics::TimingKind::Serial,
        || {
            rayon::join(
                || {
                    parallel::branch(observer.clone(), "check.prepare.catalog", || {
                        catalog::prepare(
                            root,
                            tsconfig_path,
                            &tsconfig,
                            &config,
                            root_paths.as_ref(),
                            &sources,
                            workspace.as_deref(),
                        )
                    })
                },
                || {
                    parallel::branch(observer.clone(), "check.prepare.independent", || {
                        independent::prepare(
                            root,
                            config_path,
                            root_paths.as_ref(),
                            &config,
                            visible_paths.as_ref(),
                            &sources,
                            workspace.as_deref(),
                        )
                    })
                },
            )
        },
    )
    .0;
    let playwright = no_mistakes::playwright::rules::configured(&config)
        .then(|| {
            no_mistakes::playwright::rules::prepare_from_snapshot_with_catalog(
                root,
                config_path,
                &config,
                Arc::clone(&visible_paths),
                Arc::new(tsconfig.clone()),
                Arc::clone(&tsconfig_catalog),
                Arc::clone(
                    workspace
                        .as_ref()
                        .expect("configured Playwright rules require a workspace projection"),
                ),
            )
            .context("failed to prepare Playwright shared facts")
        })
        .transpose()?
        .flatten();
    let (vitest_from_calls, playwright_projects) =
        no_mistakes::codebase::rules::forbidden_calls::prepare_root_catalogs(
            root,
            &config,
            visible_paths.as_ref(),
            &tsconfig_catalog,
        );
    let vitest_projects = vitest_from_calls.or_else(|| {
        (config.rule_configured(no_mistakes::codebase::rules::VITEST_PROJECT_MAPPING)
            || config.rule_configured(no_mistakes::codebase::rules::VITEST_CI_PATH_COVERAGE))
        .then(|| {
            no_mistakes::codebase::rules::prepare_vitest_project_catalog(
                root,
                &config,
                visible_paths.as_ref(),
                &tsconfig_catalog,
            )
        })
    });
    Ok(PreparedCheckInputs {
        visible_paths,
        inferred_roots,
        config,
        config_path: config_path.map(Path::to_path_buf),
        codebase_config,
        playwright,
        react,
        tsconfig,
        tsconfig_catalog,
        vitest_projects,
        playwright_projects,
        workflow_documents,
        tsconfig_gate_project_inputs,
    })
}
