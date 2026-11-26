use super::{Options, RULE_ID};
use crate::codebase::rules::path_filter::GlobMatcher;
use anyhow::{ensure, Result};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NameFilters {
    Allow,
    Reject,
}

#[derive(Debug)]
pub(super) struct Policy {
    pub(super) project: String,
    pub(super) workflows: GlobMatcher,
    pub(super) name_filters: NameFilters,
}

#[derive(Debug)]
pub(in crate::codebase::rules) struct Compiled {
    pub(super) projects: Vec<Policy>,
}

pub(in crate::codebase::rules) fn compile(options: &Options) -> Result<Compiled> {
    let mut seen = BTreeSet::new();
    let mut projects = Vec::new();
    for (index, entry) in options.projects.iter().enumerate() {
        let project = entry.project.trim();
        ensure!(
            !project.is_empty(),
            "{RULE_ID}: projects[{index}].project is required and must not be empty"
        );
        ensure!(
            seen.insert(project.to_string()),
            "{RULE_ID}: projects[{index}].project `{project}` is duplicated"
        );
        ensure!(
            !entry.workflows.is_empty(),
            "{RULE_ID}: projects[{index}].workflows must list at least one workflow"
        );
        let mut workflows = Vec::new();
        for (workflow_index, workflow) in entry.workflows.iter().enumerate() {
            let pattern = workflow.trim().trim_start_matches("./");
            ensure!(
                valid_workflow(pattern),
                "{RULE_ID}: projects[{index}].workflows[{workflow_index}] must be a relative workflow path"
            );
            workflows.push(pattern.to_string());
        }
        let name_filters = name_filters(index, entry.name_filters.as_deref())?;
        projects.push(Policy {
            project: project.to_string(),
            workflows: GlobMatcher::new(
                &workflows,
                &format!("{RULE_ID} projects[{index}].workflows"),
            )?,
            name_filters,
        });
    }
    Ok(Compiled { projects })
}

fn name_filters(index: usize, value: Option<&str>) -> Result<NameFilters> {
    match value.map(str::trim) {
        None => Ok(NameFilters::Allow),
        Some("allow") => Ok(NameFilters::Allow),
        Some("reject") => Ok(NameFilters::Reject),
        Some(other) => anyhow::bail!(
            "{RULE_ID}: projects[{index}].nameFilters must be `allow` or `reject`, found `{other}`"
        ),
    }
}

fn valid_workflow(pattern: &str) -> bool {
    !pattern.is_empty()
        && !pattern.starts_with('/')
        && !pattern.contains('\\')
        && !pattern.split('/').any(|part| part == "..")
}
