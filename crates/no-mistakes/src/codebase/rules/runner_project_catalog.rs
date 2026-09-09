use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::{StringOrList, TestProjectPolicy};
use crate::integration_tests::types::ConfigProject;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(crate) fn config_projects_required(
    root: &Path,
    configs: Option<&StringOrList>,
    projects: &BTreeMap<String, TestProjectPolicy>,
) -> bool {
    configs.is_none()
        || projects.is_empty()
        || configs.is_some_and(|configs| configs.values().iter().any(|raw| root.join(raw).exists()))
        || projects.values().any(|policy| policy.include.is_empty())
}

pub(crate) fn matching_catalog_files(
    root: &Path,
    projects: &[ConfigProject],
    names: &[String],
    files: &[PathBuf],
    unknown_name_error: &str,
) -> Result<Vec<PathBuf>> {
    let selected = projects
        .iter()
        .filter(|project| {
            names.is_empty()
                || project
                    .policy_name
                    .as_deref()
                    .is_some_and(|name| names.iter().any(|selected| selected == name))
        })
        .collect::<Vec<_>>();
    if !names.is_empty() && selected.len() != names.len() {
        anyhow::bail!("{unknown_name_error}");
    }
    let filters = selected
        .iter()
        .map(|project| {
            crate::codebase::test_discovery::ProjectTestFilter::from_project_ref(project)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut matched = files
        .iter()
        .filter(|file| {
            let relative = relative_slash_path(root, file);
            filters.iter().any(|filter| filter.is_match(&relative))
        })
        .cloned()
        .collect::<Vec<_>>();
    matched.sort();
    matched.dedup();
    Ok(matched)
}

pub(crate) fn merge_explicit_projects(
    mut projects: Vec<ConfigProject>,
    explicit: Vec<ConfigProject>,
) -> Vec<ConfigProject> {
    for project in explicit {
        projects
            .retain(|existing| existing.policy_name.as_deref() != project.policy_name.as_deref());
        projects.push(project);
    }
    projects
}
