use super::*;

pub(crate) fn prepare(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
    tsconfig: &TsConfig,
) -> PreparedIntegrationRunnerConfigs {
    prepare_inner(root, config, visible_paths, tsconfig, None)
}

pub(crate) fn prepare_with_sources(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
    tsconfig: &TsConfig,
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> PreparedIntegrationRunnerConfigs {
    prepare_inner(root, config, visible_paths, tsconfig, Some(sources))
}

fn prepare_inner(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
    tsconfig: &TsConfig,
    sources: Option<std::sync::Arc<crate::codebase::ts_source::SourceStore>>,
) -> PreparedIntegrationRunnerConfigs {
    let mut specs = Vec::new();
    add_framework_specs(
        &mut specs,
        root,
        Framework::Playwright,
        config.tests.playwright.configs.as_ref(),
        &config.tests.playwright.projects,
        visible_paths,
    );
    add_framework_specs(
        &mut specs,
        root,
        Framework::Vitest,
        config.tests.vitest.configs.as_ref(),
        &config.tests.vitest.projects,
        visible_paths,
    );
    let mut visible_files = visible_paths
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<HashSet<_>>();
    // Explicit runner configs are authoritative even when Git ignores them.
    // Add their normalized paths to the frozen request view up front; reads are
    // still memoized by the request source store and failures become prepared
    // project results rather than triggering a second filesystem discovery.
    visible_files.extend(specs.iter().map(|spec| spec.path.clone()));
    PreparedIntegrationRunnerConfigs {
        root: root.to_path_buf(),
        specs,
        tsconfig: tsconfig.clone(),
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
    visible_paths: &[PathBuf],
) {
    let needs_projects = policies
        .values()
        .any(|policy| !policy.integration_suites.is_empty() && policy.include.is_empty());
    if !needs_projects {
        return;
    }
    let raw_configs = configs.map_or_else(
        || project_config::discovered_config_paths(root, framework, visible_paths),
        StringOrList::values,
    );
    specs.extend(raw_configs.into_iter().map(|raw| RunnerConfigSpec {
        framework,
        path: crate::codebase::ts_resolver::normalize_path(&root.join(&raw)),
        raw,
    }));
}
