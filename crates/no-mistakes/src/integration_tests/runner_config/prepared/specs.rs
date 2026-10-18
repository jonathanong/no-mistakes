use super::*;

pub(super) fn add_framework_specs(
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
