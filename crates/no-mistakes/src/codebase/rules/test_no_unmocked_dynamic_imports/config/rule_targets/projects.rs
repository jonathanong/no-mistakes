use crate::codebase::check_facts::RunnerConfigFacts;
use crate::config::v2::schema::{StringOrList, TestProjectPolicy};
use crate::integration_tests::project_config;
use crate::integration_tests::types::{ConfigProject, Framework};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) fn load(
    root: &Path,
    framework: Framework,
    configs: Option<&StringOrList>,
    policies: &BTreeMap<String, TestProjectPolicy>,
    project_names: &[&String],
    prepared: Option<(&[PathBuf], &RunnerConfigFacts)>,
) -> Result<Vec<ConfigProject>> {
    if project_names.is_empty() {
        return Ok(Vec::new());
    }
    let mut unresolved = BTreeSet::new();
    let mut projects = Vec::new();
    for name in project_names.iter().copied().collect::<BTreeSet<_>>() {
        if let Some(project) = policies.get(name).and_then(|policy| {
            crate::integration_tests::config::configured_project(root, name, policy)
        }) {
            projects.push(project);
        } else {
            unresolved.insert(name.as_str());
        }
    }
    if !unresolved.is_empty() {
        let recovered = match prepared {
            Some((visible, facts)) => from_facts(root, framework, configs, visible, facts)?,
            None => project_config::load_projects(root, framework, configs)?,
        };
        projects.extend(recovered.into_iter().filter(|project| {
            unresolved.contains(project.policy_name.as_deref().unwrap_or("default"))
        }));
    }
    Ok(projects)
}

fn from_facts(
    root: &Path,
    framework: Framework,
    configs: Option<&StringOrList>,
    visible: &[PathBuf],
    facts: &RunnerConfigFacts,
) -> Result<Vec<ConfigProject>> {
    let paths = configs.map_or_else(
        || project_config::discovered_config_paths(root, framework, visible),
        |configs| {
            project_config::expand_explicit_config_values(
                root,
                &configs.values(),
                &visible.iter().cloned().collect(),
            )
        },
    );
    let mut projects = Vec::new();
    for raw in paths {
        let path = crate::codebase::ts_resolver::normalize_path(&root.join(raw));
        let result = facts
            .get(&path)
            .and_then(|facts| facts.projects_for(framework))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "missing prepared {} config projects for {}",
                    framework.as_str(),
                    path.display()
                )
            })?;
        projects.extend(result.map_err(anyhow::Error::msg)?);
    }
    Ok(projects)
}
