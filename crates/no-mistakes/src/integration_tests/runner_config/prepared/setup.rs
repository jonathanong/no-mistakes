use super::*;

pub(crate) fn prepare(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> PreparedIntegrationRunnerConfigs {
    prepare_inner(
        root,
        config,
        visible_paths,
        std::sync::Arc::new(crate::codebase::ts_resolver::TsConfigCatalog::forced(
            root,
            tsconfig.clone(),
            None,
        )),
        None,
        false,
    )
}

pub(crate) fn prepare_with_catalog_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
    tsconfig_catalog: std::sync::Arc<crate::codebase::ts_resolver::TsConfigCatalog>,
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> PreparedIntegrationRunnerConfigs {
    prepare_inner(
        root,
        config,
        visible_paths,
        tsconfig_catalog,
        Some(sources),
        false,
    )
}

fn prepare_inner(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
    tsconfig_catalog: std::sync::Arc<crate::codebase::ts_resolver::TsConfigCatalog>,
    sources: Option<std::sync::Arc<crate::codebase::ts_source::SourceStore>>,
    deadlines: bool,
) -> PreparedIntegrationRunnerConfigs {
    let mut specs = Vec::new();
    let mut visible_files = visible_paths
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<crate::fx::PathSet>();
    add_framework_specs(
        &mut specs,
        root,
        Framework::Playwright,
        config.tests.playwright.configs.as_ref(),
        &config.tests.playwright.projects,
        &visible_files,
        deadlines && config.tests.playwright.configs.is_some(),
    );
    add_framework_specs(
        &mut specs,
        root,
        Framework::Vitest,
        config.tests.vitest.configs.as_ref(),
        &config.tests.vitest.projects,
        &visible_files,
        (deadlines && config.tests.vitest.configs.is_some())
            || !config.tests.playwright.route_coverage_sources.is_empty()
            || config
                .tests
                .playwright
                .apps
                .values()
                .any(|app| !app.route_coverage_sources.is_empty()),
    );
    // Explicit runner configs are authoritative even when Git ignores them.
    // Add their normalized paths to the frozen request view up front; reads are
    // still memoized by the request source store and failures become prepared
    // project results rather than triggering a second filesystem discovery.
    visible_files.extend(specs.iter().map(|spec| spec.path.clone()));
    PreparedIntegrationRunnerConfigs {
        root: root.to_path_buf(),
        specs,
        deadline_runners: [
            (
                "playwright",
                deadlines && config.tests.playwright.configs.is_some(),
            ),
            ("vitest", deadlines && config.tests.vitest.configs.is_some()),
        ]
        .into_iter()
        .filter_map(|(runner, requested)| requested.then_some(runner))
        .collect(),
        tsconfig_catalog,
        visible_files,
        sources,
    }
}

fn add_framework_specs(
    specs: &mut Vec<RunnerConfigSpec>,
    root: &Path,
    framework: Framework,
    configs: Option<&StringOrList>,
    policies: &BTreeMap<String, TestProjectPolicy>,
    visible_files: &crate::fx::PathSet,
    force: bool,
) {
    let needs_projects = policies
        .values()
        .any(|policy| !policy.integration_suites.is_empty() && policy.include.is_empty());
    if !needs_projects && !force {
        return;
    }
    let raw_configs = configs.map_or_else(
        || {
            project_config::discovered_config_paths(
                root,
                framework,
                &visible_files.iter().cloned().collect::<Vec<_>>(),
            )
        },
        |configs| {
            configs
                .values()
                .into_iter()
                .flat_map(|pattern| {
                    // Retain repeated literal specs for ordinary discovery callers.
                    project_config::expand_explicit_config_values(root, &[pattern], visible_files)
                })
                .collect()
        },
    );
    specs.extend(raw_configs.into_iter().map(|raw| RunnerConfigSpec {
        framework,
        path: crate::codebase::ts_resolver::normalize_path(&root.join(&raw)),
        raw,
    }));
}

/// Extend existing configured selection; do not discover unrelated runner roots.
pub fn prepare_runner_configs_with_deadline_evidence(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
    catalog: std::sync::Arc<crate::codebase::ts_resolver::TsConfigCatalog>,
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> PreparedIntegrationRunnerConfigs {
    prepare_inner(root, config, visible_paths, catalog, Some(sources), true)
}
