use super::runner_config;
use crate::codebase::test_discovery::TestRunner;
use crate::config::v2::schema::{NoMistakesConfig, StringOrList, TestProjectPolicy};
use crate::integration_tests::project_config::prefix_globs;
use crate::integration_tests::types::{ConfigProject, VitestSetupDependency};
use std::collections::BTreeMap;
use std::path::Path;

/// Build only projects described directly by no-mistakes policy without
/// preparing their runner. This reserves explicit cross-runner ownership.
pub(in crate::codebase::test_discovery) fn explicit_policy_projects(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
) -> Vec<ConfigProject> {
    let (configs, policies) = runner_config(config, runner);
    policies
        .iter()
        .filter_map(|(name, policy)| {
            let config = single_config(configs);
            let workspace = workspace_config(config.as_deref());
            configured_project(root, name, policy, config, workspace, Vec::new())
        })
        .collect()
}

pub(super) fn apply_explicit_policy_projects(
    root: &Path,
    configs: Option<&StringOrList>,
    policies: &BTreeMap<String, TestProjectPolicy>,
    projects: &mut Vec<ConfigProject>,
) {
    for (name, policy) in policies {
        let matching = projects
            .iter()
            .filter(|candidate| candidate.policy_name.as_deref() == Some(name))
            .map(|candidate| {
                (
                    candidate.config.clone(),
                    candidate.workspace,
                    candidate.vitest_setup.clone(),
                )
            })
            .collect::<Vec<_>>();
        let configs = if matching.is_empty() {
            let config = single_config(configs);
            vec![(
                config.clone(),
                workspace_config(config.as_deref()),
                Vec::new(),
            )]
        } else {
            matching
        };
        let configured = configs
            .into_iter()
            .filter_map(|(config, workspace, setups)| {
                configured_project(root, name, policy, config, workspace, setups)
            })
            .collect::<Vec<_>>();
        if !configured.is_empty() {
            projects.retain(|candidate| candidate.policy_name.as_deref() != Some(name));
            projects.extend(configured);
        }
    }
}

fn configured_project(
    root: &Path,
    name: &str,
    policy: &TestProjectPolicy,
    config: Option<String>,
    workspace: bool,
    vitest_setup: Vec<VitestSetupDependency>,
) -> Option<ConfigProject> {
    if policy.include.is_empty() {
        return None;
    }
    Some(ConfigProject {
        config,
        workspace,
        policy_name: Some(name.to_string()),
        runner_project_arg: Some(name.to_string()),
        // Explicit policies own their include/exclude universe. Each setup
        // retains its parsed resolution base for fallback independently.
        scope: None,
        include: prefix_globs(root, root, &policy.include),
        exclude: prefix_globs(root, root, &policy.exclude),
        vitest_setup,
    })
}

fn single_config(configs: Option<&StringOrList>) -> Option<String> {
    let values = configs?.values();
    if values.len() == 1 {
        values.into_iter().next()
    } else {
        None
    }
}

fn workspace_config(config: Option<&str>) -> bool {
    config.is_some_and(|config| {
        crate::integration_tests::is_vitest_project_array_path(Path::new(config))
    })
}
