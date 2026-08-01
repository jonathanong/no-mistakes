mod coverage_paths;
mod findings;
mod globs;
mod projects;
mod scan;
mod workflow_filters;

use super::RuleFinding;
use projects::CoverageUnit;
use serde::Deserialize;
use std::collections::BTreeMap;
use workflow_filters::WorkflowSelector;

pub use scan::check_with_files;
pub(crate) use scan::check_with_files_from_snapshot_catalog_sources_and_workflows;

pub const RULE_ID: &str = "vitest-ci-path-coverage";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) project_filters: BTreeMap<String, Vec<String>>,
    pub(crate) source_globs_by_project: BTreeMap<String, Vec<String>>,
    pub(crate) workflows: Vec<WorkflowSelector>,
    pub(crate) include_vitest_project_globs: Option<bool>,
    pub(crate) include_full_suite_triggers: Option<bool>,
    pub(crate) explicit_projects_only: bool,
}

#[cfg(test)]
mod tests;
